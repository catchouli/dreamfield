use std::collections::HashMap;

use dreamfield_system::world::{world_chunk::{ChunkIndex, VERTEX_STRIDE, INDEX_STRIDE, WorldChunk}, WorldChunkManager, aabb::Aabb};
use cgmath::{Vector3, vec2, vec3};

use ncollide3d::bounding_volume::BoundingVolume;
use ncollide3d::shape::{TriMesh, Ball, CompositeShape, Shape};
use ncollide3d::math::{Point, Isometry, Translation};
use ncollide3d::query::Contact;
use ncollide3d::query::visitors::BoundingVolumeInterferencesCollector;

use super::intersection;

// TODO: might be worth trying fixed point, for authenticity, and to see if it's more stable
// TODO: or doubles.. if really necessary for stability. quake used floats though and was very
// stable.

/// A struct for storing spherecast hits
pub struct SpherecastResult {
    hit_toi: f32,
    hit_point: Vector3<f32>,
    hit_normal: Vector3<f32>
}

impl SpherecastResult {
    pub fn new(hit_toi: f32, hit_point: Vector3<f32>, hit_normal: Vector3<f32>) -> Self {
        Self {
            hit_toi,
            hit_point,
            hit_normal
        }
    }

    pub fn toi(&self) -> f32 {
        self.hit_toi
    }

    pub fn point(&self) -> &Vector3<f32> {
        &self.hit_point
    }

    pub fn normal(&self) -> &Vector3<f32> {
        &self.hit_normal
    }
}

/// The level collision service
pub struct LevelCollision {
    chunk_meshes: HashMap<ChunkIndex, Option<(Aabb, Vec<(Aabb, TriMesh<f32>)>)>>,
}

impl Default for LevelCollision {
    fn default() -> Self {
        Self {
            chunk_meshes: HashMap::new()
        }
    }
}

impl LevelCollision {
    /// Sweep a sphere out from start to end, returning an intersection result along with a time of impact
    pub fn sweep_sphere(&mut self, world: &mut WorldChunkManager, start: &Vector3<f32>, dir: &Vector3<f32>,
        length: f32, radius: f32) -> Option<SpherecastResult>
    {
        // Construct sphere
        let sphere = intersection::Sphere::new(*start, radius);

        // Construct an aabb for the sphere's path
        let mut sphere_path_aabb = Aabb::new();
        sphere_path_aabb.expand_with_point(&(start - vec3(radius, radius, radius)));
        sphere_path_aabb.expand_with_point(&(start + vec3(radius, radius, radius)));
        sphere_path_aabb.expand_with_point(&(start + dir * length - vec3(radius, radius, radius)));
        sphere_path_aabb.expand_with_point(&(start + dir * length + vec3(radius, radius, radius)));

        // Walk aabb bounds and find all chunks that intersect the spherecast
        let (min, max) = sphere_path_aabb.min_max().unwrap();
        let (chunk_min_x, chunk_min_z) = WorldChunk::point_to_chunk_index(min);
        let (chunk_max_x, chunk_max_z) = WorldChunk::point_to_chunk_index(max);

        //// We clip this toi by each intersection until we end up with no more intersections
        let mut clipped_toi = length;
        let mut closest_point_normal: Option<(Vector3<f32>,Vector3<f32>)> = None;

        for x in chunk_min_x..=chunk_max_x {
            for z in chunk_min_z..=chunk_max_z {
                if let Some((aabb, meshes)) = self.get_chunk_meshes(world, (x, z)) {
                    // Check if the sphere is going to intersect the abbb at all and return None if not
                    if intersection::toi_moving_sphere_aabb(&sphere, aabb, dir, clipped_toi).is_none() {
                        return None;
                    }

                    // Check each mesh in the chunk for intersections
                    for (mesh_aabb, mesh) in meshes.iter() {
                        if intersection::toi_moving_sphere_aabb(&sphere, mesh_aabb, dir, clipped_toi).is_none() {
                            continue;
                        }

                        for i in 0..mesh.nparts() {
                            let triangle = intersection::Triangle::from(mesh.triangle_at(i));

                            let res = intersection::toi_moving_sphere_triangle(&sphere, &triangle, dir, clipped_toi);
                            if let Some((t0, point, normal)) = res {
                                if t0 >= 0.0 && t0 < clipped_toi {
                                    clipped_toi = t0;
                                    closest_point_normal = Some((point, normal));
                                }
                            }
                        }
                    }
                }
            }
        }

        // If we have a closest_normal that means there was at least one intersection, otherwise there was none
        closest_point_normal.map(|(point, normal)| SpherecastResult::new(clipped_toi, point, normal))
    }

