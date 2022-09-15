pub mod gl_backend;
pub mod camera;
pub mod resources;
pub mod renderer;
pub mod components;

use bevy_ecs::schedule::SystemSet;

/// The render systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(renderer::update_diagnostics)
        .with_system(renderer::renderer_system)
}

