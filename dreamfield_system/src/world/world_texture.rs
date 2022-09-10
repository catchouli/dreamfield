use std::io::Cursor;

use image::{DynamicImage, ImageBuffer};
use speedy::{Readable, Writable};

pub type TextureIndex = i32;

/// A single world texture
#[derive(Readable, Writable)]
pub struct WorldTexture {
    data: Vec<u8>,
    width: u32,
    height: u32,
    index: TextureIndex
}

impl WorldTexture {
    pub fn new(pixels: Vec<u8>, format: u32, width: u32, height: u32, index: TextureIndex) -> Self {
        let data = Self::pixels_to_png(index, &pixels, format, width, height);

        Self {
            data,
            width,
            height,
            index
        }
    }

    /// Get the pixels in rgba8 format
    pub fn pixels(&self) -> Vec<u8> {
        println!("getting pixels for world txture {}", self.index);
        Self::png_to_rgba(self.index, &self.data)
    }

    /// Get the width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the index
    pub fn index(&self) -> TextureIndex {
        self.index
    }

    /// Get the chunk filename for a given texture index
    pub fn filename(texture_index: TextureIndex) -> String {
        format!("texture_{}.texture", texture_index)
    }

    /// Convert a pixel buffer to a png
    fn pixels_to_png(index: TextureIndex, pixels: &Vec<u8>, format: u32, width: u32, height: u32) -> Vec<u8> {
        // Load image
        let image = match format {
            gl::RGB => ImageBuffer::from_raw(width, height, pixels.clone()).map(DynamicImage::ImageRgb8),
            gl::RGBA => ImageBuffer::from_raw(width, height, pixels.clone()).map(DynamicImage::ImageRgba8),
            _ => panic!("Unsupported image format {format} for world image {index}")
        }.expect(&format!("Failed to load world image {index}"));

        // Write image as png
        let mut bytes = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        image.write_to(&mut cursor, image::ImageOutputFormat::Png)
            .expect(&format!("Failed to convert world image {index} to png"));

        bytes
    }

    /// Convert a png buffer to rgba8 pixels
    fn png_to_rgba(index: TextureIndex, png: &Vec<u8>) -> Vec<u8> {
        image::io::Reader::new(Cursor::new(png))
            .with_guessed_format()
            .expect(&format!("Failed to read world image {index}"))
            .decode()
            .expect(&format!("Failed to decode world image {index}"))
            .into_rgba8()
            .into_vec()
    }
}
