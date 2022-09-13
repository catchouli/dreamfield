use include_dir::{include_dir, Dir};

use bevy_ecs::prelude::Events;
use bevy_ecs::world::World;

use dreamfield_macros::*;
use dreamfield_renderer::resources::{ShaderManager, TextureManager, ModelManager, FontManager};
use dreamfield_renderer::gl_backend::TextureParams;
use dreamfield_system::WindowSettings;
use dreamfield_system::resources::{InputState, SimTime, Diagnostics};
use dreamfield_system::systems::entity_spawner::EntitySpawnEvent;
use dreamfield_system::world::WorldChunkManager;
use dreamfield_system::world::world_collision::WorldCollision;

/// The world chunks
const WORLD_CHUNKS: Dir<'_> = include_dir!("target/world_chunks");

/// Create the world chunk manager
fn create_world_chunk_manager() -> WorldChunkManager {
    WorldChunkManager::new(&WORLD_CHUNKS)
}

/// Create the shader manager
fn create_shader_manager() -> ShaderManager {
    ShaderManager::new(vec![
        ("sky", preprocess_shader_vf!(include_bytes!("../resources/shaders/sky.glsl"))),
        ("ps1_no_tess", preprocess_shader_vf!(include_bytes!("../resources/shaders/ps1.glsl"))),
        ("ps1_tess", preprocess_shader_vtf!(include_bytes!("../resources/shaders/ps1.glsl"))),
        ("composite_yiq", preprocess_shader_vf!(include_bytes!("../resources/shaders/composite_yiq.glsl"))),
        ("composite_resolve", preprocess_shader_vf!(include_bytes!("../resources/shaders/composite_resolve.glsl"))),
        ("blit", preprocess_shader_vf!(include_bytes!("../resources/shaders/blit.glsl"))),
        ("text", preprocess_shader_vf!(include_bytes!("../resources/shaders/text.glsl"))),
    ])
}

/// Create the texture manager
fn create_texture_manager() -> TextureManager {
    TextureManager::new_with_textures(vec![
        ("sky", (include_bytes!("../resources/textures/skydark_small.png"), TextureParams::repeat_nearest(), true, None)),
    ])
}

/// Create the model manager
fn create_model_manager() -> ModelManager {
    ModelManager::new_with_models(vec![
        ("fire_orb", include_bytes!("../resources/models/fire_orb.glb")),
        ("tree", include_bytes!("../resources/models/tree.glb")),
        ("white_sphere", include_bytes!("../resources/models/white_sphere.glb")),
        ("samy", include_bytes!("../resources/models/samy_diamond.glb")),
        ("elf", include_bytes!("../resources/models/elf.glb")),
        ("minecart", include_bytes!("../resources/models/minecart.glb")),
        ("capsule", include_bytes!("../resources/models/capsule.glb")),
    ])
}

/// Create the font manager
fn create_font_manager() -> FontManager {
    const MEDIEVAL_FONT_TEX: &'static [u8] = include_bytes!("../resources/fonts/0xDB_medievalish_chonker_8x8_1bpp_bmp_font_packed.png");
    const MEDIEVAL_FONT_MAP: &'static [u8] = include_bytes!("../resources/fonts/0xDB_medievalish_chonker_8x8_1bpp_bmp_font_packed.csv");
    FontManager::new(vec![
        ("medieval", MEDIEVAL_FONT_TEX, MEDIEVAL_FONT_MAP)
    ])
}

/// Initialise resources
pub fn add_resources(world: &mut World) {
    // System resources
    world.insert_resource(WindowSettings::default());
    world.insert_resource(InputState::new());
    world.insert_resource(SimTime::new(0.0, super::FIXED_UPDATE_TIME));
    world.insert_resource(Events::<EntitySpawnEvent>::default());
    world.init_resource::<Diagnostics>();

    // Resource managers for our game
    world.insert_resource(create_shader_manager());
    world.insert_resource(create_texture_manager());
    world.insert_resource(create_model_manager());
    world.insert_resource(create_world_chunk_manager());
    world.insert_resource(create_font_manager());

    // Our own resources
    world.init_resource::<WorldCollision>();
}
