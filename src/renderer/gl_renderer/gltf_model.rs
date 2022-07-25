use gl::types::*;
use gltf::Semantic;
use gltf::image::Format;
use gltf::accessor::DataType;
use gltf::json::extras::RawValue;
use gltf::khr_lights_punctual::Kind;
use super::texture::{Texture, TextureParams};
use super::uniform_buffer::{UniformBuffer, GlobalParams, MaterialParams, LightParams, Light};
use super::bindings;
use super::lights::LightType;
use cgmath::{SquareMatrix, Matrix4, Vector3, vec3, vec4, Vector4};
use super::to_std140::*;

/// A gltf model
pub struct GltfModel {
    buffers: Vec<u32>,
    textures: Vec<Texture>,
    meshes: Vec<GltfMesh>,
    drawables: Vec<GltfDrawable>,
    lights: Vec<GltfLight>,
    materials: Vec<UniformBuffer<MaterialParams>>,
    ubo_lights: Option<UniformBuffer<LightParams>>,
    default_material: UniformBuffer<MaterialParams>,
    transform: Matrix4<f32>
}

pub struct GltfDrawable {
    name: String,
    mesh: usize,
    transform: Matrix4<f32>,
    extras: Option<Box<RawValue>>
}

pub struct GltfLight {
    light_type: LightType,
    light_pos: Vector3<f32>,
    light_dir: Vector3<f32>,
    color: Vector3<f32>,
    intensity: f32,
    range: Option<f32>,
    inner_cone_angle: Option<f32>,
    outer_cone_angle: Option<f32>
}

struct GltfMesh {
    primitives: Vec<GltfMeshPrimitive>
}

struct GltfMeshPrimitive {
    vao: u32,
    indexed_offset_length: Option<(i32, i32)>,
    material_index: Option<usize>,
    base_color_texture: Option<usize>,
    primitive_count: Option<i32>
}

impl GltfModel {
    /// Load a model from a gltf file embedded in a buffer
    /// https://kcoley.github.io/glTF/specification/2.0/figures/gltfOverview-2.0.0a.png
    pub fn from_buf(data: &[u8]) -> Result<GltfModel, gltf::Error> {
        let (doc, buffer_data, image_data) = gltf::import_slice(data)?;

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

        // Build scene drawables and lights
        let (drawables, lights) = {
            let mut drawables: Vec<GltfDrawable> = Vec::new();
            let mut lights: Vec<GltfLight> = Vec::new();

            for scene in doc.scenes() {
                for node in scene.nodes() {
                    Self::build_scene_recursive(&node, &Matrix4::identity(), &mut drawables, &mut lights);
                }
            }

            (drawables, lights)
        };

        // Load all lights
        let ubo_lights = match lights.len() {
            0 => None,
            _ => {
                let mut ubo_lights = UniformBuffer::new();

                for (i, light) in lights.iter().enumerate() {
                    let light_range = light.range.unwrap_or(0.0);
                    ubo_lights.set_lights(i, &Light {
                        enabled: true.to_std140(),
                        light_pos: light.light_pos.to_std140(),
                        light_dir: light.light_dir.to_std140(),
                        light_type: (light.light_type as i32).to_std140(),
                        color: vec3(light.color.x, light.color.y, light.color.z).to_std140(),
                        intensity: light.intensity.to_std140(),
                        range: light_range.to_std140(),
                        inner_cone_angle: light.inner_cone_angle.unwrap_or(0.0).to_std140(),
                        outer_cone_angle: light.outer_cone_angle.unwrap_or(0.0).to_std140()
                    });
                }

                Some(ubo_lights)
            }
        };

        // Create default material
        let default_material = UniformBuffer::<MaterialParams>::new();

        Ok(GltfModel {
            buffers,
            textures,
            meshes,
            drawables,
            lights,
            materials,
            ubo_lights,
            default_material,
            transform: SquareMatrix::identity()
        })
    }

