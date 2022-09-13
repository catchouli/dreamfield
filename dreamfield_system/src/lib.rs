pub mod input;
pub mod resources;
pub mod world;
pub mod components;
pub mod systems;
mod fixed_timestep;
mod glfw_system;
mod game_host;

use bevy_ecs::schedule::SystemSet;
pub use fixed_timestep::*;
pub use glfw_system::*;
pub use game_host::*;
use systems::entity_spawner::entity_spawner_system;

/// The system systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(entity_spawner_system)
}

