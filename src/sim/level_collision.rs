use std::{collections::HashMap, rc::Rc};

use byteorder::{ReadBytesExt, LittleEndian};
use ncollide3d::{shape::{TriMesh, Capsule}, math::{Point, Isometry}, query::{Ray, RayCast, RayIntersection, self, Contact}, bounding_volume::{AABB, BoundingVolume}};
use cgmath::{SquareMatrix, Matrix4, vec4, Vector3, vec3};
use gltf::Semantic;

const GRID_NODE_SIZE: f32 = 10.0;

pub struct LevelCollision {
    aabb_grid: HashMap<(i32, i32), LevelGridNode>
}

pub struct LevelGridNode {
    index: (i32, i32),
    aabb: AABB<f32>,
    pub tri_meshes: Vec<Rc<TriMesh<f32>>>
}

impl LevelCollision {
    /// Create a new LevelCollision from a gltf model
    pub fn new(gltf_model: &[u8]) -> Self {
        let mut collision = LevelCollision {
            aabb_grid: HashMap::new()
        };

        collision.load_level_collision(gltf_model);

        collision
    }

    /// Raycast into the level
    pub fn raycast(&self, origin: &Vector3<f32>, direction: &Vector3<f32>, max_dist: f32) -> Option<f32> {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        let grid_node = self.get_aabb_node(origin)?;

        // Raycast the overall grid node
        let identity = Isometry::identity();
        grid_node.aabb.toi_with_ray(&identity, &ray, max_dist, true)?;

        // Now raycast the meshes and get the closest hit
        let mut hit: Option<f32> = None;

        for tri_mesh in grid_node.tri_meshes.iter() {
            if let Some(toi) = tri_mesh.toi_with_ray(&identity, &ray, max_dist, true) {
                if let Some(old_toi) = hit && toi < old_toi {
                    hit = Some(toi);
                }
                else {
                    hit = Some(toi);
                }
            }
        }

        hit
    }

    /// Raycast into the level, and obtain a normal
    pub fn raycast_normal(&self, origin: &Vector3<f32>, direction: &Vector3<f32>, max_dist: f32) -> Option<RayIntersection<f32>> {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        let grid_node = self.get_aabb_node(origin)?;

        // Raycast the overall grid node
        let identity = Isometry::identity();
        grid_node.aabb.toi_with_ray(&identity, &ray, max_dist, true)?;

        // Now raycast the meshes and get the closest hit
        let mut hit: Option<RayIntersection<f32>> = None;

        for tri_mesh in grid_node.tri_meshes.iter() {
            if let Some(new_hit) = tri_mesh.toi_and_normal_with_ray(&identity, &ray, max_dist, true) {
                if let Some(old_hit) = hit && new_hit.toi < old_hit.toi {
                    hit = Some(new_hit);
                }
                else {
                    hit = Some(new_hit);
                }
            }
        }

        hit
    }

    /// Contact
    pub fn contact(&self, origin: &Vector3<f32>, direction: &Vector3<f32>, collider: &Capsule<f32>, m: Isometry<f32>,
        max_dist: f32) -> Option<Contact<f32>>
    {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        let grid_node = self.get_aabb_node(origin)?;

        // Raycast the overall grid node
        let identity = Isometry::identity();
        grid_node.aabb.toi_with_ray(&identity, &ray, max_dist, true)?;

        // Now contact the meshes and get the closest hit
        let mut hit: Option<Contact<f32>> = None;

        for tri_mesh in grid_node.tri_meshes.iter() {
            let contact_result = query::contact_composite_shape_shape(&identity, tri_mesh.as_ref(), &m, collider,
                max_dist);
            if let Some(_) = contact_result {
                hit = contact_result;
                break;
            }
        }

        hit
    }

    /// Load level collision
    /// TODO: lots of unsafe unwraps and stuff
    fn load_level_collision(&mut self, gltf_model: &[u8]) {
        let (doc, buffer_data, _) = gltf::import_slice(gltf_model).unwrap();

        for node in doc.nodes() {
            self.load_level_geometry_recursive(&doc, &buffer_data, &node, &SquareMatrix::identity());
        }
    }

    /// Load the level geometry into buffers recursively
    fn load_level_geometry_recursive(&mut self, doc: &gltf::Document, buffers: &Vec<gltf::buffer::Data>,
        node: &gltf::Node, parent_world_transform: &Matrix4<f32>)
    {
        // Calculate world transform
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());
        let world_transform = parent_world_transform * local_transform;

        // Add this world transform to the aabb grid
        //let pos = world_transform.w.truncate();
        //let mut grid_node = self.get_or_add_aabb_node(&pos);
        //grid_node.aabb.inflate(&pos);

