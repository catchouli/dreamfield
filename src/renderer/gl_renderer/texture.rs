use std::path::Path;
use std::error::Error;
use gl::types::*;

/// A texture
pub struct Texture {
    id: u32
}

pub struct TextureParams {
    pub horz_wrap: u32,
    pub vert_wrap: u32,
    pub min_filter: u32,
    pub mag_filter: u32
}

impl Texture {
    /// Texture params with repeat wrapping and nearest filtering
    pub const NEAREST_WRAP: TextureParams = TextureParams {
        horz_wrap: gl::REPEAT,
        vert_wrap: gl::REPEAT,
        min_filter: gl::NEAREST,
        mag_filter: gl::NEAREST
    };

    /// Load a new texture from a file
    pub fn new_from_file(path: &str, params: TextureParams) -> Result<Texture, Box<dyn Error>> {
        let img = image::open(&Path::new(path))?;
        let width = img.width() as i32;
        let height = img.height() as i32;
        let data = img.into_bytes();

        Texture::new_from_buf(&data, width, height, gl::RGB, gl::RGB, params)
    }

    /// Load a new texture from a buffer
    pub fn new_from_buf(buf: &[u8], width: i32, height: i32, source_format: u32, dest_format: u32,
                        params: TextureParams) -> Result<Texture, Box<dyn Error>>
    {
        unsafe {
            let mut texture = 0;
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, params.horz_wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, params.vert_wrap as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, params.min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, params.mag_filter as i32);

            gl::TexImage2D(gl::TEXTURE_2D,
                           0,
                           dest_format as i32,
                           width,
                           height,
                           0,
                           source_format,
                           gl::UNSIGNED_BYTE,
                           &buf[0] as *const u8 as *const GLvoid);

            Ok(Texture { id: texture })
        }
    }

    /// Bind texture
    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.id)
        }
    }
}

