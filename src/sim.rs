pub mod player_movement;
pub mod ball;
mod entity_spawner;

use dreamfield_system::{resources::{InputState, InputName}, world::{world_collision::WorldCollision, WorldChunkManager}, math::intersection::Capsule};
use bevy_ecs::{schedule::SystemSet, prelude::Component, system::{Res, Query, ResMut, ParamSet}};
use cgmath::{Vector3, vec3, ElementWise};
use dreamfield_system::components::Transform;

// Components
pub use player_movement::{PlayerMovement, PlayerMovementMode};
pub use ball::Ball;

/// Sim systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .label("sim")
        .with_system(player_movement::player_update)
        .with_system(ball::ball_update)
        .with_system(entity_spawner::entity_spawner)
        //.with_system(capsule_test_system)
}

/// Capsule collision test
#[derive(Component)]
pub struct CapsuleA;

#[derive(Component)]
pub struct CapsuleB;

/// A capsule collider defined by a sphere swept along a line segment
#[derive(Component)]
pub struct CapsuleCollider {
    pub capsule: Capsule,
    pub cbm: Vector3<f32>,
}

impl CapsuleCollider {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>, radius: f32) -> Self {
        // Calculate change of basis matrix to convert from world coordinates to the vector space
        // where the spheres of this capsule are unit spheres
        let cbm = vec3(1.0 / radius, 1.0 / radius, 1.0 / radius);

        Self {
            capsule: Capsule::new(a, b, radius),
            cbm,
        }
    }
}

fn _capsule_test_system(input: Res<InputState>, mut collision: ResMut<WorldCollision>, mut world: ResMut<WorldChunkManager>,
    mut set: ParamSet<(Query<(&CapsuleA, &CapsuleCollider, &mut Transform)>, Query<(&PlayerMovement, &Transform)>)>)
{
    if input.is_held(InputName::Run) {
        // Get sweep start and velocity
        let (start, velocity) = {
            let player_query = set.p1();
            let (player_movement, player_transform) = player_query.single();
            let start = player_transform.pos + vec3(0.0, 1.7, 0.0);
            let velocity = player_movement.forward() * 15.0;
            (start, velocity)
        };

        // Collide capsule with world
        {
            let mut capsule_query = set.p0();
            let (_, collider, mut transform) = capsule_query.single_mut();

            let radius = collider.capsule.radius;

            let cbm = vec3(1.0 / radius, 1.0 / radius, 1.0 / radius);

            let a = (start + collider.capsule.a).mul_element_wise(cbm);
            let b = (start + collider.capsule.b).mul_element_wise(cbm);
            let velocity = velocity.mul_element_wise(cbm);

            if let Some(res) = collision.sweep_unit_capsule(&mut world, a, b, velocity, cbm) {
                let hit_pos = (a + velocity * res.toi()).div_element_wise(cbm);
                transform.pos = hit_pos - vec3(0.0, 0.5, 0.0);
            };
        }
    }
}
