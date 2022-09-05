use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, InnerSpace};
use dreamfield_system::world::WorldChunkManager;

use super::level_collision::LevelCollision;
use dreamfield_renderer::components::{PlayerCamera, Camera};
use dreamfield_system::resources::{SimTime, InputName, InputState};

/// The character height
const CHAR_HEIGHT: f32 = 1.8;

/// The character's collider radius
const COLLIDER_RADIUS: f32 = 0.5;

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 0.5;

/// The camera fast look speed
const CAM_LOOK_SPEED_FAST: f32 = 1.5;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 4.0;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 12.0;

/// The gravity acceleration
//const GRAVITY_ACCELERATION: f32 = 9.8;
const GRAVITY_ACCELERATION: f32 = 0.0;

/// The character eye level. According to a cursory google, this is on average 4.5" below the top
/// of your head, which is just over 10cm
const CHAR_EYE_LEVEL: f32 = CHAR_HEIGHT - 0.10;

/// The PlayerMovement component
#[derive(Component)]
pub struct PlayerMovement {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>
}

impl PlayerMovement {
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>) -> Self {
        PlayerMovement {
            position,
            velocity
        }
    }
}

/// The player update system
pub fn player_update(mut level_collision: ResMut<LevelCollision>, mut world: ResMut<WorldChunkManager>,
    input_state: Res<InputState>, sim_time: Res<SimTime>, mut query: Query<(&mut PlayerCamera, &mut PlayerMovement)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    for (mut cam, mut player_movement) in query.iter_mut() {
        let camera = &mut cam.camera;

        // Update look direction (buttons)
        let (cam_look_horizontal, cam_look_vertical) = input_state.get_look_input();

        let cam_look_speed = match input_state.inputs[InputName::CamSpeed as usize] {
            false => CAM_LOOK_SPEED,
            true => CAM_LOOK_SPEED_FAST,
        };

        let (mut pitch, mut yaw) = camera.get_pitch_yaw();
        pitch += cam_look_vertical * cam_look_speed * time_delta;
        yaw += cam_look_horizontal * cam_look_speed * time_delta;
        camera.set_pitch_yaw(pitch, yaw);

        // Get camera movement input
        let (forward_cam_movement, right_cam_movement) = input_state.get_movement_input();

        let cam_speed = match input_state.inputs[InputName::CamSpeed as usize] {
            false => CAM_MOVE_SPEED,
            true => CAM_MOVE_SPEED_FAST,
        };

        let cam_movement = forward_cam_movement * cam_speed * camera.forward()
            + right_cam_movement * cam_speed * camera.right();

        // Update velocity with cam movement and gravity
        player_movement.velocity.x = cam_movement.x;
        player_movement.velocity.z = cam_movement.z;
        player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;

        // Now solve the y movement and xz movement separately
        let mut pos = player_movement.position;

        // Print the camera position
        log::trace!("Camera position: {}, {}, {}; cam rot: {}, {}", pos.x, pos.y, pos.z, pitch, yaw);

        // Bump out of walls as a backup
        //let collider_pos = vec3(pos.x, pos.y + COLLIDER_RADIUS, pos.z);
        //if let Some(contact) = level_collision.sphere_contact_any(&mut world, &collider_pos, COLLIDER_RADIUS, None) {
        //    if contact.depth > 0.0 {
        //        println!("bumping out of wall, contact depth {}", contact.depth);
        //        let normal = vec3(contact.normal.x, contact.normal.y, contact.normal.z);
        //        pos += normal * contact.depth;
        //    }
        //}

        // Resolve horizontal motion
        let mut movement = time_delta * vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z);
        //println!("\nresolving horizontal motion: {movement:?}");
        for i in 0..3 {
            movement.y = 0.0;
            movement = resolve_horizontal_movement(level_collision.as_mut(), world.as_mut(), &pos, &movement);
            //println!("resolved {i}: {movement:?}");
        }
        // Zero y motion or we can be redirected downwards as a result of collisions
        pos += vec3(movement.x, 0.0, movement.z);

        // Resolve vertical motion
        let movement_y = player_movement.velocity.y * time_delta;
        let (movement, on_ground) = resolve_vertical_movement(level_collision.as_mut(), world.as_mut(),
            &pos, movement_y);

        pos += movement;

        if on_ground {
            //println!("on ground, resetting velocity to 0");
            player_movement.velocity.y = 0.0;
        }

        // Update camera position
        player_movement.position = pos;

        let cam_pos = vec3(pos.x, pos.y + CHAR_EYE_LEVEL, pos.z);
        camera.set_pos(&cam_pos);
        camera.update();
    }
}

/// Resolve the vertical movement, returns the resolved movement vector and whether the collider is
/// now on the ground.
fn resolve_vertical_movement(level_collision: &mut LevelCollision, world: &mut WorldChunkManager, pos: &Vector3<f32>,
    movement_y: f32) -> (Vector3<f32>, bool)
{
    let movement = vec3(0.0, movement_y, 0.0);

    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);
    let target_pos = collider_pos + movement;

    if let Some(hit) = level_collision.spherecast(world, &collider_pos, &target_pos, COLLIDER_RADIUS) {
        //println!("collided with ground");
        (hit.toi() * vec3(0.0, -1.0, 0.0), true)
    }
    else {
        (movement, false)
    }
}

fn resolve_horizontal_movement(level_collision: &mut LevelCollision, world: &mut WorldChunkManager, pos: &Vector3<f32>,
    movement: &Vector3<f32>) -> Vector3<f32>
{
    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);
    let target_pos = collider_pos + movement;

    let movement_dir = (target_pos - collider_pos).normalize();

    if let Some(hit) = level_collision.spherecast(world, &collider_pos, &target_pos, COLLIDER_RADIUS) {
        let dot = hit.normal().dot(movement_dir);
        //println!("collision dot: {dot}, toi: {}, normal: {:?}", hit.toi(), hit.normal());

        let movement_to_wall = hit.toi() * movement_dir;// + 0.01 * hit.normal();
        let remaining_movement = movement - movement_to_wall;
        let subtracted_movement = hit.normal() * remaining_movement.dot(*hit.normal());

        movement_to_wall + remaining_movement - subtracted_movement
        //let wall_dir_movement = movement * movement.dot(*hit.normal());
        //movement - wall_dir_movement
    }
    else {
        //println!("spherecast didnt hit");
        *movement
    }
}
