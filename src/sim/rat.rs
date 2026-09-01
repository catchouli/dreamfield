use bevy_ecs::prelude::Component;
use cgmath::vec3;

use dreamfield_system::intersection::{Collider, Shape};

use super::melee_enemy::MeleeEnemyParams;

/// The speed the rat wanders at
const WANDER_SPEED: f32 = 1.5;

/// The speed the rat scurries at when chasing, so it's easy to escape
const CHASE_SPEED: f32 = 2.4;

/// The distance at which the rat notices the player and starts chasing
const AGGRO_RADIUS: f32 = 8.0;

/// The distance at which the rat starts an attack
const ATTACK_RANGE: f32 = 1.0;

/// The maximum distance at which a strike still connects
const ATTACK_STRIKE_RANGE: f32 = 1.5;

/// The duration of the rat's attack windup, in seconds
const ATTACK_WINDUP_TIME: f32 = 0.35;

/// The time between rat attacks, in seconds
const ATTACK_COOLDOWN_TIME: f32 = 1.0;

/// The damage dealt by a rat attack
const RAT_ATTACK_DAMAGE: f32 = 6.0;

/// How far the rat leans back during the attack windup, in radians
const LEAN_ANGLE: f32 = 0.3;

/// The radius of the rat's collision sweep
const RAT_RADIUS: f32 = 0.25;

/// A tag component identifying rats
#[derive(Component)]
pub struct Rat;

impl Rat {
    /// The rat's collider, matching the low-slung ~1.6m long model
    pub fn collider() -> Collider {
        Collider::new(Shape::BoundingSpheroid(vec3(0.0, 0.25, 0.0), vec3(0.3, 0.25, 0.4)))
    }

    /// The rat's melee AI parameters
    pub fn params() -> MeleeEnemyParams {
        MeleeEnemyParams {
            wander_speed: WANDER_SPEED,
            chase_speed: CHASE_SPEED,
            aggro_radius: AGGRO_RADIUS,
            attack_range: ATTACK_RANGE,
            attack_strike_range: ATTACK_STRIKE_RANGE,
            attack_windup_time: ATTACK_WINDUP_TIME,
            attack_cooldown_time: ATTACK_COOLDOWN_TIME,
            attack_damage: RAT_ATTACK_DAMAGE,
            lean_angle: LEAN_ANGLE,
            move_radius: RAT_RADIUS,
        }
    }
}
