pub mod input;
pub mod level_collision;

use cgmath::{vec3, Vector3, Zero, InnerSpace};
use crate::renderer::camera::{FpsCamera, Camera};
use crate::renderer::gl_renderer::resources;

use self::input::{InputName, InputState};
use self::level_collision::LevelCollision;

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 1.0;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 0.1;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 0.5;

/// The gravity acceleration
const GRAVITY_ACCELERATION: f32 = 0.01;

/// The game state
pub struct GameState {
    pub time: f64,
    pub camera: FpsCamera,
    pub ball_pos: f32,
    pub level_collision: LevelCollision,
    pub velocity: Vector3<f32>
}

impl GameState {
    /// Create a new, default game state
    pub fn new() -> GameState {
        // Create camera
        let camera = FpsCamera::new_with_pos_rot(vec3(0.0, 10.0, 13.0), -0.17, 0.0, CAM_LOOK_SPEED);

        // Load level collision
        let level_collision = LevelCollision::new(resources::MODEL_DEMO_SCENE);

        GameState {
            time: 0.0,
            camera,
            ball_pos: 0.0,
            level_collision,
            velocity: Vector3::zero()
        }
    }

    /// Simulate the game state
    pub fn simulate(&mut self, sim_time: f64, input_state: &InputState) {
        // Update time
        let time_delta = sim_time - self.time;
        self.time = sim_time;

        // Update ball
        self.ball_pos += 0.02;

        // Simulate character
        self.simulate_character(input_state, time_delta as f32);
    }

    /// Get the movement input
    fn movement_input(&mut self, input_state: &InputState) -> (f32, f32) {
        let inputs = input_state.inputs;

        let cam_speed = match inputs[InputName::CamSpeed as usize] {
            false => CAM_MOVE_SPEED,
            true => CAM_MOVE_SPEED_FAST,
        };

        let cam_forwards = inputs[InputName::CamForwards as usize];
        let cam_backwards = inputs[InputName::CamBackwards as usize];
        let cam_left = inputs[InputName::CamLeft as usize];
        let cam_right = inputs[InputName::CamRight as usize];

        let forward_cam_movement = match (cam_forwards, cam_backwards) {
            (true, false) => cam_speed,
            (false, true) => -cam_speed,
            _ => 0.0
        };

        let right_cam_movement = match (cam_left, cam_right) {
            (true, false) => -cam_speed,
            (false, true) => cam_speed,
            _ => 0.0
        };

        (forward_cam_movement, right_cam_movement)
    }

    /// Simulate the character movement
    fn simulate_character(&mut self, input_state: &InputState, time_delta: f32) {
        /// The character camera height
        const CHAR_HEIGHT: f32 = 1.8;

        /// Minimum distance to stop before walls
        const MIN_DIST: f32 = 0.5;

        // Update look direction
        if input_state.cursor_captured {
            let (dx, dy) = input_state.mouse_diff;
            self.camera.mouse_move(dx as f32, dy as f32);
            self.camera.update_matrices();
        }

        // Get camera movement input
        let (forward_cam_movement, right_cam_movement) = self.movement_input(input_state);
        let cam_movement = forward_cam_movement * self.camera.forward() + right_cam_movement * self.camera.right();

        // Apply gravity to velocity
        let mut velocity_y = self.velocity.y - GRAVITY_ACCELERATION;
        let velocity_y_len = f32::abs(velocity_y);

        // Now solve the y movement and xz movement separately
        let mut pos = *self.camera.pos();

        // Do y first
        if velocity_y < 0.0 {
            let velocity_y_dir = vec3(0.0, -1.0, 0.0);

            let stop_dist = self.level_collision
                .raycast(&(pos + vec3(0.0, 1.0, 0.0)), &velocity_y_dir, velocity_y_len + 2.0)
                .map(|t| t - CHAR_HEIGHT)
                .filter(|t| *t < velocity_y_len);

            (pos, velocity_y) = match stop_dist {
                Some(t) => (pos + t * velocity_y_dir, 0.0),
                _ => (pos + vec3(0.0, velocity_y, 0.0), velocity_y)
            };
        }

        // Then xz
        let velocity_xz = vec3(cam_movement.x, 0.0, cam_movement.z);
        if velocity_xz.x != 0.0 || velocity_xz.y != 0.0 || velocity_xz.z != 0.0 {
            let velocity_xz_len = velocity_xz.magnitude();
            let velocity_xz_dir = velocity_xz / velocity_xz_len;

            let stop_dist = self.level_collision
                .raycast(&pos, &velocity_xz_dir, velocity_xz_len + 0.5) 
                .map(|t| t - MIN_DIST)
                .filter(|t| *t < velocity_xz_len);

            pos = match stop_dist {
                Some(t) => pos + t * velocity_xz_dir,
                _ => pos + velocity_xz
            };
        }

        // Accumulate y velocity
        self.velocity.y = velocity_y;

        // Update camera position
        self.camera.set_pos(&pos);
        self.camera.update();
    }
}