    /// Render a model
    pub fn render(&mut self, ubo_global: &mut UniformBuffer<GlobalParams>, patches: bool) {
        // Bind global ubo
        ubo_global.bind(bindings::UniformBlockBinding::GlobalParams);

        // Figure out primitive type
        let prim_type = match patches {
            true => gl::PATCHES,
            false => gl::TRIANGLES
        };

        // Bind lights if the model has any
        if let Some(ubo_lights) = &mut self.ubo_lights {
            ubo_lights.bind(bindings::UniformBlockBinding::LightParams);
        }

        // Render all prims
        for drawable in self.drawables.iter_mut() {
            let mesh = &mut self.meshes[drawable.mesh];

            // Calculate world transform of drawable and bind model mat
            let model_mat = self.transform * drawable.transform;
            ubo_global.set_mat_model_derive(&model_mat);
            ubo_global.upload_changed();

            // Draw
            for primitive in mesh.primitives.iter_mut() {
                // Bind material ubo
                primitive.material_index
                    .map(|mat| &mut self.materials[mat])
                    .unwrap_or(&mut self.default_material)
                    .bind(bindings::UniformBlockBinding::MaterialParams);

                // Bind textures, or unbind if None
                if let Some(base_color_texture_index) = primitive.base_color_texture {
                    self.textures[base_color_texture_index].bind(bindings::TextureSlot::BaseColor);
                }

                // Indexed draw
                if let Some((offset, length)) = primitive.indexed_offset_length {
                    unsafe {
                        gl::BindVertexArray(primitive.vao);
                        gl::DrawElements(prim_type,
                                         length,
                                         gl::UNSIGNED_SHORT,
                                         offset as *const GLvoid);
                    }
                }
                // Non-indexed
                else if let Some(count) = primitive.primitive_count {
                    unsafe {
                        gl::BindVertexArray(primitive.vao);
                        gl::DrawArrays(prim_type, 0, count);
                    }
                }
                else {
                    println!("No index data or primitive count for model");
                }
            }
        }
    }

    /// Set the model's transform
    pub fn set_transform(&mut self, transform: &Matrix4<f32>) {
        self.transform = *transform
    }

