mod renderer;
mod system;

use std::ffi::CString;
use glfw::{Action, Context, Key};
use system::glfw_system::Window;
use renderer::gl::{ShaderProgram, Texture, Mesh, VertexAttrib};

/// The width of the window
const WINDOW_WIDTH: u32 = 1024;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768;

/// The GL renderer
pub struct GLRenderer {
}

impl GLRenderer {
    /// Create a new GLRenderer
    pub fn new() -> GLRenderer {
        GLRenderer { }
    }
}

// Entry point
fn main() {
    // Create window
    let mut window = Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW);

    // Load shaders
    let shader_program = ShaderProgram::new_from_vf("resources/shaders/sky_rectangle.glsl");

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

    // Start main loop
    while !window.window.should_close() {
        // Handle events
        for event in window.poll_events() {
            handle_window_event(&mut window, event);
        }

        // Render
        unsafe {
            gl::ClearColor(0.06, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            sky_texture.bind(0);
            shader_program.use_program();

            let time = window.glfw.get_time() as f32;
            let green = time.sin() / 2.0 + 0.5;
            let uni_color = CString::new("uni_color").unwrap();
            let color_location = gl::GetUniformLocation(shader_program.id(), uni_color.as_ptr());
            gl::Uniform4f(color_location, 0.0, green, 0.0, 1.0);

            full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
        }
        window.window.swap_buffers();
    }
}

/// Handle events
fn handle_window_event(window: &mut Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::FramebufferSize(width, height) => {
            unsafe { gl::Viewport(0, 0, width, height) }
        }
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.window.set_should_close(true)
        }
        _ => {}
    }
}

