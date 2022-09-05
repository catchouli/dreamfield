use speedy::{Readable, Writable};

pub type TextureIndex = i32;

/// A single world texture
#[derive(Readable, Writable)]
pub struct WorldTexture {
    pixels: Vec<u8>,
    format: u32,
    width: u32,
    height: u32,
    index: TextureIndex
}

impl WorldTexture {
    pub fn new(pixels: Vec<u8>, format: u32, width: u32, height: u32, index: TextureIndex) -> Self {
        Self {
            pixels,
            format,
            width,
            height,
            index
        }
    }

    /// Get the pixels
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Get the gl format
    pub fn format(&self) -> u32 {
        self.format
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
}
