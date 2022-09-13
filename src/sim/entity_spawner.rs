use bevy_ecs::{prelude::EventReader, system::Commands};
use cgmath::{Matrix4, Quaternion, Matrix3, Vector3};
use dreamfield_renderer::components::{Visual, Animation};
use dreamfield_system::{systems::entity_spawner::EntitySpawnEvent, components::Transform};

/// The entity spawner
pub fn entity_spawner(mut commands: Commands, mut reader: EventReader<EntitySpawnEvent>) {
    for event in reader.iter() {
        let (pos, rot) = decompose_transform(event.entity_info.world_transform());
        match event.entity_info.object_id() {
            "Elf" => {
                commands.spawn()
                    .insert(Transform::new(pos, rot))
                    .insert(Visual::new_with_anim("elf", false, Animation::Loop("Idle".to_string())));
            },
            "Minecart" => {
                commands.spawn()
                    .insert(Transform::new(pos, rot))
                    .insert(Visual::new("minecart", false));
            },
            _ => {
                log::warn!("Asked to spawn unknown entity: {:?}", event.entity_info);
            }
        }
        println!("spawn event: {:?}", event.entity_info);
    }
}

/// Decompose a transform into a position and orientation
fn decompose_transform(transform: &Matrix4<f32>) -> (Vector3<f32>, Quaternion<f32>) {
    let pos = transform.w.truncate();

    let rot_mat = Matrix3::from_cols(
        transform.x.truncate(),
        transform.y.truncate(),
        transform.z.truncate());
    let rot = Quaternion::from(rot_mat);

    (pos, rot)
}
