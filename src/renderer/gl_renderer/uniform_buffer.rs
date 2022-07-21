use super::bindings;
use super::to_std140::{ToStd140, FromStd140};
use cgmath::{SquareMatrix, Matrix3, Matrix4, Matrix, vec3};
use dreamfield_macros::UniformSetters;
use dreamfield_traits::UniformSetters;
use rangemap::RangeSet;
use gl::types::*;
use super::lights::LIGHT_COUNT;

/// Uniform buffer wrapper
pub struct UniformBuffer<T: Default + UniformSetters> {
    ubo: u32,
    data: T,
    field_offsets: Vec<(usize, usize)>,
    modified_ranges: RangeSet<usize>
}

impl<T: Default + UniformSetters> UniformBuffer<T> {
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

        // Calculate field offsets and lengths for uniform setters
        let field_offsets = {
            let field_sizes: Vec<usize> = T::calculate_field_offsets();

            let mut cur_offset: usize = 0;
            let mut vec: Vec<(usize, usize)> = Vec::with_capacity(field_sizes.len());

            for field_size in field_sizes {
                vec.push((cur_offset, cur_offset + field_size));
                cur_offset += field_size;
            }

            vec
        };

        let uniform_buffer = UniformBuffer::<T> {
            ubo,
            data: Default::default(),
            field_offsets,
            modified_ranges: RangeSet::new()
        };

        uniform_buffer.upload_all();

        uniform_buffer
    }

    /// Bind this ubo to a binding
    pub fn bind(&mut self, binding: bindings::UniformBlockBinding) {
        self.upload_changed();
        unsafe { gl::BindBufferBase(gl::UNIFORM_BUFFER, binding as u32, self.ubo) }
    }

    /// Upload all data to the buffer
    pub fn upload_all(&self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            gl::BufferSubData(gl::UNIFORM_BUFFER,
                              0,
                              std::mem::size_of::<T>() as isize,
                              &self.data as *const T as *const std::ffi::c_void);
        }
    }

    // Upload modified ranges to the buffer
    pub fn upload_changed(&mut self) {
        unsafe { gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo) }
        for range in self.modified_ranges.iter() {
            unsafe {
                // Exciting undefined behaviour
                let mut ptr_int = &self.data as *const T as usize;
                ptr_int += range.start;
                gl::BufferSubData(gl::UNIFORM_BUFFER,
                                  range.start as isize,
                                  (range.end - range.start) as isize,
                                  ptr_int as *const GLvoid);
            }
        }
        self.modified_ranges = RangeSet::new();
    }
}

impl<T: Default + UniformSetters> Drop for UniformBuffer<T> {
    /// Clean up opengl buffers
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.ubo) }
    }
}

/// Base render params
#[std140::repr_std140]
#[derive(UniformSetters)]
pub struct GlobalParams {
    pub mat_proj: std140::mat4x4,
    pub mat_view: std140::mat4x4,
    pub mat_view_proj: std140::mat4x4,
    pub mat_view_proj_inv: std140::mat4x4,
    pub mat_model: std140::mat4x4,
    pub mat_model_view_proj: std140::mat4x4,
    pub mat_normal: std140::mat3x3,
    pub sim_time: std140::float,
    pub vp_aspect: std140::float
}

impl Default for GlobalParams {
    fn default() -> Self {
        GlobalParams {
            mat_proj: Matrix4::identity().to_std140(),
            mat_view: Matrix4::identity().to_std140(),
            mat_view_proj: Matrix4::identity().to_std140(),
            mat_view_proj_inv: Matrix4::identity().to_std140(),
            mat_model: Matrix4::identity().to_std140(),
            mat_model_view_proj: Matrix4::identity().to_std140(),
            mat_normal: Matrix3::identity().to_std140(),
            sim_time: (0.0).to_std140(),
            vp_aspect: (1.0).to_std140()
        }
    }
}

impl UniformBuffer<GlobalParams> {
    /// Set the view and projection matrices, and also derive any other matrices as needed. Note
    /// that the projection matrix must be set first.
    pub fn set_mat_view_derive(&mut self, view: &Matrix4<f32>) {
        let proj = self.data.mat_proj.from_std140();
        let view_proj = proj * view;
        let view_proj_inv = view_proj.invert().unwrap();

        self.set_mat_view(view);
        self.set_mat_view_proj(&view_proj);
        self.set_mat_view_proj_inv(&view_proj_inv);
    }

    /// Set the model matrix, and also derive any other matrices as needed. Note that the
    /// projection and view matrices must be set first.
    pub fn set_mat_model_derive(&mut self, model: &Matrix4<f32>) {
        let proj = self.data.mat_proj.from_std140();
        let view = self.data.mat_view.from_std140();
        let model_view_proj = proj * view * model;

        self.set_mat_model(model);
        self.set_mat_normal(&{
            // https://learnopengl.com/Lighting/Basic-lighting
            let v = model.invert().unwrap().transpose();
            Matrix3::from_cols(v.x.truncate(), v.y.truncate(), v.z.truncate())
        });
        self.set_mat_model_view_proj(&model_view_proj);
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

/// Light params
#[std140::repr_std140]
#[derive(UniformSetters)]
pub struct LightParams {
    pub ambient_light: std140::vec3,
    pub lights: std140::array<Light, 20>
}

#[std140::repr_std140]
#[derive(Copy, Clone)]
pub struct Light {
    pub enabled: std140::boolean,
    pub light_type: std140::int,

    pub intensity: std140::float,
    pub range: std140::float,

    pub inner_cone_angle: std140::float,
    pub outer_cone_angle: std140::float,

    pub color: std140::vec3,
    pub light_dir: std140::vec3,
    pub light_pos: std140::vec3
}

impl Default for LightParams {
    fn default() -> Self {
        let light_default = Default::default();
        LightParams {
            ambient_light: vec3(1.0, 1.0, 1.0).to_std140(),
            lights: [light_default; LIGHT_COUNT].to_std140()
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Light {
            enabled: false.to_std140(),
            light_type: 1.to_std140(),
            light_pos: vec3(0.0, 0.0, 0.0).to_std140(),
            light_dir: vec3(0.0, 0.0, 0.0).to_std140(),
            color: vec3(0.0, 0.0, 0.0).to_std140(),
            intensity: (0.0).to_std140(),
            range: (0.0).to_std140(),
            inner_cone_angle: (0.0).to_std140(),
            outer_cone_angle: (0.0).to_std140()
        }
    }
}
