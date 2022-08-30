/// The renderer settings resource
pub struct RendererSettings {
    pub window_size: (i32, i32),
    pub wireframe_enabled: bool
}

impl RendererSettings {
    pub fn with_window_size(window_size: (i32, i32)) -> Self {
        RendererSettings {
            window_size,
            wireframe_enabled: false
        }
    }
}

