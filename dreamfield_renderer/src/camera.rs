use std::f32::consts::PI;

use cgmath::{Matrix4, SquareMatrix, Vector3, Zero, vec3, Rad, Quaternion, Rotation3};

/// The world forward direction
const WORLD_FORWARD: Vector3<f32> = vec3(0.0, 0.0, -1.0);

/// The world up direction
const WORLD_UP: Vector3<f32> = vec3(0.0, 1.0, 0.0);

/// The world right direction
const WORLD_RIGHT: Vector3<f32> = vec3(1.0, 0.0, 0.0);

/// The base scale for camera look speed
const BASE_CAM_LOOK_SPEED_SCALE: f32 = 0.002;

/// The min limit for pitch
const PITCH_MIN: f32 = -PI * 0.4;

/// The min limit for pitch
const PITCH_MAX: f32 = PI * 0.4;

/// A camera trait for providing something that can be used to obtain a view matrix
pub trait Camera {
    fn update(&mut self);
    fn get_view_matrix(&self) -> Matrix4<f32>;
}

/// A simple fps camera
#[derive(Copy, Clone)]
pub struct FpsCamera {
    look_speed: f32,
    pos: Vector3<f32>,
    pitch: f32,
    yaw: f32,
    matrices: FpsCameraMatrices,
    dirty: bool
}

#[derive(Copy, Clone)]
struct FpsCameraMatrices {
    cam: Matrix4<f32>,
    view: Matrix4<f32>,
    forward: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>
}

impl FpsCamera {
    /// Make a new fps camera at the origin
    pub fn new() -> Self {
        Self::new_with_pos_rot(Vector3::zero(), 0.0, 0.0, 1.0)
    }

    /// Make a new fps camera at the specified position and orientation
    pub fn new_with_pos_rot(pos: Vector3<f32>, pitch: f32, yaw: f32, look_speed: f32) -> Self {
        let mut cam = FpsCamera {
            look_speed,
            pos,
            pitch,
            yaw,
            matrices: FpsCameraMatrices {
                cam: Matrix4::identity(),
                view: Matrix4::identity(),
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
                right: vec3(-1.0, 0.0, 1.0)
            },
            dirty: true
        };

        cam.update_matrices();

        cam
    }

    /// Get forward vector
    pub fn forward(&self) -> &Vector3<f32> {
        &self.matrices.forward
    }

    /// Get up vector
    pub fn up(&self) -> &Vector3<f32> {
        &self.matrices.up
    }

    /// Get right vector
    pub fn right(&self) -> &Vector3<f32> {
        &self.matrices.right
    }

    /// Get the camera position
    pub fn pos(&self) -> &Vector3<f32> {
        &self.pos
    }

    /// Set the camera position
    pub fn set_pos(&mut self, new_pos: &Vector3<f32>) {
        self.pos = *new_pos;
        self.dirty = true;
    }

    /// Get the camera pitch and yaw
    pub fn get_pitch_yaw(&self) -> (f32, f32) {
        (self.pitch, self.yaw)
    }

    /// Set the camera pitch and yaw
    pub fn set_pitch_yaw(&mut self, pitch: f32, yaw: f32) {
        self.pitch = f32::clamp(pitch, PITCH_MIN, PITCH_MAX);
        self.yaw = yaw;
        self.dirty = true;
    }

    /// Move the camera in its axis
    pub fn move_camera(&mut self, forward: f32, right: f32, up: f32) {
        let new_pos = self.pos + forward * self.forward() + right * self.right() + up * self.right();
        self.set_pos(&new_pos)
    }

    /// Input mouse movement
    pub fn mouse_move(&mut self, dx: f32, dy: f32) {
        let (mut pitch, mut yaw) = self.get_pitch_yaw();
        pitch -= dy * self.look_speed * BASE_CAM_LOOK_SPEED_SCALE;
        yaw -= dx * self.look_speed * BASE_CAM_LOOK_SPEED_SCALE;
        self.set_pitch_yaw(pitch, yaw);
        self.dirty = true;
    }

    /// Update the matrices if necessary
    pub fn update_matrices(&mut self) {
        if self.dirty {
            self.dirty = false;

            let translation = cgmath::Matrix4::from_translation(self.pos);

            let pitch = Quaternion::from_axis_angle(WORLD_RIGHT, Rad(self.pitch));
            let yaw = Quaternion::from_axis_angle(WORLD_UP, Rad(self.yaw));
            let orientation = yaw * pitch;

            self.matrices.cam = translation * Matrix4::from(orientation);
            self.matrices.view = self.matrices.cam.invert().unwrap();
            self.matrices.forward = orientation * WORLD_FORWARD;
            self.matrices.up = orientation * WORLD_UP;
            self.matrices.right = orientation * WORLD_RIGHT;
        }
    }
}

impl Camera for FpsCamera {
    fn update(&mut self) {
        self.update_matrices();
    }

    fn get_view_matrix(&self) -> Matrix4<f32> {
        if self.dirty {
            panic!("Matrices out of date, call camera.update()");
        }
        self.matrices.view
    }
}
