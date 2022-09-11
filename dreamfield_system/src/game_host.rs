use std::time::Instant;

use bevy_ecs::{world::World, schedule::Schedule};
use glfw::{Action, Context, Key};
use crate::fixed_timestep::FixedTimestep;
use crate::resources::{SimTime, Diagnostics};
use crate::input::{InputState, InputName};
use crate::glfw_system::GlfwWindow;

use bevy_ecs::prelude::*;

/// The window settings resource
pub struct WindowSettings {
    pub window_size: (i32, i32),
    pub wireframe_enabled: bool
}

impl WindowSettings {
    pub fn with_window_size(window_size: (i32, i32)) -> Self {
        Self {
            window_size,
            wireframe_enabled: false
        }
    }
}

/// A game host that creates a window, and then runs updates at a fixed timestep,
/// while rendering as fast as it can (or at the user's vsync setting)  
pub struct GameHost {
    window: GlfwWindow,
    update_timestep: f64
}

impl GameHost {
    pub fn new(window_width: i32, window_height: i32, update_timestep: f64) -> Self {
        // Create window
        let gl_debug_level = gl::DEBUG_SEVERITY_LOW - 500;
        let window = GlfwWindow::new_with_context(window_width, window_height, "Dreamfield", gl_debug_level);

        Self {
            window,
            update_timestep
        }
    }

    pub fn run(&mut self, mut world: World, mut update_schedule: Schedule, mut render_schedule: Schedule) {
        // Set up fixed timestep
        let mut fixed_timestep = FixedTimestep::new(self.update_timestep, self.window.glfw.get_time());

        // Mouse movement
        let (mut mouse_x, mut mouse_y) = self.window.window.get_cursor_pos();

        // Colemak mode for luci (hax) until we support key rebinding
        let mut colemak_mode = false;

        // Start main loop
        while !self.window.window.should_close() {
            // Handle events
            for event in self.window.poll_events() {
                world.resource_scope(|world, mut input_state| {
                    world.resource_scope(|_, mut render_settings| {
                        Self::handle_window_event(&mut self.window, event, &mut input_state, &mut render_settings, &mut colemak_mode);
                    });
                });
            }

            // Handle mouse movement
            world.resource_scope(|_, mut input_state| {
                (mouse_x, mouse_y) = Self::handle_mouse_movement(&self.window, (mouse_x, mouse_y), &mut input_state);
            });

            // Update at fixed timestep
            fixed_timestep.update_actual_time(self.window.glfw.get_time());
            while fixed_timestep.should_update() {
                // Update sim time
                world.resource_scope(|_, mut sim_time: Mut<SimTime>| {
                    sim_time.sim_time = fixed_timestep.sim_time();
                });

                // Simulate game state
                let update_start = Instant::now();
                update_schedule.run(&mut world);
                let update_time = update_start.elapsed();

                // Update diagnostics
                world.resource_scope(|_, mut diagnostics: Mut<Diagnostics>| {
                    diagnostics.update_time = update_time;
                });

                // Save old input states, we do this after each update so that we don't have a
                // 'first input' in multiple updates.
                world.resource_scope(|_, mut input_state: Mut<InputState>| {
                    for i in 0..input_state.inputs.len() {
                        input_state.last_inputs[i] = input_state.inputs[i];
                    }
                });
            }

            // Render
            let render_start = Instant::now();
            render_schedule.run(&mut world);
            let render_time = render_start.elapsed();
            self.window.window.swap_buffers();

            // Update diagnostics
            world.resource_scope(|_, mut diagnostics: Mut<Diagnostics>| {
                diagnostics.render_time = render_time;
            });
        }
    }

    /// Handle events
    fn handle_window_event(window: &mut GlfwWindow, event: glfw::WindowEvent, input_state: &mut Mut<InputState>,
                           renderer_settings: &mut Mut<WindowSettings>, colemak_mode: &mut bool)
    {
        let input_map_func = match colemak_mode {
            true => Self::map_game_inputs_colemak,
            false => Self::map_game_inputs_default
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
            Key::LeftShift => Some(InputName::Run),
            Key::Space => Some(InputName::Jump),
            Key::U => Some(InputName::Debug),
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
            Key::LeftShift => Some(InputName::Run),
            Key::Space => Some(InputName::Jump),
            _ => None
        }
    }

    /// Handle mouse movement
    fn handle_mouse_movement(window: &GlfwWindow, (old_mouse_x, old_mouse_y): (f64, f64),
                             input_state: &mut InputState) -> (f64, f64)
    {
        let (mouse_x, mouse_y) = window.window.get_cursor_pos();
        let (mouse_dx, mouse_dy) = (mouse_x - old_mouse_x, mouse_y - old_mouse_y);

        input_state.mouse_diff = (mouse_dx, mouse_dy);

        (mouse_x, mouse_y)
    }
}
