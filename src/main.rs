pub mod renderer;
pub mod system;
pub mod sim;
pub mod rewindable_game_state;

use std::collections::VecDeque;

use glfw::{Action, Context, Key};
use system::glfw_system::Window;
use sim::{GameState, input::{InputEvent, InputName, InputState}};
use renderer::gl_renderer::GLRenderer;
use rewindable_game_state::RewindableGameState;

/// The width of the window
const WINDOW_WIDTH: u32 = 1024 * 2;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768 * 2;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 60;

/// The fixed update target tim
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

// Entry point
fn main() {
    // Create window
    let mut window = Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW - 500);

    // Create renderer
    let (win_width, win_height) = window.window.get_size();
    let mut renderer = GLRenderer::new(win_width, win_height);

    // The input state
    let mut input_events = VecDeque::<InputEvent>::new();

    // The game state
    let mut game_state = RewindableGameState::<GameState>::new();

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
            handle_window_event(&mut window, &mut renderer, event, &mut input_events);
        }

        // Handle mouse movement
        (mouse_x, mouse_y) = handle_mouse_movement(&window, (mouse_x, mouse_y), &mut input_events);

        // Update input state
        for event in input_events.iter() {
            handle_input_event(*event, &mut input_state)
        }

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
        renderer.render(game_state.cur_state());
        window.window.swap_buffers();
    }
}

/// Handle events
fn handle_window_event(window: &mut Window, renderer: &mut GLRenderer, event: glfw::WindowEvent,
                       input_events: &mut VecDeque<InputEvent>)
{
    match event {
        glfw::WindowEvent::FramebufferSize(width, height) => {
            renderer.set_viewport(width, height)
        }
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.window.set_should_close(true)
        }
        glfw::WindowEvent::MouseButton(_, Action::Press, _) => {
            if !window.is_mouse_captured() {
                window.set_mouse_captured(true);
                input_events.push_back(InputEvent::CursorCaptured(true));
            }
        }
        glfw::WindowEvent::Key(Key::LeftAlt, _, Action::Press, _) | glfw::WindowEvent::Focus(false) => {
            if window.is_mouse_captured() {
                window.set_mouse_captured(false);
                input_events.push_back(InputEvent::CursorCaptured(false));
            }
        }
        glfw::WindowEvent::Key(key, _, Action::Press, _) => {
            if let Some(input) = map_game_inputs_key(key) {
                input_events.push_back(InputEvent::GameInput(input, true));
            }
        }
        glfw::WindowEvent::Key(key, _, Action::Release, _) => {
            if let Some(input) = map_game_inputs_key(key) {
                input_events.push_back(InputEvent::GameInput(input, false));
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
                         input_events: &mut VecDeque<InputEvent>) -> (f64, f64)
{
    let (mouse_x, mouse_y) = window.window.get_cursor_pos();
    let (mouse_dx, mouse_dy) = (mouse_x - old_mouse_x, mouse_y - old_mouse_y);

    if window.is_mouse_captured() && (mouse_dx != 0.0 || mouse_dy != 0.0) {
        input_events.push_back(InputEvent::CursorMoved(mouse_dx, mouse_dy));
    }

    (mouse_x, mouse_y)
}

/// Handle an input event
fn handle_input_event(event: InputEvent, input_state: &mut InputState) {
    match event {
        InputEvent::CursorMoved(dx, dy) => {
            input_state.mouse_diff = (dx, dy);
        }
        InputEvent::CursorCaptured(captured) => {
            input_state.cursor_captured = captured;
        }
        InputEvent::GameInput(input_name, is_down) => {
            input_state.inputs[input_name as usize] = is_down;
        }
    }
}
