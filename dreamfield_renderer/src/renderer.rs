mod renderer_resources;

use std::sync::Arc;
use std::time::Duration;

use bevy_ecs::schedule::SystemSet;
use bevy_ecs::system::{Local, Res, Query, ResMut};
use cgmath::{SquareMatrix, Matrix4, vec2, InnerSpace, vec4, vec3};
use renderer_resources::RendererResources;
use crate::gl_backend::*;
use crate::gl_backend::bindings::AttribBinding;
use crate::resources::{ModelManager, TextureManager, ShaderManager, FontManager};
use crate::components::{PlayerCamera, Visual, ScreenEffect, RunTime, TextBox, DiagnosticsTextBox};
use dreamfield_system::WindowSettings;
use dreamfield_system::world::WorldChunkManager;
use dreamfield_system::world::world_chunk::{WorldChunk, WorldChunkMesh, ChunkIndex};
use dreamfield_system::world::world_texture::WorldTexture;
use dreamfield_system::world::wrapped_vectors::WrappedVector3;
use dreamfield_system::resources::{SimTime, Diagnostics};
use dreamfield_system::components::Transform;

pub const RENDER_WIDTH: i32 = 320;
pub const RENDER_HEIGHT: i32 = 240;

pub const RENDER_ASPECT: f32 = 4.0 / 3.0;

pub const FOV: f32 = 60.0;

pub const NEAR_CLIP: f32 = 0.1;
pub const FAR_CLIP: f32 = 35.0;

// Calculated values
pub const FOG_START: f32 = FAR_CLIP - 10.0;
pub const FOG_END: f32 = FAR_CLIP - 5.0;

pub const FOV_RADIANS: f32 = FOV * std::f32::consts::PI / 180.0;
pub const HALF_FOV_RADIANS: f32 = FOV_RADIANS / 2.0;

/// The render systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(update_diagnostics)
        .with_system(renderer_system)
}

/// The renderer system
fn renderer_system(mut local: Local<RendererResources>, window_settings: Res<WindowSettings>,
    sim_time: Res<SimTime>, models: Res<ModelManager>, mut textures: ResMut<TextureManager>,
    fonts: Res<FontManager>, mut world: ResMut<WorldChunkManager>, mut shaders: ResMut<ShaderManager>,
    mut effect_query: Query<&mut ScreenEffect>, player_query: Query<&PlayerCamera>,
    mut visuals_query: Query<(&Transform, &mut Visual)>, text_query: Query<&TextBox>)
{
    let local = &mut *local;

    // Update window size if it's changed
    if window_settings.is_added() || window_settings.is_changed() {
        let (width, height) = window_settings.window_size;
        local.ubo_global.set_window_aspect(&(width as f32 / height as f32));
    }

    // Get player camera
    let player_camera = player_query.get_single().expect("Expected one player camera");

    // Create framebuffers if they don't exist
    if local.framebuffer.is_none() {
        local.framebuffer = Some(Framebuffer::new(player_camera.render_res.x as i32, player_camera.render_res.y as i32,
            gl::SRGB8, TextureParams::new(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE, gl::NEAREST, gl::NEAREST)));
    }
    if local.yiq_framebuffer.is_none() {
        local.yiq_framebuffer = Some(Framebuffer::new(player_camera.render_res.x as i32, player_camera.render_res.y as i32,
            gl::RGBA32F, TextureParams::new(gl::CLAMP_TO_EDGE, gl::CLAMP_TO_EDGE, gl::LINEAR_MIPMAP_LINEAR, gl::NEAREST)));
    }

    // Render game
    // Update global params
    local.ubo_global.set_fog_color(&player_camera.fog_color);
    local.ubo_global.set_fog_dist(&player_camera.fog_range);

    local.ubo_global.set_target_aspect(&player_camera.render_aspect);
    local.ubo_global.set_render_res(&player_camera.render_res);
    local.ubo_global.set_render_fov(&player_camera.render_fov_rad);

    local.ubo_global.set_mat_proj(&player_camera.proj);

    local.ubo_global.set_sim_time(&(sim_time.sim_time as f32));
    local.ubo_global.set_mat_proj(&player_camera.proj);
    local.ubo_global.set_mat_view_derive(&player_camera.view);
    local.ubo_global.upload_changed();

    // Bind framebuffer and clear
    unsafe { gl::Viewport(0, 0, player_camera.render_res.x as i32, player_camera.render_res.y as i32) };
    local.framebuffer.as_ref().unwrap().bind_draw();
    unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB); }

    // Enable or disable wireframe mode
    let polygon_mode = match window_settings.wireframe_enabled {
        true => gl::LINE,
        false => gl::FILL
    };
    unsafe { gl::PolygonMode(gl::FRONT_AND_BACK, polygon_mode) }

    // Clear screen
    unsafe {
        gl::ClearColor(0.05, 0.05, 0.05, 1.0);
        gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    // Render pre-scene effects
    render_screen_effects(RunTime::PreScene, local, &mut textures, &mut shaders, &mut effect_query);

    // Draw world
    draw_world(local, &mut world, &models, &player_camera);

    // Draw visuals
    draw_visuals(local, &sim_time, &models, &mut visuals_query);

    // Render post-scene effects
    render_screen_effects(RunTime::PostScene, local, &mut textures, &mut shaders, &mut effect_query);

    // Render text
    unsafe { gl::Enable(gl::SCISSOR_TEST); }
    for text_box in text_query.iter() {
        render_text(local, &player_camera, &fonts, &mut shaders, text_box);
    }
    unsafe { gl::Disable(gl::SCISSOR_TEST); }

    // Run final composite
    final_composite(local, &window_settings);
}

