use std::ffi::CStr;
use std::sync::mpsc::Receiver;
use gl::types::*;
use glfw::Context;

/// A window
pub struct GlfwWindow {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    pub events: Receiver<(f64, glfw::WindowEvent)>,
    mouse_captured: bool
}

impl GlfwWindow {
    /// Create a new window with an opengl context with the given width and height
    pub fn new_with_context(size: Option<(i32, i32)>, title: &str, debug_log_level: u32) -> GlfwWindow {
        log::info!("Creating window");

        // Initialise glfw
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

        // Get the initial window size
        let (width, height) = size.unwrap_or_else(|| {
            // If unspecified, use a percentage of the primary monitor size, or a default value
            let (mut width, mut height) = (1024, 768);
            glfw.with_primary_monitor(|_, monitor| {
                if let Some(monitor) = monitor {
                    if let Some(video_mode) = monitor.get_video_mode() {
                        // Set height to 3/4 of monitor height, and width to the corresponding 4:3
                        // resolution, as widescreen monitors are common and this is more likely to
                        // result in a sensible default.
                        height = video_mode.height as i32 * 3 / 4;
                        width = height * 4 / 3;
                    }
                }
            });
            (width, height)
        });

        // Create window and gl context
        let (mut window, events) = glfw.create_window(width as u32, height as u32, title, glfw::WindowMode::Windowed)
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

        GlfwWindow {
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
                gl::DebugMessageCallback(Some(GlfwWindow::handle_debug_message), debug_log_level as *const GLvoid);
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
