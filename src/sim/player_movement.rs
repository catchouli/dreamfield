use std::f32::consts::PI;

use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, Query};
use cgmath::{Vector3, vec3, InnerSpace, Vector2, Quaternion, Rad, Rotation3, Matrix4, SquareMatrix};

use super::level_collision::LevelCollision;
use dreamfield_renderer::components::PlayerCamera;
use dreamfield_system::resources::{SimTime, InputName, InputState};

/// The character height
const CHAR_HEIGHT: f32 = 1.7;

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

/// The world up vector
const WORLD_UP: Vector3<f32> = vec3(0.0, 1.0, 0.0);

/// The world right vector
const WORLD_RIGHT: Vector3<f32> = vec3(1.0, 0.0, 0.0);

/// The world forward vector
const WORLD_FORWARD: Vector3<f32> = vec3(0.0, 0.0, -1.0);

/// The PlayerMovement component
#[derive(Component)]
pub struct PlayerMovement {
    pub position: Vector3<f32>,
    pub pitch_yaw: Vector2<f32>,
    pub velocity: Vector3<f32>
}

impl PlayerMovement {
    pub fn new(position: Vector3<f32>, pitch_yaw: Vector2<f32>) -> Self {
        PlayerMovement {
            position,
            pitch_yaw,
            velocity: vec3(0.0, 0.0, 0.0)
        }
    }

    // TODO: could cache these
    pub fn orientation(&self) -> Quaternion<f32> {
        let pitch = Quaternion::from_axis_angle(WORLD_RIGHT, Rad(self.pitch_yaw.x));
        let yaw = Quaternion::from_axis_angle(WORLD_UP, Rad(self.pitch_yaw.y));
        yaw * pitch
    }

    pub fn forward(&self) -> Vector3<f32> {
        self.orientation() * WORLD_FORWARD
    }

    pub fn right(&self) -> Vector3<f32> {
        self.orientation() * WORLD_RIGHT
    }
}

/// The player update system
pub fn player_update(level_collision: Res<LevelCollision>, input_state: Res<InputState>, sim_time: Res<SimTime>,
    mut query: Query<(&mut PlayerCamera, &mut PlayerMovement)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    for (mut cam, mut player_movement) in query.iter_mut() {
        // Update look direction (buttons)
        let (cam_look_horizontal, cam_look_vertical) = input_state.get_look_input();

        let cam_look_speed = match input_state.inputs[InputName::CamSpeed as usize] {
            false => CAM_LOOK_SPEED,
            true => CAM_LOOK_SPEED_FAST,
        };

        let pitch_yaw = &mut player_movement.pitch_yaw;
        pitch_yaw.x += cam_look_vertical * cam_look_speed * time_delta;
        pitch_yaw.y += cam_look_horizontal * cam_look_speed * time_delta;

        // Get camera movement input
        let (forward_cam_movement, right_cam_movement) = input_state.get_movement_input();

        let cam_speed = match input_state.inputs[InputName::CamSpeed as usize] {
            false => CAM_MOVE_SPEED,
            true => CAM_MOVE_SPEED_FAST,
        };

        let cam_movement = forward_cam_movement * cam_speed * player_movement.forward()
            + right_cam_movement * cam_speed * player_movement.right();

        // Update velocity with cam movement and gravity
        player_movement.velocity.x = cam_movement.x;
        player_movement.velocity.z = cam_movement.z;
        player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;

        // Now solve the y movement and xz movement separately
        let mut pos = player_movement.position + vec3(0.0, CHAR_HEIGHT, 0.0);

        // Resolve horizontal motion
        let mut movement = time_delta * vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z);
        for _ in 0..2 {
            if movement.x != 0.0 || movement.y != 0.0 || movement.z != 0.0 {
                movement = resolve_horizontal_movement(level_collision.as_ref(), &pos, &movement);
            }
        }
        pos += movement;

        // Resolve vertical motion
        if player_movement.velocity.y < 0.0 {
            let movement_y = player_movement.velocity.y * time_delta;
            (pos, player_movement.velocity.y) = resolve_vertical_movement(level_collision.as_ref(), &pos,
                &player_movement.velocity, &movement_y);
        }

        // Bump out of wall
        const BUMP_STEPS: i32 = 4;
        const BUMP_RADIUS: f32 = 0.5;
        for i in 0..BUMP_STEPS {
            let angle = (i as f32 / BUMP_STEPS as f32) * 2.0 * PI;

            let x_offset = f32::sin(angle);
            let z_offset = f32::cos(angle);

            let collider_dir = vec3(x_offset, 0.0, z_offset);

            if let Some(hit) = level_collision.raycast_normal(&pos, &collider_dir, BUMP_RADIUS) {
                pos = pos + (hit.toi - BUMP_RADIUS) * collider_dir;
            }
        }

        // Update position
        player_movement.position = pos - vec3(0.0, CHAR_HEIGHT, 0.0);

        // Update camera position
        update_camera(&player_movement, &mut cam);
    }
}

fn resolve_vertical_movement(level_collision: &LevelCollision, pos: &Vector3<f32>, vel: &Vector3<f32>, movement_y: &f32)
    -> (Vector3<f32>, f32)
{
    let movement_y_len = f32::abs(*movement_y);
    let movement_y_dir = vec3(0.0, -1.0, 0.0);

    let stop_dist = level_collision
        .raycast(pos, &movement_y_dir, movement_y_len + CHAR_HEIGHT)
        .map(|toi| toi - CHAR_HEIGHT);

    match stop_dist {
        Some(toi) => {
            (pos + toi * movement_y_dir, 0.0)
        },
        _ => {
            (pos + vec3(0.0, *movement_y, 0.0), vel.y)
        }
    }
}

fn resolve_horizontal_movement(level_collision: &LevelCollision, pos: &Vector3<f32>, movement: &Vector3<f32>)
    -> Vector3<f32>
{
    let movement_len = movement.magnitude();
    let movement_dir = movement / movement_len;

    let ray_start = pos;
    let ray_dist = movement_len;

    match level_collision.raycast_normal(&ray_start, &movement_dir, ray_dist) {
        Some(ray_hit) => {
            let hit_normal = vec3(ray_hit.normal.x, ray_hit.normal.y, ray_hit.normal.z);

            let movement_to_wall = ray_hit.toi * movement_dir * 0.99;
            let remaining_movement = movement - movement_to_wall;
            let subtracted_movement = hit_normal * remaining_movement.dot(hit_normal);

            movement_to_wall + remaining_movement - subtracted_movement
        },
        _ => {
            *movement
        }
    }
}

fn update_camera(player_movement: &PlayerMovement, player_camera: &mut PlayerCamera) {
    let cam_pos = player_movement.position + vec3(0.0, CHAR_HEIGHT, 0.0);
    let cam_transform = Matrix4::from_translation(cam_pos) * Matrix4::from(player_movement.orientation());
    player_camera.view = cam_transform.invert().unwrap();
}
