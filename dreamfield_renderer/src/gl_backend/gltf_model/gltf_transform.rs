use std::sync::{Arc, Mutex};
use cgmath::{SquareMatrix, Vector3, Matrix4, Quaternion, vec3};

/// The transform hierarchy for gltf nodes
pub struct GltfTransformHierarchy {
    root: Arc<Mutex<GltfTransform>>,
    nodes_by_index: Vec<Option<Arc<Mutex<GltfTransform>>>>
}

/// A single node's transform, forming a tree
pub struct GltfTransform {
    parent: Option<Arc<Mutex<GltfTransform>>>,

    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,

    local_transform: Matrix4<f32>,
    local_transform_dirty: bool,

    world_transform: Matrix4<f32>,
    world_transform_dirty: bool
}

impl GltfTransformHierarchy {
    /// Create a new transform hierarchy with its root transform at the origin and no child nodes
    pub fn new() -> Self {
        GltfTransformHierarchy {
            root: Arc::new(Mutex::new(GltfTransform::from_local(None, Matrix4::identity()))),
            nodes_by_index: Vec::new()
        }
    }

    /// Get the root transform
    pub fn root(&self) -> &Arc<Mutex<GltfTransform>> {
        &self.root
    }

    /// Add a new child node
    pub fn add_at_index(&mut self, index: usize, transform: Arc<Mutex<GltfTransform>>) {
        if self.nodes_by_index.len() <= index {
            self.nodes_by_index.resize(index + 1, None);
        }

        self.nodes_by_index[index] = Some(transform);
    }

    /// Get a node's transform from its json index
    pub fn node_by_index(&self, index: usize) -> &Option<Arc<Mutex<GltfTransform>>> {
        self.nodes_by_index.get(index).unwrap_or(&None)
    }
}

impl GltfTransform {
    /// Create a new GltfTransform from a local transformation matrix
    pub fn from_local(parent: Option<Arc<Mutex<GltfTransform>>>, mat: Matrix4<f32>) -> Self {
        GltfTransform {
            parent,
            translation: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
            local_transform: mat,
            local_transform_dirty: false,
            world_transform: Matrix4::identity(),
            world_transform_dirty: true
        }
    }

    /// Get the local transform of the node
    pub fn local_transform(&mut self) -> &Matrix4<f32> {
        if self.local_transform_dirty {
            self.local_transform_dirty = false;
            self.local_transform =
                Matrix4::from_translation(self.translation) *
                Matrix4::from(self.rotation) *
                Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        }
        &self.local_transform
    }

    /// Get the world transform of the node
    pub fn world_transform(&mut self) -> &Matrix4<f32> {
        if self.world_transform_dirty {
            let parent_world_transform = self.parent
                .as_ref()
                .map(|t| *t.lock().unwrap().world_transform())
                .unwrap_or(Matrix4::identity());

            self.world_transform = parent_world_transform * self.local_transform();
        }
        &self.world_transform
    }

    /// Set the local transform, this overrides the translation/rotation/scale and causes them to
    /// be ignored. We could fix this if we decomposed this every time, but it seems like a bit of
    /// a waste of cycles, so for now just avoid mixing and matching them.
    pub fn set_transform(&mut self, transform: Matrix4<f32>) {
        self.local_transform = transform;
        self.local_transform_dirty = false;
        self.world_transform_dirty = true;
    }

    /// Set the translation of the node
    pub fn set_translation(&mut self, translation: Vector3<f32>) {
        self.translation = translation;
        self.local_transform_dirty = true;
        self.world_transform_dirty = true;
    }

    /// Set the rotation of the node
    pub fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
        self.local_transform_dirty = true;
        self.world_transform_dirty = true;
    }

    /// Set the scale of the node
    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
        self.local_transform_dirty = true;
        self.world_transform_dirty = true;
    }
}

