pub mod input;

use cgmath::vec3;
use crate::renderer::camera::{FpsCamera, Camera};

use self::input::{InputName, InputState};
use super::rewindable_game_state::SnapshotState;

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 1.0;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 0.1;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 0.5;

/// The game state
#[derive(Copy, Clone)]
pub struct GameState {
    pub time: f64,
    pub camera: FpsCamera,
    pub ball_pos: f32
}

impl GameState {
    /// Create a new, default game state
    pub fn new() -> GameState {
        // Create camera
        let camera = FpsCamera::new_with_pos_rot(vec3(0.0, 1.0, 10.0), 0.0, 0.0, CAM_LOOK_SPEED);

        GameState {
            time: 0.0,
            camera,
            ball_pos: 0.0
        }
    }

    /// Simulate the game state
    pub fn simulate(&mut self, sim_time: f64, input_state: &InputState) {
        // Update time
        self.time = sim_time;

        // Update ball
        self.ball_pos += 0.02;

        // Simulate camera
        if input_state.cursor_captured {
            self.simulate_camera(input_state);
        }
    }

    /// Simulate the camera
    fn simulate_camera(&mut self, input_state: &InputState) {
        // Update camera
        let inputs = input_state.inputs;

        if input_state.cursor_captured {
            let (dx, dy) = input_state.mouse_diff;
            self.camera.mouse_move(dx as f32, dy as f32);
            self.camera.update_matrices();
        }

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

        self.camera.move_camera(forward_cam_movement, right_cam_movement, 0.0);
        self.camera.update();
    }
}

impl SnapshotState<GameState> for GameState {
    fn new() -> Self {
        Self::new()
    }

    fn snapshot(&self) -> Self {
        *self
    }

    fn simulate(&mut self, sim_time: f64, input_state: &InputState) {
        self.simulate(sim_time, input_state)
    }
}
