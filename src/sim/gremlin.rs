use std::f32::consts::TAU;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy_ecs::{prelude::{Component, Entity}, query::Without, system::{ParamSet, Query, Res, ResMut}};
use cgmath::{Vector3, vec3, InnerSpace, Zero, Matrix3, SquareMatrix};

use dreamfield_system::components::Transform;
use dreamfield_system::resources::SimTime;
use dreamfield_system::world::{WorldChunkManager, world_collision::WorldCollision};

use super::player_movement::PlayerMovement;

/// The speed the gremlin wanders at
const WANDER_SPEED: f32 = 1.0;

/// The speed the gremlin flees at, slower than the player's walk speed so it can be caught
const FLEE_SPEED: f32 = 3.4;

/// The distance at which the gremlin notices the player and starts fleeing
const FLEE_RADIUS: f32 = 5.0;

/// The minimum time between picking new wander directions, in seconds
const WANDER_TIME_MIN: f32 = 2.0;

/// The maximum time between picking new wander directions, in seconds
const WANDER_TIME_MAX: f32 = 5.0;

/// The radius of the gremlin's collision sweep
const GREMLIN_RADIUS: f32 = 0.3;

/// The height of the gremlin's collider center above its feet
const GREMLIN_COLLIDER_HEIGHT: f32 = 0.25;

/// Counter for assigning each gremlin a unique pseudo-random seed
static GREMLIN_SEED_COUNTER: AtomicU32 = AtomicU32::new(0);

/// The gremlin component
#[derive(Component)]
pub struct Gremlin {
    /// Seconds until the gremlin picks a new wander direction
    wander_timer: f32,

    /// The direction the gremlin wanders in
    dir: Vector3<f32>,

    /// Per-entity seed for pseudo-random wander directions
    seed: u32,
}

impl Gremlin {
    /// Create a new Gremlin
    pub fn new() -> Self {
        Gremlin {
            wander_timer: 0.0,
            dir: Vector3::zero(),
            seed: GREMLIN_SEED_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// The gremlin update system
pub fn gremlin_update(sim_time: Res<SimTime>,
                      mut collision: ResMut<WorldCollision>,
                      mut world: ResMut<WorldChunkManager>,
                      mut param_set: ParamSet<(
                          Query<(&Transform, &PlayerMovement), Without<Gremlin>>,
                          Query<(Entity, &mut Gremlin, &mut Transform)>)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    // Get the player position
    let player_pos = {
        let query = param_set.p0();
        let (transform, _) = query.single();
        transform.pos
    };

    for (entity, mut gremlin, mut transform) in param_set.p1().iter_mut() {
        // Flee from the player if they're close, otherwise wander
        let to_player = player_pos - transform.pos;
        let to_player_flat = vec3(to_player.x, 0.0, to_player.z);
        let dist_to_player = to_player_flat.magnitude();

        let (direction, speed) = if dist_to_player < FLEE_RADIUS && dist_to_player > 0.001 {
            (-to_player_flat / dist_to_player, FLEE_SPEED)
        }
        else {
            gremlin.wander_timer -= time_delta;
            if gremlin.wander_timer <= 0.0 {
                gremlin.wander_timer = WANDER_TIME_MIN
                    + pseudo_random(gremlin.seed as f32 + sim_time.sim_time as f32)
                        * (WANDER_TIME_MAX - WANDER_TIME_MIN);
                gremlin.dir = random_xz_direction(gremlin.seed, sim_time.sim_time);
            }
            (gremlin.dir, WANDER_SPEED)
        };

        // Face the direction of movement, using the same math as the minecart
        let up = vec3(0.0, 1.0, 0.0);
        let forward = -direction;
        let right = up.cross(forward);
        let look_at = Matrix3::new(right.x, up.x, forward.x, right.y, up.y, forward.y,
            right.z, up.z, forward.z);
        // I'm not really sure why we have to invert this... (copied from the minecart)
        transform.rot = look_at.invert().unwrap();

        // Sweep a small sphere through the world to find out how far we can move
        let start = transform.pos + vec3(0.0, GREMLIN_COLLIDER_HEIGHT, 0.0);
        let velocity = direction * speed * time_delta;

        transform.pos = match collision.sweep_sphere(&mut world, start, velocity,
            vec3(GREMLIN_RADIUS, GREMLIN_RADIUS, GREMLIN_RADIUS), Some(entity))
        {
            // Slide along whatever we hit
            Some(hit) => {
                let normal = *hit.normal();
                let flat_normal = vec3(normal.x, 0.0, normal.z);
                let slide_dir = direction - flat_normal * direction.dot(flat_normal);

                if slide_dir.magnitude2() > 0.0001 {
                    gremlin.dir = slide_dir.normalize();
                    transform.pos + slide_dir.normalize() * speed * time_delta
                }
                else {
                    transform.pos
                }
            },
            None => transform.pos + velocity,
        };
    }
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
