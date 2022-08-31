use std::sync::Arc;

use bevy_ecs::prelude::Component;
use cgmath::Vector3;
pub use crate::camera::{Camera, FpsCamera};
use crate::gl_backend::GltfModel;

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
    pub tessellate: bool,
    pub cur_anim: Option<Animation>,

    // Internal: the model
    pub model: Option<Arc<GltfModel>>,

    /// Internal: the animation state
    pub anim_state: Option<AnimationState>
}

#[derive(Clone)]
pub enum Animation {
    Once(String),
    Loop(String)
}

impl Animation {
    pub fn name(&self) -> &str {
        match &self {
            Animation::Once(name) => &name,
            Animation::Loop(name) => &name
        }
    }
}

pub struct AnimationState {
    /// The current animation
    pub cur_anim: Animation,

    /// Time that the animation started
    pub anim_start: f32,

    /// The current animation time
    pub anim_time: f32
}

impl AnimationState {
    pub fn should_loop(&self) -> bool {
        match &self.cur_anim {
            Animation::Once(_) => false,
            Animation::Loop(_) => true
        }
    }
}

impl Visual {
    pub fn new(model_name: &str, tessellate: bool) -> Self {
        Self {
            model_name: model_name.to_string(),
            tessellate,
            cur_anim: None,
            model: None,
            anim_state: None
        }
    }

    pub fn new_with_anim(model_name: &str, tessellate: bool, anim: Animation) -> Self {
        Self {
            model_name: model_name.to_string(),
            tessellate,
            cur_anim: Some(anim),
            model: None,
            anim_state: None
        }
    }

    /// Update animation based on cur_anim, updating anim_state. Returns whether the animation
    /// needs to be updated.
    pub fn animate(&mut self, time: f32) -> bool {
        // Check if an animation is supposed to be playing
        if let Some(cur_anim) = &self.cur_anim {
            // Check if we currently have an animation in progress
            if let Some(anim_state) = &mut self.anim_state {
                // If it's not the same animation, start the new one
                if anim_state.cur_anim.name() != cur_anim.name() {
                    self.anim_state = Some(AnimationState {
                        cur_anim: cur_anim.clone(),
                        anim_start: time,
                        anim_time: 0.0
                    });
                    return true;
                }
                // Otherwise, update the animation time
                else {
                    let anim_time = time - anim_state.anim_start;
                    let anim_updated = anim_time != anim_state.anim_time;
                    anim_state.anim_time = anim_time;
                    return anim_updated;
                }
            }
            // If we don't, start it
            else {
                self.anim_state = Some(AnimationState {
                    cur_anim: cur_anim.clone(),
                    anim_start: time,
                    anim_time: 0.0
                });
                return true;
            }
        }
        // If there's no animation supposed to be playing, make sure the anim state reflects that
        else if self.anim_state.is_some() {
            self.anim_state = None;
            return true;
        }
        else {
            return false;
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