    /// Get the drawables list
    pub fn drawables(&self) -> &Vec<GltfDrawable> {
        &self.drawables
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
            let indexed_offset_length = prim.indices().map(|accessor| {
                // Note: we're not handling sparse accessors, hence the unwrap
                let buffer_view = accessor.view().unwrap();
                let buffer_index = buffer_view.buffer().index();

                // Bind the right buffer
                unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers[buffer_index]) }

                // Get the data type size in bytes
                let data_type_size = Self::data_type_size(accessor.data_type());

                // Return the offset and length in elements
                let offset = buffer_view.offset() as i32;
                let length = (buffer_view.length() / data_type_size) as i32;

                (offset, length)
            });

            // Bind buffers
            for (prim_type, accessor) in prim.attributes() {
                // Note: we're not handling sparse accessors, hence the unwrap
                let buffer_view = accessor.view().unwrap();
                let buffer = buffer_view.buffer();
                let buffer_index = buffer.index();

                let attrib_index = Self::attribute_index(&prim_type);
                let attrib_size = Self::attribute_size(&prim_type);
                let attrib_stride = buffer_view.stride().unwrap_or(0) as i32;

                // Assuming that it's always gl::FLOAT but I might be wrong
                let attrib_type = gl::FLOAT;

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

            // Figure out primitive count for non-indexed drawing, I don't know how else to do this
            // but maybe the information is in the gltf model somewhere. We divide the buffer view
            // length for each primitive by the size in bytes to figure out the number of
            // primitives, and if they're mismatched we log a warning and choose the smallest one.
            let primitive_count: Option<i32> = prim.attributes()
                .flat_map(|(name, accessor)| accessor.view().map(|view| (name, view)))
                .fold(None, |prev: Option<(Semantic, i32)>, (name, view)| {
                    // Calculate attribute count
                    let attrib_size = Self::attribute_size(&name);
                    let attrib_size_bytes = attrib_size * (std::mem::size_of::<f32>() as i32);
                    let attrib_count = (view.length() as i32) / attrib_size_bytes;

                    // If it's a different size to the previous, warn and use the smaller one
                    if let Some((prev_name, prev_count)) = prev {
                        if attrib_count != prev_count {
                            println!("Attribute count mismatch: {}={}, {}={}. Using smallest.",
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
                .map(|tex_info| tex_info.texture().index());

            // Get material index
            let material_index = prim.material().index();

            GltfMeshPrimitive {
                vao, indexed_offset_length, base_color_texture, material_index, primitive_count
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
            Semantic::Weights(_) => 0,
            Semantic::Extras(_) => 0
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

    /// Get the size of a data type
    fn data_type_size(data_type: gltf::accessor::DataType) -> usize {
        match data_type {
            DataType::I8 => 1,
            DataType::U8 => 1,
            DataType::I16 => 2,
            DataType::U16 => 2,
            DataType::U32 => 4,
            DataType::F32 => 4
        }
    }

    /// Build the list of drawables recursively
    fn build_scene_recursive(node: &gltf::Node, parent_world_transform: &Matrix4<f32>,
        out_drawables: &mut Vec<GltfDrawable>, out_lights: &mut Vec<GltfLight>)
    {
        const WORLD_FORWARD: Vector4<f32> = vec4(0.0, 0.0, -1.0, 0.0);

        // Calculate world transform
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());
        let world_transform = parent_world_transform * local_transform;

        if let Some(light) = node.light() {
            let light_pos = vec3(world_transform[3][0], world_transform[3][1], world_transform[3][2]);
            let light_dir = (world_transform * WORLD_FORWARD).truncate();

            let (light_type, inner_cone_angle, outer_cone_angle) = match light.kind() {
                Kind::Directional => (LightType::DirectionalLight, None, None),
                Kind::Point => (LightType::PointLight, None, None),
                Kind::Spot { inner_cone_angle, outer_cone_angle } =>
                    (LightType::SpotLight, Some(inner_cone_angle), Some(outer_cone_angle))
            };

            out_lights.push(GltfLight {
                light_type,
                light_pos,
                light_dir,
                color: Vector3::from(light.color()),
                intensity: light.intensity(),
                range: light.range(),
                inner_cone_angle,
                outer_cone_angle
            });
        }

        // Add drawable if this node has a mesh
        if let Some(mesh) = node.mesh() {
            // Create drawable
            out_drawables.push(GltfDrawable {
                name: mesh.name().unwrap_or("").to_string(),
                mesh: mesh.index(),
                transform: world_transform,
                extras: node.extras().clone()
            });
        }

        // Recurse into children
        for child in node.children() {
            Self::build_scene_recursive(&child, &world_transform, out_drawables, out_lights);
        }
    }

    /// Load material to a ubo
    fn load_material(mat: &gltf::Material) -> UniformBuffer<MaterialParams> {
        let mut ubo = UniformBuffer::<MaterialParams>::new();
        ubo.set_has_base_color_texture(&mat.pbr_metallic_roughness().base_color_texture().is_some());
        ubo
    }
}

impl GltfDrawable {
    /// Get the name of the drawable
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the transform for this drawable
    pub fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    /// Get the extra fields
    pub fn extras(&self) -> &Option<Box<RawValue>> {
        &self.extras
    }
}

impl Drop for GltfModel {
    /// Clean up gl objects
    fn drop(&mut self) {
        // Collect VAOs and buffers, and then delete them
        let vaos: Vec<u32> = self.meshes.iter().flat_map(|mesh| mesh.primitives.iter().map(|prim| prim.vao)).collect();

        println!("Deleting {} VAOs and {} buffers from GltfModel", vaos.len(), self.buffers.len());

        unsafe {
            gl::DeleteVertexArrays(vaos.len() as i32, vaos.as_ptr());
            gl::DeleteVertexArrays(self.buffers.len() as i32, self.buffers.as_ptr());
        }
    }
}
