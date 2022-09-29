use super::bindings;
use super::TextureParams;

/// A framebuffer
pub struct Framebuffer
{
    color_tex: u32,
    depth_buffer: u32,
    framebuffer_object: u32
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, color_format: u32, texture_params: TextureParams) -> Self {
        // Create framebuffer object
        let mut framebuffer_object: u32 = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer_object);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer_object);
        }

        // Create color texture
        let mut color_tex: u32 = 0;

        unsafe {
            gl::GenTextures(1, &mut color_tex);
            gl::BindTexture(gl::TEXTURE_2D, color_tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, texture_params.horz_wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, texture_params.vert_wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, texture_params.min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, texture_params.mag_filter as i32);

            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           color_format as i32,
                           width,
                           height,
                           0,
                           gl::RGBA,
                           gl::UNSIGNED_BYTE,
                           std::ptr::null());

            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, color_tex, 0);
        }

        // Create depth renderbuffer
        let mut depth_buffer: u32 = 0;

        unsafe {
            gl::GenRenderbuffers(1, &mut depth_buffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, depth_buffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);

            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, depth_buffer);
        }

        // Check if the current configuration is supported
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer is not complete");
        }

        // Unbind fbo
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };

        Framebuffer {
            color_tex,
            depth_buffer,
            framebuffer_object
        }
    }

    pub fn bind_draw(&self) {
        unsafe { gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.framebuffer_object) }
    }

    pub fn bind_read(&self) {
        unsafe { gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_object) }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) }
    }

    pub fn bind_color_tex(&self, slot: bindings::TextureSlot) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
            gl::BindTexture(gl::TEXTURE_2D, self.color_tex)
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        log::debug!("Cleaning up fbo");
        unsafe {
            gl::DeleteTextures(1, &self.color_tex);
            gl::DeleteRenderbuffers(1, &self.depth_buffer);
            gl::DeleteFramebuffers(1, &self.framebuffer_object);
        }
    }
}
