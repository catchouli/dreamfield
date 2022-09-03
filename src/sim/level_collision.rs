use std::collections::HashMap;

use dreamfield_system::world::{world_chunk::{ChunkIndex, VERTEX_STRIDE, INDEX_STRIDE, WorldChunk}, WorldChunkManager};
use ncollide3d::{shape::TriMesh, math::{Point, Isometry}, query::{Ray, RayCast, RayIntersection}};
use cgmath::Vector3;

pub struct LevelCollision {
    chunk_meshes: HashMap<ChunkIndex, Option<Vec<TriMesh<f32>>>>
}

impl Default for LevelCollision {
    fn default() -> Self {
        Self {
            chunk_meshes: HashMap::new()
        }
    }
}

impl LevelCollision {
    /// Raycast into the level
    pub fn raycast(&mut self, world: &mut WorldChunkManager, origin: &Vector3<f32>, direction: &Vector3<f32>,
        max_dist: f32) -> Option<f32>
    {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        // Raycast the current chunk's trimeshes
        // TODO: we probably need to check every chunk along the ray
        let chunk_index = WorldChunk::point_to_chunk_index(origin);
        if let Some(meshes) = self.get_chunk_meshes(world, chunk_index) {
            let mut nearest: Option<f32> = None;

            for mesh in meshes.iter() {
                let mesh_hit = mesh.toi_with_ray(&Isometry::identity(), &ray, max_dist, true);
                if let Some(hit) = mesh_hit {
                    if nearest.is_none() || hit < nearest.unwrap() {
                        nearest = mesh_hit;
                    }
                }
            }

            nearest
        }
        else {
            None
        }
    }

    /// Raycast into the level, and obtain a normal
    pub fn raycast_normal(&mut self, world: &mut WorldChunkManager, origin: &Vector3<f32>, direction: &Vector3<f32>,
        max_dist: f32) -> Option<RayIntersection<f32>>
    {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        // Raycast the current chunk's trimeshes
        // TODO: we probably need to check every chunk along the ray
        let chunk_index = WorldChunk::point_to_chunk_index(origin);
        if let Some(meshes) = self.get_chunk_meshes(world, chunk_index) {
            let mut nearest: Option<RayIntersection<f32>> = None;

            for mesh in meshes.iter() {
                let hit = mesh.toi_and_normal_with_ray(&Isometry::identity(), &ray, max_dist, true);
                if let Some(new_hit) = hit {
                    if nearest.is_none() || new_hit.toi < nearest.unwrap().toi {
                        nearest = hit;
                    }
                }
            }

            nearest
        }
        else {
            None
        }
    }

