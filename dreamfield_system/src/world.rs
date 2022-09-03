pub mod world_builder;
pub mod world_chunk;
pub mod aabb;

use std::collections::HashMap;
use speedy::Readable;
use include_dir::Dir;
use world_chunk::{WorldChunk, ChunkIndex};

/// The size of a world chunk in each dimension
pub use world_chunk::CHUNK_SIZE;

/// The world chunk manager
pub struct WorldChunkManager {
    world_chunks_dir: &'static Dir<'static>,
    loaded_chunks: HashMap<ChunkIndex, Option<WorldChunk>>
}

impl WorldChunkManager {
    /// Create new WorldChunkManager
    pub fn new(world_chunks_dir: &'static Dir<'static>) -> Self {
        Self {
            world_chunks_dir,
            loaded_chunks: HashMap::new()
        }
    }

    /// Get the specified chunk, loading it if necessary
    pub fn get_chunk(&mut self, (x, z): ChunkIndex) -> &Option<WorldChunk> {
        self.loaded_chunks
            .entry((x, z))
            .or_insert_with(|| {
                log::info!("Loading world chunk {}, {}", x, z);
                let chunk_filename = WorldChunk::filename((x, z));
                if let Some(file) = self.world_chunks_dir.get_file(&chunk_filename) {
                    let chunk = WorldChunk::read_from_buffer(file.contents()).expect("Failed to load world chunk");
                    Some(chunk)
                }
                else {
                    log::info!("No such chunk {}, {}", x, z);
                    None
                }
            })
    }
}
