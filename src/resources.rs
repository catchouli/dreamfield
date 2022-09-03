use dreamfield_renderer::resources::{ShaderManager, TextureManager, ModelManager};
use dreamfield_renderer::gl_backend::TextureParams;
use dreamfield_macros::*;
use dreamfield_system::world::WorldChunkManager;
use include_dir::{include_dir, Dir};

/// The world chunks
pub const WORLD_CHUNKS: Dir<'_> = include_dir!("target/world_chunks");

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
        ("blit", preprocess_shader_vf!(include_bytes!("../resources/shaders/blit.glsl")))
    ])
}

/// Create the texture manager
pub fn create_texture_manager() -> TextureManager {
    TextureManager::new_with_textures(vec![
        ("sky", (include_bytes!("../resources/textures/skydark_small.png"), TextureParams::repeat_nearest(), true, None))
    ])
}

/// Create the model manager
pub fn create_model_manager() -> ModelManager {
    ModelManager::new_with_models(vec![
        ("demo_scene", include_bytes!("../resources/models/demo_scene.glb")),
        ("fire_orb", include_bytes!("../resources/models/fire_orb.glb")),
        ("scene_a", include_bytes!("../resources/models/scene_a.glb")),
        ("scene_b", include_bytes!("../resources/models/scene_b.glb")),
        ("scene_c", include_bytes!("../resources/models/scene_c.glb")),
        ("scene_d", include_bytes!("../resources/models/scene_d.glb")),
        ("scene_e", include_bytes!("../resources/models/scene_e.glb")),
    ])
}