/// Draw the world
fn draw_world(local: &mut RendererResources, mut world: &mut ResMut<WorldChunkManager>, models: &Res<ModelManager>,
    camera: &PlayerCamera)
{
    local.ubo_global.bind(bindings::UniformBlockBinding::GlobalParams);
    local.ubo_joints.bind(bindings::UniformBlockBinding::JointParams);
    local.ubo_material.bind(bindings::UniformBlockBinding::MaterialParams);

    unsafe { gl::Enable(gl::DEPTH_TEST); }
    local.ps1_tess_shader.use_program();

    // Get camera pos
    let cam_transform = camera.view.invert().unwrap();
    let pos = cam_transform.w.truncate();
    let forward = cam_transform * vec4(0.0, 0.0, -1.0, 0.0);

    // Work out what chunks can be seen by creating a triangle between the camera position and the
    // corners of the far clip plane, in 2D, and then walk all the chunks between them and draw them
    let pos_xz = vec2(pos.x, pos.z);
    let forward_xz = vec2(forward.x, forward.z).normalize();

    // The view of the world grid (in 2d) forms a triangle between the corners of the far clip
    // plane and the camera position. To draw the chunks within the camera view, we want to figure
    // out those two points in the distance.
    //
    // To do this, we can divide this triangle into two right-angle triangles, where the line from
    // the camera position to the point `far_point` straight ahead of the camera on the far clip plane
    // forms one edge, and the clip plane corners form the third point on each triangle.
    //
    // To figure out the corner points, we then first figure out what far point is, and then rotate
    // the forward vector by 90 degrees to get right_xz:
    let far_point = pos_xz + forward_xz * FAR_CLIP;
    let right_xz = vec2(-forward_xz.y, forward_xz.x);

    // We then calculate the "half width" of the far clip plane using trigonometry, which is the
    // distance between far_point and the corner point.
    // This can't be a const right now but it could be if f32::tan was...
    let far_clip_half_width: f32 = FAR_CLIP * f32::tan(HALF_FOV_RADIANS);

    // And then we multiply this by the right vector and add it to get the corner point, and then
    // do the opposite to get the other corner point.
    let corner_a = far_point + right_xz * far_clip_half_width;
    let corner_b = far_point - right_xz * far_clip_half_width;

    // Then, take the min and max of all three points, and use it to create an AABB for the view.
    // We can then draw all world chunks that intersect this AABB. As an optimization, we could
    // draw only the ones that are actually within the triangle, but I don't think it's necessary.
    let view_aabb_min = vec2(
        f32::min(pos_xz.x, f32::min(corner_a.x, corner_b.x)),
        f32::min(pos_xz.y, f32::min(corner_a.y, corner_b.y))
    );
    let view_aabb_max = vec2(
        f32::max(pos_xz.x, f32::max(corner_a.x, corner_b.x)),
        f32::max(pos_xz.y, f32::max(corner_a.y, corner_b.y))
    );

    // Get chunk indexes at corners
    let (view_min_chunk_x, view_min_chunk_z) = WorldChunk::point_to_chunk_index_2d(&view_aabb_min);
    let (view_max_chunk_x, view_max_chunk_z) = WorldChunk::point_to_chunk_index_2d(&view_aabb_max);

    for chunk_x in view_min_chunk_x..=view_max_chunk_x {
        for chunk_z in view_min_chunk_z..=view_max_chunk_z {
            draw_world_chunk(local, &mut world, &models, (chunk_x, chunk_z));
        }
    }
}

