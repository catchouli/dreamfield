use super::bindings;

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
