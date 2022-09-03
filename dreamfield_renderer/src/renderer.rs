mod renderer_resources;

use std::sync::Arc;

use bevy_ecs::schedule::SystemSet;
use bevy_ecs::system::{Local, Res, Query, ResMut};
use cgmath::{SquareMatrix, Matrix4, vec2, InnerSpace};
use dreamfield_system::world::world_chunk::WorldChunk;
use renderer_resources::RendererResources;
use crate::gl_backend::*;
use crate::gl_backend::bindings::AttribBinding;
use crate::camera::Camera;
use crate::resources::{ModelManager, TextureManager, ShaderManager};
use crate::components::{PlayerCamera, Position, Visual, ScreenEffect, RunTime};
use dreamfield_system::WindowSettings;
use dreamfield_system::world::WorldChunkManager;
use dreamfield_system::resources::SimTime;

pub const RENDER_WIDTH: i32 = 320;
pub const RENDER_HEIGHT: i32 = 240;

pub const RENDER_ASPECT: f32 = 4.0 / 3.0;

pub const FOV: f32 = 60.0;

pub const NEAR_CLIP: f32 = 0.1;
pub const FAR_CLIP: f32 = 35.0;

pub const FOG_START: f32 = FAR_CLIP - 10.0;
pub const FOG_END: f32 = FAR_CLIP - 5.0;

/// The render systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(renderer_system)
}

/// The renderer system
fn renderer_system(mut local: Local<RendererResources>, window_settings: Res<WindowSettings>,
    sim_time: Res<SimTime>, models: Res<ModelManager>, mut textures: ResMut<TextureManager>,
    mut world: ResMut<WorldChunkManager>, mut shaders: ResMut<ShaderManager>,
    mut effect_query: Query<&mut ScreenEffect>, player_query: Query<&PlayerCamera>,
    mut visuals_query: Query<(&Position, &mut Visual)>)
{
    let local = &mut *local;

    // Update window size if it's changed
    if window_settings.is_changed() {
        let (width, height) = window_settings.window_size;
        local.ubo_global.set_window_aspect(&(width as f32 / height as f32));
    }

    // Get player camera
    let player_camera = player_query.get_single().expect("Expected one player camera");

    // Render game
    // Update global params
    local.ubo_global.set_sim_time(&(sim_time.sim_time as f32));
    local.ubo_global.set_mat_view_derive(&player_camera.camera.get_view_matrix());
    local.ubo_global.upload_changed();

    // Bind framebuffer and clear
    unsafe { gl::Viewport(0, 0, RENDER_WIDTH, RENDER_HEIGHT) };
    local.framebuffer.bind_draw();
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

    // Draw visuals
    draw_visuals(local, &sim_time, &models, &mut visuals_query);
    draw_world(local, &mut world, &player_camera);

    // Render pre-scene effects
    render_screen_effects(RunTime::PostScene, local, &mut textures, &mut shaders, &mut effect_query);

    // Run final composite
    final_composite(local, &window_settings);
}

