mod renderer_resources;

use std::sync::Arc;

use bevy_ecs::schedule::SystemSet;
use bevy_ecs::system::{Local, Res, Query, ResMut};
use cgmath::Matrix4;
use crate::gl_backend::*;
use crate::resources::{ModelManager, TextureManager, ShaderManager};
use dreamfield_system::WindowSettings;
use renderer_resources::RendererResources;
use dreamfield_system::resources::SimTime;
use crate::components::{PlayerCamera, Position, Visual, ScreenEffect, RunTime};

/// The render systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(renderer_system)
}

/// The renderer system
fn renderer_system(mut local: Local<RendererResources>, window_settings: Res<WindowSettings>,
    sim_time: Res<SimTime>, models: Res<ModelManager>, mut textures: ResMut<TextureManager>,
    mut shaders: ResMut<ShaderManager>, mut effect_query: Query<&mut ScreenEffect>, player_query: Query<&PlayerCamera>,
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

    // Draw glfw models
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

    // Render pre-scene effects
    render_screen_effects(RunTime::PostScene, local, &mut textures, &mut shaders, &mut effect_query);

    // Run final composite
    final_composite(local, &window_settings);
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
