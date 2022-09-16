use bevy_ecs::prelude::Component;
use cgmath::{Vector3, vec3, Matrix3, SquareMatrix};

/// A component for representing an entities name
#[derive(Component)]
pub struct EntityName {
    pub name: String
}

impl EntityName {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

/// A component for representing object transforms
#[derive(Component)]
pub struct Transform {
    pub pos: Vector3<f32>,
    pub rot: Matrix3<f32>,
}

impl Transform {
    pub fn new(pos: Vector3<f32>, rot: Matrix3<f32>) -> Self {
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
            rot: Matrix3::identity()
        }
    }
}
