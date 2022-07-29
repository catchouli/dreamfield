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
    CamSpeed,
    Rewind,
    Last
}

/// The input state
#[derive(Copy, Clone)]
pub struct InputState {
    pub cursor_captured: bool,
    pub inputs: [bool; InputName::Last as usize],
    pub mouse_diff: (f64, f64)
}

impl InputState {
    pub fn new() -> Self {
        Self {
            cursor_captured: false,
            inputs: [false; InputName::Last as usize],
            mouse_diff: (0.0, 0.0)
        }
    }
}
