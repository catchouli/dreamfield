use std::time::Duration;

use cgmath::{Vector3, Vector2, vec3, vec2};

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
    pub player_pos: Vector3<f32>,
    pub player_pitch_yaw: Vector2<f32>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            update_time: Default::default(),
            render_time: Default::default(),
            player_pos: vec3(0.0, 0.0, 0.0),
            player_pitch_yaw: vec2(0.0, 0.0),
        }
    }
}
