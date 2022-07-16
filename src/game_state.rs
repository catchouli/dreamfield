/// The game state
#[derive(Copy, Clone)]
pub struct GameState {
    pub time: f32
}

impl GameState {
    /// Create a new, default game state
    pub fn new() -> GameState {
        GameState {
            time: 0.0
        }
    }
}