/// Draw a WorldChunk
fn draw_world_chunk(local: &mut RendererResources, world: &mut ResMut<WorldChunkManager>, models: &Res<ModelManager>,
    chunk_index: ChunkIndex)
{
    let mut textures_to_load = Vec::new();

    if let Some(chunk) = world.get_or_load_chunk(chunk_index) {
        // Draw instances in chunk
        for instance in chunk.instances().iter() {
            // Get reference to model from the renderer resources cache, loading it if it's not in there
            let model = local.models
                .entry(instance.mesh_name().to_string())
                .or_insert_with(|| {
                    let data = models.get(instance.mesh_name()).unwrap();
                    Arc::new(GltfModel::from_buf(data).unwrap())
                });

            for WrappedVector3(point) in instance.points().iter() {
                let transform = Matrix4::from_translation(*point);
                model.render(&transform, &mut local.ubo_global, &mut local.ubo_joints, true);
            }
        }

        // Draw meshes in chunk
        local.ubo_global.set_mat_model_derive(&Matrix4::identity());
        local.ubo_global.upload_changed();
        local.ubo_joints.set_skinning_enabled(&false);
        local.ubo_joints.upload_changed();

        for mesh in chunk.meshes().iter() {
            // Bind material
            local.ubo_material.set_has_base_color_texture(&false);
            if let Some(material) = mesh.material() {
                if let Some(texture_id) = material.base_color_tex() {
                    if let Some(texture) = local.world_textures.get(texture_id) {
                        local.ubo_material.set_has_base_color_texture(&true);
                        texture.bind(TextureSlot::BaseColor);
                    }
                    else {
                        // Annoyingly we can't just load it now as we still have world borrowed
                        // mutably through the reference to chunk
                        textures_to_load.push(*texture_id);
                    }
                }
                else {
                }

                local.ubo_material.set_base_color(material.base_color().as_vec());
                local.ubo_material.bind(bindings::UniformBlockBinding::MaterialParams);
            }
            else {
                local.ubo_material.set_base_color(&vec4(1.0, 1.0, 1.0, 1.0));
                local.ubo_material.set_has_base_color_texture(&false);
                local.ubo_material.bind(bindings::UniformBlockBinding::MaterialParams);
            }
            local.ubo_material.upload_changed();

            // Draw mesh
            let count = mesh.indices().len() as i32;
            let mesh = get_gl_mesh(local, &mesh);
            mesh.draw_indexed(gl::PATCHES, count);
        }
    }

    // Load textures, a bit late but otherwise we end up borrowing world twice because we're still
    // iterating the chunk's meshes... sigh
    for tex_idx in textures_to_load {
        if let Some(texture) = world.get_or_load_texture(tex_idx) {
            get_gl_texture(local, &texture);
        }
    }
}

// Get the gl mesh for a world mesh
fn get_gl_mesh<'a>(local: &'a mut RendererResources, mesh: &WorldChunkMesh) -> &'a Mesh {
    local.world_meshes
        .entry(mesh.index())
        .or_insert_with(|| {
            // TODO: pregenerate them as u32, or just load them as u16
            let index_buffer = mesh.indices().iter().map(|i| *i as u32).collect::<Vec<u32>>();
            let buffer_layout = vec![
                VertexAttrib {
                    index: AttribBinding::Positions as u32,
                    attrib_type: gl::FLOAT,
                    size: 3
                },
                VertexAttrib {
                    index: AttribBinding::Normals as u32,
                    attrib_type: gl::FLOAT,
                    size: 3
                },
                VertexAttrib {
                    index: AttribBinding::TexCoords as u32,
                    attrib_type: gl::FLOAT,
                    size: 2
                },
                VertexAttrib {
                    index: AttribBinding::Colors as u32,
                    attrib_type: gl::FLOAT,
                    size: 4
                }
            ];

            Mesh::new_indexed(mesh.vertices(), &index_buffer, &buffer_layout)
        })
}

