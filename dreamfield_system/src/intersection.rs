mod intersection_tests;

use std::collections::HashSet;

use bevy_ecs::{prelude::{Component, Entity}, system::{ResMut, Query}, query::Changed};
use cgmath::Vector3;

pub use intersection_tests::*;

use crate::{world::{world_chunk::ChunkIndex, WorldChunkManager}, components::{EntityName, Transform}};

/// An ADT of collision shapes
#[derive(Debug, Clone)]
pub enum Shape {
    /// A spheroid with an offset and radiuses
    BoundingSpheroid(Vector3<f32>, Vector3<f32>),
    /// A bounding obx
    BoundingBox
}

/// A component for representing an entity's collider
#[derive(Component)]
pub struct Collider {
    /// The shape of the collider
    pub shape: Shape,
    /// The chunks the collider is currently in
    pub chunks_in: HashSet<ChunkIndex>,
}

impl Collider {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            chunks_in: Default::default(),
        }
    }
}

/// A system for updating the world chunks an entity is in
pub fn update_world_chunks_system(mut world: ResMut<WorldChunkManager>,
    mut query: Query<(Entity, &Transform, &mut Collider, Option<&EntityName>),
                      Changed<Transform>>)
{
    for (e, transform, mut collider, name) in query.iter_mut() {
        world.update_entity_location(e, transform, &mut collider, name);
    }
}