/// Draw the world
fn draw_world(local: &mut RendererResources, world: &mut ResMut<WorldChunkManager>, camera: &PlayerCamera) {
    unsafe { gl::Enable(gl::DEPTH_TEST); }
    local.ps1_tess_shader.use_program();

    local.ubo_global.set_mat_model_derive(&Matrix4::identity());
    local.ubo_global.upload_changed();

    // Get camera pos
    let pos = camera.camera.pos();
    let forward = camera.camera.forward();

    // Work out what chunks can be seen by creating a triangle between the camera position and the
    // corners of the far clip plane, in 2D, and then walk all the chunks between them and draw them
    let pos_xz = vec2(pos.x, pos.z);
    let forward_xz = vec2(forward.x, forward.z).normalize();

    // The view of the world grid (in 2d) forms a triangle between the corners of the far clip
    // plane and the camera position. To draw the chunks within the camera view, we want to figure
    // out those two points in the distance.
    //
    // To do this, we can divide this triangle into two right-angle triangles, where the line from
    // the camera position to the point straight ahead of the camera on the far clip plane form one
    // edge, and the hypotenuse of each triangle then points from the camera position to the corner
    // of the far clip plane.
    //
    // To figure out the corner points, we then rotate the forward vector of the camera by half of
    // the FOV to get a direction to it, and then use trigonometry to work out the length of the
    // hypotenuse:
    //   cos(fov / 2) = FAR / corner_dist
    //   corner_dist = FAR / cos(fov / 2)
    //   triangle_corner = camera_pos + triangle_edge_dir * corner_dist
    let half_fov = FOV / 2.0;

    let triangle_edge_dir_1 = vec2(
        forward_xz.x * f32::cos(half_fov) - forward_xz.y * f32::sin(half_fov),
        forward_xz.x * f32::sin(half_fov) + forward_xz.y * f32::cos(half_fov)
    ).normalize();
    let triangle_edge_dir_2 = vec2(
        forward_xz.x * f32::cos(-half_fov) - forward_xz.y * f32::sin(-half_fov),
        forward_xz.x * f32::sin(-half_fov) + forward_xz.y * f32::cos(-half_fov)
    ).normalize();

    let corner_dist = FAR_CLIP / f32::cos(half_fov);

    let triangle_corner_1 = pos_xz + triangle_edge_dir_1 * corner_dist;
    let triangle_corner_2 = pos_xz + triangle_edge_dir_2 * corner_dist;

    // Then, take the min and max of all three points, and use it to create an AABB for the view.
    // We can then draw all world chunks that intersect this AABB. As an optimization, we could
    // draw only the ones that are actually within the triangle, but I don't think it's necessary.
    let view_aabb_min = vec2(
        f32::min(pos_xz.x, f32::min(triangle_corner_1.x, triangle_corner_2.x)),
        f32::min(pos_xz.y, f32::min(triangle_corner_1.y, triangle_corner_2.y))
    );
    let view_aabb_max = vec2(
        f32::max(pos_xz.x, f32::max(triangle_corner_1.x, triangle_corner_2.x)),
        f32::max(pos_xz.y, f32::max(triangle_corner_1.y, triangle_corner_2.y))
    );

    // Get chunk indexes at corners
    let (view_min_chunk_x, view_min_chunk_z) = WorldChunk::point_to_chunk_index_2d(&view_aabb_min);
    let (view_max_chunk_x, view_max_chunk_z) = WorldChunk::point_to_chunk_index_2d(&view_aabb_max);

    let chunk_count = (view_max_chunk_x - view_min_chunk_x) * (view_max_chunk_z - view_min_chunk_z);

    println!("drawing chunks from {}, {} to {}, {} ({} chunks)", view_min_chunk_x, view_min_chunk_z, view_max_chunk_x, view_max_chunk_z, chunk_count);

    for chunk_x in view_min_chunk_x..=view_max_chunk_x {
        for chunk_z in view_min_chunk_z..=view_max_chunk_z {
            if let Some(chunk) = world.get_chunk((chunk_x, chunk_z)) {
                draw_world_chunk(local, &chunk);
            }
        }
    }
}

/// Draw a WorldChunk
fn draw_world_chunk(local: &mut RendererResources, chunk: &WorldChunk) {
    // Render each mesh in chunk
    for mesh in chunk.meshes().iter() {
        // Get or load gl mesh
        let gl_mesh = local.world_meshes
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
                let mesh = Mesh::new_indexed(mesh.vertices(), &index_buffer, &buffer_layout);

                mesh
            });

        // TODO: bind materials/textures and set up state properly
        gl_mesh.draw_indexed(gl::PATCHES, mesh.indices().len() as i32);
    }
}

/// Draw the visuals
fn draw_visuals(local: &mut RendererResources, sim_time: &Res<SimTime>, models: &Res<ModelManager>,
    visuals_query: &mut Query<(&Position, &mut Visual)>)
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
        let transform = Matrix4::from_translation(pos.pos);
        model.render(&transform, ubo_global, ubo_joints, visual.tessellate);
    }
}

/// Render a screen effect
/// TODO: these aren't that useful for anything but the sky if you can't read the framebuffer :)
pub fn render_screen_effects(run_time: RunTime, local: &RendererResources, texture_manager: &mut ResMut<TextureManager>,
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
pub fn final_composite(local: &RendererResources, window_settings: &Res<WindowSettings>) {
    // Disable depth test for blitting operations
    unsafe {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        gl::Disable(gl::DEPTH_TEST);
    }

    // Composite simulation: convert rgb to yiq color space
    // No SRGB conversion, since we're outputting colors in the YIQ color space. Additionally
    // we're writing to an f32 framebuffer already anyway to avoid precision issues.
    unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB) };
    local.yiq_framebuffer.bind_draw();
    local.framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    local.composite_yiq_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);

    // Composite simulation: resolve back to regular framebuffer
    // This time we're outputting back to our srgb framebuffer so we enable sRGB again.
    // Annoyingly the YIQ conversion already outputs sRGB colors, so we have to convert them
    // back to linear in the shader, just for them to be converted back into sRGB. Oh well.
    unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB); }
    local.framebuffer.bind_draw();
    local.yiq_framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    unsafe { gl::GenerateMipmap(gl::TEXTURE_2D) };
    local.composite_resolve_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
    local.framebuffer.unbind();

    // Render framebuffer to screen
    let (window_width, window_height) = window_settings.window_size;
    unsafe { gl::Viewport(0, 0, window_width, window_height) };
    local.framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    local.blit_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
}

/// Update an animation
pub fn update_animation(model: &GltfModel, name: &str, time: f32) {
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
