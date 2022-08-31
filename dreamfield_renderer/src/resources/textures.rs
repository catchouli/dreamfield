use std::{collections::HashMap, sync::Arc};
use crate::gl_backend::{Texture, TextureParams};

// Texture manager
pub struct TextureManager {
    entries: HashMap<String, Arc<Texture>>
}

impl TextureManager {
    pub fn new_with_textures(sources: Vec<(&str, (&[u8], TextureParams, bool, Option<u8>))>) -> Self {
        let entries = sources.into_iter().map(|(name, (buf, params, srgb, downsample_bits))| {
            let texture = Texture::new_from_image_buf(buf, params, srgb, downsample_bits)
                .expect(&format!("Failed to load texture {}", name));
            (name.to_string(), Arc::new(texture))
        }).collect();

        Self {
            entries
        }
    }

    pub fn get(&self, name: &str) -> Result<&Arc<Texture>, String> {
        self.entries
            .get(name)
            .ok_or(format!("No such texture {}", name))
    }
}
