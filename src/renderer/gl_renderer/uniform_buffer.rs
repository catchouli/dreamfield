use super::bindings;
use super::to_std140::ToStd140;
use cgmath::{SquareMatrix, Matrix3, Matrix4};

/// Uniform buffer wrapper
pub struct UniformBuffer<T: Default> {
    ubo: u32,
    pub data: T
}

impl<T: Default> UniformBuffer<T> {
    /// Create a new UniformBuffer
    pub fn new() -> Self {
        let mut ubo: u32 = 0;
        
        unsafe {
            gl::CreateBuffers(1, &mut ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
            gl::BufferData(gl::UNIFORM_BUFFER,
                           std::mem::size_of::<T>() as isize,
                           std::ptr::null(),
                           gl::STATIC_DRAW);
        }

        UniformBuffer::<T> {
            ubo,
            data: Default::default()
        }
    }

    /// Bind this ubo to a binding
    pub fn bind(&self, binding: bindings::UniformBlockBinding) {
        unsafe { gl::BindBufferBase(gl::UNIFORM_BUFFER, binding as u32, self.ubo) }
    }

    /// Upload all data to the buffer
    pub fn upload(&self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,
                              0,
                              std::mem::size_of::<T>() as isize,
                              &self.data as *const T as *const std::ffi::c_void);
        }
    }
}

impl<T: Default> Drop for UniformBuffer<T> {
    /// Clean up opengl buffers
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.ubo) }
    }
}

/// Base render params
#[std140::repr_std140]
pub struct GlobalParams {
    pub sim_time: std140::float,
    pub mat_proj: std140::mat4x4,
    pub mat_view: std140::mat4x4
}

impl Default for GlobalParams {
    fn default() -> Self {
        GlobalParams {
            sim_time: (0.0).to_std140(),
            mat_proj: cgmath::Matrix4::identity().to_std140(),
            mat_view: Matrix4::identity().to_std140()
        }
    }
}

/// Object render params
#[std140::repr_std140]
pub struct ModelParams {
    pub mat_model: std140::mat4x4,
    pub mat_normal: std140::mat3x3
}

impl Default for ModelParams {
    fn default() -> Self {
        ModelParams {
            mat_model: Matrix4::identity().to_std140(),
            mat_normal: Matrix3::identity().to_std140()
        }
    }
}

/// Material render params
#[std140::repr_std140]
pub struct MaterialParams {
    pub has_base_color_texture: std140::boolean
}

impl Default for MaterialParams {
    fn default() -> Self {
        MaterialParams {
            has_base_color_texture: false.to_std140()
        }
    }
}

