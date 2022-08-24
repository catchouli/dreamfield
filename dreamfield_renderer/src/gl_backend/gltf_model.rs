use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use gl::types::*;
use gltf::Semantic;
use gltf::animation::Property;
use gltf::image::Format;
use gltf::accessor::DataType;
use gltf::json::extras::RawValue;
use gltf::khr_lights_punctual::Kind;
use gltf::material::AlphaMode;
use super::texture::{Texture, TextureParams};
use super::uniform_buffer::{UniformBuffer, GlobalParams, MaterialParams};
use super::{bindings, JointParams, Joint, ToStd140};
use super::lights::LightType;
use cgmath::{SquareMatrix, Matrix4, Vector3, vec3, vec4, Vector4, Matrix, Quaternion, VectorSpace};
use serde::{Deserialize, Serialize, Deserializer};
use byteorder::{LittleEndian, ReadBytesExt};

/// Size of an f32
const F32_SIZE: usize = std::mem::size_of::<f32>();

/// A gltf model
pub struct GltfModel {
    buffers: Vec<u32>,
    drawables: Vec<GltfDrawable>,
    lights: Vec<GltfLight>,
    transform: Matrix4<f32>,
    ubo_joints: UniformBuffer<JointParams>,
    animations: HashMap<String, GltfAnimation>,
    transform_hierarchy: Vec<Option<Rc<RefCell<GltfTransform>>>>
}

pub struct GltfTransform {
    parent: Option<Rc<RefCell<GltfTransform>>>,
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
    local_transform: Matrix4<f32>
}

pub struct GltfDrawable {
    name: String,
    mesh: Rc<GltfMesh>,
    skin: Option<Rc<RefCell<GltfSkin>>>,
    transform: Matrix4<f32>,
    raw_extras: Option<Box<RawValue>>
}

pub struct GltfLight {
    pub light_type: LightType,
    pub light_pos: Vector3<f32>,
    pub light_dir: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub range: Option<f32>,
    pub inner_cone_angle: Option<f32>,
    pub outer_cone_angle: Option<f32>
}

struct GltfMesh {
    primitives: Vec<GltfMeshPrimitive>,
    parsed_extras: GltfMeshExtras
}

struct GltfMeshPrimitive {
    vao: u32,
    indexed_offset_length: Option<(i32, i32)>,
    material: Rc<RefCell<GltfMaterial>>,
    base_color_texture: Option<Rc<Texture>>,
    primitive_count: Option<i32>,
    alpha_blend: bool
}

struct GltfMaterial {
    uniform_buffer: UniformBuffer<MaterialParams>
}

struct GltfSkin {
    joints: Vec<GltfJoint>
}

struct GltfJoint {
    joint_index: usize,
    inverse_bind_matrix: Matrix4<f32>
}

pub struct GltfAnimation {
    name: String,
    length: f32,
    channels: Vec<GltfAnimationChannel> 
}

struct GltfAnimationChannel {
    target_node: usize,
    frames: Vec<GltfAnimationKeyframe>
}

enum GltfAnimationKeyframe {
    Translation(f32, Vector3<f32>),
    Rotation(f32, Quaternion<f32>),
    Scale(f32, Vector3<f32>)
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct GltfMeshExtras {
    #[serde(default, deserialize_with = "GltfModel::bool_from_int")]
    is_billboard: bool,
    #[serde(default, deserialize_with = "GltfModel::bool_from_int")]
    keep_upright: bool
}

impl GltfModel {
    /// Load a model from a gltf file
    pub fn from_file(path: &str, downscale_textures: bool) -> Result<GltfModel, gltf::Error> {
        Self::import(gltf::import(path)?, downscale_textures)
    }

    /// Load a model from a gltf file embedded in a buffer
    pub fn from_buf(data: &[u8], downscale_textures: bool) -> Result<GltfModel, gltf::Error> {
        Self::import(gltf::import_slice(data)?, downscale_textures)
    }

