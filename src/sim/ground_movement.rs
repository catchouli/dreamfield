use bevy_ecs::prelude::Entity;
use cgmath::{Vector3, vec3, InnerSpace};

use dreamfield_system::world::{WorldChunkManager, world_collision::{SpherecastResult, WorldCollision}};

/// How far the ground snap sweeps down below the entity, in metres
const GROUND_SNAP_DISTANCE: f32 = 3.0;

/// How far above the entity's sphere the ground probe starts, so slightly embedded entities
/// still find the ground below them
const GROUND_PROBE_MARGIN: f32 = 0.1;

/// How flat the terrain needs to be relative to the movement direction to walk on it
const MIN_WALK_ALIGNMENT: f32 = 0.0001;

/// Move a ground-walking entity through the world, following the terrain like the player does.
/// The entity's position is at its feet, and it is moved along the ground plane by a sphere of
/// the given radius, sliding along walls it hits and snapping to the ground below it.
pub fn move_on_ground(collision: &mut WorldCollision, world: &mut WorldChunkManager, entity: Entity,
    pos: Vector3<f32>, direction: Vector3<f32>, speed: f32, radius: f32, time_delta: f32) -> Vector3<f32>
{
    let center = pos + vec3(0.0, radius, 0.0);

    // Find the ground plane under the entity, so we can follow the terrain
    let (_ground_point, ground_normal) = match sweep(collision, world, entity,
        center + vec3(0.0, GROUND_PROBE_MARGIN, 0.0),
        vec3(0.0, -(GROUND_SNAP_DISTANCE + GROUND_PROBE_MARGIN), 0.0), radius)
    {
        Some(hit) => (*hit.point(), *hit.normal()),
        // If there's no ground below us, don't move, so we can't walk off into the void
        None => return pos,
    };

    // Project the movement direction onto the ground plane, so we follow slopes up and down
    let flat_direction = vec3(direction.x, 0.0, direction.z);
    let direction = flat_direction - ground_normal * flat_direction.dot(ground_normal);
    if direction.magnitude2() < MIN_WALK_ALIGNMENT {
        return pos;
    }
    let direction = direction.normalize();

    // Sweep through the world, sliding along whatever we hit
    let velocity = direction * speed * time_delta;
    let mut new_center = center + velocity;

    if let Some(hit) = sweep(collision, world, entity, center, velocity, radius) {
        let hit_normal = *hit.normal();
        let moved = velocity * hit.toi();
        let rest = velocity - moved;
        let rest = rest - hit_normal * rest.dot(hit_normal);

        new_center = center + moved + rest;
    }

    // Snap to the ground below the new position
    match sweep(collision, world, entity, new_center + vec3(0.0, GROUND_PROBE_MARGIN, 0.0),
        vec3(0.0, -(GROUND_SNAP_DISTANCE + GROUND_PROBE_MARGIN), 0.0), radius)
    {
        Some(hit) => vec3(new_center.x, hit.point().y, new_center.z),
        None => new_center - vec3(0.0, radius, 0.0),
    }
}

/// Sweep a sphere through the world
fn sweep(collision: &mut WorldCollision, world: &mut WorldChunkManager, entity: Entity,
    start: Vector3<f32>, velocity: Vector3<f32>, radius: f32) -> Option<SpherecastResult>
{
    collision.sweep_sphere(world, start, velocity, vec3(radius, radius, radius), Some(entity))
}