// Get the gl texture for a world texture
fn get_gl_texture<'a>(local: &'a mut RendererResources, texture: &WorldTexture) -> &'a Texture {
    local.world_textures
        .entry(texture.index())
        .or_insert_with(|| {
            let dest_format = gl::SRGB8_ALPHA8;
            let tex_params = TextureParams::repeat_nearest();
            Texture::new_from_buf(&texture.pixels(), texture.width() as i32, texture.height() as i32, texture.format(),
                gl::UNSIGNED_BYTE, dest_format, tex_params).expect("Failed to create world texture")
        })
}

/// Draw the visuals
fn draw_visuals(local: &mut RendererResources, sim_time: &Res<SimTime>, models: &Res<ModelManager>,
    visuals_query: &mut Query<(&Transform, &mut Visual)>)
{
    unsafe { gl::Enable(gl::DEPTH_TEST); }

    let ubo_global = &mut local.ubo_global;
    let ubo_joints = &mut local.ubo_joints;
    for (pos, mut visual) in visuals_query.iter_mut() {
        let anim_changed = visual.animate(sim_time.sim_time as f32);

        // Get model, loading it if it isn't already loaded
        let model = {
            // Initialise model if it's not already
            if visual.internal_model.is_none() {
                // Get reference to model from the renderer resources cache, loading it if it's not in there
                let model = local.models
                    .entry(visual.model_name.to_string())
                    .or_insert_with(|| {
                        let data = models.get(&visual.model_name).unwrap();
                        Arc::new(GltfModel::from_buf(data).unwrap())
                    });

                visual.internal_model = Some(model.clone());
            }
            visual.internal_model.as_ref().unwrap()
        };

        // Animate model if an animation is playing
        if anim_changed {
            if let Some(anim_state) = &visual.internal_anim_state {
                let anim_time = anim_state.anim_time;
                update_animation(&model, &anim_state.cur_anim.name(), anim_time);
            }
        }

        // Bind shader
        let shader = match visual.tessellate {
            true => &local.ps1_tess_shader,
            false => &local.ps1_no_tess_shader
        };
        shader.use_program();

        // Draw model
        let transform = Matrix4::from_translation(pos.pos) * Matrix4::from(pos.rot);
        model.render(&transform, ubo_global, ubo_joints, visual.tessellate);
    }
}

/// Render a screen effect
/// TODO: these aren't that useful for anything but the sky if you can't read the framebuffer :)
fn render_screen_effects(run_time: RunTime, local: &RendererResources, texture_manager: &mut ResMut<TextureManager>,
    shader_manager: &mut ResMut<ShaderManager>, effect_query: &mut Query<&mut ScreenEffect>)
{
    unsafe { gl::Disable(gl::DEPTH_TEST); }
    for mut effect in effect_query.iter_mut() {
        if effect.run_time == run_time {
            if let Some(texture) = effect.get_texture(texture_manager.as_mut()) {
                texture.bind(bindings::TextureSlot::BaseColor);
            }
            if let Some(shader) = effect.get_shader(shader_manager.as_mut()) {
                shader.use_program();
                local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
            }
        }
    }
}

/// Run final compositing and blit operations, including ntsc composite emulation
fn final_composite(local: &RendererResources, window_settings: &Res<WindowSettings>) {
    // Disable depth test for blitting operations
    unsafe {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        gl::Disable(gl::DEPTH_TEST);
    }

    let yiq_framebuffer = local.yiq_framebuffer.as_ref().unwrap();
    let framebuffer = local.framebuffer.as_ref().unwrap();

    // Composite simulation: convert rgb to yiq color space
    // No SRGB conversion, since we're outputting colors in the YIQ color space. Additionally
    // we're writing to an f32 framebuffer already anyway to avoid precision issues.
    unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB) };
    yiq_framebuffer.bind_draw();
    framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    local.composite_yiq_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);

    // Composite simulation: resolve back to regular framebuffer
    // This time we're outputting back to our srgb framebuffer so we enable sRGB again.
    // Annoyingly the YIQ conversion already outputs sRGB colors, so we have to convert them
    // back to linear in the shader, just for them to be converted back into sRGB. Oh well.
    unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB); }
    framebuffer.bind_draw();
    yiq_framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) };
    local.composite_resolve_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
    framebuffer.unbind();

    // Render framebuffer to screen
    let (window_width, window_height) = window_settings.window_size;
    unsafe { gl::Viewport(0, 0, window_width, window_height) };
    framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    local.blit_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
}

