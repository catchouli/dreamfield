use cgmath::{vec3, vec2, Deg, perspective};
use crate::renderer::{shaders, resources};
use dreamfield_renderer::gl_backend::{Mesh, VertexAttrib, ShaderProgram, Texture, TextureParams, GltfModel,
    UniformBuffer, Framebuffer, GlobalParams, bindings};

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
    pub sky_shader: ShaderProgram,
    pub ps1_shader: ShaderProgram,
    pub ps1_shader_tess: ShaderProgram,
    pub blit_shader: ShaderProgram,
    pub composite_yiq_shader: ShaderProgram,
    pub composite_resolve_shader: ShaderProgram,
    pub sky_texture: Texture,
    pub demo_scene_model: GltfModel,
    pub fire_orb_model: GltfModel,
    pub ubo_global: UniformBuffer<GlobalParams>,
    pub framebuffer: Framebuffer,
    pub yiq_framebuffer: Framebuffer
}

impl Default for RendererResources {
    fn default() -> Self {
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

        // Load shaders
        let sky_shader = ShaderProgram::new_from_vf(shaders::SHADER_SKY);
        let ps1_shader = ShaderProgram::new_from_vf(shaders::SHADER_PS1);
        let ps1_shader_tess = ShaderProgram::new_from_vtf(shaders::SHADER_PS1_TESSELLATE);
        let blit_shader = ShaderProgram::new_from_vf(shaders::SHADER_BLIT);
        let composite_yiq_shader = ShaderProgram::new_from_vf(shaders::SHADER_COMPOSITE_YIQ);
        let composite_resolve_shader = ShaderProgram::new_from_vf(shaders::SHADER_COMPOSITE_RESOLVE);

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
        let sky_params = TextureParams::repeat_nearest();
        let sky_texture = Texture::new_from_image_buf(resources::TEXTURE_CLOUD, sky_params, true, None)
            .expect("Failed to load sky texture");

        // Load models
        let demo_scene_model = GltfModel::from_buf(resources::MODEL_DEMO_SCENE).unwrap();
        let fire_orb_model = GltfModel::from_buf(resources::MODEL_FIRE_ORB).unwrap();

        // Create framebuffer
        let framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, gl::SRGB8,
            TextureParams::new(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE, gl::NEAREST, gl::NEAREST));
        let yiq_framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, gl::RGBA32F,
            TextureParams::new(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE, gl::LINEAR_MIPMAP_LINEAR, gl::NEAREST));

        RendererResources {
            full_screen_rect,
            sky_shader,
            sky_texture,
            ps1_shader,
            ps1_shader_tess,
            blit_shader,
            composite_yiq_shader,
            composite_resolve_shader,
            demo_scene_model,
            fire_orb_model,
            ubo_global,
            framebuffer,
            yiq_framebuffer
        }
    }
}