        // Add drawable if this node has a mesh
        if let Some(mesh) = node.mesh() {
            let mesh = doc.meshes().nth(mesh.index()).expect("mesh");

            for prim in mesh.primitives() {
                let indices = prim.indices().map(|accessor| {
                    // Note: we're not handling sparse accessors, hence the unwrap
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

                let vertices = prim.attributes()
                    .find(|(attr_type, _)| *attr_type == Semantic::Positions)
                    .map(|(_, accessor)| {
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

                        vertices
                    });

                let indices = indices.unwrap();
                let vertices = vertices.unwrap();

                let mut mesh_points: Vec<Point<f32>> = Vec::new();
                let mut mesh_indices: Vec<Point<usize>> = Vec::new();
                let mut mesh_aabb: Option<AABB<f32>> = None;

                for i in (0..indices.len()).step_by(3) {
                    mesh_indices.push(Point::new(
                        indices[i] as usize,
                        indices[i+1] as usize,
                        indices[i+2] as usize));
                }

                for i in (0..vertices.len()).step_by(3) {
                    let v = (world_transform * vec4(vertices[i], vertices[i+1], vertices[i+2], 1.0)).truncate();
                    mesh_points.push(Point::<f32>::new(v.x, v.y, v.z));

                    if let Some(mesh_aabb) = &mut mesh_aabb {
                        mesh_aabb.take_point(Point::new(v.x, v.y, v.z));
                    }
                    else {
                        let point = Point::new(v.x, v.y, v.z);
                        mesh_aabb = Some(AABB::new(point, point));
                    }
                }

                // If the node has an aabb, add it to the relevant aabbs
                if let Some(mesh_aabb) = &mesh_aabb {
                    let min = &mesh_aabb.mins;
                    let max = &mesh_aabb.maxs;

                    let start_idx_x = (min.x / GRID_NODE_SIZE) as i32;
                    let end_idx_x = (max.x / GRID_NODE_SIZE) as i32;
                    let start_idx_y = (min.z / GRID_NODE_SIZE) as i32;
                    let end_idx_y = (max.z / GRID_NODE_SIZE) as i32;

                    let tri_mesh = Rc::new(TriMesh::new(mesh_points, mesh_indices, None));

                    for x in start_idx_x..=end_idx_x {
                        for y in start_idx_y..=end_idx_y {
                            let grid_node = self.get_or_add_aabb_node_at_idx((x, y));
                            grid_node.aabb.merge(mesh_aabb);
                            grid_node.tri_meshes.push(tri_mesh.clone());
                        }
                    }
                }

            }
        }

        // Recurse into children
        for child in node.children() {
            self.load_level_geometry_recursive(doc, buffers, &child, &world_transform);
        }
    }

    /// Get aabb node for point
    pub fn get_aabb_node(&self, point: &Vector3<f32>) -> Option<&LevelGridNode> {
        self.aabb_grid.get(&Self::grid_index(point))
    }

    /// Get or add an aabb node, returning a mutable reference
    fn get_or_add_aabb_node_at_idx(&mut self, (x, y): (i32, i32)) -> &mut LevelGridNode {
        self.aabb_grid
            .entry((x, y))
            .or_insert_with(|| {
                LevelGridNode::for_index(x, y)
            })
    }

    /// Get the grid index for a point
    fn grid_index(point: &Vector3<f32>) -> (i32, i32) {
        ((point.x / GRID_NODE_SIZE) as i32, (point.z / GRID_NODE_SIZE) as i32)
    }
}

impl LevelGridNode {
    /// Create a new LevelGridNode for the given node index (x, y)
    fn for_index(x: i32, y: i32) -> Self {
        let min = vec3(x as f32, 0.0, y as f32) * GRID_NODE_SIZE;
        let max = min + vec3(GRID_NODE_SIZE, 0.0, GRID_NODE_SIZE);

        //let aabb = Aabb::from_min_max(&min, &max);
        let aabb = AABB::new(Point::new(min.x, min.y, min.z), Point::new(max.x, max.y, max.z));

        LevelGridNode {
            index: (x, y),
            aabb,
            tri_meshes: Vec::new()
        }
    }

    /// Get the aabb
    pub fn aabb(&self) -> &AABB<f32> {
        &self.aabb
    }

    /// Get the index
    pub fn index(&self) -> (i32, i32) {
        self.index
    }
}

//impl Aabb {
//    /// Create a new aabb from the given corners
//    fn from_min_max(min: &Vector3<f32>, max: &Vector3<f32>) -> Self {
//        let mut min = *min;
//        let mut max = *max;
//
//        Self::swap_if_greater(&mut min.x, &mut max.x);
//        Self::swap_if_greater(&mut min.y, &mut max.y);
//        Self::swap_if_greater(&mut min.z, &mut max.z);
//
//        Aabb {
//            min,
//            max
//        }
//    }
//
//    fn swap_if_greater(a: &mut f32, b: &mut f32) {
//        if a > b {
//            let tmp = *a;
//            *a = *b;
//            *b = tmp;
//        }
//    }
//
//    /// Inflate to include point
//    fn inflate(&mut self, point: &Vector3<f32>) {
//        self.min.x = f32::min(self.min.x, point.x);
//        self.min.y = f32::min(self.min.y, point.y);
//        self.min.z = f32::min(self.min.z, point.z);
//        self.max.x = f32::max(self.max.x, point.x);
//        self.max.y = f32::max(self.max.y, point.y);
//        self.max.z = f32::max(self.max.z, point.z);
//    }
//
//    /// Inflate to include aabbb
//    fn inflate_by_aabb(&mut self, other: &AABB) {
//        self.inflate(&other.min);
//        self.inflate(&other.max);
//    }
//
//    /// Get the bbox min
//    pub fn min(&self) -> &Vector3<f32> {
//        &self.min
//    }
//
//    /// Get the bbox max
//    pub fn max(&self) -> &Vector3<f32> {
//        &self.max
//    }
//}
