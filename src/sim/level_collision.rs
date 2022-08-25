use byteorder::{ReadBytesExt, LittleEndian};
use ncollide3d::{shape::TriMesh, math::{Point, Isometry}, query::{Ray, RayCast, RayIntersection}};
use cgmath::{SquareMatrix, Matrix4, vec4, Vector3};
use gltf::Semantic;

pub struct LevelCollision {
    pub level_tri_mesh: TriMesh<f32>
}

impl LevelCollision {
    /// Create a new LevelCollision from a gltf model
    pub fn new(gltf_model: &[u8]) -> Self {
        LevelCollision {
            level_tri_mesh: Self::load_level_collision(gltf_model)
        }
    }

    /// Raycast into the level
    pub fn raycast(&self, origin: &Vector3<f32>, direction: &Vector3<f32>, max_dist: f32) -> Option<f32> {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        self.level_tri_mesh.toi_with_ray(&Isometry::identity(), &ray, max_dist, true)
    }

    /// Raycast into the level, and obtain a normal
    pub fn raycast_normal(&self, origin: &Vector3<f32>, direction: &Vector3<f32>, max_dist: f32) -> Option<RayIntersection<f32>> {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        self.level_tri_mesh.toi_and_normal_with_ray(&Isometry::identity(), &ray, max_dist, true)
    }

    /// Load level collision
    /// TODO: lots of unsafe unwraps and stuff
    fn load_level_collision(gltf_model: &[u8]) -> TriMesh<f32> {
        let (doc, buffer_data, _) = gltf::import_slice(gltf_model).unwrap();

        let mut points = Vec::new();
        let mut indices = Vec::new();

        for node in doc.nodes() {
            Self::load_level_geometry_recursive(&doc, &buffer_data, &node, &SquareMatrix::identity(), &mut points,
                &mut indices);
        }

        TriMesh::new(points, indices, None)
    }

    /// Load the level geometry into buffers recursively
    fn load_level_geometry_recursive(doc: &gltf::Document, buffers: &Vec<gltf::buffer::Data>, node: &gltf::Node,
        parent_world_transform: &Matrix4<f32>, out_points: &mut Vec<Point<f32>>, out_indices: &mut Vec<Point<usize>>)
    {
        // Calculate world transform
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());
        let world_transform = parent_world_transform * local_transform;

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

                let existing_vertex_count = out_points.len();

                for i in (0..indices.len()).step_by(3) {
                    out_indices.push(Point::new(indices[i] as usize + existing_vertex_count,
                                                indices[i+1] as usize + existing_vertex_count,
                                                indices[i+2] as usize + existing_vertex_count));
                }

                for i in (0..vertices.len()).step_by(3) {
                    let v = world_transform * vec4(vertices[i], vertices[i+1], vertices[i+2], 1.0);
                    out_points.push(Point::<f32>::new(v.x, v.y, v.z));
                }
            }
        }

        // Recurse into children
        for child in node.children() {
            Self::load_level_geometry_recursive(doc, buffers, &child, &world_transform, out_points, out_indices);
        }
    }

}
