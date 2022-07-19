use super::bindings;
use super::to_std140::ToStd140;
use cgmath::{SquareMatrix, Matrix3, Matrix4};
use dreamfield_macros::UniformSetters;
use dreamfield_traits::UniformSetters;
use rangemap::RangeSet;
use gl::types::*;

/// Uniform buffer wrapper
pub struct UniformBuffer<T: Default + UniformSetters + std140::Std140Struct> {
    ubo: u32,
    data: Vec<T>,
    modified_ranges: RangeSet<usize>
}

impl<T: Default + UniformSetters + std140::Std140Struct> UniformBuffer<T> {
    /// Create a new UniformBuffer with a single blocks
    pub fn new_single() -> Self {
        Self::new_multi(1)
    }

    /// Create a new UniformBuffer with count blocks
    pub fn new_multi(count: usize) -> Self {
        let mut ubo: u32 = 0;
        
        unsafe {
            gl::CreateBuffers(1, &mut ubo);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);
            gl::BufferData(gl::UNIFORM_BUFFER,
                           std::mem::size_of::<T>() as isize,
                           std::ptr::null(),
                           gl::STATIC_DRAW);
        }

        let mut vec: Vec<T> = Vec::with_capacity(count);
        for i in 0..count {
            vec.push(Default::default());
        }

        let uniform_buffer = UniformBuffer::<T> {
            ubo,
            data: vec,
            modified_ranges: RangeSet::new()
        };

        uniform_buffer.upload_all();

        uniform_buffer
    }

    /// Bind this ubo to a binding
    pub fn bind(&self, binding: bindings::UniformBlockBinding) {
        unsafe { gl::BindBufferBase(gl::UNIFORM_BUFFER, binding as u32, self.ubo) }
    }

    /// Bind an element of this ubo to a binding
    pub fn bind_n(&self, idx: isize, binding: bindings::UniformBlockBinding) {
        let stride = 4;//self.data_stride();
        let offset = 4;//idx * stride;
        println!("binding idx {idx}, stride={stride}, offset={offset}");
        println!("blah {}", gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT);
        unsafe { gl::BindBufferRange(gl::UNIFORM_BUFFER, binding as u32, self.ubo, offset, stride) }
    }

    /// Upload all data to the buffer
    pub fn upload_all(&self) {
        let stride = self.data_stride();
        println!("upload all {stride}");

        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,
                              0,
                              stride * (self.data.len() as isize),
                              self.data.as_ptr() as *const std::ffi::c_void);
        }
    }

    // Upload modified ranges to the buffer
    pub fn upload_changed(&mut self) {
        unsafe { gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo) }
        for range in self.modified_ranges.iter() {
            unsafe {
                // Exciting undefined behaviour
                let mut ptr_int = self.data.as_ptr() as usize;
                ptr_int += range.start;
                gl::BufferSubData(gl::UNIFORM_BUFFER,
                                  range.start as isize,
                                  (range.end - range.start) as isize,
                                  ptr_int as *const GLvoid);
            }
        }
        self.modified_ranges = RangeSet::new();
    }

    /// The offset between elements in data, coinciding with std140 alignment rules
    fn data_stride(&self) -> isize {
        if self.data.len() > 1 {
            // Exciting undefined behavior
            (&self.data[1] as *const T as isize) - (&self.data[0] as *const T as isize)
        }
        else {
            // If there's only on element we can't do that, but the stride is also not as relevant
            std::mem::size_of::<T>() as isize
        }

    }
}

impl<T: Default + UniformSetters + std140::Std140Struct> Drop for UniformBuffer<T> {
    /// Clean up opengl buffers
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.ubo) }
    }
}

/// Base render params
#[std140::repr_std140]
#[derive(UniformSetters)]
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
#[derive(UniformSetters)]
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
#[derive(UniformSetters)]
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

