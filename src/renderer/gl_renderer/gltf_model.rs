use gl::types::*;
use gltf::Semantic;
use gltf::image::Format;
use super::texture::{Texture, TextureParams};
use super::uniform_buffer::{UniformBuffer, ModelParams, MaterialParams};
use super::bindings;
use cgmath::{SquareMatrix, Matrix, Matrix3, Matrix4};
use super::to_std140::ToStd140;

/// A gltf model
pub struct GltfModel {
    path: String,
    buffers: Vec<u32>,
    textures: Vec<Texture>,
    meshes: Vec<GltfMesh>,
    drawables: Vec<GltfDrawable>,
    materials: Vec<UniformBuffer<MaterialParams>>,
    default_material: UniformBuffer<MaterialParams>
}

struct GltfDrawable {
    ubo_model: UniformBuffer<ModelParams>,
    mesh: usize
}

struct GltfMesh {
    primitives: Vec<GltfMeshPrimitive>
}

struct GltfMeshPrimitive {
    vao: u32,
    offset: i32,
    length: i32,
    material_index: Option<usize>,
    base_color_texture: Option<usize>
}

impl GltfModel {
    /// Load a model from a gltf file
    /// https://kcoley.github.io/glTF/specification/2.0/figures/gltfOverview-2.0.0a.png
    pub fn load_gltf(path: &str) -> Result<GltfModel, gltf::Error> {
        println!("Loading GltfModel {path}");
        let (doc, buffer_data, image_data) = gltf::import(path)?;

        // Load all buffers
        let buffers: Vec<u32> = unsafe {
            let mut buffers = vec![0; buffer_data.len()];
            gl::GenBuffers(buffer_data.len() as i32, buffers.as_mut_ptr());

            for (i, buffer) in buffer_data.iter().enumerate() {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffers[i]);
                gl::BufferData(gl::ARRAY_BUFFER,
                               buffer.len() as GLsizeiptr,
                               buffer.as_ptr() as *const GLvoid,
                               gl::STATIC_DRAW);
            }

            buffers
        };

        // Load all textures
        let textures = doc.textures().map(|tex| Self::load_gltf_texture(&tex, &image_data)).collect();

        // Load all meshes
        let meshes = doc.meshes().map(|mesh| Self::load_mesh(&mesh, &buffers)).collect();

        // Load all materials
        let materials = doc.materials().map(|mat| Self::load_material(&mat)).collect();

        // Build list of drawables
        let drawables = {
            let mut drawables: Vec<GltfDrawable> = Vec::new();
            for scene in doc.scenes() {
                for node in scene.nodes() {
                    Self::build_drawables_recursive(&node, &mut drawables, &Matrix4::identity());
                }
            }
            drawables
        };

        // Create default material
        let default_material = UniformBuffer::<MaterialParams>::new();

