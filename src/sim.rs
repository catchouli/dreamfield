pub mod input;

use std::collections::VecDeque;

use cgmath::vec3;
use crate::renderer::camera::{FpsCamera, Camera};

use self::input::{InputEvent, InputName};

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
    pub input_state: InputState,
    pub camera: FpsCamera
}

#[derive(Copy, Clone)]
pub struct InputState {
    cursor_captured: bool,
    inputs: [bool; 25]
}

impl GameState {
    /// Create a new, default game state
    pub fn new() -> GameState {
        // Create camera
        let camera = FpsCamera::new_with_pos_rot(vec3(0.0, 0.0, 10.0), 0.0, 0.0, CAM_LOOK_SPEED);

        // Create input state
        let input_state = InputState {
            cursor_captured: false,
            inputs: [false; 25]
        };

        GameState {
            time: 0.0,
            input_state,
            camera
        }
    }

    pub fn simulate(&self, sim_time: f64, input_events: &mut VecDeque<InputEvent>) -> GameState
    {
        // Create new game state
        let mut new_state = self.clone();
        new_state.time = sim_time;

        // Handle input events
        while let Some(event) = input_events.pop_back() {
            self.handle_input_event(&mut new_state, event);
        }

        // Update camera
        if self.input_state.cursor_captured {
            let inputs = &self.input_state.inputs;

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

            new_state.camera.move_camera(forward_cam_movement, right_cam_movement, 0.0);
            new_state.camera.update();
        }

        new_state
    }

    fn handle_input_event(&self, new_state: &mut GameState, event: InputEvent) {
        match event {
            InputEvent::CursorMoved(dx, dy) => {
                if self.input_state.cursor_captured {
                    new_state.camera.mouse_move(dx as f32, dy as f32);
                }
            }
            InputEvent::CursorCaptured(captured) => {
                new_state.input_state.cursor_captured = captured;
            }
            InputEvent::GameInput(input_name, is_down) => {
                new_state.input_state.inputs[input_name as usize] = is_down;
            }
        }
    }
}
