use bevy_ecs::{prelude::{Component, Commands, Entity}, query::{With, Without}, system::{Res, ParamSet, Query}};
use cgmath::{Vector3, vec3, InnerSpace, ElementWise, Quaternion, Rad, Rotation3, Matrix3, VectorSpace};

use dreamfield_system::components::{Transform, EntityName};
use dreamfield_system::intersection::{Collider, Shape, toi_unit_sphere_point};
use dreamfield_system::resources::{SimTime, InputState, InputName};

use super::health::Health;
use super::player_movement::{PlayerMovement, CHAR_EYE_LEVEL};

/// The duration of a sword swing, in seconds
const SWING_TIME: f32 = 0.4;

/// The fraction of the swing at which damage is dealt
const SWING_DAMAGE_FRACTION: f32 = 0.45;

/// The damage dealt by a sword swing
const ATTACK_DAMAGE: f32 = 40.0;

/// The radius of the attack's collision sweep
const ATTACK_RADIUS: f32 = 0.4;

/// The distance from the eye at which the attack sweep starts
const ATTACK_START_DISTANCE: f32 = 0.3;

/// The length of the attack's collision sweep
const ATTACK_REACH: f32 = 1.6;

/// The camera-space rest position of the sword viewmodel (right, up, forward)
const SWORD_REST_POS: Vector3<f32> = vec3(0.42, -0.32, -0.72);

/// The camera-space strike position of the sword viewmodel (right, up, forward)
const SWORD_STRIKE_POS: Vector3<f32> = vec3(-0.30, -0.55, -0.80);

/// The camera-space axis the sword swings around, sweeping the blade from upper-right to
/// lower-left
const SWING_AXIS: Vector3<f32> = vec3(1.0, 1.0, -0.7);

/// The sword swing angle at rest, in radians
const SWING_START_ANGLE: f32 = -0.3;

/// The sword swing angle at the end of the swing, in radians
const SWING_END_ANGLE: f32 = -2.2;

/// The player attack component
#[derive(Component)]
pub struct PlayerAttack {
    /// Seconds elapsed in the current swing, if one is in progress
    swing_timer: Option<f32>,

    /// Whether damage has been dealt during the current swing
    damage_dealt: bool,
}

impl PlayerAttack {
    /// Create a new PlayerAttack
    pub fn new() -> Self {
        PlayerAttack {
            swing_timer: None,
            damage_dealt: false,
        }
    }
}

/// A tag component for the first-person sword viewmodel entity
#[derive(Component)]
pub struct SwordViewmodel;

/// The player attack update system
pub fn attack_update(sim_time: Res<SimTime>,
                     input: Res<InputState>,
                     mut commands: Commands,
                     mut param_set: ParamSet<(
                         Query<(&Transform, &PlayerMovement, &mut PlayerAttack), Without<SwordViewmodel>>,
                         Query<&mut Transform, With<SwordViewmodel>>,
                         Query<(Entity, &Transform, &Collider, &mut Health, Option<&EntityName>),
                               Without<SwordViewmodel>>)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    // Update the swing state
    let (eye_pos, forward, swing_progress, swing_damage) = {
        let mut p0 = param_set.p0();
        let (player_transform, player_movement, mut attack) = p0.single_mut();

        // Start a new swing
        if input.is_just_pressed(InputName::Attack) && attack.swing_timer.is_none() && player_movement.enabled {
            attack.swing_timer = Some(0.0);
            attack.damage_dealt = false;
        }

        // Update the swing progress
        let swing_progress = match &mut attack.swing_timer {
            Some(timer) => {
                *timer += time_delta;
                f32::min(*timer / SWING_TIME, 1.0)
            },
            None => 0.0,
        };

        // Deal damage partway through the swing
        let swing_damage = swing_progress >= SWING_DAMAGE_FRACTION && !attack.damage_dealt;
        if swing_damage {
            attack.damage_dealt = true;
        }

        if swing_progress >= 1.0 {
            attack.swing_timer = None;
        }

        let eye_pos = player_transform.pos + vec3(0.0, CHAR_EYE_LEVEL, 0.0);
        let forward = player_movement.forward();

        (eye_pos, forward, swing_progress, swing_damage)
    };

    // Update the sword viewmodel pose
    {
        let (offset, rotation) = sword_pose(swing_progress);
        let orientation = param_set.p0().single().1.orientation();

        let mut p1 = param_set.p1();
        let mut sword_transform = p1.single_mut();
        sword_transform.pos = eye_pos + orientation * offset;
        sword_transform.rot = Matrix3::from(orientation * rotation);
    }

    // Deal damage to anything hit by the swing
    if swing_damage {
        let start = eye_pos + forward * ATTACK_START_DISTANCE;
        let velocity = forward * ATTACK_REACH;

        for (entity, transform, collider, mut health, name) in param_set.p2().iter_mut() {
            let (offset, radii) = match collider.shape {
                Shape::BoundingSpheroid(offset, radii) => (offset, radii),
                _ => panic!("attack: collider shape not supported for melee")
            };

            if swept_sphere_spheroid_toi(start, velocity, ATTACK_RADIUS, transform.pos, offset, radii).is_some() {
                let name = name.map_or("unknown".to_string(), |name| name.name.clone());
                let died = health.damage(ATTACK_DAMAGE);

                if died {
                    log::info!("{name} died");
                    commands.entity(entity).despawn();
                }
                else {
                    log::info!("Hit {name} for {ATTACK_DAMAGE} damage");
                }
            }
        }
    }
}

/// Get the camera-space position offset and rotation of the sword viewmodel for a given swing
/// progress, from 0 (rest) to 1 (end of swing)
fn sword_pose(progress: f32) -> (Vector3<f32>, Quaternion<f32>) {
    // Smoothstep ease, so the swing starts and ends smoothly
    let eased = progress * progress * (3.0 - 2.0 * progress);

    let offset = SWORD_REST_POS.lerp(SWORD_STRIKE_POS, eased);
    let angle = SWING_START_ANGLE + (SWING_END_ANGLE - SWING_START_ANGLE) * eased;
    let rotation = Quaternion::from_axis_angle(SWING_AXIS.normalize(), Rad(angle));

    (offset, rotation)
}

/// Get the time of impact of a sphere swept through a bounding spheroid, if any
fn swept_sphere_spheroid_toi(start: Vector3<f32>, velocity: Vector3<f32>, radius: f32,
    pos: Vector3<f32>, offset: Vector3<f32>, radii: Vector3<f32>) -> Option<f32>
{
    // Convert to the space where the attack sphere is a unit sphere
    let cbm = vec3(1.0 / radius, 1.0 / radius, 1.0 / radius);

    // Convert to the space where the sum of the two radii is a unit sphere, mirroring
    // WorldCollision::sweep_unit_sphere_entity
    let self_to_combined_cbm = vec3(
        1.0 / (1.0 + radii.x * cbm.x),
        1.0 / (1.0 + radii.y * cbm.y),
        1.0 / (1.0 + radii.z * cbm.z));

    let start_es = start.mul_element_wise(cbm).mul_element_wise(self_to_combined_cbm);
    let velocity_es = velocity.mul_element_wise(cbm).mul_element_wise(self_to_combined_cbm);
    let center_es = (pos + offset).mul_element_wise(cbm).mul_element_wise(self_to_combined_cbm);

    toi_unit_sphere_point(start_es, velocity_es, center_es)
        .filter(|toi| *toi >= 0.0 && *toi <= 1.0)
}
