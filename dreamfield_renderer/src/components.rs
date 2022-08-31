use bevy_ecs::prelude::Component;
use cgmath::Vector3;
pub use crate::camera::{Camera, FpsCamera};

/// A component for representing 3d positions
#[derive(Component)]
pub struct Position {
    pub pos: Vector3<f32>
}

impl Position {
    pub fn new(pos: Vector3<f32>) -> Self {
        Self {
            pos
        }
    }
}

/// A component for representing visible models
#[derive(Component)]
pub struct Visual {
    pub model_name: String,
    pub tessellate: bool
}

impl Visual {
    pub fn new(model_name: &str, tessellate: bool) -> Self {
        Self {
            model_name: model_name.to_string(),
            tessellate
        }
    }
}

/// A component for representing a camera
#[derive(Component)]
pub struct PlayerCamera {
    pub camera: FpsCamera
}

impl PlayerCamera {
    pub fn new(pos: Vector3<f32>, pitch: f32, yaw: f32) -> Self {
        PlayerCamera {
            camera: FpsCamera::new_with_pos_rot(pos, pitch, yaw, 0.0)
        }
    }
}

