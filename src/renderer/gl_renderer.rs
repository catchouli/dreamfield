pub mod shader;
pub mod mesh;
pub mod texture;
pub mod uniform_buffer;
pub mod gltf_model;
pub mod to_std140;
pub mod bindings;

use cgmath::*;

use shader::*;
use mesh::*;
use texture::*;
use uniform_buffer::*;
use gltf_model::*;

/// The GL renderer
pub struct GLRenderer {
    full_screen_rect: Mesh,
    sky_rectangle_shader: ShaderProgram,
    glfw_model_shader: ShaderProgram,
    sky_texture: Texture,
    suzanne: GltfModel,
    triangle: GltfModel,
    ubo_global: UniformBuffer<GlobalParams>
}

impl GLRenderer {
    /// Create a new GLRenderer
    pub fn new() -> GLRenderer {
        // Create uniform buffers
        let ubo_global = UniformBuffer::<GlobalParams>::new_single();
        ubo_global.bind(bindings::UniformBlockBinding::GlobalParams);

        // Load shaders
        let sky_rectangle_shader = ShaderProgram::new_from_vf("resources/shaders/sky_rectangle.glsl");
        let glfw_model_shader = ShaderProgram::new_from_vf("resources/shaders/glfw_model.glsl");

        // Load meshes
        let full_screen_rect = Mesh::new_indexed(
            &vec![
                 1.0,  1.0, 0.0, 0.0, 0.0,  // top right
                 1.0, -1.0, 0.0, 0.0, 1.0,  // bottom right
                -1.0, -1.0, 0.0, 1.0, 1.0,  // bottom left
                -1.0,  1.0, 0.0, 1.0, 0.0,  // top left
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
        let sky_texture = Texture::new_from_file("resources/textures/cloud.jpg", Texture::NEAREST_WRAP)
            .expect("Failed to load sky texture");

        // Load suzanne
        let suzanne = GltfModel::load_gltf("resources/models/suzanne.glb").unwrap();
        let triangle = GltfModel::load_gltf("resources/models/TriangleWithoutIndices.gltf").unwrap();

        GLRenderer {
           full_screen_rect,
           sky_rectangle_shader,
           glfw_model_shader,
           sky_texture,
           suzanne,
           triangle,
           ubo_global
        }
    }

    /// Render the game
    pub fn render(&mut self, game_state: crate::GameState) {
        unsafe {
            gl::ClearColor(0.06, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Set up global render parameters
        let mat_proj: Matrix4<f32> = perspective(Deg(90.0), 1.0, 0.1, 100.0);
        let mat_cam: Matrix4<f32> = Matrix4::from_angle_y(Rad(game_state.time.sin() * 0.5))
            * Matrix4::from_translation(vec3(0.0, 0.0, 7.0));
        let mat_view = mat_cam.invert().unwrap();

        self.ubo_global.set_sim_time_n(0, &game_state.time);
        self.ubo_global.set_mat_proj(&mat_proj);
        self.ubo_global.set_mat_view(&mat_view);
        self.ubo_global.upload_changed();

        // Draw background
        unsafe { gl::Disable(gl::DEPTH_TEST) }
        self.sky_texture.bind(bindings::TextureSlot::BaseColor);
        self.sky_rectangle_shader.use_program();
        self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
        unsafe { gl::Enable(gl::DEPTH_TEST) }

        // Draw glfw models
        self.glfw_model_shader.use_program();
        self.suzanne.render();
        self.triangle.render();
    }
}

