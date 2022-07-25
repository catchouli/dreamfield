use dreamfield_macros::{preprocess_shader_vf, preprocess_shader_vtf};

pub const SHADER_SKY: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../resources/shaders/sky.glsl"));
pub const SHADER_PBR: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../resources/shaders/pbr.glsl"));
pub const SHADER_PS1: (&str, &str, &str, &str) = preprocess_shader_vtf!(include_bytes!("../../resources/shaders/ps1.glsl"));
pub const SHADER_BLIT: (&str, &str) = preprocess_shader_vf!(include_bytes!("../../resources/shaders/blit.glsl"));
