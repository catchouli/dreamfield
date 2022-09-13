use std::collections::HashSet;

use bevy_ecs::{prelude::{Component, EventWriter}, system::{Local, Query, ResMut}};
use cgmath::vec3;

use crate::world::{world_chunk::{WorldChunkEntity, EntityId, WorldChunk}, WorldChunkManager};
use crate::components::Transform;

/// An event sent by the system that instructs the game to spawn an entity
pub struct EntitySpawnEvent {
    pub entity_info: WorldChunkEntity
}

/// A component that can be used to tag the player, so that entities in nearby world chunks get
/// spawned automatically when they come nearby
#[derive(Component)]
pub struct EntitySpawnRadius {
    pub radius: f32
}

impl EntitySpawnRadius {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

/// A local resource for the entity spawner system, so it can keep track of spawned entities
#[derive(Default)]
pub struct EntitySpawnResource {
    spawned_entities: HashSet<EntityId>
}

/// The entity spawner system. Watches for entities with an EntitySpawnRadius and a Transform (e.g.
/// the player or camera entity), and spawns entities in world chunks when it comes nearby
pub fn entity_spawner_system(mut local: Local<EntitySpawnResource>,
                             query: Query<(&Transform, &EntitySpawnRadius)>,
                             mut chunks: ResMut<WorldChunkManager>,
                             mut writer: EventWriter<EntitySpawnEvent>)
{
    for (transform, radius) in query.iter() {
        let radius = radius.radius;
        let min = transform.pos - vec3(radius, radius, radius);
        let max = transform.pos + vec3(radius, radius, radius);

        let (min_chunk_x, min_chunk_y) = WorldChunk::point_to_chunk_index(&min);
        let (max_chunk_x, max_chunk_y) = WorldChunk::point_to_chunk_index(&max);

        for x in min_chunk_x..=max_chunk_x {
            for y in min_chunk_y..=max_chunk_y {
                if let Some(chunk) = chunks.get_or_load_chunk((x, y)) {
                    for entity in chunk.entities().iter() {
                        let entity_id = entity.entity_id();
                        if local.spawned_entities.contains(&entity_id) {
                            continue;
                        }

                        log::info!("Spawning entity {} ({})", entity.object_id(), entity.entity_id());
                        writer.send(EntitySpawnEvent {
                            entity_info: entity.clone()
                        });
                        local.spawned_entities.insert(entity_id);
                    }
                }
            }
        }
    }
}