    /// Get the meshes for a chunk, loading them if necessary from the world chunk manager
    fn get_chunk_meshes(&mut self, world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> &Option<Vec<TriMesh<f32>>>
    {
        self.chunk_meshes
            .entry(chunk_index)
            .or_insert_with(|| { Self::load_chunk_meshes(world, chunk_index) })
    }

    /// Load the meshes for a chunk from the world chunk manager
    fn load_chunk_meshes(world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> Option<Vec<TriMesh<f32>>>
    {
        world.get_chunk(chunk_index)
            .as_ref()
            .map(|chunk| {
                log::info!("Loading {} chunk meshes for chunk {}, {}", chunk.meshes().len(), chunk_index.0, chunk_index.1);
                chunk.meshes().iter().map(|mesh| {
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

                    TriMesh::new(points, indices, None)
                }).collect()
            })
    }

    ///// Load level collision
    ///// TODO: lots of unsafe unwraps and stuff
    //fn load_level_collision(gltf_model: &[u8]) -> TriMesh<f32> {
    //    let (doc, buffer_data, _) = gltf::import_slice(gltf_model).unwrap();

    //    let mut points = Vec::new();
    //    let mut indices = Vec::new();

    //    for node in doc.nodes() {
    //        Self::load_level_geometry_recursive(&doc, &buffer_data, &node, &SquareMatrix::identity(), &mut points,
    //            &mut indices);
    //    }

    //    TriMesh::new(points, indices, None)
    //}

    ///// Load the level geometry into buffers recursively
    //fn load_level_geometry_recursive(doc: &gltf::Document, buffers: &Vec<gltf::buffer::Data>, node: &gltf::Node,
    //    parent_world_transform: &Matrix4<f32>, out_points: &mut Vec<Point<f32>>, out_indices: &mut Vec<Point<usize>>)
    //{
    //    // Calculate world transform
    //    let local_transform = cgmath::Matrix4::from(node.transform().matrix());
    //    let world_transform = parent_world_transform * local_transform;

    //    // Add drawable if this node has a mesh
    //    if let Some(mesh) = node.mesh() {
    //        let mesh = doc.meshes().nth(mesh.index()).expect("mesh");

    //        for prim in mesh.primitives() {
    //            let indices = prim.indices().map(|accessor| {
    //                // Note: we're not handling sparse accessors, hence the unwrap
    //                let buffer_view = accessor.view().unwrap();
    //                let buffer_index = buffer_view.buffer().index();

    //                let buffer = &buffers[buffer_index];

    //                if accessor.data_type() != gltf::accessor::DataType::U16 {
    //                    panic!("not u16 mesh indices: {:?}", accessor.data_type());
    //                }
    //                let data_type_size = std::mem::size_of::<u16>();

    //                let offset = buffer_view.offset();
    //                let length_bytes = buffer_view.length();
    //                let length_elements = length_bytes / data_type_size;

    //                let mut indices = vec![0; length_elements];
    //                let mut slice = &buffer[offset..offset+length_bytes];
    //                slice.read_u16_into::<LittleEndian>(&mut indices).expect("Failed!");

    //                indices
    //            });

    //            let vertices = prim.attributes()
    //                .find(|(attr_type, _)| *attr_type == Semantic::Positions)
    //                .map(|(_, accessor)| {
    //                    // Note: we're not handling sparse accessors, hence the unwrap
    //                    let buffer_view  = accessor.view().unwrap();
    //                    let buffer_index = buffer_view.buffer().index();

    //                    let buffer = &buffers[buffer_index];

    //                    let attrib_stride = buffer_view.stride().unwrap_or(0) as i32;
    //                    if attrib_stride != 0 {
    //                        panic!("unhandled");
    //                    }

    //                    // Assuming that it's always gl::FLOAT but I might be wrong
    //                    let data_size_type = std::mem::size_of::<f32>();

    //                    let offset = buffer_view.offset();
    //                    let length_bytes = buffer_view.length();
    //                    let length_elements = length_bytes / data_size_type;

    //                    let mut vertices = vec![0.0; length_elements];
    //                    let mut slice = &buffer[offset..offset+length_bytes];
    //                    slice.read_f32_into::<LittleEndian>(&mut vertices).expect("Failed!");

    //                    vertices
    //                });

    //            let indices = indices.unwrap();
    //            let vertices = vertices.unwrap();

    //            let existing_vertex_count = out_points.len();

    //            for i in (0..indices.len()).step_by(3) {
    //                out_indices.push(Point::new(indices[i] as usize + existing_vertex_count,
    //                                            indices[i+1] as usize + existing_vertex_count,
    //                                            indices[i+2] as usize + existing_vertex_count));
    //            }

    //            for i in (0..vertices.len()).step_by(3) {
    //                let v = world_transform * vec4(vertices[i], vertices[i+1], vertices[i+2], 1.0);
    //                out_points.push(Point::<f32>::new(v.x, v.y, v.z));
    //            }
    //        }
    //    }

    //    // Recurse into children
    //    for child in node.children() {
    //        Self::load_level_geometry_recursive(doc, buffers, &child, &world_transform, out_points, out_indices);
    //    }
    //}

}
