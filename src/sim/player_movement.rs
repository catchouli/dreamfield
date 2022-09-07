use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, InnerSpace};
use dreamfield_system::world::WorldChunkManager;

use crate::sim::intersection::Plane;

use super::TestSphere;
use super::level_collision::LevelCollision;
use dreamfield_renderer::components::{PlayerCamera, Camera, Position};
use dreamfield_system::resources::{SimTime, InputName, InputState};

/// The character height
const CHAR_HEIGHT: f32 = 1.8;

/// The character's collider radius
const COLLIDER_RADIUS: f32 = 0.5;

/// Height that we can step up onto objects
const _STEP_OFFSET: f32 = 0.2;

/// The maximum angle for a face to be considered the floor
const MAX_FLOOR_ANGLE: f32 = 30.0;

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
    pub velocity: Vector3<f32>,
    pub ground_normal: Option<Vector3<f32>>
}

impl PlayerMovement {
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>) -> Self {
        PlayerMovement {
            position,
            velocity,
            ground_normal: None
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
        let bump = true;
        let vert = true;
        let horz = true;

        // Bump out of walls
        if bump {
            //player_movement.position = bump_out_of_walls(level_collision.as_mut(), world.as_mut(), &player_movement);
            player_movement.position = nudge_position(level_collision.as_mut(), world.as_mut(), &player_movement);
        }

        // Update grounded state
        player_movement.ground_normal = get_ground_normal(level_collision.as_mut(), world.as_mut(), &player_movement);

        // Cancel out vertical velocity if we're on the ground
        if player_movement.ground_normal.is_some() && player_movement.velocity.y < 0.0 {
            player_movement.velocity.y = 0.0;
        }

        // Apply horizontal movement
        if horz {
            player_movement.position = apply_horizontal_motion(level_collision.as_mut(), world.as_mut(),
                &player_movement, time_delta);
        }

        // Apply vertical movement
        if vert {
            player_movement.position = apply_vertical_motion(level_collision.as_mut(), world.as_mut(),
                &player_movement, time_delta);
        }

        // Update camera position
        let feet_pos = &player_movement.position;
        let cam_pos = vec3(feet_pos.x, feet_pos.y + CHAR_EYE_LEVEL, feet_pos.z);
        camera.set_pos(&cam_pos);
        camera.update();

        // Debug spherecast
        if input_state.inputs[InputName::Rewind as usize] {//|| true {
            let (_, mut pos) = test_sphere.single_mut();

            let start = cam_pos;
            let dir = *camera.forward();

            let res = level_collision.sweep_sphere(world.as_mut(), &start, &dir, 10.0, 0.5);
            if let Some(res) = res {
                pos.pos = start + dir * res.toi();
            }
            else {
                pos.pos = vec3(9.0, 0.0, -9.0);
            }
        }
    }
}

