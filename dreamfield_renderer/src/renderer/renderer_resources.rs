use std::collections::HashMap;
use std::sync::Arc;
use bevy_ecs::world::{FromWorld, World};
use crate::gl_backend::{Mesh, EditableMesh, VertexAttrib, Texture, GltfModel, UniformBuffer,
    Framebuffer, GlobalParams, JointParams, ShaderProgram, MaterialParams};
use crate::resources::ShaderManager;

/// The renderer state resource
pub struct RendererResources {
    pub full_screen_rect: Mesh,
    pub ubo_global: UniformBuffer<GlobalParams>,
    pub ubo_joints: UniformBuffer<JointParams>,
    pub ubo_material: UniformBuffer<MaterialParams>,
    pub framebuffer_size: Option<(i32, i32)>,
    pub framebuffer: Option<Framebuffer>,
    pub yiq_framebuffer: Option<Framebuffer>,
    pub ps1_tess_shader: Arc<ShaderProgram>,
    pub composite_yiq_shader: Arc<ShaderProgram>,
    pub composite_resolve_shader: Arc<ShaderProgram>,
    pub blit_shader: Arc<ShaderProgram>,
    pub models: HashMap<String, Arc<GltfModel>>,
    pub world_meshes: HashMap<i32, Mesh>,
    pub world_textures: HashMap<i32, Texture>,
    pub text_mesh: EditableMesh,
    pub portal_meshes: HashMap<i32, Mesh>,
}

impl FromWorld for RendererResources {
    fn from_world(world: &mut World) -> Self {
        log::info!("Creating renderer resources");

        // Create uniform buffers
        let ubo_global = UniformBuffer::<GlobalParams>::new();
        let ubo_joints = UniformBuffer::<JointParams>::new();
        let ubo_material = UniformBuffer::<MaterialParams>::new();

        // Load meshes
        let full_screen_rect = Mesh::new_indexed(
            &vec![
            1.0,  1.0, 0.0, 1.0, 1.0,  // top right
            1.0, -1.0, 0.0, 1.0, 0.0,  // bottom right
            -1.0, -1.0, 0.0, 0.0, 0.0,  // bottom left
            -1.0,  1.0, 0.0, 0.0, 1.0,  // top left
            ],
            &vec![
            0, 1, 3,
            1, 2, 3,
            ],
            &vec![
            VertexAttrib { index: 0, size: 3, attrib_type: gl::FLOAT },
            VertexAttrib { index: 1, size: 2, attrib_type: gl::FLOAT },
            ]);

        let text_mesh = EditableMesh::new(vec![
            VertexAttrib { index: 0, size: 3, attrib_type: gl::FLOAT },
            VertexAttrib { index: 1, size: 2, attrib_type: gl::FLOAT },
        ]);

        // Load shaders
        // TODO: it would be nice if the shaders were specified by components on entities instead
        // of hardcoded here, and the composite/resolve were converted to screen-space effects
        let mut shaders = world.get_resource_mut::<ShaderManager>().expect("Failed to get shader manager");
        let ps1_tess_shader = shaders.get("ps1_tess").unwrap().clone();
        let composite_yiq_shader = shaders.get("composite_yiq").unwrap().clone();
        let composite_resolve_shader = shaders.get("composite_resolve").unwrap().clone();
        let blit_shader = shaders.get("blit").unwrap().clone();

        RendererResources {
            full_screen_rect,
            ubo_global,
            ubo_joints,
            ubo_material,
            framebuffer_size: None,
            framebuffer: None,
            yiq_framebuffer: None,
            ps1_tess_shader,
            composite_yiq_shader,
            composite_resolve_shader,
            blit_shader,
            models: HashMap::new(),
            world_meshes: HashMap::new(),
            world_textures: HashMap::new(),
            text_mesh,
            portal_meshes: HashMap::new(),
        }
    }
}

