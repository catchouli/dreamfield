use super::world_chunk::{WorldChunk, WorldChunkMesh, ChunkIndex, CHUNK_SIZE, VERTEX_STRIDE, INDEX_STRIDE};
use super::aabb::Aabb;
use std::{collections::HashMap, path::Path};
use gltf::{import_slice, buffer, image, Semantic, Node};
use cgmath::{Matrix4, SquareMatrix, Vector3, vec4, vec3, vec2, InnerSpace};
use byteorder::{ReadBytesExt, LittleEndian};
use speedy::Writable;
use crate::build_log;

/// Include a world model at compile time, for use in build.rs to specify what models to build into
/// the world chunks
#[macro_export]
macro_rules! include_world_model {
    ($($tokens: tt)*) => {
        WorldModel::new($($tokens)*, include_bytes!($($tokens)*))
    }
}

/// Some bullshit log thing
/// https://github.com/rust-lang/cargo/issues/985#issuecomment-1071667472
#[macro_export]
macro_rules! build_log {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

/// A world model - i.e. a gltf model embedded in a build script that we should build into the game world
pub struct WorldModel {
    filename: &'static str,
    data: &'static [u8]
}

impl WorldModel {
    /// Create a new world model, use include_world_model! instead of calling this directly
    pub const fn new(filename: &'static str, data: &'static [u8]) -> Self {
        Self {
            filename,
            data
        }
    }
}

/// World builder
pub struct WorldBuilder {
    out_dir: &'static str,
    models: &'static [WorldModel],
    chunks: HashMap<ChunkIndex, WorldChunk>
}

impl WorldBuilder {
    /// Create a new world builder
    pub fn new(out_dir: &'static str, models: &'static [WorldModel]) -> Self {
        Self {
            out_dir,
            models,
            chunks: HashMap::new()
        }
    }

    // Build world models
    pub fn build_world_models(&mut self) {
        std::fs::remove_dir_all(self.out_dir).unwrap();
        std::fs::create_dir_all(self.out_dir).unwrap();

        // Tell cargo to rerun build.rs if any of the models change
        for model in self.models {
            println!("cargo:rerun-if-changed={}", model.filename);
        }

        let mut world_mesh_count: i32 = 0;

        // Iterate all models and add their meshes to the world chunks
        for model in self.models.iter() {
            build_log!("Processing model {}", model.filename);
            let (doc, buffer_data, image_data) = import_slice(model.data).unwrap();
            for scene in doc.scenes() {
                for n in scene.nodes() {
                    self.walk_nodes(&Matrix4::identity(), &n, &buffer_data, &image_data, &mut world_mesh_count);
                }
            }
        }

        for ((x, z), chunk) in self.chunks.iter() {
            let chunk_filename = WorldChunk::filename((*x, *z));
            let chunk_path = Path::new(self.out_dir).join(chunk_filename);
            chunk.write_to_file(chunk_path).unwrap();
        }
    }

    // Walk model hierarchy, adding geometry to chunks
    fn walk_nodes(&mut self, parent_world_transform: &Matrix4<f32>, node: &Node, buffers: &[buffer::Data],
        image_data: &[image::Data], world_mesh_count: &mut i32)
    {
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());
        let world_transform = parent_world_transform * local_transform;

        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                // Read indices for mesh
                let indices = prim.indices().map(|accessor| {
                    let buffer_view = accessor.view().unwrap();
                    let buffer_index = buffer_view.buffer().index();

                    let buffer = &buffers[buffer_index];

                    if accessor.data_type() != gltf::accessor::DataType::U16 {
                        panic!("not u16 mesh indices: {:?}", accessor.data_type());
                    }
                    let data_type_size = std::mem::size_of::<u16>();

                    let offset = buffer_view.offset();
                    let length_bytes = buffer_view.length();
                    let length_elements = length_bytes / data_type_size;

                    let mut indices = vec![0; length_elements];
                    let mut slice = &buffer[offset..offset+length_bytes];
                    slice.read_u16_into::<LittleEndian>(&mut indices).expect("Failed!");

                    indices
                });

                // Read vertex attributes for mesh
                let attribs = prim.attributes()
                    .map(|(semantic, accessor)| {
                        // Note: we're not handling sparse accessors, hence the unwrap
                        let buffer_view  = accessor.view().unwrap();
                        let buffer_index = buffer_view.buffer().index();

                        let buffer = &buffers[buffer_index];

                        let attrib_stride = buffer_view.stride().unwrap_or(0) as i32;
                        if attrib_stride != 0 {
                            panic!("unhandled");
                        }

                        // Assuming that it's always gl::FLOAT but I might be wrong
                        let data_size_type = std::mem::size_of::<f32>();

                        let offset = buffer_view.offset();
                        let length_bytes = buffer_view.length();
                        let length_elements = length_bytes / data_size_type;

                        let mut vertices = vec![0.0; length_elements];
                        let mut slice = &buffer[offset..offset+length_bytes];
                        slice.read_f32_into::<LittleEndian>(&mut vertices).expect("Failed!");

                        (semantic, vertices)
                    })
                    .collect::<HashMap<Semantic, Vec<f32>>>();

                // Enforce that we now have indices and vertices, if we want to support non-indexed
                // meshes (which are uncommon and blender's gltf exporter doesn't output them),
                // we'll have to change this
                let indices = indices.expect("Mesh must have indices");
                let positions = attribs.get(&Semantic::Positions).expect("Need positions");
                let normals = attribs.get(&Semantic::Normals).expect("Need normals");
                let uvs = attribs.get(&Semantic::TexCoords(0));
                let colors = attribs.get(&Semantic::Colors(0));

                let vertex_count = positions.len() / 3;

                assert!(positions.len() % 3 == 0);
                assert!(normals.len() / 3 == vertex_count);
                assert!(uvs.map(|uvs| uvs.len() / 2).unwrap_or(vertex_count) == vertex_count);
                assert!(colors.map(|colors| colors.len() / 4).unwrap_or(vertex_count) == vertex_count);

                // Transform vertices to world space, and calculate bounding box
                let vertex_components = 3 + 3 + 2 + 4;
                let mut vertices = Vec::with_capacity(vertex_components * vertex_count);
                let mut aabb = Aabb::new();

                for i in 0..vertex_count {
                    // Calculate world position
                    let local_pos = vec3(positions[i*3], positions[i*3+1], positions[i*3+2]);
                    let world_pos = (world_transform * vec4(local_pos.x, local_pos.y, local_pos.z, 1.0)).truncate();

                    // Expand mesh aabb
                    aabb.expand_with_point(&world_pos);

                    // Get normal
                    let normal = vec3(normals[i*3], normals[i*3+1], normals[i*3+2]);

                    // Get uv
                    let uv = match uvs {
                        Some(uvs) => vec2(uvs[i*2], uvs[i*2+1]),
                        None => vec2(0.0, 0.0)
                    };

                    // Get color
                    let color = match colors {
                        Some(colors) => vec4(colors[i*4], colors[i*4+1], colors[i*4+2], colors[i*4+3]),
                        None => vec4(1.0, 1.0, 1.0, 1.0)
                    };

                    // Add to vertex buffer
                    vertices.push(world_pos.x);
                    vertices.push(world_pos.y);
                    vertices.push(world_pos.z);
                    vertices.push(normal.x);
                    vertices.push(normal.y);
                    vertices.push(normal.z);
                    vertices.push(uv.x);
                    vertices.push(uv.y);
                    vertices.push(color.x);
                    vertices.push(color.y);
                    vertices.push(color.z);
                    vertices.push(color.w);
                };

                // Add the mesh to each chunk that the mesh overlaps
                if let Some((min, max)) = aabb.min_max().map(|(a, b)| (a.clone(), b.clone())) {
                    // Get the min and max
                    let (chunk_x_min, chunk_z_min) = WorldChunk::point_to_chunk_index(&min);
                    let (chunk_x_max, chunk_z_max) = WorldChunk::point_to_chunk_index(&max);

                    for x in chunk_x_min..=chunk_x_max {
                        for z in chunk_z_min..=chunk_z_max {
                            let chunk = self.get_chunk((x, z));

                            let chunk_bounds_min = vec3(x as f32 * CHUNK_SIZE, -1000.0, z as f32 * CHUNK_SIZE);
                            let chunk_bounds_max = vec3(chunk_bounds_min.x + CHUNK_SIZE, 1000.0, chunk_bounds_min.z + CHUNK_SIZE);

                            // Build vertex and index buffer for this chunk
                            let mut chunk_mesh_aabb = Aabb::new();
                            let mut chunk_mesh_vertices = Vec::new();
                            let mut chunk_mesh_indices = Vec::new();

                            // A map of original mesh indices to new mesh indices, since we're
                            // going to be filtering some of them out and need to remap them
                            let mut chunk_index_map: HashMap<u16, u16> = HashMap::new();

                            // Insert a vertex into the new mesh, returning the index of the vertex
                            let mut insert_chunk_mesh_vertex = |data: &[f32]| -> u16 {
                                let index = (chunk_mesh_vertices.len() / VERTEX_STRIDE) as u16;

                                for i in 0..VERTEX_STRIDE {
                                    chunk_mesh_vertices.push(data[i]);
                                }

                                index
                            };

                            // Iterate through each triangle in the original mesh and build a mesh of just triangles
                            // that intersect or are contained within the chunk's aabb
                            assert!(vertices.len() % VERTEX_STRIDE == 0);
                            assert!(indices.len() % INDEX_STRIDE == 0);
                            for tri in indices.chunks_exact(INDEX_STRIDE) {
                                let i1 = tri[0];
                                let i2 = tri[1];
                                let i3 = tri[2];

                                let i1_offset = i1 as usize * VERTEX_STRIDE;
                                let i2_offset = i2 as usize * VERTEX_STRIDE;
                                let i3_offset = i3 as usize * VERTEX_STRIDE;

                                let v1_data = &vertices[i1_offset .. i1_offset + VERTEX_STRIDE];
                                let v2_data = &vertices[i2_offset .. i2_offset + VERTEX_STRIDE];
                                let v3_data = &vertices[i3_offset .. i3_offset + VERTEX_STRIDE];

                                let v1 = vec3(v1_data[0], v1_data[1], v1_data[2]);
                                let v2 = vec3(v2_data[0], v2_data[1], v2_data[2]);
                                let v3 = vec3(v3_data[0], v3_data[1], v3_data[2]);

                                if Self::triangle_intersects_aabb(&chunk_bounds_min, &chunk_bounds_max, &v1, &v2, &v3) {
                                    // And then remap each index into a new place in the vertex buffer.
                                    let i1 = *chunk_index_map
                                        .entry(i1)
                                        .or_insert_with(|| { insert_chunk_mesh_vertex(v1_data) });

                                    let i2 = *chunk_index_map
                                        .entry(i2)
                                        .or_insert_with(|| { insert_chunk_mesh_vertex(v2_data) });

                                    let i3 = *chunk_index_map
                                        .entry(i3)
                                        .or_insert_with(|| { insert_chunk_mesh_vertex(v3_data) });

                                    chunk_mesh_indices.push(i1);
                                    chunk_mesh_indices.push(i2);
                                    chunk_mesh_indices.push(i3);

                                    chunk_mesh_aabb.expand_with_point(&v1);
                                    chunk_mesh_aabb.expand_with_point(&v2);
                                    chunk_mesh_aabb.expand_with_point(&v3);
                                }
                            }

                            // Create the mesh
                            assert!(chunk_mesh_indices.len() % INDEX_STRIDE == 0);
                            assert!(chunk_mesh_vertices.len() % VERTEX_STRIDE == 0);

                            if chunk_mesh_indices.len() > 0 {
                                //build_log!("inserting mesh with {} indices and {} vertices", chunk_mesh_indices.len(), chunk_mesh_vertices.len());

                                let mesh = WorldChunkMesh::new(chunk_mesh_aabb.clone(), *world_mesh_count,
                                    chunk_mesh_vertices, chunk_mesh_indices);
                                *world_mesh_count += 1;

                                chunk.add_mesh(mesh.clone());
                            }
                        }
                    }
                }
            }
        }

        for child in node.children() {
            self.walk_nodes(&world_transform, &child, &buffers, &image_data, world_mesh_count);
        }
    }

    /// Check whether a triangle intersects an aabb
    /// https://gdbooks.gitbooks.io/3dcollisions/content/Chapter4/aabb-triangle.html
    fn triangle_intersects_aabb(aabb_min: &Vector3<f32>, aabb_max: &Vector3<f32>,
        a: &Vector3<f32>, b: &Vector3<f32>, c: &Vector3<f32>) -> bool
    {
        // Convert AABB to center-extents form
        let center = 0.5 * aabb_min + 0.5 * aabb_max;
        let extents = 0.5 * aabb_max - 0.5 * aabb_min;

        // Translate triangle as conceptually moving aabb to origin
        let v0 = a - center;
        let v1 = b - center;
        let v2 = c - center;

        // Compute the edge vectors of the triangle
        let f0 = v1 - v0;
        let f1 = v2 - v1;
        let f2 = v0 - v2;

        // Compute face normals of the aabb (they're just axis aligned)
        let u0 = Vector3::new(1.0, 0.0, 0.0);
        let u1 = Vector3::new(0.0, 1.0, 0.0);
        let u2 = Vector3::new(0.0, 0.0, 1.0);

        // A helper function for doing SAT tests of a given axis against the triangle
        let axis_separated = |axis: Vector3<f32>| -> bool {
            let p0 = v0.dot(axis);
            let p1 = v1.dot(axis);
            let p2 = v2.dot(axis);

            let r = extents.x * f32::abs(u0.dot(axis)) +
                    extents.y * f32::abs(u1.dot(axis)) +
                    extents.z * f32::abs(u2.dot(axis));

            f32::max(-f32::max(p0, f32::max(p1, p2)), f32::min(p0, f32::min(p1, p2))) > r
        };

        // Do the sat tests for:
        // * The 9 axes separating the edges of the aabb and the triangle's edges
        // * The three face normals from the AABB
        // * The axis of the face normal of the triangle
        let any_separated =
            axis_separated(u0.cross(f0)) ||
            axis_separated(u0.cross(f1)) ||
            axis_separated(u0.cross(f2)) ||
            axis_separated(u1.cross(f0)) ||
            axis_separated(u1.cross(f1)) ||
            axis_separated(u1.cross(f2)) ||
            axis_separated(u2.cross(f0)) ||
            axis_separated(u2.cross(f1)) ||
            axis_separated(u2.cross(f2)) ||
            axis_separated(u0) ||
            axis_separated(u1) ||
            axis_separated(u2) ||
            axis_separated(f0.cross(f1));

        // If any axis was separated, the triangle did not intersect the aabb
        !any_separated
    }

    /// Get the chunk for a given chunk index
    fn get_chunk(&mut self, idx: ChunkIndex) -> &mut WorldChunk {
        self.chunks
            .entry(idx)
            .or_insert(WorldChunk::new())
    }
}
