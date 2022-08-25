use std::io::Cursor;
use std::path::Path;
use std::error::Error;
use gl::types::*;
use image::EncodableLayout;
use image::io::Reader;
use super::bindings;

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

        Texture::new_from_buf(&data, width, height, gl::RGB, gl::UNSIGNED_BYTE, gl::RGB, params)
    }

    /// Load a new texture from an image in a buffer
    pub fn new_from_image_buf(buf: &[u8], params: TextureParams) -> Result<Texture, Box<dyn Error>> {
        let img = Reader::new(Cursor::new(buf)).with_guessed_format()?.decode()?;
        let rgb_img = img.to_rgb8();

        let width = rgb_img.width() as i32;
        let height = rgb_img.height() as i32;
        let data = rgb_img.as_bytes();

        Texture::new_from_buf(&data, width, height, gl::RGB, gl::UNSIGNED_BYTE, gl::RGB, params)
    }

    /// Load a new texture from a buffer
    pub fn new_from_buf(buf: &[u8], width: i32, height: i32, source_format: u32, source_type: u32,
                        dest_format: u32, params: TextureParams) -> Result<Texture, Box<dyn Error>>
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
                           source_type,
                           &buf[0] as *const u8 as *const GLvoid);

            Ok(Texture { id: texture })
        }
    }

    /// Generate mipmaps
    pub fn gen_mipmaps(&self) {
        unsafe { gl::GenerateTextureMipmap(self.id) }
    }

    /// Bind texture
    pub fn bind(&self, slot: bindings::TextureSlot) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
            gl::BindTexture(gl::TEXTURE_2D, self.id)
        }
    }

    /// Unbind texture
    pub fn unbind(slot: bindings::TextureSlot) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot as u32);
            gl::BindTexture(gl::TEXTURE_2D, 0)
        }
    }

    /// Convert RGBA8 to RGBA5551
    pub fn convert_rgba8_to_rgba5551(buf: &[u8], out_vec: &mut Vec<u8>) {
        let input_bytes = buf.len();

        if (input_bytes % 4) != 0 {
            panic!("convert_rgba8_to_rgba5551: Input buffer needs to be a multiple of 4");
        }

        let output_bytes = input_bytes / 2;
        out_vec.resize(output_bytes, 0);

        for i in 0 .. buf.len() / 4 {
            let mut r = buf[i * 4] as u16;
            let mut g = buf[i * 4 + 1] as u16;
            let mut b = buf[i * 4 + 2] as u16;
            let mut a = buf[i * 4 + 3] as u16;

            // Convert from 0..255 to 0..31
            r = r * 31 / 255;
            g = g * 31 / 255;
            b = b * 31 / 255;

            // Alpha maps 0 to 0, and any other value to 1
            a = match a {
                0 => 0,
                _ => 1
            };

            // Now convert it back to 2 bytes
            r = r << 11;
            g = g << 6;
            b = b << 1;
            let converted = r | g | b | a;

            out_vec[i * 2 + 1] = (converted >> 8) as u8;
            out_vec[i * 2 + 0] = (converted & 0b11111111) as u8;
        }
    }

    /// Quantize a RGBA888 texture to a certain bit depth, leaving it as RGBA8888
    pub fn quantize_to_bit_depth(buf: &[u8], out_vec: &mut Vec<u8>, target_component_depth: u8) {
        if target_component_depth < 1 || target_component_depth > 8 {
            panic!("quantize_to_bit_depth: target_component_depth should be (1..8)");
        }

        if (buf.len() % 4) != 0 {
            panic!("quantize_to_bit_depth: Input buffer needs to be a multiple of 4");
        }

        out_vec.resize(buf.len(), 0);

        const DEFAULT_DOMAIN: i32 = 256;
        let target_domain = 1 << target_component_depth as i32;
        let multiplier = (target_domain as f32) / (DEFAULT_DOMAIN as f32);

        for i in 0 .. buf.len() / 4 {
            let mut r = buf[i * 4] as f32;
            let mut g = buf[i * 4 + 1] as f32;
            let mut b = buf[i * 4 + 2] as f32;
            let mut a = buf[i * 4 + 3] as f32;

            r = f32::floor(r * multiplier) / multiplier;
            g = f32::floor(g * multiplier) / multiplier;
            b = f32::floor(b * multiplier) / multiplier;
            a = f32::floor(a * multiplier) / multiplier;

            out_vec[i * 4] = r as u8;
            out_vec[i * 4 + 1] = g as u8;
            out_vec[i * 4 + 2] = b as u8;
            out_vec[i * 4 + 3] = a as u8;
        }
    }
}

