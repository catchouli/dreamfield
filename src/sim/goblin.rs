use std::f32::consts::TAU;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy_ecs::{prelude::{Component, Entity}, query::Without, system::{ParamSet, Query, Res, ResMut}};
use cgmath::{Vector3, vec3, InnerSpace, Zero, Matrix3, Rad, SquareMatrix};

use dreamfield_system::components::Transform;
use dreamfield_system::intersection::{Collider, Shape};
use dreamfield_system::resources::{Diagnostics, SimTime};
use dreamfield_system::world::{WorldChunkManager, world_collision::WorldCollision};

use super::damage_flash::FLASH_TRIGGER_INTENSITY;
use super::ground_movement::move_on_ground;
use super::health::Health;
use super::player_movement::PlayerMovement;

/// The maximum player health
pub const PLAYER_MAX_HEALTH: f32 = 100.0;

/// The position the player respawns at when killed (the village entrance)
const PLAYER_SPAWN_POS: Vector3<f32> = vec3(-125.1, 5.8, 123.8);

/// The speed the goblin wanders at
const WANDER_SPEED: f32 = 1.0;

/// The speed the goblin chases at, slower than the player's walk speed so it can be escaped
const CHASE_SPEED: f32 = 3.5;

/// The distance at which the goblin notices the player and starts chasing
const AGGRO_RADIUS: f32 = 12.0;

/// The distance at which the goblin starts an attack
const ATTACK_RANGE: f32 = 1.4;

/// The maximum distance at which a strike still connects
const ATTACK_STRIKE_RANGE: f32 = 1.9;

/// The duration of the attack windup, in seconds
const ATTACK_WINDUP_TIME: f32 = 0.45;

/// The time between attacks, in seconds
const ATTACK_COOLDOWN_TIME: f32 = 1.4;

/// The damage dealt by a goblin attack
const GOBLIN_ATTACK_DAMAGE: f32 = 10.0;

/// How far the goblin leans back during the attack windup, in radians
const LEAN_ANGLE: f32 = 0.45;

/// The minimum time between picking new wander directions, in seconds
const WANDER_TIME_MIN: f32 = 2.0;

/// The maximum time between picking new wander directions, in seconds
const WANDER_TIME_MAX: f32 = 5.0;

/// The radius of the goblin's collision sweep
const GOBLIN_RADIUS: f32 = 0.45;

/// Counter for assigning each goblin a unique pseudo-random seed
static GOBLIN_SEED_COUNTER: AtomicU32 = AtomicU32::new(0);

/// The goblin component
#[derive(Component)]
pub struct Goblin {
    /// Seconds until the goblin picks a new wander direction
    wander_timer: f32,

    /// The direction the goblin wanders in
    dir: Vector3<f32>,

    /// Seconds elapsed in the current attack windup, if attacking
    attack_windup: Option<f32>,

    /// Seconds until the goblin can attack again
    attack_cooldown: f32,

    /// Per-entity seed for pseudo-random wander directions
    seed: u32,
}

