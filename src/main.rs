pub mod renderer;
pub mod system;
pub mod game_state;

use glfw::{Action, Context, Key};
use system::glfw_system::Window;
use game_state::GameState;
use renderer::gl_renderer::GLRenderer;
use renderer::camera::FpsCamera;
use cgmath::vec3;

use crate::renderer::camera::Camera;

/// The width of the window
const WINDOW_WIDTH: u32 = 1024 * 2;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768 * 2;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 60;

/// The fixed update target tim
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 1.0;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 0.1;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 0.5;

// Entry point
fn main() {
    // Create window
    let mut window = Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW - 500);

    // Create renderer
    let (win_width, win_height) = window.window.get_size();
    let mut renderer = GLRenderer::new(win_width, win_height);

    // The input state
    let mut key_state: Vec<bool> = vec![false; glfw::ffi::KEY_LAST as usize];

    // The game state
    let mut game_state = GameState::new();

    // The camera
    let mut camera = FpsCamera::new_with_pos_rot(vec3(0.0, 0.0, 10.0), 0.0, 0.0, CAM_LOOK_SPEED);

    // Fixed timestep - https://gafferongames.com/post/fix_your_timestep/
    let mut current_time = window.glfw.get_time();
    let mut sim_time = 0.0;
    let mut accumulator = 0.0;

    // Mouse movement
    let (mut mouse_x, mut mouse_y) = window.window.get_cursor_pos();

    window.window.set_cursor_mode(glfw::CursorMode::Disabled);

    // Start main loop
    while !window.window.should_close() {
        // Handle events
        for event in window.poll_events() {
            handle_window_event(&mut window, &mut renderer, &mut key_state, event);
        }

        // Fixed timestep
        let new_time = window.glfw.get_time();
        let frame_time = new_time - current_time;

        current_time = new_time;
        accumulator += frame_time;

        while accumulator >= FIXED_UPDATE_TIME {
            // Update game state
            game_state.time = sim_time as f32;

            // Mouse movement
            let (old_mouse_x, old_mouse_y) = (mouse_x, mouse_y);
            (mouse_x, mouse_y) = window.window.get_cursor_pos();
            let (mouse_dx, mouse_dy) = (mouse_x - old_mouse_x, mouse_y - old_mouse_y);
            
            // Update camera
            let cam_speed = match key_state[Key::LeftShift as usize] {
                false => CAM_MOVE_SPEED,
                true => CAM_MOVE_SPEED_FAST,
            };

            let forward_cam_movement = match (key_state[Key::W as usize], key_state[Key::S as usize]) {
                (true, false) => cam_speed,
                (false, true) => -cam_speed,
                _ => 0.0
            };

            let right_cam_movement = match (key_state[Key::A as usize], key_state[Key::D as usize]) {
                (true, false) => -cam_speed,
                (false, true) => cam_speed,
                _ => 0.0
            };

            camera.move_camera(forward_cam_movement, right_cam_movement, 0.0);
            camera.mouse_move(mouse_dx as f32, mouse_dy as f32);
            game_state.view_matrix = camera.get_view_matrix();

            // Consume accumulated time
            accumulator -= FIXED_UPDATE_TIME;
            sim_time += FIXED_UPDATE_TIME;
        }

        // Render
        renderer.render(game_state);
        window.window.swap_buffers();
    }
}

/// Handle events
fn handle_window_event(window: &mut Window, renderer: &mut GLRenderer, key_state: &mut [bool],
                       event: glfw::WindowEvent)
{
    match event {
        glfw::WindowEvent::FramebufferSize(width, height) => {
            renderer.set_viewport(width, height)
        }
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.window.set_should_close(true)
        }
        glfw::WindowEvent::Key(key, _, Action::Press, _) => {
            key_state[key as usize] = true;
        }
        glfw::WindowEvent::Key(key, _, Action::Release, _) => {
            key_state[key as usize] = false;
        }
        glfw::WindowEvent::MouseButton(_, _, _) => {
            println!("button 1");
        }
        _ => {
            println!("{:?}", event);
        }
    }
}

