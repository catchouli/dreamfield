pub mod shader;
pub mod mesh;
pub mod texture;
pub mod uniform_buffer;
pub mod gltf_model;
pub mod to_std140;
pub mod bindings;
pub mod resources;
pub mod lights;

use cgmath::*;

use shader::*;
use mesh::*;
use texture::*;
use uniform_buffer::*;
use gltf_model::*;

use super::camera::Camera;

/// The GL renderer
pub struct GLRenderer {
    full_screen_rect: Mesh,
    sky_shader: ShaderProgram,
    pbr_shader: ShaderProgram,
    sky_texture: Texture,
    demo_scene_model: GltfModel,
    fire_orb_model: GltfModel,
    ubo_global: UniformBuffer<GlobalParams>
}

impl GLRenderer {
    /// Create a new GLRenderer
    pub fn new(width: i32, height: i32) -> GLRenderer {
        // Create uniform buffers
        let mut ubo_global = UniformBuffer::<GlobalParams>::new();
        ubo_global.bind(bindings::UniformBlockBinding::GlobalParams);

        // Load shaders
        let sky_shader = ShaderProgram::new_from_vf(resources::SHADER_SKY);
        let pbr_shader = ShaderProgram::new_from_vf(resources::SHADER_PBR);

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
        let sky_texture = Texture::new_from_image_buf(resources::TEXTURE_CLOUD, Texture::NEAREST_WRAP)
            .expect("Failed to load sky texture");

        // Load models
        let demo_scene_model = GltfModel::from_buf(resources::MODEL_DEMO_SCENE).unwrap();
        let fire_orb_model = GltfModel::from_buf(resources::MODEL_FIRE_ORB).unwrap();

        // Look for extra fields
        for drawable in demo_scene_model.drawables().iter() {
            if let Some(extra) = drawable.extras() {
                let raw = extra.get();
                println!("Node {} has extras: {:?}", drawable.name(), raw);
            }
        }

        // Create renderer struct
        let mut renderer = GLRenderer {
           full_screen_rect,
           sky_shader,
           pbr_shader,
           sky_texture,
           demo_scene_model,
           fire_orb_model,
           ubo_global
        };

        // Set viewport
        renderer.set_viewport(width, height);

        renderer
    }

    /// Render the game
    pub fn render(&mut self, game_state: &crate::GameState) {
        unsafe {
            gl::ClearColor(0.06, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Update global params
        self.ubo_global.set_sim_time(&(game_state.time as f32));
        self.ubo_global.set_mat_view_derive(&game_state.camera.get_view_matrix());
        self.ubo_global.upload_changed();

        // Draw background
        unsafe { gl::Disable(gl::DEPTH_TEST) }
        self.sky_texture.bind(bindings::TextureSlot::BaseColor);
        self.sky_shader.use_program();
        self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
        unsafe { gl::Enable(gl::DEPTH_TEST) }

        // Draw glfw models
        self.pbr_shader.use_program();
        self.demo_scene_model.render(&mut self.ubo_global);
        self.fire_orb_model.set_transform(&Matrix4::from_translation(vec3(0.0, game_state.time as f32, 0.0)));
        self.fire_orb_model.render(&mut self.ubo_global);
    }

    /// Update the viewport
    pub fn set_viewport(&mut self, width: i32, height: i32) {
        println!("Setting viewport to {width} * {height}");
        unsafe { gl::Viewport(0, 0, width, height) };

        // Update projection matrix and aspect
        let aspect = width as f32 / height as f32;
        self.ubo_global.set_mat_proj(&perspective(Deg(60.0), aspect, 0.1, 100.0));
        self.ubo_global.set_vp_aspect(&aspect);
    }
}

