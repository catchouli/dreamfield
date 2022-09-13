use bevy_ecs::prelude::Component;
use cgmath::{Vector3, Quaternion, vec3};

/// A component for representing object transforms
#[derive(Component)]
pub struct Transform {
    pub pos: Vector3<f32>,
    pub rot: Quaternion<f32>,
}

impl Transform {
    pub fn new(pos: Vector3<f32>, rot: Quaternion<f32>) -> Self {
        Self {
            pos,
            rot,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: vec3(0.0, 0.0, 0.0),
            rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }
}
