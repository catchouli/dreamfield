use include_dir::{include_dir, Dir};

use dreamfield_macros::*;
use dreamfield_renderer::resources::{ShaderManager, TextureManager, ModelManager, FontManager};
use dreamfield_renderer::gl_backend::TextureParams;
use dreamfield_system::world::WorldChunkManager;

/// The world chunks
const WORLD_CHUNKS: Dir<'_> = include_dir!("target/world_chunks");

/// Create the world chunk manager
pub fn create_world_chunk_manager() -> WorldChunkManager {
    WorldChunkManager::new(&WORLD_CHUNKS)
}

/// Create the shader manager
pub fn create_shader_manager() -> ShaderManager {
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
pub fn create_texture_manager() -> TextureManager {
    TextureManager::new_with_textures(vec![
        ("sky", (include_bytes!("../resources/textures/skydark_small.png"), TextureParams::repeat_nearest(), true, None)),
    ])
}

/// Create the model manager
pub fn create_model_manager() -> ModelManager {
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
pub fn create_font_manager() -> FontManager {
    FontManager::new(vec![
        ("medieval", include_bytes!("../resources/fonts/medieval.png"), include_bytes!("../resources/fonts/medieval.csv")),
        ("medieval_2x", include_bytes!("../resources/fonts/medieval_2x.png"), include_bytes!("../resources/fonts/medieval_2x.csv")),
        ("medieval_4x", include_bytes!("../resources/fonts/medieval_4x.png"), include_bytes!("../resources/fonts/medieval_4x.csv")),
    ])
}
