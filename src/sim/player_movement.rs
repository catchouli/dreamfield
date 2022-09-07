use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, InnerSpace};
use dreamfield_system::world::WorldChunkManager;

use super::TestSphere;
use super::level_collision::LevelCollision;
use dreamfield_renderer::components::{PlayerCamera, Camera, Position};
use dreamfield_system::resources::{SimTime, InputName, InputState};

/// The character height
const CHAR_HEIGHT: f32 = 1.8;

/// The character's collider radius
const COLLIDER_RADIUS: f32 = 0.5;

/// Height that we can step up onto objects
const STEP_OFFSET: f32 = 0.2;

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 0.5;

/// The camera fast look speed
const CAM_LOOK_SPEED_FAST: f32 = 1.5;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 4.0;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 12.0;

/// The gravity acceleration
const GRAVITY_ACCELERATION: f32 = 9.8;

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
    input_state: Res<InputState>, sim_time: Res<SimTime>, mut query: Query<(&mut PlayerCamera, &mut PlayerMovement)>,
    mut test_sphere: Query<(&TestSphere, &mut Position)>)
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
        let bump = false;
        let vert = false;
        let horz = false;

        // Bump out of walls
        if bump {
            //player_movement.position = bump_out_of_walls(level_collision.as_mut(), world.as_mut(), &player_movement);
            player_movement.position = nudge_position(level_collision.as_mut(), world.as_mut(), &player_movement);
        }

        // Apply horizontal movement
        if horz {
            player_movement.position = apply_horizontal_motion(level_collision.as_mut(), world.as_mut(), &player_movement, time_delta);
        }

        // Apply vertical movement
        if vert {
            let on_ground;
            (player_movement.position, on_ground) = apply_vertical_motion(level_collision.as_mut(), world.as_mut(), &player_movement, time_delta);

            // Cancel out vertical velocity if we're on the ground
            if on_ground && player_movement.velocity.y < 0.0 {
                player_movement.velocity.y = 0.0;
            }
        }

        // Update camera position
        let feet_pos = &player_movement.position;
        let cam_pos = vec3(feet_pos.x, feet_pos.y + CHAR_EYE_LEVEL, feet_pos.z);
        camera.set_pos(&cam_pos);
        camera.update();

        // Debug spherecast
        let mut vel = player_movement.velocity.clone();
        vel.y = 0.0;
        player_movement.position += vel * time_delta;
        if input_state.inputs[InputName::Rewind as usize] || true {
            let (_, mut pos) = test_sphere.single_mut();

            //let start = cam_pos;
            //let dir = *camera.forward();

            let start = vec3(-51.240383, 6.7, 17.681177);
            let dir = vec3(0.89094263, -0.45288646, -0.033392847);

            let res = level_collision.sweep_sphere(world.as_mut(), &start, &dir, 10.0, 0.5);
            if let Some(res) = res {
                pos.pos = start + dir * res.toi();
            }
            else {
                pos.pos = vec3(9.0, 0.0, -9.0);
            }
            //println!("firing test spherecast: {start:?}, dir: {dir:?}");
            //println!("camera pos: {:?}, dir: {:?}", camera.pos(), camera.get_pitch_yaw());
        }
    }
}

fn nudge_position(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement) -> Vector3<f32>
{
    const BUMP_SIZE: f32 = 1.0 / 32.0;
    const SIGN: [i32; 3] = [0, -1, 1];

    let base = player_movement.position;

    for z in 0..3 {
        for x in 0..3 {
            for y in 0..3 {
                let pos = vec3(
                    base.x + (SIGN[x] as f32) * BUMP_SIZE,
                    base.y + (SIGN[y] as f32) * BUMP_SIZE,
                    base.z + (SIGN[z] as f32) * BUMP_SIZE
                );

                let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);
                if level_collision.sphere_contact_any(world, &collider_pos, COLLIDER_RADIUS).is_none() {
                    return pos;
                }
                else {
                    //println!("in wall, nudging");
                }
            }
        }
    }

    base
}

/// Apply the horizontal movement, returning an updated position
fn apply_horizontal_motion(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement, time_delta: f32) -> Vector3<f32>
{
    const HORIZONTAL_ITERATIONS: i32 = 2;

    // Construct horizontal movement vector
    let pos = &player_movement.position;
    let mut movement = time_delta * vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z);

    // Apply a few iterations of this, as the first slide may result in a movement vector that
    // slides us through a wall, requiring us to test again
    for _ in 0..HORIZONTAL_ITERATIONS {
        if movement.x != 0.0 || movement.y != 0.0 || movement.z != 0.0 {
            movement = resolve_horizontal_movement(level_collision, world, pos, &movement);
        }

        if movement.y < 0.0 {
            movement.y = 0.0;
        }
    }

    pos + movement
}

fn resolve_horizontal_movement(level_collision: &mut LevelCollision, world: &mut WorldChunkManager, pos: &Vector3<f32>,
    movement: &Vector3<f32>) -> Vector3<f32>
{
    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);

    let move_dist = movement.magnitude();
    let move_dir = movement / move_dist;

    let res = level_collision.sweep_sphere(world, &collider_pos, &move_dir, move_dist, COLLIDER_RADIUS);
    if let Some(hit) = res {
        let movement_to_wall = hit.toi() * move_dir * 0.99;
        let remaining_movement = (movement - movement_to_wall) * 0.99;
        let subtracted_movement = hit.normal() * remaining_movement.dot(*hit.normal());

        movement_to_wall + remaining_movement - subtracted_movement
    }
    else {
        *movement
    }
}

/// Apply the vertical movement, returning an updated position, and whether we're on the ground
fn apply_vertical_motion(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement, time_delta: f32) -> (Vector3<f32>, bool)
{
    let pos = &player_movement.position;
    let movement_y = player_movement.velocity.y * time_delta;
    let (movement, on_ground) = resolve_vertical_movement(level_collision, world, pos, movement_y);
    (pos + movement, on_ground)
}

/// Resolve the vertical movement, returns the resolved movement vector and whether the collider is
/// now on the ground.
fn resolve_vertical_movement(level_collision: &mut LevelCollision, world: &mut WorldChunkManager, pos: &Vector3<f32>,
    movement_y: f32) -> (Vector3<f32>, bool)
{
    let movement = vec3(0.0, movement_y, 0.0);
    let move_dist = movement.magnitude();
    let move_dir = movement / move_dist;

    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);

    if let Some(hit) = level_collision.sweep_sphere(world, &collider_pos, &move_dir, move_dist, COLLIDER_RADIUS) {
        (hit.toi() * move_dir, true)
    }
    else {
        (movement, false)
    }
}


/// "Bump" out of walls as a backup for if the current position is obstructed, returning an updated
/// position
fn bump_out_of_walls(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement) -> Vector3<f32>
{
    let feet_pos = &player_movement.position;
    let collider_pos = vec3(feet_pos.x, feet_pos.y + COLLIDER_RADIUS, feet_pos.z);

    level_collision.sphere_contact_any(world, &collider_pos, COLLIDER_RADIUS)
        .filter(|contact| contact.depth > 0.0)
        .map(|contact| {
            feet_pos + contact.depth * vec3(contact.normal.x, contact.normal.y, contact.normal.z)
        })
        .unwrap_or(*feet_pos)
}
