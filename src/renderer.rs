pub mod resources;
pub mod shaders;
mod renderer_settings;
mod renderer_resources;

use bevy_ecs::schedule::SystemSet;
use bevy_ecs::system::{Local, Res, Query};
use cgmath::*;
use dreamfield_renderer::gl_backend::*;
use dreamfield_renderer::camera::Camera;

pub use renderer_settings::RendererSettings;
use renderer_resources::{RendererResources, RENDER_WIDTH, RENDER_HEIGHT};
use crate::sim::{PlayerCamera, SimTime, Ball};

/// The render systems
pub fn systems() -> SystemSet {
    SystemSet::new()
        .with_system(renderer_system)
}

/// The renderer system
fn renderer_system(mut local: Local<RendererResources>, renderer_settings: Res<RendererSettings>,
    sim_time: Res<SimTime>, player_query: Query<&PlayerCamera>, ball_query: Query<&Ball>)
{
    let local = &mut *local;

    // Update window size if it's changed
    if renderer_settings.is_changed() {
        let (width, height) = renderer_settings.window_size;
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
    let polygon_mode = match renderer_settings.wireframe_enabled {
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

    // Draw background
    unsafe { gl::Disable(gl::DEPTH_TEST) }
    local.sky_texture.bind(bindings::TextureSlot::BaseColor);
    local.sky_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);

    // Draw glfw models
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }

    // Update animations
    update_animation(&local.demo_scene_model, "Idle", sim_time.sim_time as f32);
    update_animation(&local.fire_orb_model, "Orb", sim_time.sim_time as f32);

    // Draw world
    local.ps1_shader_tess.use_program();
    let ubo_global = &mut local.ubo_global;
    let demo_scene_model = &mut local.demo_scene_model;
    demo_scene_model.render(ubo_global, true);

    // Draw other models
    local.ps1_shader.use_program();

    // Draw balls
    for ball in ball_query.iter() {
        local.fire_orb_model.set_transform(&Matrix4::from_translation(ball.pos));
        local.fire_orb_model.render(&mut local.ubo_global, false);
    }

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
    let (window_width, window_height) = renderer_settings.window_size;
    unsafe { gl::Viewport(0, 0, window_width, window_height) };
    local.framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
    local.blit_shader.use_program();
    local.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
}

/// Update an animation
/// TODO: have some sort of animation updater thingy instead
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