        Ok(GltfModel {
            path: path.to_string(),
            buffers,
            textures,
            meshes,
            drawables,
            materials,
            default_material
        })
    }

    /// Render a model
    pub fn render(&self) {
        // Render all prims
        for drawable in &self.drawables {
            let mesh = &self.meshes[drawable.mesh];

            // Bind drawable model ubo
            drawable.ubo_model.bind(bindings::UniformBlockBinding::ModelParams);

            // Draw
            for primitive in &mesh.primitives {
                // Bind material ubo
                primitive.material_index
                    .map(|mat| &self.materials[mat])
                    .unwrap_or(&self.default_material)
                    .bind(bindings::UniformBlockBinding::MaterialParams);

                // Bind textures, or unbind if None
                if let Some(base_color_texture_index) = primitive.base_color_texture {
                    self.textures[base_color_texture_index].bind(bindings::TextureSlot::BaseColor);
                }

                unsafe {
                    // Bind vao and draw elements
                    // TODO: support non-indexed
                    gl::BindVertexArray(primitive.vao);
                    gl::DrawElements(gl::TRIANGLES,
                                     primitive.length,
                                     gl::UNSIGNED_SHORT,
                                     primitive.offset as *const GLvoid);
                }
            }
        }
    }

    /// Load a mesh into a vao
    fn load_mesh(mesh: &gltf::Mesh, buffers: &[u32]) -> GltfMesh {
        // Create primitive VAOs
        let primitives = mesh.primitives().map(|prim| {
            // Create VAO for primitive
            let vao = unsafe {
                let mut vao: u32 = 0;
                gl::GenVertexArrays(1, &mut vao);
                gl::BindVertexArray(vao);
                vao
            };

            // Bind element buffer, and get offset and length to render
            // TODO: support non-indexed
            let (offset, length) = prim.indices().map(|accessor| {
                // Note: we're not handling sparse accessors, hence the unwrap
                let buffer_view = accessor.view().unwrap();
                let buffer_index = buffer_view.buffer().index();

                // Bind the right buffer
                unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[buffer_index]) }

                // Return the offset and length
                // TODO: the length is divided by 2 because the index type is short, but we should
                // check first and make sure we draw using the right type and do this calculation
                // correctly.
                let offset = buffer_view.offset() as i32;
                let length = (buffer_view.length() / 2) as i32;

                (offset, length)
            }).unwrap();

            // Bind buffers
            for (prim_type, accessor) in prim.attributes() {
                // Note: we're not handling sparse accessors, hence the unwrap
                let buffer_view = accessor.view().unwrap();
                let buffer = buffer_view.buffer();
                let buffer_index = buffer.index();

                let attrib_index = Self::attribute_index(&prim_type);
                let attrib_size = Self::attribute_size(&prim_type);
                let attrib_type = gl::FLOAT;
                let attrib_stride = buffer_view.stride().unwrap_or(0) as i32;

                let offset = buffer_view.offset();

                unsafe {
                    gl::BindBuffer(gl::ARRAY_BUFFER, buffers[buffer_index]);
                    gl::EnableVertexAttribArray(attrib_index);
                    gl::VertexAttribPointer(attrib_index,
                                            attrib_size,
                                            attrib_type,
                                            gl::FALSE,
                                            attrib_stride,
                                            offset as *const GLvoid);
                }
            }

            // Look up the textures to use, so we don't have to do this again to render
            let base_color_texture = prim
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|tex_info| tex_info.texture().index());

            // Get material index
            let material_index = prim.material().index();

            GltfMeshPrimitive {
                vao, offset, length, base_color_texture, material_index
            }
        }).collect();

        GltfMesh {
            primitives
        }
    }

    /// Get the attribute size of a prim_type (1, 2, 3, or 4)
    fn attribute_size(prim_type: &gltf::Semantic) -> i32 {
        match prim_type {
            Semantic::Positions => 3,
            Semantic::Normals => 3,
            Semantic::Tangents => 4,
            Semantic::Colors(_) => 3,
            Semantic::TexCoords(_) => 2,
            Semantic::Joints(_) => 0,
            Semantic::Weights(_) => 0
        }
    }

    /// Get the attribute index of a primitive
    fn attribute_index(prim_type: &gltf::Semantic) -> u32 {
        match prim_type {
            Semantic::Positions => 0,
            Semantic::Normals => 1,
            Semantic::TexCoords(_) => 3,
            Semantic::Tangents => 4,
            Semantic::Colors(_) => 5,
            Semantic::Joints(_) => 6,
            Semantic::Weights(_) => 7
        }
    }

    /// Load a gltf texture
    fn load_gltf_texture(tex: &gltf::Texture, image_data: &[gltf::image::Data]) -> Texture {
        let data = &image_data[tex.source().index()];
        let sampler = tex.sampler();

        // Load texture
        let tex_params = TextureParams {
            horz_wrap: sampler.wrap_s().as_gl_enum(),
            vert_wrap: sampler.wrap_t().as_gl_enum(),
            min_filter: sampler.min_filter().map(|f| f.as_gl_enum()).unwrap_or(gl::NEAREST),
            mag_filter: sampler.mag_filter().map(|f| f.as_gl_enum()).unwrap_or(gl::NEAREST)
        };

        let tex = Texture::new_from_buf(&data.pixels, data.width as i32, data.height as i32,
            Self::source_format(data.format), gl::RGBA, tex_params).expect("Failed to load gltf texture");

        // Generate mipmaps - the mag_filter is often one which needs them
        tex.gen_mipmaps();

        tex
    }

    /// Get gl format from gltf::image::Format
    fn source_format(format: gltf::image::Format) -> u32 {
        match format {
            Format::R8 => gl::RED,
            Format::R8G8 => gl::RG,
            Format::R8G8B8 => gl::RGB,
            Format::R8G8B8A8 => gl::RGBA,
            Format::B8G8R8 => gl::BGR,
            Format::B8G8R8A8 => gl::BGRA,
            Format::R16 => gl::R16,
            Format::R16G16 => gl::RG16,
            Format::R16G16B16 => gl::RGB16,
            Format::R16G16B16A16 => gl::RGBA16,
        }
    }

    /// Build the list of drawables recursively
    fn build_drawables_recursive(node: &gltf::Node, out: &mut Vec<GltfDrawable>, parent_world_transform: &Matrix4<f32>) {
        // Calculate world transform
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());
        let world_transform = parent_world_transform * local_transform;

        // Add drawable if this node has a mesh
        if let Some(mesh) = node.mesh() {
            // Create UBO
            let mut ubo_model = UniformBuffer::<ModelParams>::new();
            ubo_model.data.mat_model = world_transform.to_std140();
            ubo_model.data.mat_normal = {
                // https://learnopengl.com/Lighting/Basic-lighting
                let v = world_transform.invert().unwrap().transpose();
                Matrix3::from_cols(v.x.truncate(), v.y.truncate(), v.z.truncate())
            }.to_std140();
            ubo_model.upload();

            // Create drawable
            out.push(GltfDrawable {
                ubo_model,
                mesh: mesh.index()
            });
        }

        // Recurse into children
        for child in node.children() {
            Self::build_drawables_recursive(&child, out, &world_transform);
        }
    }

    /// Load material to a ubo
    fn load_material(mat: &gltf::Material) -> UniformBuffer<MaterialParams> {
        let mut ubo = UniformBuffer::<MaterialParams>::new();
        ubo.data.has_base_color_texture = mat.pbr_metallic_roughness().base_color_texture().is_some().to_std140();
        ubo.upload();
        ubo
    }
}

impl Drop for GltfModel {
    /// Clean up gl objects
    fn drop(&mut self) {
        // Collect VAOs and buffers, and then delete them
        let vaos: Vec<u32> = self.meshes.iter().flat_map(|mesh| mesh.primitives.iter().map(|prim| prim.vao)).collect();

        println!("Deleting {} VAOs and {} buffers for texture {}", vaos.len(), self.buffers.len(), self.path);

        unsafe {
            gl::DeleteVertexArrays(vaos.len() as i32, vaos.as_ptr());
            gl::DeleteVertexArrays(self.buffers.len() as i32, self.buffers.as_ptr());
        }
    }
}