    /// Load from a (doc, buffer_data, image_data)
    /// https://kcoley.github.io/glTF/specification/2.0/figures/gltfOverview-2.0.0a.png
    fn import((doc, buffer_data, image_data): (gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>),
              downscale_textures: bool) -> Result<GltfModel, gltf::Error>
    {
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
        let textures = doc.textures()
            .map(|tex| {
                let texture = Self::load_gltf_texture(&tex, &image_data, downscale_textures);
                Rc::new(texture)
            })
            .collect();

        // Load all materials
        let materials: Vec<Rc<RefCell<GltfMaterial>>> = doc.materials().map(|mat| {
            let mat = Self::load_material(&mat);
            Rc::new(RefCell::new(mat))
        }).collect();

        // Create default material
        let default_material = Rc::new(RefCell::new(GltfMaterial {
            uniform_buffer: UniformBuffer::<MaterialParams>::new()
        }));

        // Load all meshes
        let meshes = doc.meshes().map(|mesh| {
            let mesh = Self::load_mesh(&materials, &default_material, &textures, &mesh, &buffers);
            Rc::new(mesh)
        }).collect();

        // Calculate world transforms
        let mut transform_hierarchy = Vec::new();
        for scene in doc.scenes() {
            for node in scene.nodes() {
                Self::build_transform_hierarchy(&node, None, &mut transform_hierarchy);
            }
        }

        // Load all skins
        let skins = doc.skins().map(|skin| {
            let skin = Self::load_skin(&skin, &buffer_data);
            Rc::new(RefCell::new(skin))
        }).collect();

        // Build scene drawables and lights
        let (drawables, lights) = {
            let mut drawables: Vec<GltfDrawable> = Vec::new();
            let mut lights: Vec<GltfLight> = Vec::new();

            for scene in doc.scenes() {
                for node in scene.nodes() {
                    Self::build_scene_recursive(&node, &Matrix4::identity(), &meshes, &skins, &mut drawables, &mut lights);
                }
            }

            (drawables, lights)
        };

        // Load animations
        let animations = doc.animations().map(|anim| {
            Self::load_animation(&anim, &buffer_data)
        })
        .map(|anim| (anim.name.to_string(), anim))
        .collect();

        Ok(GltfModel {
            buffers,
            drawables,
            lights,
            transform: SquareMatrix::identity(),
            ubo_joints: UniformBuffer::new(),
            animations,
            transform_hierarchy
        })
    }

    /// Build transform hierarchy
    pub fn build_transform_hierarchy(node: &gltf::Node, parent: Option<&Rc<RefCell<GltfTransform>>>,
        out_transform_hierarchy: &mut Vec<Option<Rc<RefCell<GltfTransform>>>>)
    {
        let node_index = node.index();
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());

        if out_transform_hierarchy.len() <= node_index {
            out_transform_hierarchy.resize(node_index + 1, None);
        }

        let transform = Rc::new(RefCell::new(GltfTransform::from_matrix(parent.map(|rc| rc.clone()), local_transform)));

        for child in node.children() {
            Self::build_transform_hierarchy(&child, Some(&transform), out_transform_hierarchy);
        }

        out_transform_hierarchy[node_index] = Some(transform);
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

