pub mod world_builder;
pub mod world_chunk;
pub mod world_texture;
pub mod aabb;
pub mod wrapped_vectors;
pub mod world_collision;

use std::collections::{HashMap, HashSet};
use bevy_ecs::prelude::Entity;
use cgmath::Vector3;
use speedy::Readable;
use include_dir::Dir;
use world_chunk::{WorldChunk, ChunkIndex};
use world_texture::{WorldTexture, TextureIndex};

/// The size of a world chunk in each dimension
pub use world_chunk::CHUNK_SIZE;

use crate::{components::{Transform, EntityName}, intersection::{Collider, Shape}};

/// The world chunk manager
pub struct WorldChunkManager {
    world_chunks_dir: &'static Dir<'static>,
    loaded_chunks: HashMap<ChunkIndex, Option<WorldChunk>>,
    loaded_textures: HashMap<TextureIndex, Option<WorldTexture>>,
    entity_locations: HashMap<Entity, EntityLocation>,
    chunk_entities: HashMap<ChunkIndex, HashSet<Entity>>,
    empty_entity_hashset: HashSet<Entity>,
}

struct EntityLocation {
    entity_id: Entity,
    pos: Vector3<f32>,
    shape: Shape
}

impl WorldChunkManager {
    /// Create new WorldChunkManager
    pub fn new(world_chunks_dir: &'static Dir<'static>) -> Self {
        Self {
            world_chunks_dir,
            loaded_chunks: HashMap::new(),
            loaded_textures: HashMap::new(),
            entity_locations: HashMap::new(),
            chunk_entities: HashMap::new(),
            empty_entity_hashset: HashSet::new(),
        }
    }

    /// Get the specified chunk, loading it if necessary
    pub fn get_or_load_chunk(&mut self, (x, z): ChunkIndex) -> &Option<WorldChunk> {
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

    /// Get the specified texture, loading it if necessary
    pub fn get_or_load_texture(&mut self, idx: TextureIndex) -> &Option<WorldTexture> {
        self.loaded_textures
            .entry(idx)
            .or_insert_with(|| {
                log::info!("Loading world texture {}", idx);
                let texture_filename = WorldTexture::filename(idx);
                if let Some(file) = self.world_chunks_dir.get_file(&texture_filename) {
                    let texture = WorldTexture::read_from_buffer(file.contents()).expect("Failed to load world texture");
                    Some(texture)
                }
                else {
                    log::info!("No such texture {}", idx);
                    None
                }
            })
    }

    /// Update a live entity's location in the world, for collision purposes
    pub fn update_entity_location(&mut self, entity_id: Entity, transform: &Transform, collider: &mut Collider,
        entity_name: Option<&EntityName>)
    {
        // Update entity location, adding it if we don't already have a record of it
        let entity_location = self.entity_locations
            .entry(entity_id)
            .or_insert_with(|| {
                EntityLocation {
                    entity_id,
                    pos: transform.pos,
                    shape: collider.shape.clone(),
                }
            });
        entity_location.pos = transform.pos;

        // Get the aabb of the collider
        let (pos_min, pos_max) = match collider.shape {
            Shape::BoundingSpheroid(offset, radius) => {
                let center = transform.pos + offset;
                (center - radius, center + radius)
            }
            _ => panic!("update_world_chunks_system: Unsupported shape {:?}", collider.shape)
        };

        // Get the min and max world chunk this entity can be intersecting
        let (chunk_min_x, chunk_min_y) = WorldChunk::point_to_chunk_index(&pos_min);
        let (chunk_max_x, chunk_max_y) = WorldChunk::point_to_chunk_index(&pos_max);
        
        // Remove entity from chunks it's no longer in
        collider.chunks_in.retain(|(x, y)| {
            let still_in_chunk = *x >= chunk_min_x && *x <= chunk_max_x && *y >= chunk_min_y && *y <= chunk_max_y;
            if !still_in_chunk {
                self.remove_entity_from_chunk(entity_id, (*x, *y));
                let entity_name = entity_name.as_ref().map(|n| n.name.as_str()).unwrap_or("no-name");
                log::info!("Entity {entity_name} ({entity_id:?}) left chunk {x}, {y}");
            }
            still_in_chunk
        });

        // Add any chunks that it's moved into
        for x in chunk_min_x..=chunk_max_x {
            for y in chunk_min_y..=chunk_max_y {
                if !collider.chunks_in.contains(&(x, y)) {
                    collider.chunks_in.insert((x, y));
                    self.add_entity_to_chunk(entity_id, (x, y));
                    let entity_name = entity_name.as_ref().map(|n| n.name.as_str()).unwrap_or("no-name");
                    log::info!("Entity {entity_name} ({entity_id:?}) entered chunk {x}, {y}");
                }
            }
        }
    }

    /// Add an entity to a chunk
    fn add_entity_to_chunk(&mut self, entity_id: Entity, chunk: ChunkIndex) {
        let chunk_entities = self.chunk_entities
            .entry(chunk)
            .or_insert_with(HashSet::new);

        chunk_entities.insert(entity_id);
    }

    /// Remove an entity from a chunk
    fn remove_entity_from_chunk(&mut self, entity_id: Entity, chunk: ChunkIndex) {
        let chunk_entities = self.chunk_entities
            .entry(chunk)
            .or_insert_with(HashSet::new);

        chunk_entities.remove(&entity_id);
    }

    /// Get entities in a chunk
    fn get_entities_in_chunk(&mut self, chunk: ChunkIndex) -> impl Iterator<Item=&EntityLocation> + '_ {
        self.chunk_entities
            .get(&chunk)
            .unwrap_or(&self.empty_entity_hashset)
            .iter()
            .map(|entity_id| {
                self.entity_locations.get(entity_id)
                    .expect("WorldChunkManager: Internal error: Live entity in chunk but found no entity location")
            })
    }
}
