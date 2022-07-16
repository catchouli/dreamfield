pub mod shader;
pub mod mesh;
pub mod texture;
pub mod gltf_model;

use std::ffi::CString;
use cgmath::{SquareMatrix, Matrix4, vec3, Deg, perspective};

use shader::*;
use mesh::*;
use texture::*;
use gltf_model::*;

/// The GL renderer
pub struct GLRenderer {
    full_screen_rect: Mesh,
    sky_rectangle_shader: ShaderProgram,
    sky_texture: Texture,
    suzanne: GltfModel
}

impl GLRenderer {
    /// Create a new GLRenderer
    pub fn new() -> GLRenderer {
        // Load shaders
        let sky_rectangle_shader = ShaderProgram::new_from_vf("resources/shaders/sky_rectangle.glsl");

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
        let sky_texture = Texture::new_from_file("resources/textures/cloud.jpg", Texture::NEAREST_WRAP);

        // Load suzanne
        let suzanne = GltfModel::load_gltf("resources/models/suzanne.glb").unwrap();

        GLRenderer {
           full_screen_rect,
           sky_rectangle_shader,
           sky_texture,
           suzanne
        }
    }

    /// Render the game
    pub fn render(&self, game_state: crate::GameState) {
        unsafe {
            gl::ClearColor(0.06, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            self.sky_texture.bind(0);
            self.sky_rectangle_shader.use_program();

            let uni_time = CString::new("uni_time").unwrap();
            let time_location = gl::GetUniformLocation(self.sky_rectangle_shader.id(), uni_time.as_ptr());
            gl::Uniform1f(time_location, game_state.time);

            // Set up matrices
            let view: Matrix4<f32> = Matrix4::from_translation(vec3(0.0, 0.0, -1.0 - game_state.time*0.1));
            let proj: Matrix4<f32> = perspective(Deg(90.0), 1.0, 0.1, 100.0);
            let model: Matrix4<f32> = SquareMatrix::identity();

            gl::UniformMatrix4fv(self.sky_rectangle_shader.get_loc("uni_proj"), 1, gl::FALSE, &proj[0][0]);
            gl::UniformMatrix4fv(self.sky_rectangle_shader.get_loc("uni_view"), 1, gl::FALSE, &view[0][0]);
            gl::UniformMatrix4fv(self.sky_rectangle_shader.get_loc("uni_model"), 1, gl::FALSE, &model[0][0]);

            //self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);

            self.suzanne.render();
        }
    }
}

