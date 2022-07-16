mod renderer;

use std::ffi::CString;
use std::str;
use glfw::{Action, Context, Key};

/// The width of the window
const WINDOW_WIDTH: u32 = 1024;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768;

/// The vertex shader
const VERTEX_SHADER: &str = r#"
    #version 330 core

    layout (location = 0) in vec3 in_pos;
    layout (location = 1) in vec2 in_uv;

    out vec2 var_uv;

    void main() {
        var_uv = in_uv;
        gl_Position = vec4(in_pos.x, in_pos.y, in_pos.z, 1.0);
    }
"#;

/// The fragment shader
const FRAGMENT_SHADER: &str = r#"
    #version 330 core

    #define M_PI 3.1415926535897932384626433832795

    uniform vec4 uni_color;

    uniform sampler2D tex_skybox;

    in vec2 var_uv;

    out vec4 out_frag_color;

    void main() {
        vec3 offset = vec3(var_uv-0.5,0)*2;
        vec3 d = vec3(0.0, 0.0, -1.0);
        vec2 tx = vec2(0.5 + atan(d.z, sqrt(d.x*d.x + d.y*d.y))/(2.0 * M_PI), 0.5 + atan(d.y, d.x)/(2.0 * M_PI));

        vec4 c = texture2D( tex_skybox, tx);

        out_frag_color = texture(tex_skybox, var_uv);
    }
"#;

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
    let mut window = renderer::Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW);

    // Load shaders
    let shader_program = renderer::ShaderProgram::new_from_vf(VERTEX_SHADER, FRAGMENT_SHADER);

    // Load meshes
    let full_screen_rect = renderer::Mesh::new_indexed(
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
            renderer::VertexAttrib { index: 0, size: 3, attrib_type: gl::FLOAT },
            renderer::VertexAttrib { index: 1, size: 2, attrib_type: gl::FLOAT },
        ]);

    // Load textures
    let sky_texture = renderer::Texture::new_from_file("cloud.jpg", renderer::Texture::NEAREST_WRAP);

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
fn handle_window_event(window: &mut renderer::Window, event: glfw::WindowEvent) {
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

