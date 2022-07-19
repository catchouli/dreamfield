/// Input events
pub enum InputEvent {
    CursorMoved(f64, f64),
    CursorCaptured(bool),
    GameInput(InputName, bool)
}

/// Input names
pub enum InputName {
    CamForwards,
    CamBackwards,
    CamLeft,
    CamRight,
    CamSpeed,
    Last
}
