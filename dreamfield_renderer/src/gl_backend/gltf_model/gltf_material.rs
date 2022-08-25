use super::{bindings, UniformBuffer, MaterialParams};
use cgmath::vec4;

/// A gltf material
pub struct GltfMaterial {
    uniform_buffer: UniformBuffer<MaterialParams>
}

impl GltfMaterial {
    /// Create a new default material
    pub fn new() -> Self {
        GltfMaterial {
            uniform_buffer: UniformBuffer::new()
        }
    }

    /// Load a material from a gltf node
    pub fn load(mat: &gltf::Material) -> Self {
        // Create uniform buffer
        let mut ubo = UniformBuffer::<MaterialParams>::new();
        let pbr = mat.pbr_metallic_roughness();
        let base_color = pbr.base_color_factor();
        ubo.set_has_base_color_texture(&pbr.base_color_texture().is_some());
        ubo.set_base_color(&vec4(base_color[0], base_color[1], base_color[2], base_color[3]));

        GltfMaterial {
            uniform_buffer: ubo
        }
    }
    /// Bind the material
    pub fn bind(&mut self) {
        self.uniform_buffer.bind(bindings::UniformBlockBinding::MaterialParams);
    }
}

impl Drop for GltfMaterial {
    fn drop(&mut self) {
        drop(&mut self.uniform_buffer);
    }
}
