pub mod level_collision;
pub mod player_movement;
pub mod ball;
pub mod intersection;

use bevy_ecs::schedule::SystemSet;

// Components
pub use player_movement::{PlayerMovement, PlayerMovementMode};
pub use ball::Ball;
pub use ball::TestSphere;

// Sim systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .label("sim")
        .with_system(player_movement::player_update)
        .with_system(ball::ball_update)
}
