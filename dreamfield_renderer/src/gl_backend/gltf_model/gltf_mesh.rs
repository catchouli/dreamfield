use std::sync::{Arc, Mutex};
use super::GltfMaterial;
use super::Texture;
use gltf::{Semantic, material::AlphaMode};
use serde::{Deserialize, Serialize, Deserializer};
use crate::gl_backend::bindings;
use gl::types::GLvoid;

/// A gltf mesh
pub struct GltfMesh {
    primitives: Vec<GltfMeshPrimitive>,
    parsed_extras: GltfMeshExtras
}

/// A single primitive from a mesh
pub struct GltfMeshPrimitive {
    vao: u32,
    indexed_offset_length: Option<(i32, i32)>,
    material: Arc<Mutex<GltfMaterial>>,
    base_color_texture: Option<Arc<Texture>>,
    primitive_count: Option<i32>,
    alpha_blend: bool
}

/// The mesh extras we support
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GltfMeshExtras {
    #[serde(default, deserialize_with = "GltfMeshExtras::bool_from_int")]
    pub is_billboard: bool,
    #[serde(default, deserialize_with = "GltfMeshExtras::bool_from_int")]
    pub keep_upright: bool
}

impl GltfMesh {
    /// Load a mesh from a gltf document
    pub fn load(materials: &Vec<Arc<Mutex<GltfMaterial>>>, default_material: &Arc<Mutex<GltfMaterial>>,
        textures: &Vec<Arc<Texture>>, mesh: &gltf::Mesh, buffers: &[u32])
        -> GltfMesh
    {
        log::trace!("Loading mesh {}", mesh.name().unwrap_or("no-name"));

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
            let indexed_offset_length = prim.indices().map(|accessor| {
                // Note: we're not handling sparse accessors, hence the unwrap
                let buffer_view = accessor.view().unwrap();
                let buffer_index = buffer_view.buffer().index();

                // Bind the right buffer
                unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[buffer_index]) }

                // Return the offset and length in elements
                let offset = buffer_view.offset() as i32;
                let length = (buffer_view.length() / accessor.data_type().size()) as i32;

                (offset, length)
            });

            // Bind buffers
            for (prim_type, accessor) in prim.attributes() {
                // Note: we're not handling sparse accessors, hence the unwrap
                let buffer_view = accessor.view().unwrap();
                let buffer = buffer_view.buffer();
                let buffer_index = buffer.index();

                let data_type = accessor.data_type();

                let attrib_index = Self::attribute_index(&prim_type);
                let attrib_size = accessor.dimensions().multiplicity() as i32;
                let attrib_type = data_type.as_gl_enum();
                let attrib_stride = buffer_view.stride().unwrap_or(0) as i32;

                let offset = buffer_view.offset();

                // Ignore extra UV properties
                let ignored = match prim_type {
                    Semantic::Colors(_) => false,
                    Semantic::TexCoords(0) => false,
                    Semantic::TexCoords(1) => true,
                    _ =>  false
                };

                if ignored {
                    continue;
                }

                // Log buffers being bound
                // TODO: use a real logging library, with log levels
                log::trace!("Binding buffer for attrib {:?} (type: {:?}, index: {attrib_index}, size: {attrib_size}, type: {attrib_type}, stride: {attrib_stride})", prim_type, data_type);

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

            // Figure out primitive count for non-indexed drawing
            let primitive_count: Option<i32> = prim.attributes()
                .fold(None, |prev: Option<(Semantic, i32)>, (name, accessor)| {
                    let attrib_count = accessor.count() as i32;

                    // If it's a different size to the previous, warn and use the smaller one
                    if let Some((prev_name, prev_count)) = prev {
                        if attrib_count != prev_count {
                            log::warn!("Attribute count mismatch: {}={}, {}={}. Using smallest.",
                                     prev_name.to_string(),
                                     prev_count,
                                     name.to_string(),
                                     attrib_count);
                        }
                        Some((prev_name, std::cmp::min(prev_count, attrib_count)))
                    }
                    else {
                        Some((name, attrib_count))
                    }
                })
            .map(|(_, count)| count);

            // Look up the textures to use, so we don't have to do this again to render
            let base_color_texture = prim
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|tex_info| tex_info.texture().index())
                .map(|idx| textures[idx].clone());

            // Look up whether it's alpha blended
            let alpha_blend = prim.material().alpha_mode() == AlphaMode::Blend;

            // Get material index
            let material_index = prim.material().index();
            let material = material_index
                .map(|idx| &materials[idx])
                .unwrap_or(default_material)
                .clone();

            GltfMeshPrimitive {
                vao,
                indexed_offset_length,
                base_color_texture,
                material,
                primitive_count,
                alpha_blend
            }
        }).collect();

        // Parse extras
        let parsed_extras = mesh.extras().as_ref().map(|extras| {
            serde_json::from_str(extras.get()).unwrap()
        }).unwrap_or(Default::default());

        GltfMesh {
            primitives,
            parsed_extras
        }
    }

    /// Get the parsed extras
    pub fn extras(&self) -> &GltfMeshExtras {
        &self.parsed_extras
    }

    /// Draw the mesh
    pub fn draw(&self, patches: bool) {
        for primitive in self.primitives.iter() {
            primitive.draw(patches);
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
            Semantic::Weights(_) => 7,
            Semantic::Extras(_) => 8
        }
    }
}

impl GltfMeshPrimitive {
    /// Draw the primitive
    pub fn draw(&self, patches: bool) {
        // Figure out primitive type
        let prim_type = match patches {
            true => gl::PATCHES,
            false => gl::TRIANGLES
        };

        // Bind material
        self.material.lock().unwrap().bind();

        // Enable or disable alpha blending
        unsafe {
            if self.alpha_blend {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::DepthMask(gl::FALSE);
            }
            else {
                gl::Disable(gl::BLEND);
                gl::DepthMask(gl::TRUE);
            }
        }

        // Bind textures, or unbind if None
        if let Some(texture) = &self.base_color_texture {
            texture.bind(bindings::TextureSlot::BaseColor);
        }

        // Indexed draw
        if let Some((offset, length)) = self.indexed_offset_length {
            unsafe {
                gl::BindVertexArray(self.vao);
                gl::DrawElements(prim_type,
                                 length,
                                 gl::UNSIGNED_SHORT,
                                 offset as *const GLvoid);
            }
        }
        // Non-indexed
        else if let Some(count) = self.primitive_count {
            unsafe {
                gl::BindVertexArray(self.vao);
                gl::DrawArrays(prim_type, 0, count);
            }
        }
        else {
            log::warn!("No index data or primitive count for model");
        }
    }
}

impl GltfMeshExtras {
    /// Can be used to deserialize ints to bools
    fn bool_from_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        match u8::deserialize(deserializer)? {
            0 => Ok(false),
            _ => Ok(true)
        }
    }
}

impl Drop for GltfMesh {
    fn drop(&mut self) {
        for prim in self.primitives.iter() {
            unsafe {
                gl::DeleteVertexArrays(1, &prim.vao);
            }
        }
    }
}
