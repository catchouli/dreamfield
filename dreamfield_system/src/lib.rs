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

use bevy_ecs::{schedule::SystemSet, world::World, prelude::Events};
use input::InputState;
use resources::{SimTime, Diagnostics};
use systems::entity_spawner::EntitySpawnEvent;
use world::world_collision::WorldCollision;

/// Initialise resources etc
pub fn init(world: &mut World) {
    // Resources
    world.init_resource::<SimTime>();
    world.init_resource::<InputState>();
    world.init_resource::<WindowSettings>();
    world.init_resource::<Diagnostics>();
    world.init_resource::<WorldCollision>();

    // Events
    world.init_resource::<Events::<EntitySpawnEvent>>();
}

/// The system systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(systems::entity_spawner::entity_spawner_system)
        .with_system(intersection::update_world_chunks_system)
}

