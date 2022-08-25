use std::rc::Rc;
use std::cell::RefCell;
use cgmath::{Vector3, Vector4, vec4, Matrix4};

use super::LightType;
use super::gltf_transform::GltfTransform;

/// A light (KHR_PUNCTUAL_LIGHTS)
pub struct GltfLight {
    transform: Option<Rc<RefCell<GltfTransform>>>,
    light_type: LightType,
    color: Vector3<f32>,
    intensity: f32,
    range: Option<f32>,
    inner_cone_angle: Option<f32>,
    outer_cone_angle: Option<f32>
}

impl GltfLight {
    /// Create a new light
    pub fn new(transform: Option<Rc<RefCell<GltfTransform>>>, light_type: LightType, color: Vector3<f32>,
        intensity: f32, range: Option<f32>, inner_cone_angle: Option<f32>, outer_cone_angle: Option<f32>)
        -> Self
    {
        GltfLight {
            transform,
            light_type,
            color,
            intensity,
            range,
            inner_cone_angle,
            outer_cone_angle
        }
    }

    /// Get the light's world transform
    pub fn world_transform(&self) -> Option<Matrix4<f32>> {
        self.transform.as_ref().map(|t| t.borrow_mut().world_transform().clone())
    }

    /// Get the light's position
    pub fn light_pos(&self) -> Option<Vector3<f32>> {
        self.world_transform().map(|t| t.w.truncate())
    }

    /// Get the light's direction
    pub fn light_dir(&self) -> Option<Vector3<f32>> {
        const WORLD_FORWARD: Vector4<f32> = vec4(0.0, 0.0, -1.0, 0.0);
        self.world_transform().map(|t| (t * WORLD_FORWARD).truncate())
    }

    /// Get the light type
    pub fn light_type(&self) -> &LightType {
        &self.light_type
    }

    /// Get the light color
    pub fn color(&self) -> &Vector3<f32> {
        &self.color
    }

    /// Get the light intensity
    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    /// Get the light range
    pub fn range(&self) -> &Option<f32> {
        &self.range
    }

    /// Get the inner cone angle
    pub fn inner_cone_angle(&self) -> &Option<f32> {
        &self.inner_cone_angle
    }

    /// Get the outer cone angle
    pub fn outer_cone_angle(&self) -> &Option<f32> {
        &self.outer_cone_angle
    }
}
