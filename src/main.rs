pub mod renderer;
pub mod sim;
mod fixed_timestep;

use cgmath::vec3;
use glfw::{Action, Context, Key};
use dreamfield_renderer::gl_backend::glfw_system::Window;
use sim::{SimTime, PlayerCamera, PlayerMovement, Ball};
use sim::input::{InputState, InputName};
use sim::level_collision::LevelCollision;
use renderer::RendererSettings;
use bevy_ecs::prelude::*;
use bevy_ecs::world::World;
use fixed_timestep::FixedTimestep;

/// The width of the window
const WINDOW_WIDTH: u32 = 1024 * 2;

/// The height of the window
const WINDOW_HEIGHT: u32 = 768 * 2;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 15;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

/// Initialize bevy world
fn init_world() -> World {
    // Create bevy world
    let mut world = World::default();

    // Initialise renderer settings
    world.insert_resource(RendererSettings::with_window_size((WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)));

    // Register other resources
    world.insert_resource(InputState::new());
    world.insert_resource(SimTime::new(0.0, FIXED_UPDATE_TIME));
    world.insert_resource(LevelCollision::new(renderer::resources::MODEL_DEMO_SCENE));

    // Create player entity
    world.spawn()
        // Entrance to village
        .insert(PlayerCamera::new(vec3(-125.1, 5.8, 123.8), 0.063, 0.099))
        // Entrance to cathedral
        //.insert(PlayerCamera::new(vec3(-99.988, 6.567, 75.533), -0.0367, 0.8334))
        // In corridor, going out
        //.insert(PlayerCamera::new(vec3(-45.99, 5.75, 17.37), 0.163, 1.7323))
        // Looking at torch
        //.insert(PlayerCamera::new(vec3(-33.04357, 4.42999, 15.564), 0.563, 2.499))
        // Looking at corridor
        //.insert(PlayerCamera::new(vec3(5.2, 0.8, 12.8), 0.03, 2.0))
        // Default dungeon pos
        //.insert(PlayerCamera::new(vec3(0.0, 1.0, 10.0), -0.17, 0.0))
        // Going outside
        //.insert(PlayerCamera::new(vec3(-53.925, 5.8, 19.56), 0.097, 1.57))
        .insert(PlayerMovement::default());

    // Create ball entity
    world.spawn()
        .insert(Ball::default());

    world
}

/// Entry point
fn main() {
    // Initialise logging
    env_logger::init();
    log::info!("Welcome to Dreamfield!");

    // Create window
    let mut window = Window::new_with_context(WINDOW_WIDTH, WINDOW_HEIGHT, "Dreamfield", gl::DEBUG_SEVERITY_LOW - 500);

    // Initialise bevy ecs world
    let mut world = init_world();

    // Create update schedule
    let mut update_schedule = create_update_schedule();

    // Create render schedule
    // (this is separate from the update schedule, as we update at a fixed rate which is separate from the render)
    let mut render_schedule = create_render_schedule();

    // Set up fixed timestep
    let mut fixed_timestep = FixedTimestep::new(FIXED_UPDATE_TIME, window.glfw.get_time());

    // Mouse movement
    let (mut mouse_x, mut mouse_y) = window.window.get_cursor_pos();

    // Colemak mode for luci (hax)
    // TODO: make this more modular (a system?) and refactor it out
    let mut colemak_mode = false;

    // Start main loop
    while !window.window.should_close() {
        // Handle events
        for event in window.poll_events() {
            world.resource_scope(|world, mut input_state| {
                world.resource_scope(|_, mut render_settings| {
                    handle_window_event(&mut window, event, &mut input_state, &mut render_settings, &mut colemak_mode);
                });
            });
        }

        // Handle mouse movement
        world.resource_scope(|_, mut input_state| {
            (mouse_x, mouse_y) = handle_mouse_movement(&window, (mouse_x, mouse_y), &mut input_state);
        });

        // Update at fixed timestep
        fixed_timestep.update_actual_time(window.glfw.get_time());
        while fixed_timestep.should_update() {
            // Update sim time
            let mut sim_time_res: Mut<SimTime> = world.get_resource_mut().unwrap();
            sim_time_res.sim_time = fixed_timestep.sim_time();

            // Simulate game state
            update_schedule.run(&mut world);
        }

        // Render
        render_schedule.run(&mut world);
        window.window.swap_buffers();
    }
}

