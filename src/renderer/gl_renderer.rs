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
use to_std140::*;

/// The GL renderer
pub struct GLRenderer {
    full_screen_rect: Mesh,
    sky_rectangle_shader: ShaderProgram,
    glfw_model_shader: ShaderProgram,
    sky_texture: Texture,
    suzanne: GltfModel,
    ubo_global: UniformBuffer<GlobalRenderParams>,
    ubo_model: UniformBuffer<ModelRenderParams>
}

/// Base render params
#[std140::repr_std140]
struct GlobalRenderParams {
    sim_time: std140::float,
    mat_proj: std140::mat4x4,
    mat_view: std140::mat4x4
}

impl Default for GlobalRenderParams {
    fn default() -> Self {
        GlobalRenderParams {
            sim_time: (0.0).to_std140(),
            mat_proj: Matrix4::identity().to_std140(),
            mat_view: Matrix4::identity().to_std140()
        }
    }
}

/// Object render params
#[std140::repr_std140]
struct ModelRenderParams {
    mat_model: std140::mat4x4,
    mat_normal: std140::mat3x3
}

impl Default for ModelRenderParams {
    fn default() -> Self {
        ModelRenderParams {
            mat_model: Matrix4::identity().to_std140(),
            mat_normal: Matrix3::identity().to_std140()
        }
    }
}

impl GLRenderer {
    /// Create a new GLRenderer
    pub fn new() -> GLRenderer {
        // Create uniform buffers
        let ubo_global = UniformBuffer::<GlobalRenderParams>::new();
        ubo_global.bind(bindings::UniformBlockBinding::GlobalRenderParams);
        let ubo_model = UniformBuffer::<ModelRenderParams>::new();
        ubo_model.bind(bindings::UniformBlockBinding::ModelRenderParams);

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

        GLRenderer {
           full_screen_rect,
           sky_rectangle_shader,
           glfw_model_shader,
           sky_texture,
           suzanne,
           ubo_global,
           ubo_model
        }
    }

    /// Render the game
    pub fn render(&mut self, game_state: crate::GameState) {
        unsafe {
            gl::ClearColor(0.06, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Set up global render parameters
            let mat_proj: Matrix4<f32> = perspective(Deg(90.0), 1.0, 0.1, 100.0);
            let mat_cam: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 0.0, 2.0 + game_state.time.sin()))
                * Matrix4::from_angle_x(Rad(game_state.time.sin() * 0.25));
            let mat_view = mat_cam.invert().unwrap();

            self.ubo_global.data.sim_time = game_state.time.to_std140();
            self.ubo_global.data.mat_proj = mat_proj.to_std140();
            self.ubo_global.data.mat_view = mat_view.to_std140();
            self.ubo_global.upload();

            // Draw background
            gl::Disable(gl::DEPTH_TEST);
            self.sky_texture.bind(0);
            self.sky_rectangle_shader.use_program();
            self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
            gl::Enable(gl::DEPTH_TEST);

            // Draw suzanne
            self.glfw_model_shader.use_program();

            let mat_model: Matrix4<f32> = Matrix4::from_angle_y(Rad(game_state.time));
            let mat_normal = {
                // https://learnopengl.com/Lighting/Basic-lighting
                let v = mat_model.invert().unwrap().transpose();
                Matrix3::from_cols(v.x.truncate(), v.y.truncate(), v.z.truncate())
            };

            self.ubo_model.data.mat_model = mat_model.to_std140();
            self.ubo_model.data.mat_normal = mat_normal.to_std140();
            self.ubo_model.upload();

            self.suzanne.render();
        }
    }
}

