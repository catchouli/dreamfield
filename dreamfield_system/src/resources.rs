use std::time::Duration;

pub use crate::input::{InputState, InputName};

/// The SimTime resource
pub struct SimTime {
    pub sim_time: f64,
    pub sim_time_delta: f64
}

impl SimTime {
    pub fn new(sim_time: f64, sim_time_delta: f64) -> Self {
        Self {
            sim_time,
            sim_time_delta
        }
    }
}

/// The Diagnostics resource (e.g. update and render time, etc)
pub struct Diagnostics {
    pub update_time: Duration,
    pub render_time: Duration,
    pub frame_time: Duration,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            update_time: Default::default(),
            render_time: Default::default(),
            frame_time: Default::default()
        }
    }
}
