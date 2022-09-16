pub mod gl_backend;
pub mod camera;
pub mod resources;
pub mod renderer;
pub mod components;

use bevy_ecs::{schedule::SystemSet, world::World};
use dreamfield_system::world::WorldChunkManager;
use resources::{ModelManager, ShaderManager, TextureManager, FontManager};

/// Initialise resources etc
pub fn init(world: &mut World, models: ModelManager, shaders: ShaderManager, textures: TextureManager,
    fonts: FontManager, chunks: WorldChunkManager)
{
    world.insert_resource(models);
    world.insert_resource(shaders);
    world.insert_resource(textures);
    world.insert_resource(fonts);
    world.insert_resource(chunks);
}

/// The render systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(renderer::update_diagnostics)
        .with_system(renderer::renderer_system)
}

