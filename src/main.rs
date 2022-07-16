mod renderer;
mod system;
mod game_state;

use glfw::{Action, Context, Key};
use system::glfw_system::Window;
use game_state::GameState;
use renderer::gl_renderer::GLRenderer;

/// The width of the window
const WINDOW_WIDTH: u32 = 1024;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768;

// Entry point
fn main() {
    // Create window
    let mut window = Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW);

    // Create renderer
    let renderer = GLRenderer::new();

    // The game state
    let mut game_state = GameState::new();

    // Start main loop
    while !window.window.should_close() {
        // Handle events
        for event in window.poll_events() {
            handle_window_event(&mut window, event);
        }

        // Update game state
        game_state.time = window.glfw.get_time() as f32;

        // Render
        renderer.render(game_state);
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