    /// Sphere contact with the level, returning true as soon as it finds an object that intersects with the sphere.
    /// Note that this isn't guaranteed to be the closest contact, as we stop when we find a single contact point.
    pub fn _sphere_contact_any(&mut self, world: &mut WorldChunkManager, center: &Vector3<f32>, radius: f32)
        -> Option<Contact<f32>>
    {
        let mut found_contact = None;

        self._sphere_contact_all(world, &center, radius, |c| {
            found_contact = Some(c);
            false
        });

        found_contact
    }

    /// Sphere contact with the level, calling a closure for every intersected triangle. Returning
    /// false from that closure will cause the intersection test to end without calling the
    /// closure again.
    pub fn _sphere_contact_all<F>(&mut self, world: &mut WorldChunkManager, center: &Vector3<f32>, radius: f32, mut f: F)
    where
        F: FnMut(Contact<f32>) -> bool
    {
        let chunk_index = WorldChunk::point_to_chunk_index_2d(&vec2(center.x, center.z));
        if let Some((chunk_aabb, meshes)) = self.get_chunk_meshes(world, chunk_index).as_ref() {
            // Check if the chunk aabb intersects the sphere
            if !chunk_aabb.intersects_sphere(&center, radius) {
                return;
            }

            // Calculate query parameters
            let ball = Ball::new(radius);
            let ball_transform = Isometry::from(Translation::new(center.x, center.y, center.z));
            let level_transform = Isometry::identity();

            for (aabb, mesh) in meshes.iter() {
                if !aabb.intersects_sphere(&center, radius) {
                    continue;
                }

                Self::_contact_trimesh_ball(&level_transform, mesh, &ball_transform, &ball, 0.0, |c| {
                    f(c)
                });
            }
        }
    }

    /// Get the meshes for a chunk, loading them if necessary from the world chunk manager
    fn get_chunk_meshes(&mut self, world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> &Option<(Aabb, Vec<(Aabb, TriMesh<f32>)>)>
    {
        self.chunk_meshes
            .entry(chunk_index)
            .or_insert_with(|| { Self::load_chunk_meshes(world, chunk_index) })
    }

    /// Load the meshes for a chunk from the world chunk manager
    fn load_chunk_meshes(world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> Option<(Aabb, Vec<(Aabb, TriMesh<f32>)>)>
    {
        world.get_or_load_chunk(chunk_index)
            .as_ref()
            .map(|chunk| {
                log::info!("Loading {} chunk meshes for chunk {}, {}", chunk.meshes().len(), chunk_index.0, chunk_index.1);

                let meshes = chunk.meshes().iter().map(|mesh| {
                    let mut points = Vec::with_capacity(mesh.vertices().len() / VERTEX_STRIDE);
                    let mut indices = Vec::with_capacity(mesh.indices().len() / INDEX_STRIDE);

                    assert!(mesh.vertices().len() % VERTEX_STRIDE == 0);
                    for v in mesh.vertices().chunks_exact(VERTEX_STRIDE) {
                        points.push(Point::new(v[0], v[1], v[2]));
                    }

                    assert!(mesh.indices().len() % INDEX_STRIDE == 0);
                    for i in mesh.indices().chunks(INDEX_STRIDE) {
                        indices.push(Point::new(i[0] as usize, i[1] as usize, i[2] as usize));
                    }

                    (mesh.aabb().clone(), TriMesh::new(points, indices, None))
                }).collect();

                (chunk.aabb().clone(), meshes)
            })
    }

    /// Best contact between a composite shape (`Mesh`, `Compound`) and any other shape.
    pub fn _contact_trimesh_ball<F: FnMut(Contact<f32>) -> bool> (
        m1: &Isometry<f32>,
        g1: &TriMesh<f32>,
        m2: &Isometry<f32>,
        g2: &Ball<f32>,
        prediction: f32,
        mut f: F,
    ) -> Option<Contact<f32>>
    {
        // Find new collisions
        let ls_m2 = m1.inverse() * m2.clone();
        let ls_aabb2 = g2.aabb(&ls_m2).loosened(prediction);

        let mut interferences = Vec::new();
        {
            let mut visitor = BoundingVolumeInterferencesCollector::new(&ls_aabb2, &mut interferences);
            g1.bvh().visit(&mut visitor);
        }

        for i in interferences.into_iter() {
            if let Some(c) = ncollide3d::query::contact(&m1, &g1.triangle_at(i), &m2, g2, prediction) {
                if !f(c) {
                    return Some(c);
                }
            }
        }

        None
    }
}
