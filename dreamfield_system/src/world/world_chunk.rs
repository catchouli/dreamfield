use cgmath::{Vector3, Vector2, Matrix4};
use speedy::{Readable, Writable};
use super::{aabb::Aabb, wrapped_vectors::{WrappedVector4, WrappedVector3, WrappedMatrix4}};

/// World chunk size
pub const CHUNK_SIZE: f32 = 16.0;

// Stride for world meshes is pos (3) + normals (3) + uv (2) + color (4)
// We could split these into separate buffers since this part only needs positions
pub const VERTEX_STRIDE: usize = 3 + 3 + 2 + 4;

// For indices it's just 3 because they're triangles
pub const INDEX_STRIDE: usize = 3;

/// Type for chunk indexes
pub type ChunkIndex = (i32, i32);

/// Type for entity IDs
pub type EntityId = i32;

/// A single world chunk
#[derive(Readable, Writable, Debug)]
pub struct WorldChunk {
    aabb: Aabb,
    meshes: Vec<WorldChunkMesh>,
    instances: Vec<WorldChunkInstance>,
    entities: Vec<WorldChunkEntity>,
}

impl WorldChunk {
    /// Create a new WorldChunk with no meshes and an empty aabb
    pub fn new() -> Self {
        Self {
            aabb: Aabb::new(),
            meshes: Vec::new(),
            instances: Vec::new(),
            entities: Vec::new(),
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

    /// Get the chunk's instances
    pub fn instances(&self) -> &[WorldChunkInstance] {
        &self.instances
    }

    /// Get the chunk's entities
    pub fn entities(&self) -> &[WorldChunkEntity] {
        &self.entities
    }

    /// Add a mesh to a world chunk
    pub fn add_mesh(&mut self, mesh: WorldChunkMesh) {
        self.aabb.expand_with_aabb(mesh.aabb());
        self.meshes.push(mesh);
    }

    /// Add an instance to a world chunk
    pub fn add_instances(&mut self, instance: WorldChunkInstance) {
        for WrappedVector3(point) in instance.points().iter() {
            self.aabb.expand_with_point(point)
        }
        self.instances.push(instance);
    }

    /// Add an entity to a world chunk
    pub fn add_entity(&mut self, entity: WorldChunkEntity) {
        self.entities.push(entity);
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

    /// Get the chunk index for a 2d point
    pub fn point_to_chunk_index_2d(point: &Vector2<f32>) -> ChunkIndex {
        (f32::floor(point.x / CHUNK_SIZE as f32) as i32,
         f32::floor(point.y / CHUNK_SIZE as f32) as i32)
    }

    /// Get the chunk index for a 3d point
    pub fn point_to_chunk_index(point: &Vector3<f32>) -> ChunkIndex {
        (f32::floor(point.x / CHUNK_SIZE as f32) as i32,
         f32::floor(point.z / CHUNK_SIZE as f32) as i32)
    }
}

/// A mesh within a world chunk
#[derive(Clone, Readable, Writable, Debug)]
pub struct WorldChunkMesh {
    aabb: Aabb,
    index: i32,
    vertices: Vec<f32>,
    indices: Vec<u16>,
    material: Option<WorldChunkMaterial>
}

impl WorldChunkMesh {
    /// Create a new mesh
    pub fn new(aabb: Aabb, index: i32, vertices: Vec<f32>, indices: Vec<u16>, material: Option<WorldChunkMaterial>)
        -> Self
    {
        Self {
            aabb,
            index,
            vertices,
            indices,
            material
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

    /// Get the material of this mesh
    pub fn material(&self) -> &Option<WorldChunkMaterial> {
        &self.material
    }
}

/// A material within a world chunk
#[derive(Clone, Readable, Writable, Debug)]
pub struct WorldChunkMaterial {
    base_color: WrappedVector4,
    base_color_tex: Option<i32>
}

impl WorldChunkMaterial {
    pub fn new(base_color: WrappedVector4, base_color_tex: Option<i32>) -> Self {
        Self {
            base_color,
            base_color_tex
        }
    }

    pub fn base_color(&self) -> &WrappedVector4 {
        &self.base_color
    }

    pub fn base_color_tex(&self) -> &Option<i32> {
        &self.base_color_tex
    }
}

/// A mesh instanced in the world at various points
#[derive(Clone, Readable, Writable, Debug)]
pub struct WorldChunkInstance {
    mesh_name: String,
    points: Vec<WrappedVector3>
}

impl WorldChunkInstance {
    pub fn new(mesh_name: String, points: Vec<WrappedVector3>) -> Self {
        Self {
            mesh_name,
            points
        }
    }

    pub fn mesh_name(&self) -> &str {
        &self.mesh_name
    }

    pub fn points(&self) -> &Vec<WrappedVector3> {
        &self.points
    }
}

/// An entity to be spawned in if the player is near this chunk
#[derive(Clone, Readable, Writable, Debug)]
pub struct WorldChunkEntity {
    /// The unique ID for this entity that allows us to track if we've already spawned it
    entity_id: EntityId,
    /// The object ID to spawn (passed to a closure from the game itself in order to spawn them)
    object_id: String,
    /// The world transform of the entity
    world_transform: WrappedMatrix4,
    /// The mesh positions for the entity from the gltf node, might be useful for some stuff
    mesh: Option<Vec<WrappedVector3>>,
    /// The gltf extras for this entity
    extras: Option<String>,
}

impl WorldChunkEntity {
    pub fn new(entity_id: EntityId, object_id: String, world_transform: Matrix4<f32>,
        mesh: Option<Vec<WrappedVector3>>, extras: Option<String>) -> Self
    {
        Self {
            entity_id,
            object_id,
            world_transform: WrappedMatrix4(world_transform),
            mesh,
            extras,
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    pub fn world_transform(&self) -> &Matrix4<f32> {
        self.world_transform.as_mat()
    }

    pub fn mesh(&self) -> Option<&Vec<WrappedVector3>> {
        self.mesh.as_ref()
    }

    pub fn extras(&self) -> Option<&String> {
        self.extras.as_ref()
    }
}
