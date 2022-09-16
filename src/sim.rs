pub mod player_movement;
pub mod ball;
mod entity_spawner;
mod minecart;

use bevy_ecs::schedule::SystemSet;

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
        .with_system(minecart::update_minecart)
}

// Test code for testing collisions, I'll leave it here for now until I'm sure I'm done...
// A capsule collider defined by a sphere swept along a line segment
//#[derive(Component)]
//pub struct SpheroidCollider {
//    radius: Vector3<f32>,
//    cbm: Vector3<f32>,
//}

//impl SpheroidCollider {
//    pub fn new(radius: Vector3<f32>) -> Self {
//        // Calculate change of basis matrix to convert from world coordinates to the vector space
//        // where the spheres of this capsule are unit spheres
//        let cbm = vec3(1.0 / radius.x, 1.0 / radius.y, 1.0 / radius.z);
//
//        Self {
//            radius,
//            cbm,
//        }
//    }
//
//    pub fn radius(&self) -> &Vector3<f32> {
//        &self.radius
//    }
//
//    pub fn cbm(&self) -> &Vector3<f32> {
//        &self.cbm
//    }
//}

//fn _capsule_test_system(input: Res<InputState>, mut collision: ResMut<WorldCollision>, mut world: ResMut<WorldChunkManager>,
//    mut set: ParamSet<(Query<(&CapsuleB, &SpheroidCollider, &mut Transform)>,
//                       Query<(&PlayerMovement, &Transform)>,
//                       Query<(&CapsuleA, &SpheroidCollider, &Transform)>)>)
//{
//    if input.is_held(InputName::Run) {
//        // Get sweep start and velocity
//        let (start, velocity) = {
//            let player_query = set.p1();
//            let (player_movement, player_transform) = player_query.single();
//            let start = player_transform.pos + vec3(0.0, 1.7, 0.0);
//            let velocity = player_movement.forward() * 15.0;
//            (start, velocity)
//        };
//
//        {
//            let mut capsule_query = set.p0();
//            let (_, _, mut transform) = capsule_query.single_mut();
//            unsafe {
//                transform.pos = player_movement::FIRST_HIT_POS;
//                println!("positioning capsule at {:?}", transform.pos);
//            };
//        }
//
//        // Get capsule A pos and radius
//        //let (other_pos, other_radius) = {
//        //    let a_query = set.p2();
//        //    let (_, collider, transform) = a_query.single();
//        //    (transform.pos, collider.radius)
//        //};
//
//        // Collide spheroids
//        //{
//        //    let mut capsule_query = set.p0();
//        //    let (_, collider, mut transform) = capsule_query.single_mut();
//
//        //    let combined_radius = collider.radius() + other_radius;
//        //    let combined_cbm = vec3(1.0 / combined_radius.x, 1.0 / combined_radius.y, 1.0 / combined_radius.z);
//
//        //    let center = (start - vec3(0.0, 0.5 * collider.radius().y, 0.0)).mul_element_wise(combined_cbm);
//        //    let velocity = velocity.mul_element_wise(combined_cbm);
//        //    let point = other_pos.mul_element_wise(combined_cbm);
//
//        //    if let Some(res) = intersection::toi_unit_sphere_point(center, velocity, point) {
//        //        let hit_pos = (center + velocity * res).div_element_wise(combined_cbm);
//        //        transform.pos = hit_pos;
//        //    }
//        //}
//
//        // Collide capsule with world
//        //{
//        //    let mut capsule_query = set.p0();
//        //    let (_, collider, mut transform) = capsule_query.single_mut();
//
//        //    let center = (start - vec3(0.0, 0.5 * collider.radius().y, 0.0)).mul_element_wise(collider.cbm);
//        //    let velocity = velocity.mul_element_wise(collider.cbm);
//
//        //    if let Some(res) = collision.sweep_unit_sphere(&mut world, center, velocity, collider.cbm) {
//        //        let hit_pos = (center + velocity * res.toi()).div_element_wise(collider.cbm);
//        //        transform.pos = hit_pos;
//        //    };
//        //}
//    }
//}
