pub mod input;
pub mod level_collision;

use cgmath::{vec3, Vector3, Zero, InnerSpace};
use dreamfield_renderer::camera::{FpsCamera, Camera};
use crate::renderer::resources;

use self::input::{InputName, InputState};
use self::level_collision::LevelCollision;

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 0.5;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 4.0;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 50.0;

/// The gravity acceleration
const GRAVITY_ACCELERATION: f32 = 9.8;

/// The game state
pub struct GameState {
    pub time: f64,
    pub camera: FpsCamera,
    pub ball_pos: Vector3<f32>,
    pub level_collision: LevelCollision,
    pub velocity: Vector3<f32>
}

impl GameState {
    /// Create a new, default game state
    pub fn new() -> GameState {
        // Create camera looking at corridor
        let camera = FpsCamera::new_with_pos_rot(vec3(5.2, 0.8, 12.8), 0.03, 2.0, 0.0);
        // Default pos
        //let camera = FpsCamera::new_with_pos_rot(vec3(0.0, 1.0, 10.0), -0.17, 0.0, 0.0);
        // Going outside
        //let camera = FpsCamera::new_with_pos_rot(vec3(-53.925, 5.8, 19.56), 0.097, 1.57, 0.0);

        // Load level collision
        let level_collision = LevelCollision::new(resources::MODEL_DEMO_SCENE);

        GameState {
            time: 0.0,
            camera,
            ball_pos: vec3(-9.0, 0.0, 9.0),
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
        self.ball_pos.y = self.time.sin() as f32 + 2.0;

        // Simulate character
        self.simulate_character(input_state, time_delta as f32);
    }

    /// Get the movement input
    fn movement_input(&self, input_state: &InputState) -> (f32, f32) {
        let inputs = input_state.inputs;

        let cam_speed = match inputs[InputName::CamSpeed as usize] {
            false => CAM_MOVE_SPEED,
            true => CAM_MOVE_SPEED_FAST,
        };

        let cam_forwards = inputs[InputName::CamForwards as usize];
        let cam_backwards = inputs[InputName::CamBackwards as usize];
        let cam_left = inputs[InputName::CamStrafeLeft as usize];
        let cam_right = inputs[InputName::CamStrafeRight as usize];

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

    /// Get the look input
    fn look_input(&self, input_state: &InputState) -> (f32, f32) {
        let inputs = input_state.inputs;

        let cam_look_up = inputs[InputName::CamLookUp as usize];
        let cam_look_down = inputs[InputName::CamLookDown as usize];
        let cam_look_left = inputs[InputName::CamLookLeft as usize];
        let cam_look_right = inputs[InputName::CamLookRight as usize];

        let cam_look_vertical = match (cam_look_up, cam_look_down) {
            (true, false) => CAM_LOOK_SPEED,
            (false, true) => -CAM_LOOK_SPEED,
            _ => 0.0
        };

        let cam_look_horizontal = match (cam_look_left, cam_look_right) {
            (true, false) => CAM_LOOK_SPEED,
            (false, true) => -CAM_LOOK_SPEED,
            _ => 0.0
        };

        (cam_look_horizontal, cam_look_vertical)
    }

    /// Simulate the character movement
    fn simulate_character(&mut self, input_state: &InputState, time_delta: f32) {
        /// The character camera height
        const CHAR_HEIGHT: f32 = 1.8;

        // Update look direction (mouse)
        if input_state.cursor_captured {
            let (dx, dy) = input_state.mouse_diff;
            self.camera.mouse_move(dx as f32, dy as f32);
        }

        // Update look direction (buttons)
        let (cam_look_horizontal, cam_look_vertical) = self.look_input(input_state);
        let (mut pitch, mut yaw) = self.camera.get_pitch_yaw();
        pitch += cam_look_vertical * time_delta;
        yaw += cam_look_horizontal * time_delta;
        self.camera.set_pitch_yaw(pitch, yaw);

        // Get camera movement input
        let (forward_cam_movement, right_cam_movement) = self.movement_input(input_state);
        let cam_movement = forward_cam_movement * self.camera.forward() + right_cam_movement * self.camera.right();

        // Update velocity with cam movement and gravity
        self.velocity.x = cam_movement.x;
        self.velocity.z = cam_movement.z;
        self.velocity.y -= GRAVITY_ACCELERATION * time_delta;

        // Now solve the y movement and xz movement separately
        let mut pos = *self.camera.pos();

        // Resolve vertical motion
        if self.velocity.y < 0.0 {
            let movement_y = self.velocity.y * time_delta;
            let movement_y_len = f32::abs(movement_y);
            let movement_y_dir = vec3(0.0, -1.0, 0.0);

            let stop_dist = self.level_collision
                .raycast(&(pos + vec3(0.0, 1.0, 0.0)), &movement_y_dir, movement_y_len + 10.0)
                .map(|t| t - CHAR_HEIGHT)
                .filter(|t| *t < movement_y_len);

            (pos, self.velocity.y) = match stop_dist {
                Some(t) => (pos + t * movement_y_dir, 0.0),
                _ => (pos + vec3(0.0, movement_y, 0.0), self.velocity.y)
            };
        }

        // Resolve horizontal motion
        let mut movement = time_delta * vec3(self.velocity.x, 0.0, self.velocity.z);
        for _ in 0..2 {
            if movement.x != 0.0 || movement.y != 0.0 || movement.z != 0.0 {
                movement = self.resolve_movement(&pos, &movement);
            }
        }
        pos += movement;

        // Update camera position
        self.camera.set_pos(&pos);
        self.camera.update();
    }

    fn resolve_movement(&self, pos: &Vector3<f32>, movement: &Vector3<f32>) -> Vector3<f32> {
        let movement_len = movement.magnitude();
        let movement_dir = movement / movement_len;

        let ray_start = pos;
        let ray_dist = movement_len;

        match self.level_collision.raycast_normal(&ray_start, &movement_dir, ray_dist) {
            Some(ray_hit) => {
                let hit_normal = vec3(ray_hit.normal.x, ray_hit.normal.y, ray_hit.normal.z);
                let hit_dot = hit_normal.dot(*movement);
                movement - (hit_dot * hit_normal)
            },
            _ => {
                *movement
            }
        }
    }
}
