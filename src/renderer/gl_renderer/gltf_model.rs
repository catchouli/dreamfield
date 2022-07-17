use gl::types::*;
use gltf::Semantic;
use gltf::image::Format;
use super::texture::Texture;

/// A gltf model
pub struct GltfModel {
    path: String,
    doc: gltf::Document,
    buffers: Vec<u32>,
    textures: Vec<Texture>,
    meshes: Vec<GltfMesh>
}

struct GltfMesh {
    primitives: Vec<GltfMeshPrimitive>
}

struct GltfMeshPrimitive {
    vao: u32,
    offset: i32,
    length: i32
}

impl GltfModel {
    /// Load a model from a gltf file
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
        let textures = image_data.iter().map(|data| Self::load_gltf_image(data)).collect();

        // Load all meshes
        let meshes = doc.meshes().map(|mesh| Self::load_mesh(&mesh, &buffers)).collect();

        Ok(GltfModel {
            path: path.to_string(),
            doc,
            buffers,
            textures,
            meshes
        })
    }

    /// Render a model
    pub fn render(&self) {
        unsafe {
            // TODO: render hierarchy
            // TODO: materials
            for object in self.doc.nodes() {
                if let Some(mesh) = object.mesh() {
                    // Get our mesh
                    let gl_mesh = &self.meshes[mesh.index()];

                    for primitive in mesh.primitives() {
                        // Bind texture
                        let mat = primitive.material();
                        let pbr = mat.pbr_metallic_roughness();
                        if let Some(tex_info) = pbr.base_color_texture() {
                            let tex = tex_info.texture();
                            let tex_index = tex.index();
                            self.textures[tex_index].bind(0);
                        }

                        // Get our primitive
                        let gl_primitive = &gl_mesh.primitives[primitive.index()];

                        // Bind vao and draw elements
                        // TODO: support non-indexed
                        gl::BindVertexArray(gl_primitive.vao);
                        gl::DrawElements(gl::TRIANGLES,
                                         gl_primitive.length,
                                         gl::UNSIGNED_SHORT,
                                         gl_primitive.offset as *const GLvoid);
                    }
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

            GltfMeshPrimitive {
                vao, offset, length
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

    /// Load a gltf image data
    fn load_gltf_image(data: &gltf::image::Data) -> Texture {
        Texture::new_from_buf(&data.pixels, data.width as i32, data.height as i32, Self::source_format(data.format),
            gl::RGBA, Texture::NEAREST_WRAP).expect("Failed to load gltf texture")
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
