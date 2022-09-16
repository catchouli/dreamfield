use bevy_ecs::{prelude::EventReader, system::Commands};
use cgmath::{Matrix4, Matrix3, Vector3, vec3};
use dreamfield_renderer::components::{Visual, Animation};
use dreamfield_system::{systems::entity_spawner::EntitySpawnEvent, components::{Transform, EntityName}, intersection::{Collider, Shape}};

use super::minecart::Minecart;

/// The entity spawner
pub fn entity_spawner(mut commands: Commands, mut reader: EventReader<EntitySpawnEvent>) {
    for event in reader.iter() {
        let (pos, rot) = decompose_transform(event.entity_info.world_transform());
        match event.entity_info.object_id() {
            "Elf" => {
                commands.spawn()
                    .insert(Transform::new(pos, rot))
                    .insert(EntityName::new("Elf"))
                    .insert(Collider::new(Shape::BoundingSpheroid(vec3(0.0, 1.0, 0.0), vec3(0.25, 1.0, 0.25))))
                    .insert(Visual::new_with_anim("elf", false, Animation::Loop("Idle".to_string())));
            },
            "Minecart" => {
                let mut entity = commands.spawn();
                entity.insert(EntityName::new("Minecart"))
                      .insert(Transform::new(pos, rot))
                      .insert(Visual::new("minecart", false));

                if let Some(points) = event.entity_info.mesh() {
                    let track_points = points.iter().map(|p| p.as_vec().clone()).collect();
                    entity.insert(Minecart::new(track_points));
                }
                else {
                    log::warn!("Minecart has no points");
                }
            },
            _ => {
                log::warn!("Asked to spawn unknown entity: {:?}", event.entity_info);
            }
        }
    }
}

/// Decompose a transform into a position and orientation
fn decompose_transform(transform: &Matrix4<f32>) -> (Vector3<f32>, Matrix3<f32>) {
    let pos = transform.w.truncate();

    let rot = Matrix3::from_cols(
        transform.x.truncate(),
        transform.y.truncate(),
        transform.z.truncate());

    (pos, rot)
}