        // Render all prims
        for drawable in self.drawables.iter_mut() {
            let mesh = &mut drawable.mesh;
            let model_mat = self.transform * drawable.transform;

            // Set model matrix based on whether this is a billboard or not
            if mesh.parsed_extras.is_billboard {
                let view_mat = ubo_global.get_mat_view();
                let billboard_mat = Self::calc_billboard_matrix(&view_mat, &model_mat, mesh.parsed_extras.keep_upright);
                ubo_global.set_mat_model_derive(&billboard_mat);
            }
            else {
                ubo_global.set_mat_model_derive(&model_mat);
            }
            ubo_global.upload_changed();

            // Update joint matrices for skinned drawables
            if let Some(skin) = &drawable.skin {
                self.ubo_joints.set_skinning_enabled(&true);

                for (i, joint) in skin.borrow().joints.iter().enumerate() {
                    let joint_world_transform = if let Some(joint_transform) = &self.transform_hierarchy[joint.joint_index] {
                        joint_transform.borrow().world_transform()
                    }
                    else {
                        Matrix4::identity()
                    };

                    let joint_matrix = joint_world_transform * joint.inverse_bind_matrix;
                    self.ubo_joints.set_joints(i, &Joint {
                        joint_matrix: joint_matrix.to_std140()
                    });
                }
            }
            else {
                self.ubo_joints.set_skinning_enabled(&false);
            }
            self.ubo_joints.bind(bindings::UniformBlockBinding::JointParams);

            // Draw
            for primitive in mesh.primitives.iter() {
                // Bind material ubo
                let material = &primitive.material;
                material.borrow_mut().uniform_buffer.bind(bindings::UniformBlockBinding::MaterialParams);

                // Enable or disable alpha blending
                unsafe {
                    if primitive.alpha_blend {
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
                if let Some(texture) = &primitive.base_color_texture {
                    texture.bind(bindings::TextureSlot::BaseColor);
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
                    log::warn!("No index data or primitive count for model");
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

    /// Get the model's lights
    pub fn lights(&self) -> &Vec<GltfLight> {
        &self.lights
    }

    /// Get the model's animations
    pub fn animations(&self) -> &HashMap<String, GltfAnimation> {
        &self.animations
    }

    /// Update an animation
    pub fn play_animation(&mut self, name: &str, time: f32) {
        if let Some(anim) = self.animations.get(name) {
            log::trace!("Playing animation {} at time {}", anim.name, time);

            for channel in anim.channels.iter() {
                if let Some(node) = &self.transform_hierarchy[channel.target_node] {
                    let cur_frame = Self::cur_frame(channel, time);
                    let total_frames = channel.frames.len();

                    let (left_frame, right_frame) = if cur_frame == 0 {
                        (0, 0)
                    }
                    else if cur_frame == total_frames {
                        (cur_frame - 1, cur_frame - 1)
                    }
                    else {
                        (cur_frame - 1, cur_frame)
                    };

                    let left = &channel.frames[left_frame];
                    let right = &channel.frames[right_frame];

                    match left.interpolate(right, time) {
                        GltfAnimationKeyframe::Translation(_, p) => {
                            node.borrow_mut().set_translation(p);
                        },
                        GltfAnimationKeyframe::Rotation(_, r) => {
                            node.borrow_mut().set_rotation(r);
                        },
                        GltfAnimationKeyframe::Scale(_, s) => {
                            node.borrow_mut().set_scale(s);
                        }
                    }
                }
                else {
                    log::error!("No such target node {}", channel.target_node);
                }
            }
        }
        else {
            log::error!("No such animation {name}");
        }
    }

    /// Perform a binary search to figure out the current animation frame
    fn cur_frame(channel: &GltfAnimationChannel, time: f32) -> usize {
        let mut min = 0;
        let mut max = channel.frames.len() as i32 - 1;

        while min <= max {
            let mid = min + (max - min) / 2;
            let frame_time = channel.frames[mid as usize].time();

            if frame_time == time {
                return mid as usize;
            }
            else if frame_time < time {
                min = mid + 1;
            }
            else {
                max = mid - 1;
            }
        }

        (max + 1) as usize
    }

    /// Load a mesh into a vao
    fn load_mesh(materials: &Vec<Rc<RefCell<GltfMaterial>>>, default_material: &Rc<RefCell<GltfMaterial>>,
                 textures: &Vec<Rc<Texture>>, mesh: &gltf::Mesh, buffers: &[u32]) -> GltfMesh
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
    fn load_gltf_texture(tex: &gltf::Texture, image_data: &[gltf::image::Data], downscale: bool) -> Texture {
        let data = &image_data[tex.source().index()];
        let sampler = tex.sampler();

        // Downscale textures to RGBA5551 if selected
        let mut new_pixel_vec = Vec::<u8>::new();
        let (format, ty, pixels) = match downscale {
            true => {
                if data.format != Format::R8G8B8A8 {
                    panic!("load_gltf_texture: must be RGBA8 to be downscaled");
                }
                Texture::convert_rgba8_to_rgba5551(&data.pixels, &mut new_pixel_vec);
                //Texture::quantize_to_bit_depth(&data.pixels, &mut new_pixel_vec, 4);
                (gl::RGBA, gl::UNSIGNED_SHORT_5_5_5_1, &new_pixel_vec)
            }
            false => {
                let (format, ty) = Self::source_format(data.format);
                (format, ty, &data.pixels)
            }
        };

        // Load texture
        let mut tex_params = TextureParams {
            horz_wrap: sampler.wrap_s().as_gl_enum(),
            vert_wrap: sampler.wrap_t().as_gl_enum(),
            min_filter: sampler.min_filter().map(|f| f.as_gl_enum()).unwrap_or(gl::NEAREST),
            mag_filter: sampler.mag_filter().map(|f| f.as_gl_enum()).unwrap_or(gl::NEAREST)
        };

        // TODO: find a way to disable mipmaps in blender's exporter
        tex_params.min_filter = Self::de_mipmapify(tex_params.min_filter);
        tex_params.mag_filter = Self::de_mipmapify(tex_params.mag_filter);

        let width = data.width as i32;
        let height = data.height as i32;
        let tex = Texture::new_from_buf(&pixels, width, height, format, ty, gl::RGBA, tex_params)
            .expect("Failed to load gltf texture");

        // Generate mipmaps - the mag_filter is often on which needs them
        tex.gen_mipmaps();

        tex
    }

    /// Get gl format and type from gltf::image::Format
    fn source_format(format: gltf::image::Format) -> (u32, u32) {
        match format {
            Format::R8 => (gl::RED, gl::UNSIGNED_BYTE),
            Format::R8G8 => (gl::RG, gl::UNSIGNED_BYTE),
            Format::R8G8B8 => (gl::RGB, gl::UNSIGNED_BYTE),
            Format::R8G8B8A8 => (gl::RGBA, gl::UNSIGNED_BYTE),
            Format::B8G8R8 => (gl::BGR, gl::UNSIGNED_BYTE),
            Format::B8G8R8A8 => (gl::BGRA, gl::UNSIGNED_BYTE),
            Format::R16 => (gl::RED, gl::UNSIGNED_SHORT),
            Format::R16G16 => (gl::RG, gl::UNSIGNED_SHORT),
            Format::R16G16B16 => (gl::RGB, gl::UNSIGNED_SHORT),
            Format::R16G16B16A16 => (gl::RGBA, gl::UNSIGNED_SHORT),
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
                             meshes: &Vec<Rc<GltfMesh>>, skins: &Vec<Rc<RefCell<GltfSkin>>>,
                             out_drawables: &mut Vec<GltfDrawable>, out_lights: &mut Vec<GltfLight>)
    {
        const WORLD_FORWARD: Vector4<f32> = vec4(0.0, 0.0, -1.0, 0.0);

        // Calculate world transform
        let local_transform = cgmath::Matrix4::from(node.transform().matrix());
        let world_transform = parent_world_transform * local_transform;

        // Add light if this node has a light
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
            // Get skin if there is one
            let name = mesh.name().unwrap_or("").to_string();
            let mesh = meshes[mesh.index()].clone();
            let skin = node.skin().map(|skin| skins[skin.index()].clone());
            let raw_extras = node.extras().clone();

            let drawable = GltfDrawable {
                name,
                mesh,
                skin,
                transform: world_transform,
                raw_extras
            };

            // Create drawable
            out_drawables.push(drawable);
        }

        // Recurse into children
        for child in node.children() {
            Self::build_scene_recursive(&child, &world_transform, meshes, skins, out_drawables, out_lights);
        }
    }

    /// Load material to a ubo
    fn load_material(mat: &gltf::Material) -> GltfMaterial {
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

    /// Load a skin
    fn load_skin(skin: &gltf::Skin, buffer_data: &[gltf::buffer::Data]) -> GltfSkin
    {
        // Get joint indices
        let joint_indices = skin.joints().map(|joint| {
            joint.index()
        });

        // Get inverse bind matrices
        let joint_count = skin.joints().count();
        let inverse_bind_matrices = skin.inverse_bind_matrices().map(|accessor| {
            // Get view and buffer data
            let view = accessor.view().unwrap();
            let buffer_data = &buffer_data[view.buffer().index()];

            // Read matrices
            let expected_length = 16 * joint_count * F32_SIZE;
            assert!(accessor.data_type().size() == F32_SIZE);
            assert!(view.length() == expected_length);

            let start = view.offset();
            let end = start + expected_length;
            let mut slice = buffer_data.get(start..end).unwrap();

            let matrices = (0..joint_count).map(|_| {
                let m00 = slice.read_f32::<LittleEndian>().unwrap();
                let m01 = slice.read_f32::<LittleEndian>().unwrap();
                let m02 = slice.read_f32::<LittleEndian>().unwrap();
                let m03 = slice.read_f32::<LittleEndian>().unwrap();
                let m10 = slice.read_f32::<LittleEndian>().unwrap();
                let m11 = slice.read_f32::<LittleEndian>().unwrap();
                let m12 = slice.read_f32::<LittleEndian>().unwrap();
                let m13 = slice.read_f32::<LittleEndian>().unwrap();
                let m20 = slice.read_f32::<LittleEndian>().unwrap();
                let m21 = slice.read_f32::<LittleEndian>().unwrap();
                let m22 = slice.read_f32::<LittleEndian>().unwrap();
                let m23 = slice.read_f32::<LittleEndian>().unwrap();
                let m30 = slice.read_f32::<LittleEndian>().unwrap();
                let m31 = slice.read_f32::<LittleEndian>().unwrap();
                let m32 = slice.read_f32::<LittleEndian>().unwrap();
                let m33 = slice.read_f32::<LittleEndian>().unwrap();
                Matrix4::new(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33)
            }).collect();

            // Check data is fully consumed
            assert!(slice.len() == 0);

            matrices
        })
        .unwrap_or(vec![SquareMatrix::identity(); joint_count]);

        // Get joints
        let joints = joint_indices.zip(inverse_bind_matrices)
            .map(|(joint_index, inverse_bind_matrix)| {
                GltfJoint {
                    joint_index,
                    inverse_bind_matrix
                }
            })
            .collect();

        GltfSkin {
            joints
        }
    }

    /// Load an animation
    fn load_animation(anim: &gltf::Animation, buffer_data: &[gltf::buffer::Data]) -> GltfAnimation {
        log::debug!("Loading animation {}", anim.name().unwrap());

        // Get name
        let name = anim.name().unwrap_or(&format!("unnamed_{}", anim.index())).to_string();

        // Load channels
        let mut length = 0.0;
        let channels = anim.channels().map(|channel| {
            // Get target node
            let target = &channel.target();
            let target_node = target.node().index();

            // Load frames
            let sampler = &channel.sampler();
            let (channel_length, frames) = Self::load_animation_frames(&sampler.input(), &sampler.output(),
                target.property(), &buffer_data);

            if channel_length > length {
                length = channel_length;
            }

            GltfAnimationChannel {
                target_node,
                frames
            }
        }).collect();

        GltfAnimation {
            name,
            length,
            channels
        }
    }

    /// Load animation frames
    fn load_animation_frames(input: &gltf::Accessor, output: &gltf::Accessor, property: Property,
        buffer_data: &[gltf::buffer::Data]) -> (f32, Vec<GltfAnimationKeyframe>)
    {
        // Get buffer view and data
        let frame_count = input.count();
        log::trace!("Loading {} {:?} animation frames", frame_count, property);

        let property_components = match property {
            Property::Translation => 3,
            Property::Rotation => 4,
            Property::Scale => 3,
            Property::MorphTargetWeights => panic!("not implemented")
        };

        let input_view = input.view().unwrap();
        let input_buffer_data = &buffer_data[input_view.buffer().index()];
        let input_buffer_length = frame_count * F32_SIZE;

        let output_view = output.view().unwrap();
        let output_buffer_data = &buffer_data[output_view.buffer().index()];
        let output_buffer_length = frame_count * property_components * F32_SIZE;

        // Safety checks
        assert!(input.data_type().size() == F32_SIZE);
        assert!(output.data_type().size() == F32_SIZE);
        assert!(input_view.length() == input_buffer_length);
        assert!(output_view.length() == output_buffer_length);

        // Load animation frames
        let mut input_reader = {
            let input_start = input_view.offset();
            let input_end = input_start + input_buffer_length;
            input_buffer_data.get(input_start..input_end).unwrap()
        };

        let mut output_reader = {
            let output_start = output_view.offset();
            let output_end = output_start + output_buffer_length;
            output_buffer_data.get(output_start..output_end).unwrap()
        };

        let mut res = Vec::new();
        let mut length = 0.0;

        for _ in 0..frame_count {
            let frame_time = input_reader.read_f32::<LittleEndian>().unwrap();

            let frame = match property {
                Property::Translation => {
                    GltfAnimationKeyframe::Translation(
                        frame_time,
                        vec3(
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap()
                            ))
                },
                Property::Rotation => {
                    let x = output_reader.read_f32::<LittleEndian>().unwrap();
                    let y = output_reader.read_f32::<LittleEndian>().unwrap();
                    let z = output_reader.read_f32::<LittleEndian>().unwrap();
                    let w = output_reader.read_f32::<LittleEndian>().unwrap();

                    GltfAnimationKeyframe::Rotation(frame_time, Quaternion::new(w, x, y, z))
                },
                Property::Scale => {
                    GltfAnimationKeyframe::Scale(
                        frame_time,
                        vec3(
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap(),
                            output_reader.read_f32::<LittleEndian>().unwrap()
                            ))
                },
                Property::MorphTargetWeights => panic!("not implemented")
            };

            if frame_time > length {
                length = frame_time;
            }

            res.push(frame);
        }

        // Check data is fully consumed
        assert!(input_reader.len() == 0);
        assert!(output_reader.len() == 0);

        (length, res)
    }

    /// Remove mipmap part from a texture filter
    /// TODO: find a way to disable mipmaps in blender's exporter
    fn de_mipmapify(filter: u32) -> u32 {
        match filter {
            gl::NEAREST_MIPMAP_NEAREST => gl::NEAREST,
            gl::LINEAR_MIPMAP_NEAREST => gl::LINEAR,
            gl::NEAREST_MIPMAP_LINEAR => gl::NEAREST,
            gl::LINEAR_MIPMAP_LINEAR => gl::LINEAR,
            _ => filter
        }
    }

    // Calculate a billboard matrix
    fn calc_billboard_matrix(view_mat: &Matrix4<f32>, model_mat: &Matrix4<f32>, keep_upright: bool) -> Matrix4<f32> {
        // Create billboard matrix without object translation
        let mut billboard_mat = match keep_upright {
            false => {
                // Transpose view matrix to get inverse of rotation, and clear view translation
                let mut mat = view_mat.transpose();

                mat[0][3] = 0.0;
                mat[1][3] = 0.0;
                mat[2][3] = 0.0;

                mat
            },
            true => {
                panic!("not implemented: keep_upright");
            }
        };

        // Add model translation
        billboard_mat[3][0] = model_mat[3][0];
        billboard_mat[3][1] = model_mat[3][1];
        billboard_mat[3][2] = model_mat[3][2];

        billboard_mat
    }

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
        &self.raw_extras
    }
}

impl GltfTransform {
    fn from_matrix(parent: Option<Rc<RefCell<GltfTransform>>>, mat: Matrix4<f32>) -> Self {
        GltfTransform {
            parent,
            translation: vec3(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
            local_transform: mat
        }
    }

    pub fn world_transform(&self) -> Matrix4<f32> {
        let parent_world_transform = if let Some(parent_transform) = &self.parent {
            parent_transform.borrow().world_transform()
        }
        else {
            Matrix4::identity()
        };
        parent_world_transform * self.local_transform
    }

    pub fn set_translation(&mut self, translation: Vector3<f32>) {
        self.translation = translation;
        self.recalculate_transform();
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
        self.recalculate_transform();
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
        self.recalculate_transform();
    }

    fn recalculate_transform(&mut self) {
        self.local_transform =
            Matrix4::from_translation(self.translation) *
            Matrix4::from(self.rotation) *
            Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
    }
}

impl GltfAnimation {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn length(&self) -> f32 {
        self.length
    }
}

impl GltfAnimationKeyframe {
    fn time(&self) -> f32 {
        match self {
            GltfAnimationKeyframe::Translation(time, _) => *time,
            GltfAnimationKeyframe::Rotation(time, _) => *time,
            GltfAnimationKeyframe::Scale(time, _) => *time
        }
    }

    /// Interpolate between two keyframes
    fn interpolate(&self, b: &GltfAnimationKeyframe, time: f32) -> GltfAnimationKeyframe {
        match (self, b) {
            (GltfAnimationKeyframe::Translation(t_a, p_a), GltfAnimationKeyframe::Translation(t_b, p_b)) => {
                let amount = Self::interpolation_amount(time, *t_a, *t_b);
                let position = p_a.lerp(*p_b, amount);
                GltfAnimationKeyframe::Translation(time, position)
            },
            (GltfAnimationKeyframe::Rotation(t_a, r_a), GltfAnimationKeyframe::Rotation(t_b, r_b)) => {
                let amount = Self::interpolation_amount(time, *t_a, *t_b);
                let rotation = r_a.slerp(*r_b, amount);
                GltfAnimationKeyframe::Rotation(time, rotation)
            },
            (GltfAnimationKeyframe::Scale(t_a, s_a), GltfAnimationKeyframe::Scale(t_b, s_b)) => {
                let amount = Self::interpolation_amount(time, *t_a, *t_b);
                let scale = s_a.lerp(*s_b, amount);
                GltfAnimationKeyframe::Scale(time, scale)
            }
            _ => panic!("Invalid combination of keyframes to lerp")
        }
    }

    /// Get the interpolation amount for a time between two other times
    fn interpolation_amount(time: f32, a: f32, b: f32) -> f32 {
        f32::clamp((time - a) / (b - a), 0.0, 1.0)
    }
}

impl Drop for GltfModel {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(self.buffers.len() as i32, self.buffers.as_ptr());
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

impl Drop for GltfMaterial {
    fn drop(&mut self) {
        drop(&mut self.uniform_buffer);
    }
}