/// Create update schedule
fn create_update_schedule() -> Schedule {
    let mut update_schedule = Schedule::default();

    update_schedule.add_stage("sim", SystemStage::parallel()
        .with_system_set(sim::systems())
    );

    update_schedule
}

/// Create render schedule
fn create_render_schedule() -> Schedule {
    let mut render_schedule = Schedule::default();

    render_schedule.add_stage("render", SystemStage::single_threaded()
        .with_system_set(renderer::systems())
    );

    render_schedule
}

/// Handle events
fn handle_window_event(window: &mut Window, event: glfw::WindowEvent, input_state: &mut Mut<InputState>,
    renderer_settings: &mut Mut<RendererSettings>, colemak_mode: &mut bool)
{
    let input_map_func = match colemak_mode {
        true => map_game_inputs_colemak,
        false => map_game_inputs_default
    };

    match event {
        glfw::WindowEvent::FramebufferSize(width, height) => {
            renderer_settings.window_size = (width, height);
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
        glfw::WindowEvent::Key(Key::F2, _, Action::Press, _) => {
            renderer_settings.wireframe_enabled = !renderer_settings.wireframe_enabled;
        }
        glfw::WindowEvent::Key(Key::F9, _, Action::Press, _) => {
            *colemak_mode = !(*colemak_mode);
            log::info!("Colemak mode {}", if *colemak_mode { "enabled" } else { "disabled "});
        }
        glfw::WindowEvent::Key(key, _, Action::Press, _) => {
            if let Some(input) = input_map_func(key) {
                input_state.inputs[input as usize] = true;
            }
        }
        glfw::WindowEvent::Key(key, _, Action::Release, _) => {
            if let Some(input) = input_map_func(key) {
                input_state.inputs[input as usize] = false;
            }
        }
        _ => {}
    }
}

/// Map game inputs from the keyboard
fn map_game_inputs_default(key: Key) -> Option<InputName> {
    match key {
        Key::W => Some(InputName::CamForwards),
        Key::A => Some(InputName::CamStrafeLeft),
        Key::S => Some(InputName::CamBackwards),
        Key::D => Some(InputName::CamStrafeRight),
        Key::I => Some(InputName::CamLookUp),
        Key::J => Some(InputName::CamLookLeft),
        Key::K => Some(InputName::CamLookDown),
        Key::L => Some(InputName::CamLookRight),
        Key::Up => Some(InputName::CamLookUp),
        Key::Left => Some(InputName::CamLookLeft),
        Key::Down => Some(InputName::CamLookDown),
        Key::Right => Some(InputName::CamLookRight),
        Key::LeftShift => Some(InputName::CamSpeed),
        Key::Z => Some(InputName::Rewind),
        _ => None
    }
}

/// Map game inputs from colemak (hax)
fn map_game_inputs_colemak(key: Key) -> Option<InputName> {
    match key {
        Key::W => Some(InputName::CamForwards),
        Key::A => Some(InputName::CamStrafeLeft),
        Key::R => Some(InputName::CamBackwards),
        Key::S => Some(InputName::CamStrafeRight),
        Key::U => Some(InputName::CamLookUp),
        Key::N => Some(InputName::CamLookLeft),
        Key::E => Some(InputName::CamLookDown),
        Key::I => Some(InputName::CamLookRight),
        Key::Up => Some(InputName::CamLookUp),
        Key::Left => Some(InputName::CamLookLeft),
        Key::Down => Some(InputName::CamLookDown),
        Key::Right => Some(InputName::CamLookRight),
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
