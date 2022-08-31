use std::collections::HashMap;
use std::sync::Arc;
use bevy_ecs::world::{FromWorld, World};
use cgmath::{vec3, vec2, Deg, perspective};
use crate::gl_backend::{Mesh, VertexAttrib, Texture, TextureParams, GltfModel,
    UniformBuffer, Framebuffer, GlobalParams, bindings, ShaderProgram};
use crate::resources::{ShaderManager, TextureManager};

pub const RENDER_WIDTH: i32 = 320;
pub const RENDER_HEIGHT: i32 = 240;

pub const RENDER_ASPECT: f32 = 4.0 / 3.0;

pub const FOV: f32 = 60.0;

pub const NEAR_CLIP: f32 = 0.1;
pub const FAR_CLIP: f32 = 35.0;

pub const FOG_START: f32 = FAR_CLIP - 10.0;
pub const FOG_END: f32 = FAR_CLIP - 5.0;

/// The renderer state resource
pub struct RendererResources {
    pub full_screen_rect: Mesh,
    pub sky_texture: Arc<Texture>,
    pub ubo_global: UniformBuffer<GlobalParams>,
    pub framebuffer: Framebuffer,
    pub yiq_framebuffer: Framebuffer,
    pub sky_shader: Arc<ShaderProgram>,
    pub ps1_tess_shader: Arc<ShaderProgram>,
    pub ps1_no_tess_shader: Arc<ShaderProgram>,
    pub composite_yiq_shader: Arc<ShaderProgram>,
    pub composite_resolve_shader: Arc<ShaderProgram>,
    pub blit_shader: Arc<ShaderProgram>,
    pub models: HashMap<String, GltfModel>
}

impl FromWorld for RendererResources {
    fn from_world(world: &mut World) -> Self {
        log::info!("Creating renderer resources");

        // Create uniform buffers
        let mut ubo_global = UniformBuffer::<GlobalParams>::new();
        ubo_global.set_fog_color(&vec3(0.0, 0.0, 0.0));
        ubo_global.set_fog_dist(&vec2(FOG_START, FOG_END));

        ubo_global.set_target_aspect(&RENDER_ASPECT);
        ubo_global.set_render_res(&vec2(RENDER_WIDTH as f32, RENDER_HEIGHT as f32));
        ubo_global.set_render_fov(&(FOV * std::f32::consts::PI / 180.0));

        ubo_global.set_mat_proj(&perspective(Deg(FOV), RENDER_ASPECT, NEAR_CLIP, FAR_CLIP));

        ubo_global.bind(bindings::UniformBlockBinding::GlobalParams);

        // Load meshes
        let full_screen_rect = Mesh::new_indexed(
            &vec![
            1.0,  1.0, 0.0, 1.0, 1.0,  // top right
            1.0, -1.0, 0.0, 1.0, 0.0,  // bottom right
            -1.0, -1.0, 0.0, 0.0, 0.0,  // bottom left
            -1.0,  1.0, 0.0, 0.0, 1.0,  // top left
            ],
            &vec![
            0, 1, 3,
            1, 2, 3,
            ],
            &vec![
            VertexAttrib { index: 0, size: 3, attrib_type: gl::FLOAT },
            VertexAttrib { index: 1, size: 2, attrib_type: gl::FLOAT },
            ]);

        // Load textures
        let textures = world.get_resource::<TextureManager>().expect("Failed to get texture manager");
        let sky_texture = textures.get("sky").unwrap().clone();

        // Create framebuffer
        let framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, gl::SRGB8,
            TextureParams::new(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE, gl::NEAREST, gl::NEAREST));
        let yiq_framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, gl::RGBA32F,
            TextureParams::new(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE, gl::LINEAR_MIPMAP_LINEAR, gl::NEAREST));

        // Load shaders
        let mut shaders = world.get_resource_mut::<ShaderManager>().expect("Failed to get shader manager");
        let sky_shader = shaders.get("sky").unwrap().clone();
        let ps1_tess_shader = shaders.get("ps1_tess").unwrap().clone();
        let ps1_no_tess_shader = shaders.get("ps1_no_tess").unwrap().clone();
        let composite_yiq_shader = shaders.get("composite_yiq").unwrap().clone();
        let composite_resolve_shader = shaders.get("composite_resolve").unwrap().clone();
        let blit_shader = shaders.get("blit").unwrap().clone();

        RendererResources {
            full_screen_rect,
            sky_texture,
            ubo_global,
            framebuffer,
            yiq_framebuffer,
            sky_shader,
            ps1_tess_shader,
            ps1_no_tess_shader,
            composite_yiq_shader,
            composite_resolve_shader,
            blit_shader,
            models: HashMap::new()
        }
    }
}

