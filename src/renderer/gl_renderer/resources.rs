use dreamfield_macros::preprocess_shader_vf;

pub const TEXTURE_CLOUD: &[u8] = include_bytes!("../../../resources/textures/cloud.jpg");

pub const MODEL_DEMO_SCENE: &[u8] = include_bytes!("../../../resources/models/demo_scene.glb");
pub const MODEL_FIRE_ORB: &[u8] = include_bytes!("../../../resources/models/fire_orb.glb");

pub const SHADER_SKY: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../../resources/shaders/sky.glsl"));
pub const SHADER_PBR: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../../resources/shaders/pbr.glsl"));
pub const SHADER_PS1: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../../resources/shaders/ps1_per_vertex.glsl"));
pub const SHADER_BLIT: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../../resources/shaders/blit.glsl"));