fn nudge_position(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement) -> Vector3<f32>
{
    const INITIAL_BUMP_SIZE: f32 = 1.0 / 64.0;
    const MAX_BUMP_STEPS: i32 = 5;
    const SIGN: [i32; 3] = [0, -1, 1];

    let base = player_movement.position;

    let mut depth = INITIAL_BUMP_SIZE;
    for _ in 0..MAX_BUMP_STEPS {
        depth *= 2.0;

        for z in 0..3 {
            for x in 0..3 {
                for y in 0..3 {
                    let pos = vec3(
                        base.x + (SIGN[x] as f32) * depth,
                        base.y + (SIGN[y] as f32) * depth,
                        base.z + (SIGN[z] as f32) * depth
                        );

                    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);
                    if level_collision.sphere_contact_any(world, &collider_pos, COLLIDER_RADIUS).is_none() {
                        return pos;
                    }
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
    const HORIZONTAL_ITERATIONS: i32 = 3;

    // Construct horizontal movement vector
    let pos = player_movement.position;
    let mut target_pos = pos + vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z) * time_delta;

    //let mut movement = vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z) * time_delta;
    if let Some(ground_normal) = player_movement.ground_normal {
        let plane = Plane::new_from_point_and_normal(pos, ground_normal);
        target_pos = plane.project_point(target_pos);
    }

    // Apply a few iterations of this, as the first slide may result in a movement vector that
    // slides us through a wall, requiring us to test again
    let mut movement = target_pos - pos;
    for _ in 0..HORIZONTAL_ITERATIONS {
        if movement.x != 0.0 || movement.y != 0.0 || movement.z != 0.0 {
            let allow_vert = player_movement.ground_normal.is_some();
            movement = resolve_horizontal_movement(level_collision, world, &pos, &movement, allow_vert);
        }
    }

    pos + movement
}

fn resolve_horizontal_movement(level_collision: &mut LevelCollision, world: &mut WorldChunkManager, pos: &Vector3<f32>,
    movement: &Vector3<f32>, allow_vert: bool) -> Vector3<f32>
{
    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);

    let move_dist = movement.magnitude();
    let move_dir = movement / move_dist;

    let res = level_collision.sweep_sphere(world, &collider_pos, &move_dir, move_dist, COLLIDER_RADIUS);
    if let Some(hit) = res {
        let movement_to_wall = hit.toi() * move_dir;

        let remaining_movement = movement - movement_to_wall;

        let mut normal = *hit.normal();
        if !allow_vert {
            normal.y = 0.0;
            normal = normal.normalize();
        }

        let subtracted_movement = normal * remaining_movement.dot(normal);
        let slide = remaining_movement - subtracted_movement;
        movement_to_wall + slide
    }
    else {
        *movement
    }
}

fn get_ground_normal(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement) -> Option<Vector3<f32>>
{
    const CAST_DIST: f32 = 0.1;

    let start = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    let dir = vec3(0.0, -1.0, 0.0);

    level_collision.sweep_sphere(world, &start, &dir, CAST_DIST, COLLIDER_RADIUS)
        .filter(|hit| is_valid_floor_normal(*hit.normal()))
        .map(|hit| hit.normal().map(|n| n.clone()))
}

/// Apply the vertical movement, returning an updated position, and whether we're on the ground
fn apply_vertical_motion(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
    player_movement: &PlayerMovement, time_delta: f32) -> Vector3<f32>
{
    let pos = &player_movement.position;
    let movement_y = player_movement.velocity.y * time_delta;
    let movement = resolve_vertical_movement(level_collision, world, pos, movement_y);
    pos + movement
}

/// Resolve the vertical movement, returns the resolved movement vector and whether the collider is
/// now on the ground.
fn resolve_vertical_movement(level_collision: &mut LevelCollision, world: &mut WorldChunkManager, pos: &Vector3<f32>,
    movement_y: f32) -> Vector3<f32>
{
    let movement = vec3(0.0, movement_y, 0.0);
    let move_dist = movement.magnitude();
    let move_dir = movement / move_dist;

    let collider_pos = pos + vec3(0.0, COLLIDER_RADIUS, 0.0);

    if let Some(hit) = level_collision.sweep_sphere(world, &collider_pos, &move_dir, move_dist, COLLIDER_RADIUS) {
        let move_to_floor = hit.toi() * move_dir;

        let remaining_movement = movement - move_to_floor;
        let subtracted_movement = hit.normal() * remaining_movement.dot(*hit.normal());

        let slide = remaining_movement - subtracted_movement;

        move_to_floor + slide
    }
    else {
        movement
    }
}


/// "Bump" out of walls as a backup for if the current position is obstructed, returning an updated
/// position
fn _bump_out_of_walls(level_collision: &mut LevelCollision, world: &mut WorldChunkManager,
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

fn is_valid_floor_normal(normal: Vector3<f32>) -> bool {
    let dot = normal.dot(vec3(0.0, -1.0, 0.0));
    let angle = f32::acos(f32::abs(dot)) * 180.0 / std::f32::consts::PI;
    angle < MAX_FLOOR_ANGLE
}
