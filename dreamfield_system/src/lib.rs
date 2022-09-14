pub mod input;
pub mod resources;
pub mod world;
pub mod components;
pub mod systems;
pub mod intersection;
mod fixed_timestep;
mod glfw_system;
mod game_host;

pub use fixed_timestep::*;
pub use glfw_system::*;
pub use game_host::*;

use bevy_ecs::schedule::SystemSet;

/// The system systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(systems::entity_spawner::entity_spawner_system)
        .with_system(intersection::update_world_chunks_system)
}

