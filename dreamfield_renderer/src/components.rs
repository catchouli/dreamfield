use std::sync::Arc;

use bevy_ecs::prelude::Component;
use cgmath::{Vector3, Quaternion, Matrix4, Vector2};
pub use crate::camera::{Camera, FpsCamera};
use crate::{gl_backend::{GltfModel, Texture, ShaderProgram}, resources::{ShaderManager, TextureManager}};

/// The renderer params

/// A component for representing 3d positions
#[derive(Component)]
pub struct Position {
    pub pos: Vector3<f32>,
    pub rot: Quaternion<f32>
}

impl Position {
    pub fn new(pos: Vector3<f32>, rot: Quaternion<f32>) -> Self {
        Self {
            pos,
            rot
        }
    }
}

/// A component for representing visible models
#[derive(Component)]
pub struct Visual {
    pub model_name: String,
    pub tessellate: bool,
    pub cur_anim: Option<Animation>,
    pub internal_model: Option<Arc<GltfModel>>,
    pub internal_anim_state: Option<AnimationState>
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
            internal_model: None,
            internal_anim_state: None
        }
    }

    pub fn new_with_anim(model_name: &str, tessellate: bool, anim: Animation) -> Self {
        Self {
            model_name: model_name.to_string(),
            tessellate,
            cur_anim: Some(anim),
            internal_model: None,
            internal_anim_state: None
        }
    }

    /// Update animation based on cur_anim, updating anim_state. Returns whether the animation
    /// needs to be updated.
    pub fn animate(&mut self, time: f32) -> bool {
        // Check if an animation is supposed to be playing
        if let Some(cur_anim) = &self.cur_anim {
            // Check if we currently have an animation in progress
            if let Some(anim_state) = &mut self.internal_anim_state {
                // If it's not the same animation, start the new one
                if anim_state.cur_anim.name() != cur_anim.name() {
                    self.internal_anim_state = Some(AnimationState {
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
                self.internal_anim_state = Some(AnimationState {
                    cur_anim: cur_anim.clone(),
                    anim_start: time,
                    anim_time: 0.0
                });
                return true;
            }
        }
        // If there's no animation supposed to be playing, make sure the anim state reflects that
        else if self.internal_anim_state.is_some() {
            self.internal_anim_state = None;
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
    pub proj: Matrix4<f32>,
    pub view: Matrix4<f32>,

    pub render_res: Vector2<f32>,
    pub render_aspect: f32,
    pub render_fov_rad: f32,

    pub fog_color: Vector3<f32>,
    pub fog_range: Vector2<f32>
}

/// A component for representing a pre- or post-processing effect, such as a skysphere
#[derive(Component)]
pub struct ScreenEffect {
    pub run_time: RunTime,
    shader: String,
    texture: Option<String>,
    shader_ref: Option<Arc<ShaderProgram>>,
    texture_ref: Option<Arc<Texture>>
}

#[derive(PartialEq)]
pub enum RunTime {
    PreScene,
    PostScene
}

impl ScreenEffect {
    pub fn new(run_time: RunTime, shader: &str, texture: Option<&str>) -> Self {
        Self {
            run_time,
            shader: shader.to_string(),
            texture: texture.map(|s| s.to_string()),
            shader_ref: None,
            texture_ref: None
        }
    }

    pub fn get_shader(&mut self, shaders: &mut ShaderManager) -> &Option<Arc<ShaderProgram>> {
        if !self.shader_ref.is_some() {
            let shader = shaders.get(&self.shader);

            match shader {
                Ok(shader) => {
                    self.shader_ref = Some(shader.clone());
                }
                Err(err) => {
                    log::error!("{}", err);
                }
            }
        }

        &self.shader_ref
    }

    pub fn get_texture(&mut self, textures: &mut TextureManager) -> &Option<Arc<Texture>> {
        if let Some(texture) = &self.texture {
            if !self.texture_ref.is_some() {
                let texture = textures.get(&texture);

                match texture {
                    Ok(texture) => {
                        self.texture_ref = Some(texture.clone());
                    }
                    Err(err) => {
                        log::error!("{}", err);
                    }
                }
            }
        }

        &self.texture_ref
    }
}
