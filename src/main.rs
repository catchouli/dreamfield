pub mod renderer;
pub mod system;
pub mod sim;
pub mod rewindable_game_state;

use glfw::{Action, Context, Key};
use system::glfw_system::Window;
use sim::{GameState, input::{InputState, InputName}};
use renderer::gl_renderer::GLRenderer;

/// The width of the window
const WINDOW_WIDTH: u32 = 1024 * 2;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768 * 2;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 30;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

// Entry point
fn main() {
    // Create window
    let mut window = Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW - 500);

    // Create renderer
    let (win_width, win_height) = window.window.get_size();
    let mut renderer = GLRenderer::new(win_width, win_height);

    let mut game_state = GameState::new();

    // Fixed timestep - https://gafferongames.com/post/fix_your_timestep/
    let mut current_time = window.glfw.get_time();
    let mut sim_time = 0.0;
    let mut accumulator = 0.0;

    // Mouse movement
    let (mut mouse_x, mut mouse_y) = window.window.get_cursor_pos();

    // Input state
    let mut input_state = InputState::new();

    // Start main loop
    while !window.window.should_close() {
        // Handle events
        for event in window.poll_events() {
            handle_window_event(&mut window, &mut renderer, event, &mut input_state);
        }

        // Handle mouse movement
        (mouse_x, mouse_y) = handle_mouse_movement(&window, (mouse_x, mouse_y), &mut input_state);

        // Fixed timestep
        let new_time = window.glfw.get_time();
        let frame_time = new_time - current_time;

        current_time = new_time;
        accumulator += frame_time;

        while accumulator >= FIXED_UPDATE_TIME {
            // Simulate game state
            game_state.simulate(sim_time, &input_state);

            // Consume accumulated time
            accumulator -= FIXED_UPDATE_TIME;
            sim_time += FIXED_UPDATE_TIME;
        }

        // Render
        renderer.render(&game_state);
        window.window.swap_buffers();
    }
}

/// Handle events
fn handle_window_event(window: &mut Window, renderer: &mut GLRenderer, event: glfw::WindowEvent,
                       input_state: &mut InputState)
{
    match event {
        glfw::WindowEvent::FramebufferSize(width, height) => {
            renderer.set_window_viewport(width, height)
        }
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.window.set_should_close(true)
        }
        glfw::WindowEvent::MouseButton(_, Action::Press, _) => {
            if !window.is_mouse_captured() {
                window.set_mouse_captured(true);
                input_state.cursor_captured = true;
            }
        }
        glfw::WindowEvent::Key(Key::LeftAlt, _, Action::Press, _) | glfw::WindowEvent::Focus(false) => {
            if window.is_mouse_captured() {
                window.set_mouse_captured(false);
                input_state.cursor_captured = false;
            }
        }
        glfw::WindowEvent::Key(Key::F1, _, Action::Press, _) => {
            renderer.toggle_graphics_mode();
        }
        glfw::WindowEvent::Key(Key::F2, _, Action::Press, _) => {
            renderer.toggle_wireframe_mode();
        }
        glfw::WindowEvent::Key(key, _, Action::Press, _) => {
            if let Some(input) = map_game_inputs_key(key) {
                input_state.inputs[input as usize] = true;
            }
        }
        glfw::WindowEvent::Key(key, _, Action::Release, _) => {
            if let Some(input) = map_game_inputs_key(key) {
                input_state.inputs[input as usize] = false;
            }
        }
        _ => {}
    }
}

/// Map game inputs from the keyboard
fn map_game_inputs_key(key: Key) -> Option<InputName> {
    match key {
        Key::W => Some(InputName::CamForwards),
        Key::S => Some(InputName::CamBackwards),
        Key::A => Some(InputName::CamLeft),
        Key::D => Some(InputName::CamRight),
        Key::LeftShift => Some(InputName::CamSpeed),
        Key::Z => Some(InputName::Rewind),
        _ => None
    }
}

/// Handle mouse movement
fn handle_mouse_movement(window: &Window, (old_mouse_x, old_mouse_y): (f64, f64),
                         input_state: &mut InputState) -> (f64, f64)
{
    let (mouse_x, mouse_y) = window.window.get_cursor_pos();
    let (mouse_dx, mouse_dy) = (mouse_x - old_mouse_x, mouse_y - old_mouse_y);

    input_state.mouse_diff = (mouse_dx, mouse_dy);

    (mouse_x, mouse_y)
}