/// Update an animation
fn update_animation(model: &GltfModel, name: &str, time: f32) {
    if let Some(anim) = model.animations().get(name) {
        log::trace!("Playing animation {} at time {}", anim.name(), time);

        for channel in anim.channels().iter() {
            if let Some(node) = &channel.target() {
                match channel.sample(time % anim.length()) {
                    GltfAnimationKeyframe::Translation(_, p) => {
                        node.lock().unwrap().set_translation(p);
                    },
                    GltfAnimationKeyframe::Rotation(_, r) => {
                        node.lock().unwrap().set_rotation(r);
                    },
                    GltfAnimationKeyframe::Scale(_, s) => {
                        node.lock().unwrap().set_scale(s);
                    }
                }
            }
            else {
                log::error!("No such target node for animation {}", name);
            }
        }
    }
    else {
        log::error!("No such animation {name}");
    }
}

fn render_text(local: &mut RendererResources, camera: &PlayerCamera, fonts: &FontManager, shaders: &mut ShaderManager,
    text_box: &TextBox)
{
    // TODO:
    //  * Wrap in indivisible blocks and not just on any character
    //  * Don't put spaces at the start of a line
    //  * Support drawing text in 3d for signs and such (maybe)
    let shader = shaders.get(&text_box.shader).unwrap();
    let texture = fonts.get_texture(&text_box.font_name).unwrap();
    let char_map = fonts.get_char_map(&text_box.font_name, &text_box.font_variant).unwrap();

    // Calculate clipping bounds
    let text_box_bounds = match text_box.bounds {
        Some(bounds) => bounds,
        None => vec4(0.0, 0.0, camera.render_res.x, camera.render_res.y)
    };
    let text_box_origin = vec2(text_box_bounds.x, text_box_bounds.y);
    let text_box_width = text_box_bounds.z - text_box_bounds.x;
    let text_box_height = text_box_bounds.w - text_box_bounds.y;

    // Update scissor region
    unsafe {
        // Set scissor rect, converting the top-left (0,0) coordinates to bottom-left (0,0)
        // screen coordinates... sigh at all these different coordinate spaces. We basically just
        // change it so we're specifying the bottom left corner of the box in that space, instead
        // of the top left one in our y-down space.
        let viewport_height = camera.render_res.y;
        let bottom_left_y = viewport_height - text_box_height - text_box_origin.y;
        gl::Scissor(text_box_origin.x as i32,
                    bottom_left_y as i32,
                    text_box_width as i32,
                    text_box_height as i32);
    }

    // Scale the coordinates for the texture into uv space
    let uv_scale = vec2(1.0 / texture.width() as f32, 1.0 / texture.height() as f32);

    // Scale and position the text from window space into clip space (-1..1)
    let window_scale = vec2(2.0 / camera.render_res.x as f32, -2.0 / camera.render_res.y as f32);
    let window_bias = vec2(-1.0, 1.0);

    // The origin and spacing of the text
    let origin = text_box_origin;
    let spacing = text_box.spacing.unwrap_or(vec2(0.0, 0.0));

    // Build vertex buffer
    local.text_mesh.clear_vertex_buffer();

    let mut offset = origin;
    let mut triangles = 0;

    // We use the last character height as a linebreak height, it's a bit weird tbh, we should
    // probably calculate the size of the box needed to render each line instead and use that.
    let mut line_height = None;

    for character in text_box.text.trim().chars() {
        // If this is a line break character, break to the next line
        if character == '\n' {
            offset.x = origin.x;
            offset.y += line_height.unwrap_or(0.0) + spacing.y;
            continue;
        }

        // Get character map entry for the next character
        let entry = char_map.get_entry(character)
            .expect(&format!("No character map entry for character {character}"));

        // Get the dimensions of the glyph
        let dimensions = vec2(entry.width as f32, entry.height as f32);
        line_height = Some(dimensions.y);

        // If the glyph doesn't fit on this line, add a line break
        if offset.x + dimensions.x > text_box_width {
            offset.x = origin.x;
            offset.y += line_height.unwrap_or(0.0) + spacing.y;
        }

        // Calculate window-space coordinates (with 0,0 at the top left) of this textbox
        let top_left = offset;
        let bottom_right = top_left + dimensions;

        // Move offset for the next glyph to after the current glyph
        offset.x += dimensions.x + spacing.x;

        // Calculate texture coordinates in image space
        let image_top_left = vec2(entry.source_x as f32, entry.source_y as f32);
        let image_bottom_right = image_top_left + dimensions;

        // Convert window coordinates to clip space
        let top_left = vec2(top_left.x as f32 * window_scale.x + window_bias.x,
                            top_left.y as f32 * window_scale.y + window_bias.y);
        let bottom_right = vec2(bottom_right.x as f32 * window_scale.x + window_bias.x,
                                bottom_right.y as f32 * window_scale.y + window_bias.y);

        // Convert image coordinates to uvs
        let uv_top_left = vec2(image_top_left.x as f32 * uv_scale.x,
                               image_top_left.y as f32 * uv_scale.y);
        let uv_bottom_right = vec2(image_bottom_right.x as f32 * uv_scale.x,
                                   image_bottom_right.y as f32 * uv_scale.y);

        // Push two triangles into vertex buffer
        local.text_mesh.push_vec3(vec3(top_left.x, top_left.y, 0.0));
        local.text_mesh.push_vec2(vec2(uv_top_left.x, uv_top_left.y));

        local.text_mesh.push_vec3(vec3(bottom_right.x, top_left.y, 0.0));
        local.text_mesh.push_vec2(vec2(uv_bottom_right.x, uv_top_left.y));

        local.text_mesh.push_vec3(vec3(bottom_right.x, bottom_right.y, 1.0));
        local.text_mesh.push_vec2(vec2(uv_bottom_right.x, uv_bottom_right.y));

        local.text_mesh.push_vec3(vec3(top_left.x, top_left.y, 0.0));
        local.text_mesh.push_vec2(vec2( uv_top_left.x, uv_top_left.y));

        local.text_mesh.push_vec3(vec3(bottom_right.x, bottom_right.y, 0.0));
        local.text_mesh.push_vec2(vec2(uv_bottom_right.x, uv_bottom_right.y));

        local.text_mesh.push_vec3(vec3(top_left.x, bottom_right.y, 1.0));
        local.text_mesh.push_vec2(vec2(uv_top_left.x, uv_bottom_right.y));

        triangles += 2;
    }

    // Draw buffer
    if triangles > 0 {
        unsafe { gl::Disable(gl::SCISSOR_TEST); }
        texture.bind(bindings::TextureSlot::BaseColor);
        shader.use_program();
        local.text_mesh.draw_arrays(gl::TRIANGLES, 0, triangles*3);
    }
}

/// The diagnostics system
fn update_diagnostics(diagnostics: Res<Diagnostics>, mut query: Query<(&DiagnosticsTextBox, &mut TextBox)>) {
    for (_, mut text_box) in query.iter_mut() {
        text_box.text = format!(
            "Update time: {}\nRender time: {}\nPlayer pos: {:.1}, {:.1}, {:.1}\nPlayer rot: {:.1}, {:.1}",
            format_duration(&diagnostics.update_time),
            format_duration(&diagnostics.render_time),
            diagnostics.player_pos.x, diagnostics.player_pos.y, diagnostics.player_pos.z,
            diagnostics.player_pitch_yaw.x, diagnostics.player_pitch_yaw.y);
    }
}

/// Format a Duration without using non-ascii characters (e.g. micro)
fn format_duration(duration: &Duration) -> String {
    let nanos = duration.as_nanos();

    if nanos < 1000 {
        format!("{nanos}ns")
    }
    else if nanos < 1000000 {
        let micros = nanos as f64 / 1000.0;
        format!("{micros}us")
    }
    else {
        let millis = nanos as f64 / 1000000.0;
        format!("{millis}ms")
    }
}
