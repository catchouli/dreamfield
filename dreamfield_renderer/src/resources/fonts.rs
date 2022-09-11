use std::{collections::HashMap, io::Cursor};
use crate::gl_backend::{Texture, TextureParams};
use serde::{Deserialize, Serialize};

// Font manager
pub struct FontManager {
    textures: HashMap<String, Texture>,
    maps: HashMap<CharMapKey, CharMap>,
}

// The key for the font map (font name, font variant)
type CharMapKey = (String, String);

impl FontManager {
    pub fn new(sources: Vec<(&str, &[u8], &[u8])>) -> Self {
        let mut textures = HashMap::new();
        let mut maps = HashMap::new();

        for (name, tex_buf, map_buf) in sources {
            // Load texture
            let texture = Texture::new_from_image_buf(tex_buf, TextureParams::repeat_nearest(), true, None)
                .expect(&format!("Failed to load font texture for font {name}"));

            textures.insert(name.to_string(), texture);
            maps.extend(Self::load_map(name, map_buf));
        }

        Self {
            textures,
            maps
        }
    }

    /// Get a font's texture
    pub fn get_texture(&self, font_name: &str) -> Result<&Texture, String> {
        self.textures
            .get(font_name)
            .ok_or(format!("No such font {}", font_name))
    }

    /// Get a font's CharMap
    pub fn get_char_map(&self, font_name: &str, font_variant: &str) -> Result<&CharMap, String> {
        self.maps
            .get(&(font_name.to_string(), font_variant.to_string()))
            .ok_or(format!("No such font {}", font_name))
    }


    /// Load a character maps from a buffer, returns one map for each font variant in the csv file
    fn load_map(font_name: &str, map_buf: &[u8]) -> HashMap<CharMapKey, CharMap> {
        let mut maps = HashMap::new();

        let cursor = Cursor::new(map_buf);
        let mut reader = csv::Reader::from_reader(cursor);

        for result in reader.records() {
            // Parse record
            let record = result.expect("Failed to load char map");
            let record: CharMapEntry = record.deserialize(None).expect("Failed to parse char map entry");

            // Get destination CharMap (oof, the to_strings, they're needed beacuse of the entry api..)
            let key = (font_name.to_string(), record.font_variant.to_string());
            let map = maps.entry(key).or_insert_with(CharMap::new);

            map.add(record);
        }

        maps
    }
}

// The character map for a single font variant
pub struct CharMap {
    map: HashMap<char, CharMapEntry>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CharMapEntry {
    font_variant: String,
    unicode: u8,
    pub source_x: i32,
    pub source_y: i32,
    pub width: i32,
    pub height: i32,
}

impl CharMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn add(&mut self, entry: CharMapEntry) {
        self.map.insert(entry.unicode as char, entry);
    }

    pub fn get_entry(&self, character: char) -> Option<&CharMapEntry> {
        self.map.get(&character)
    }
}
