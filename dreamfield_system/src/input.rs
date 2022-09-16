/// Input events
#[derive(Copy, Clone)]
pub enum InputEvent {
    CursorMoved(f64, f64),
    CursorCaptured(bool),
    GameInput(InputName, bool)
}

/// Input names
#[derive(Copy, Clone)]
pub enum InputName {
    CamForwards,
    CamBackwards,
    CamStrafeLeft,
    CamStrafeRight,
    CamLookUp,
    CamLookLeft,
    CamLookDown,
    CamLookRight,
    Run,
    Jump,
    Use,
    Debug,
    Last
}

/// The current input state
#[derive(Copy, Clone)]
pub struct InputState {
    pub inputs: [bool; InputName::Last as usize],
    pub last_inputs: [bool; InputName::Last as usize],
    pub cursor_captured: bool,
    pub mouse_diff: (f64, f64)
}

impl InputState {
    pub fn new() -> Self {
        Self {
            inputs: [false; InputName::Last as usize],
            last_inputs: [false; InputName::Last as usize],
            cursor_captured: false,
            mouse_diff: (0.0, 0.0)
        }
    }

    /// Get whether the input is held
    pub fn is_held(&self, name: InputName) -> bool {
        self.inputs[name as usize]
    }

    /// Get whether the input has just been released
    pub fn is_just_released(&self, name: InputName) -> bool {
        !self.inputs[name as usize] && self.last_inputs[name as usize]
    }

    /// Get whether the input has just been pressed
    pub fn is_just_pressed(&self, name: InputName) -> bool {
        self.inputs[name as usize] && !self.last_inputs[name as usize]
    }

    // Get the look input as a normalized float from 1 to -1. The first element is the left/right
    // look input, where positive is movement to the right, and the second element is up/down
    // movement, where positive is movement up.
    pub fn get_look_input(&self) -> (f32, f32) {
        let inputs = &self.inputs;

        let cam_look_up = inputs[InputName::CamLookUp as usize];
        let cam_look_down = inputs[InputName::CamLookDown as usize];
        let cam_look_left = inputs[InputName::CamLookLeft as usize];
        let cam_look_right = inputs[InputName::CamLookRight as usize];

        let cam_look_vertical = match (cam_look_up, cam_look_down) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0
        };

        let cam_look_horizontal = match (cam_look_left, cam_look_right) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0
        };

        (cam_look_horizontal, cam_look_vertical)
    }

    /// Get the movement input as a normalize float from 1 to -1. The first element is the
    /// forward/back movement where positive is forward, and the second element is left/right
    /// movement where positive is right.
    pub fn get_movement_input(&self) -> (f32, f32) {
        let inputs = self.inputs;

        let cam_forwards = inputs[InputName::CamForwards as usize];
        let cam_backwards = inputs[InputName::CamBackwards as usize];
        let cam_left = inputs[InputName::CamStrafeLeft as usize];
        let cam_right = inputs[InputName::CamStrafeRight as usize];

        let forward_cam_movement = match (cam_forwards, cam_backwards) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0
        };

        let right_cam_movement = match (cam_left, cam_right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0
        };

        (forward_cam_movement, right_cam_movement)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
