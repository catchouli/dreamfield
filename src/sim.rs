pub mod input;
pub mod level_collision;
mod sim_time;
mod player_movement;
mod ball;

use bevy_ecs::schedule::SystemSet;

// Resources
pub use sim_time::SimTime;

// Components
pub use player_movement::{PlayerCamera, PlayerMovement};
pub use ball::Ball;

// Sim systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .label("sim")
        .with_system(player_movement::player_update)
        .with_system(ball::ball_update)
}
