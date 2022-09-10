use std::collections::HashMap;

use dreamfield_system::world::{world_chunk::{ChunkIndex, VERTEX_STRIDE, INDEX_STRIDE, WorldChunk}, WorldChunkManager, aabb::Aabb};
use cgmath::{Vector3, vec3, ElementWise};

use crate::sim::intersection::Triangle;

use super::intersection;

/// A struct for storing spherecast hits
pub struct SpherecastResult {
    /// The time of impact from 0..1 along the velocity
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
    chunk_meshes: HashMap<ChunkIndex, Option<(Aabb, Vec<(Aabb, Vec<Triangle>)>)>>,
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
    pub fn sweep_sphere(&mut self, world: &mut WorldChunkManager, start: Vector3<f32>, velocity: Vector3<f32>,
        radius: f32) -> Option<SpherecastResult>
    {
        // Apply change of basis from R3 to ellipsoid space
        let cbm = vec3(1.0 / radius, 1.0 / radius, 1.0 / radius);

        let start = start.mul_element_wise(cbm);
        let velocity = velocity.mul_element_wise(cbm);

        // Perform intersection
        let mut result = self.sweep_unit_sphere(world, start, velocity, cbm);

        // Convert results back to R3
        if let Some(result) = &mut result {
            // Normal doesn't need to be transformed for a sphere
            result.hit_point = result.hit_point.div_element_wise(cbm);
        }

        result
    }

    /// Sweep a unit sphere with a given change of basis matrix
    pub fn sweep_unit_sphere(&mut self, world: &mut WorldChunkManager, start: Vector3<f32>, velocity: Vector3<f32>,
        cbm: Vector3<f32>) -> Option<SpherecastResult>
    {
        let end = start + velocity;

        // Construct an aabb for the sphere's path
        let mut sphere_path_aabb = Aabb::new();
        sphere_path_aabb.expand_with_point(&(start - vec3(1.0, 1.0, 1.0)));
        sphere_path_aabb.expand_with_point(&(start + vec3(1.0, 1.0, 1.0)));
        sphere_path_aabb.expand_with_point(&(end - vec3(1.0, 1.0, 1.0)));
        sphere_path_aabb.expand_with_point(&(end + vec3(1.0, 1.0, 1.0)));

        // Walk aabb bounds and find all chunks that intersect the spherecast
        let (min, max) = sphere_path_aabb.min_max().unwrap();
        let (chunk_min_x, chunk_min_z) = WorldChunk::point_to_chunk_index(&min.div_element_wise(cbm));
        let (chunk_max_x, chunk_max_z) = WorldChunk::point_to_chunk_index(&max.div_element_wise(cbm));

        //// We clip this toi by each intersection until we end up with no more intersections
        let mut closest_intersection: Option<(f32, Vector3<f32>, Vector3<f32>)> = None;

        for x in chunk_min_x..=chunk_max_x {
            for z in chunk_min_z..=chunk_max_z {
                if let Some((_aabb, meshes)) = self.get_chunk_meshes(world, (x, z)) {
                    // Check if the sphere is going to intersect the abbb at all and return None if not
                    //if intersection::toi_unit_sphere_aabb(start, velocity, aabb).is_none() {
                    //    return None;
                    //}

                    // Check each mesh in the chunk for intersections
                    for (mesh_aabb, mesh) in meshes.iter() {
                        let (min, max) = mesh_aabb.min_max()
                            .map(|(min, max)| (min.mul_element_wise(cbm), max.mul_element_wise(cbm)))
                            .expect("Chunk mesh aabb had no min/max");

                        if !intersection::intersect_unit_sphere_aabb(start, velocity, min, max) {
                            continue;
                        }

                        for triangle in mesh.iter() {
                            let triangle = triangle.apply_cbm(cbm);

                            let res = intersection::toi_unit_sphere_triangle(start, velocity, &triangle);
                            if let Some((toi, _, _)) = res {
                                if let Some((closest_toi, _, _)) = closest_intersection {
                                    if toi >= 0.0 && toi < closest_toi {
                                        closest_intersection = res;
                                    }
                                }
                                else if toi >= 0.0 {
                                    closest_intersection = res;
                                }
                            }
                        }
                    }
                }
            }
        }

        // If we have a closest_intersection that means there was at least one intersection, otherwise there was none
        closest_intersection.map(|(toi, point, normal)| SpherecastResult::new(toi, point, normal))
    }

    /// Get the meshes for a chunk, loading them if necessary from the world chunk manager
    fn get_chunk_meshes(&mut self, world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> &Option<(Aabb, Vec<(Aabb, Vec<Triangle>)>)>
    {
        self.chunk_meshes
            .entry(chunk_index)
            .or_insert_with(|| { Self::load_chunk_meshes(world, chunk_index) })
    }

    /// Load the meshes for a chunk from the world chunk manager
    fn load_chunk_meshes(world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> Option<(Aabb, Vec<(Aabb, Vec<Triangle>)>)>
    {
        world.get_or_load_chunk(chunk_index)
            .as_ref()
            .map(|chunk| {
                log::info!("Loading {} chunk meshes for chunk {}, {}", chunk.meshes().len(), chunk_index.0, chunk_index.1);

                let meshes = chunk.meshes().iter().map(|mesh| {
                    let vertices = mesh.vertices();

                    let mut triangles = Vec::with_capacity(mesh.indices().len() / INDEX_STRIDE);

                    assert!(mesh.indices().len() % INDEX_STRIDE == 0);
                    for i in mesh.indices().chunks(INDEX_STRIDE) {
                        let i0 = i[0] as usize * VERTEX_STRIDE;
                        let i1 = i[1] as usize * VERTEX_STRIDE;
                        let i2 = i[2] as usize * VERTEX_STRIDE;

                        let v1 = &vertices[i0..i0+3];
                        let v2 = &vertices[i1..i1+3];
                        let v3 = &vertices[i2..i2+3];

                        triangles.push(Triangle::new(
                            vec3(v1[0], v1[1], v1[2]),
                            vec3(v2[0], v2[1], v2[2]),
                            vec3(v3[0], v3[1], v3[2])
                        ));
                    }

                    // TODO: we could make the builder generate collision specific meshes
                    (mesh.aabb().clone(), triangles)
                }).collect();

                (chunk.aabb().clone(), meshes)
            })
    }
}
