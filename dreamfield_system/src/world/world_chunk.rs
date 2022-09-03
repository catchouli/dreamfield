use cgmath::Vector3;
use speedy::{Readable, Writable};
use super::aabb::Aabb;

/// World chunk size
pub const CHUNK_SIZE: f32 = 10.0;

// Stride for world meshes is pos (3) + normals (3) + uv (2) + color (4)
// We could split these into separate buffers since this part only needs positions
pub const VERTEX_STRIDE: usize = 3 + 3 + 2 + 4;

// For indices it's just 3 because they're triangles
pub const INDEX_STRIDE: usize = 3;

/// Type for chunk indexes
pub type ChunkIndex = (i32, i32);

/// A single world chunk
#[derive(Readable, Writable)]
pub struct WorldChunk {
    aabb: Aabb,
    meshes: Vec<WorldChunkMesh>
}

impl WorldChunk {
    /// Create a new WorldChunk with no meshes and an empty aabb
    pub fn new() -> Self {
        Self {
            aabb: Aabb::new(),
            meshes: Vec::new()
        }
    }

    /// Get the chunk's aabb
    pub fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    /// Get the chunk's meshes
    pub fn meshes(&self) -> &[WorldChunkMesh] {
        &self.meshes
    }

    /// Add a mesh to a world chunk
    pub fn add_mesh(&mut self, mesh: WorldChunkMesh) {
        self.aabb.expand_with_aabb(mesh.aabb());
        self.meshes.push(mesh);
    }

    /// Get the chunk filename for a given chunk index
    pub fn filename((x, z): ChunkIndex) -> String {
        format!("world_{}_{}.chunk", x, z)
    }

    /// Parse a chunk's filename back to a chunk index
    pub fn parse_filename(filename: &str) -> Option<ChunkIndex> {
        if filename.starts_with("world_") && filename.ends_with(".chunk") {
            let idx: Vec<i32> = filename[6..filename.len()-6].split("_").map(|s| s.parse::<i32>().unwrap()).collect();
            if idx.len() != 2 {
                panic!("Chunk filename split into more than two parts");
            }
            Some((idx[0], idx[1]))
        }
        else {
            None
        }
    }

    /// Get the chunk index for a point
    pub fn point_to_chunk_index(point: &Vector3<f32>) -> ChunkIndex {
        (f32::floor(point.x / CHUNK_SIZE as f32) as i32,
         f32::floor(point.z / CHUNK_SIZE as f32) as i32)
    }
}

/// A mesh within a world chunk
#[derive(Clone, Readable, Writable)]
pub struct WorldChunkMesh {
    aabb: Aabb,
    index: i32,
    vertices: Vec<f32>,
    indices: Vec<u16>
}

impl WorldChunkMesh {
    /// Create a new mesh
    pub fn new(aabb: Aabb, index: i32, vertices: Vec<f32>, indices: Vec<u16>) -> Self {
        Self {
            aabb,
            index,
            vertices,
            indices
        }
    }

    /// Get the aabb for this mesh
    pub fn aabb(&self) -> &Aabb {
        &self.aabb
    }

    /// Get the index of this mesh
    pub fn index(&self) -> i32 {
        self.index
    }

    /// Get the vertices of this mesh
    pub fn vertices(&self) -> &[f32] {
        &self.vertices
    }

    /// Get the indices of this mesh
    pub fn indices(&self) -> &[u16] {
        &self.indices
    }
}
