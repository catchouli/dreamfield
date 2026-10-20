use bevy_ecs::prelude::Component;
use cgmath::vec3;

use dreamfield_system::intersection::{Collider, Shape};

use super::melee_enemy::MeleeEnemyParams;

/// The speed the goblin wanders at
const WANDER_SPEED: f32 = 1.0;

/// The speed the goblin ambles at when chasing, so it's easy to escape
const CHASE_SPEED: f32 = 2.2;

/// The distance at which the goblin notices the player and starts chasing
const AGGRO_RADIUS: f32 = 12.0;

/// The distance at which the goblin starts an attack
const ATTACK_RANGE: f32 = 1.4;

/// The maximum distance at which a strike still connects
const ATTACK_STRIKE_RANGE: f32 = 1.9;

/// The duration of the goblin's attack windup, in seconds
const ATTACK_WINDUP_TIME: f32 = 0.45;

/// The time between goblin attacks, in seconds
const ATTACK_COOLDOWN_TIME: f32 = 1.4;

/// The damage dealt by a goblin attack
const GOBLIN_ATTACK_DAMAGE: f32 = 10.0;

/// How far the goblin leans back during the attack windup, in radians
const LEAN_ANGLE: f32 = 0.45;

/// The radius of the goblin's collision sweep
const GOBLIN_RADIUS: f32 = 0.45;

/// A tag component identifying goblins
#[derive(Component)]
pub struct Goblin;

impl Goblin {
    /// The goblin's collider, matching the ~1.5m tall model
    pub fn collider() -> Collider {
        Collider::new(Shape::BoundingSpheroid(vec3(0.0, 0.75, 0.0), vec3(0.45, 0.75, 0.45)))
    }

    /// The goblin's melee AI parameters
    pub fn params() -> MeleeEnemyParams {
        MeleeEnemyParams {
            wander_speed: WANDER_SPEED,
            chase_speed: CHASE_SPEED,
            aggro_radius: AGGRO_RADIUS,
            attack_range: ATTACK_RANGE,
            attack_strike_range: ATTACK_STRIKE_RANGE,
            attack_windup_time: ATTACK_WINDUP_TIME,
            attack_cooldown_time: ATTACK_COOLDOWN_TIME,
            attack_damage: GOBLIN_ATTACK_DAMAGE,
            lean_angle: LEAN_ANGLE,
            move_radius: GOBLIN_RADIUS,
        }
    }
}
