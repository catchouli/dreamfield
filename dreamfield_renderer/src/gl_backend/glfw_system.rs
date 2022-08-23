use std::ffi::CStr;
use std::sync::mpsc::Receiver;
use gl::types::*;
use glfw::Context;

/// A window
pub struct Window {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    pub events: Receiver<(f64, glfw::WindowEvent)>,
    mouse_captured: bool
}

impl Window {
    /// Create a new window with an opengl context with the given width and height
    pub fn new_with_context(width: u32, height: u32, title: &str, debug_log_level: u32) -> Window {
        log::info!("Creating window");

        // Initialise glfw
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        // Create window and gl context
        let (mut window, events) = glfw.create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_mouse_button_polling(true);
        window.set_focus_polling(true);
        window.make_current();

        // Load all gl functions
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        // Enable debug output
        Self::set_debug_log_level(debug_log_level);

        Window {
            glfw,
            window,
            events,
            mouse_captured: false
        }
    }

    /// Poll all events and return them as a list
    pub fn poll_events(&mut self) -> Vec<glfw::WindowEvent> {
        self.glfw.poll_events();
        glfw::flush_messages(&self.events).map(|(_, event)| event).collect()
    }

    /// Set mouse captured
    pub fn set_mouse_captured(&mut self, captured: bool) {
        if captured && !self.mouse_captured {
            log::info!("Capturing cursor");
            self.window.set_cursor_mode(glfw::CursorMode::Disabled)
        }
        else if self.mouse_captured {
            log::info!("Releasing cursor");
            self.window.set_cursor_mode(glfw::CursorMode::Normal)
        }

        self.mouse_captured = captured;
    }

    pub fn is_mouse_captured(&self) -> bool {
        self.mouse_captured
    }

    /// Set debug log level, 0 means no debugging
    fn set_debug_log_level(debug_log_level: u32) {
        unsafe {
            if debug_log_level != 0 {
                gl::Enable(gl::DEBUG_OUTPUT);
                gl::DebugMessageCallback(Some(Window::handle_debug_message), debug_log_level as *const GLvoid);
            }
        }
    }

    /// Handle debug messages and log them
    extern "system" fn handle_debug_message(_: u32, _: u32, id: u32, severity: u32, _: i32,
        message: *const i8, user_data: *mut GLvoid)
    {
        let min_severity: u32 = user_data as u32;

        if severity >= min_severity {
            let message_str = unsafe { CStr::from_ptr(message).to_str().unwrap() };
            log::warn!("[{id:#x}] {message_str}");
        }
    }
}
