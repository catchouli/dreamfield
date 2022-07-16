mod shader;
mod mesh;
mod texture;

use std::ffi::CString;

use shader::ShaderProgram;
use mesh::{Mesh, VertexAttrib};
use texture::Texture;

/// The GL renderer
pub struct GLRenderer {
    full_screen_rect: Mesh,
    sky_rectangle_shader: ShaderProgram,
    sky_texture: Texture
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

        GLRenderer {
           full_screen_rect,
           sky_rectangle_shader,
           sky_texture
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

            self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
        }
    }
}

