use std::f32::consts::TAU;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy_ecs::{prelude::{Component, Entity}, query::Without, system::{Query, Res, ResMut}};
use cgmath::{Vector3, vec3, InnerSpace, Zero, Matrix3, Rad, SquareMatrix};

use dreamfield_system::components::Transform;
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

/// The minimum time between picking new wander directions, in seconds
const WANDER_TIME_MIN: f32 = 2.0;

/// The maximum time between picking new wander directions, in seconds
const WANDER_TIME_MAX: f32 = 5.0;

/// Counter for assigning each enemy a unique pseudo-random seed
static ENEMY_SEED_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Per-enemy tuning parameters for the shared melee enemy AI
#[derive(Copy, Clone)]
pub struct MeleeEnemyParams {
    /// The speed the enemy wanders at
    pub wander_speed: f32,

    /// The speed the enemy chases at, slower than the player's walk speed so it can be escaped
    pub chase_speed: f32,

    /// The distance at which the enemy notices the player and starts chasing
    pub aggro_radius: f32,

    /// The distance at which the enemy starts an attack
    pub attack_range: f32,

    /// The maximum distance at which a strike still connects
    pub attack_strike_range: f32,

    /// The duration of the attack windup, in seconds
    pub attack_windup_time: f32,

    /// The time between attacks, in seconds
    pub attack_cooldown_time: f32,

    /// The damage dealt by an enemy attack
    pub attack_damage: f32,

    /// How far the enemy leans back during the attack windup, in radians
    pub lean_angle: f32,

    /// The radius of the enemy's collision sweep
    pub move_radius: f32,
}

/// The melee enemy AI component, shared by all ground-based melee enemies
#[derive(Component)]
pub struct MeleeEnemy {
    params: MeleeEnemyParams,

    /// Seconds until the enemy picks a new wander direction
    wander_timer: f32,

    /// The direction the enemy wanders in
    dir: Vector3<f32>,

    /// Seconds elapsed in the current attack windup, if attacking
    attack_windup: Option<f32>,

    /// Seconds until the enemy can attack again
    attack_cooldown: f32,

    /// Per-entity seed for pseudo-random wander directions
    seed: u32,
}

impl MeleeEnemy {
    /// Create a new MeleeEnemy with the given parameters
    pub fn new(params: MeleeEnemyParams) -> Self {
        MeleeEnemy {
            params,
            wander_timer: 0.0,
            dir: Vector3::zero(),
            attack_windup: None,
            attack_cooldown: 0.0,
            seed: ENEMY_SEED_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// The melee enemy update system
pub fn melee_enemy_update(sim_time: Res<SimTime>,
                          mut collision: ResMut<WorldCollision>,
                          mut world: ResMut<WorldChunkManager>,
                          mut diagnostics: ResMut<Diagnostics>,
                          mut enemies: Query<(Entity, &mut MeleeEnemy, &mut Transform), Without<PlayerMovement>>,
                          mut player: Query<(&mut Transform, &mut Health, &PlayerMovement)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    // Get the player position
    let player_pos = {
        let p0 = player.single_mut();
        p0.0.pos
    };

    // Damage dealt to the player by enemy strikes this update
    let mut player_damage = 0.0;

    for (entity, mut enemy, mut transform) in enemies.iter_mut() {
        let params = enemy.params;
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
        if let Some(windup) = &mut enemy.attack_windup {
            *windup += time_delta;

            if *windup >= params.attack_windup_time {
                if dist_to_player <= params.attack_strike_range {
                    player_damage += params.attack_damage;
                }

                enemy.attack_windup = None;
                enemy.attack_cooldown = params.attack_cooldown_time;
                transform.rot = face_direction(player_dir);
            }
            else {
                let lean = params.lean_angle * f32::min(*windup / params.attack_windup_time, 1.0);
                transform.rot = face_direction(player_dir)
                    * Matrix3::from_axis_angle(vec3(1.0, 0.0, 0.0), Rad(lean));
            }

            continue;
        }

        // Cooling down between attacks: face the player but don't move
        if enemy.attack_cooldown > 0.0 {
            enemy.attack_cooldown -= time_delta;
            transform.rot = face_direction(player_dir);
            continue;
        }

        // Chase the player and attack when close, otherwise wander
        if dist_to_player < params.aggro_radius {
            if dist_to_player <= params.attack_range {
                enemy.attack_windup = Some(0.0);
                transform.rot = face_direction(player_dir);
                continue;
            }

            let new_pos = move_on_ground(&mut collision, &mut world, entity, transform.pos,
                player_dir, params.chase_speed, params.move_radius, time_delta);
            transform.pos = new_pos;
            transform.rot = face_direction(player_dir);
        }
        else {
            enemy.wander_timer -= time_delta;
            if enemy.wander_timer <= 0.0 {
                enemy.wander_timer = WANDER_TIME_MIN
                    + pseudo_random(enemy.seed as f32 + sim_time.sim_time as f32)
                        * (WANDER_TIME_MAX - WANDER_TIME_MIN);
                enemy.dir = random_xz_direction(enemy.seed, sim_time.sim_time);
            }

            let new_pos = move_on_ground(&mut collision, &mut world, entity, transform.pos,
                enemy.dir, params.wander_speed, params.move_radius, time_delta);
            transform.pos = new_pos;
            transform.rot = face_direction(enemy.dir);
        }
    }

    // Apply strike damage to the player, respawning them if they died
    if player_damage > 0.0 {
        diagnostics.damage_flash = FLASH_TRIGGER_INTENSITY;

        let (mut transform, mut health, _) = player.single_mut();

        let died = health.damage(player_damage);
        if died {
            log::info!("You died, respawning at the village entrance");
            transform.pos = PLAYER_SPAWN_POS;
            health.health = PLAYER_MAX_HEALTH;
        }
        else {
            log::info!("Enemies hit you for {player_damage} damage");
        }
    }
}

/// Build a rotation facing the given direction on the xz plane. The model's +Z axis is its
/// front, so the rotation maps model +Z onto the facing direction.
fn face_direction(direction: Vector3<f32>) -> Matrix3<f32> {
    let up = vec3(0.0, 1.0, 0.0);
    let right = up.cross(direction);
    Matrix3::from_cols(right, up, direction)
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
