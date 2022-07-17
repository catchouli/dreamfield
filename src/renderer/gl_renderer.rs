pub mod shader;
pub mod mesh;
pub mod texture;
pub mod gltf_model;

use cgmath::{Matrix, SquareMatrix, Matrix4, Matrix3, vec3, Deg, Rad, perspective};

use shader::*;
use mesh::*;
use texture::*;
use gltf_model::*;

/// The GL renderer
pub struct GLRenderer {
    full_screen_rect: Mesh,
    sky_rectangle_shader: ShaderProgram,
    glfw_model_shader: ShaderProgram,
    sky_texture: Texture,
    suzanne: GltfModel
}

impl GLRenderer {
    /// Create a new GLRenderer
    pub fn new() -> GLRenderer {
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
           suzanne
        }
    }

    /// Render the game
    pub fn render(&self, game_state: crate::GameState) {
        unsafe {
            gl::ClearColor(0.06, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Draw background
            gl::Disable(gl::DEPTH_TEST);
            self.sky_texture.bind(0);
            self.sky_rectangle_shader.use_program();
            self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
            gl::Enable(gl::DEPTH_TEST);

            // Draw suzanne
            self.glfw_model_shader.use_program();

            // Set up matrices
            let view: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 0.0, -2.0 - game_state.time.sin()));
            let proj: Matrix4<f32> = perspective(Deg(90.0), 1.0, 0.1, 100.0);
            let model: Matrix4<f32> = Matrix4::from_angle_y(Rad(game_state.time));
            let normal = Self::model_to_normal(model);

            gl::Uniform1f(self.glfw_model_shader.get_loc("uni_time"), game_state.time);
            gl::UniformMatrix4fv(self.glfw_model_shader.get_loc("uni_proj"), 1, gl::FALSE, &proj[0][0]);
            gl::UniformMatrix4fv(self.glfw_model_shader.get_loc("uni_view"), 1, gl::FALSE, &view[0][0]);
            gl::UniformMatrix4fv(self.glfw_model_shader.get_loc("uni_model"), 1, gl::FALSE, &model[0][0]);
            gl::UniformMatrix3fv(self.glfw_model_shader.get_loc("uni_normal"), 1, gl::FALSE, &normal[0][0]);

            self.suzanne.render();
        }
    }

    /// Get normal matrix from a model matrix
    fn model_to_normal(model: Matrix4<f32>) -> Matrix3<f32> {
        // https://learnopengl.com/Lighting/Basic-lighting
        let v = model.invert().unwrap().transpose();
        Matrix3::from_cols(v.x.truncate(), v.y.truncate(), v.z.truncate())
    }
}