impl Goblin {
    /// Create a new Goblin
    pub fn new() -> Self {
        Goblin {
            wander_timer: 0.0,
            dir: Vector3::zero(),
            attack_windup: None,
            attack_cooldown: 0.0,
            seed: GOBLIN_SEED_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// The goblin's collider, matching the ~1.5m tall model
    pub fn collider() -> Collider {
        Collider::new(Shape::BoundingSpheroid(vec3(0.0, 0.75, 0.0), vec3(0.45, 0.75, 0.45)))
    }
}

/// The goblin update system
pub fn goblin_update(sim_time: Res<SimTime>,
                     mut collision: ResMut<WorldCollision>,
                     mut world: ResMut<WorldChunkManager>,
                     mut diagnostics: ResMut<Diagnostics>,
                     mut param_set: ParamSet<(
                         Query<(&mut Transform, &mut Health, &PlayerMovement), Without<Goblin>>,
                         Query<(Entity, &mut Goblin, &mut Transform)>)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    // Get the player position
    let player_pos = {
        let p0 = param_set.p0();
        let (transform, _, _) = p0.single();
        transform.pos
    };

    // Damage dealt to the player by goblin strikes this update
    let mut player_damage = 0.0;

    for (entity, mut goblin, mut transform) in param_set.p1().iter_mut() {
        let to_player = player_pos - transform.pos;
        let to_player_flat = vec3(to_player.x, 0.0, to_player.z);
        let dist_to_player = to_player_flat.magnitude();
        let player_dir = if dist_to_player > 0.001 {
            to_player_flat / dist_to_player
        }
        else {
            vec3(0.0, 0.0, 1.0)
        };

        // Attacking: stand still and lean back, then strike at the end of the windup
        if let Some(windup) = &mut goblin.attack_windup {
            *windup += time_delta;

            if *windup >= ATTACK_WINDUP_TIME {
                if dist_to_player <= ATTACK_STRIKE_RANGE {
                    player_damage += GOBLIN_ATTACK_DAMAGE;
                }

                goblin.attack_windup = None;
                goblin.attack_cooldown = ATTACK_COOLDOWN_TIME;
                transform.rot = face_direction(player_dir);
            }
            else {
                let lean = LEAN_ANGLE * f32::min(*windup / ATTACK_WINDUP_TIME, 1.0);
                transform.rot = face_direction(player_dir)
                    * Matrix3::from_axis_angle(vec3(1.0, 0.0, 0.0), Rad(lean));
            }

            continue;
        }

        // Cooling down between attacks: face the player but don't move
        if goblin.attack_cooldown > 0.0 {
            goblin.attack_cooldown -= time_delta;
            transform.rot = face_direction(player_dir);
            continue;
        }

        // Chase the player and attack when close, otherwise wander
        if dist_to_player < AGGRO_RADIUS {
            if dist_to_player <= ATTACK_RANGE {
                goblin.attack_windup = Some(0.0);
                transform.rot = face_direction(player_dir);
                continue;
            }

            let new_pos = move_on_ground(&mut collision, &mut world, entity, transform.pos,
                player_dir, CHASE_SPEED, GOBLIN_RADIUS, time_delta);
            transform.pos = new_pos;
            transform.rot = face_direction(player_dir);
        }
        else {
            goblin.wander_timer -= time_delta;
            if goblin.wander_timer <= 0.0 {
                goblin.wander_timer = WANDER_TIME_MIN
                    + pseudo_random(goblin.seed as f32 + sim_time.sim_time as f32)
                        * (WANDER_TIME_MAX - WANDER_TIME_MIN);
                goblin.dir = random_xz_direction(goblin.seed, sim_time.sim_time);
            }

            let new_pos = move_on_ground(&mut collision, &mut world, entity, transform.pos,
                goblin.dir, WANDER_SPEED, GOBLIN_RADIUS, time_delta);
            transform.pos = new_pos;
            transform.rot = face_direction(goblin.dir);
        }
    }

    // Apply strike damage to the player, respawning them if they died
    if player_damage > 0.0 {
        diagnostics.damage_flash = FLASH_TRIGGER_INTENSITY;

        let mut p0 = param_set.p0();
        let (mut transform, mut health, _) = p0.single_mut();

        let died = health.damage(player_damage);
        if died {
            log::info!("You died, respawning at the village entrance");
            transform.pos = PLAYER_SPAWN_POS;
            health.health = PLAYER_MAX_HEALTH;
        }
        else {
            log::info!("Goblin hit you for {player_damage} damage");
        }
    }
}

/// Build a rotation facing the given direction on the xz plane, using the same math as the
/// minecart
fn face_direction(direction: Vector3<f32>) -> Matrix3<f32> {
    let up = vec3(0.0, 1.0, 0.0);
    let forward = -direction;
    let right = up.cross(forward);
    let look_at = Matrix3::new(right.x, up.x, forward.x, right.y, up.y, forward.y,
        right.z, up.z, forward.z);
    // I'm not really sure why we have to invert this... (copied from the minecart)
    look_at.invert().unwrap()
}

/// Get a pseudo-random value in 0..1 from a seed
fn pseudo_random(seed: f32) -> f32 {
    (seed.sin() * 43758.5453).fract().abs()
}

/// Pick a pseudo-random direction on the xz plane
fn random_xz_direction(seed: u32, time: f64) -> Vector3<f32> {
    let angle = pseudo_random(seed as f32 + time as f32 * 0.173) * TAU;
    vec3(angle.cos(), 0.0, angle.sin())
}
