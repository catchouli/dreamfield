pub mod resources;
pub mod shaders;

use cgmath::*;
use dreamfield_renderer::gl_backend::*;
use dreamfield_renderer::camera::Camera;

const RENDER_WIDTH: i32 = 320;
const RENDER_HEIGHT: i32 = 240;

const RENDER_ASPECT: f32 = 4.0 / 3.0;

const FOV: f32 = 60.0;

const NEAR_CLIP: f32 = 0.01;
const FAR_CLIP: f32 = 35.0;

const FOG_START: f32 = FAR_CLIP - 10.0;
const FOG_END: f32 = FAR_CLIP - 5.0;

/// The renderer
pub struct Renderer {
    full_screen_rect: Mesh,
    sky_shader: ShaderProgram,
    pbr_shader: ShaderProgram,
    ps1_shader: ShaderProgram,
    blit_shader: ShaderProgram,
    sky_texture: Texture,
    demo_scene_model: GltfModel,
    fire_orb_model: GltfModel,
    ubo_global: UniformBuffer<GlobalParams>,
    framebuffer: Framebuffer,
    window_viewport: (i32, i32),
    ps1_mode: bool,
    wireframe_enabled: bool,
    ubo_lights: Option<UniformBuffer<LightParams>>
}

impl Renderer {
    /// Create a new Renderer
    pub fn new(width: i32, height: i32) -> Renderer {
        // Create uniform buffers
        let mut ubo_global = UniformBuffer::<GlobalParams>::new();
        ubo_global.set_fog_color(&vec3(0.0, 0.0, 0.0));
        ubo_global.set_fog_dist(&vec2(FOG_START, FOG_END));

        ubo_global.set_target_aspect(&RENDER_ASPECT);
        ubo_global.set_render_res(&vec2(RENDER_WIDTH as f32, RENDER_HEIGHT as f32));

        ubo_global.set_mat_proj(&perspective(Deg(FOV), RENDER_ASPECT, NEAR_CLIP, FAR_CLIP));

        ubo_global.bind(bindings::UniformBlockBinding::GlobalParams);

        // Load shaders
        let sky_shader = ShaderProgram::new_from_vf(shaders::SHADER_SKY);
        let pbr_shader = ShaderProgram::new_from_vf(shaders::SHADER_PBR);
        let ps1_shader = ShaderProgram::new_from_vtf(shaders::SHADER_PS1);
        let blit_shader = ShaderProgram::new_from_vf(shaders::SHADER_BLIT);

        // Load meshes
        let full_screen_rect = Mesh::new_indexed(
            &vec![
                 1.0,  1.0, 0.0, 1.0, 1.0,  // top right
                 1.0, -1.0, 0.0, 1.0, 0.0,  // bottom right
                -1.0, -1.0, 0.0, 0.0, 0.0,  // bottom left
                -1.0,  1.0, 0.0, 0.0, 1.0,  // top left
            ],
            &vec![
                0, 1, 3,
                1, 2, 3,
            ],
            &vec![
                VertexAttrib { index: 0, size: 3, attrib_type: gl::FLOAT },
                VertexAttrib { index: 1, size: 2, attrib_type: gl::FLOAT },
            ]);

        // Load textures
        let sky_texture = Texture::new_from_image_buf(resources::TEXTURE_CLOUD, Texture::NEAREST_WRAP)
            .expect("Failed to load sky texture");

        // Load models
        let demo_scene_model = GltfModel::from_buf(resources::MODEL_DEMO_SCENE, true).unwrap();
        println!("loading fire orb");
        let fire_orb_model = GltfModel::from_buf(resources::MODEL_FIRE_ORB, true).unwrap();

        // Build lights from demo scene (disabled)
        //let ubo_lights = Self::lights_from_models(demo_scene_model.lights());

        // Just build an empty light buffer for later
        let ubo_lights = Some(UniformBuffer::new());

        // Look for extra fields (unused so far)
        for drawable in demo_scene_model.drawables().iter() {
            if let Some(extra) = drawable.extras() {
                let raw = extra.get();
                println!("Node {} has extras: {:?}", drawable.name(), raw);
            }
        }

        // Create framebuffer
        let framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT);

        // Create renderer struct
        let mut renderer = Renderer {
           full_screen_rect,
           sky_shader,
           pbr_shader,
           sky_texture,
           ps1_shader,
           blit_shader,
           demo_scene_model,
           fire_orb_model,
           ubo_global,
           framebuffer,
           window_viewport: (width, height),
           ps1_mode: true,
           wireframe_enabled: false,
           ubo_lights
        };

        renderer.set_window_viewport(width, height);

        renderer
    }

    /// Render the game
    pub fn render(&mut self, game_state: &crate::GameState) {
        // Update global params
        self.ubo_global.set_sim_time(&(game_state.time as f32));
        self.ubo_global.set_mat_view_derive(&game_state.camera.get_view_matrix());
        self.ubo_global.upload_changed();

        // Bind framebuffer and clear
        self.set_gl_viewport(RENDER_WIDTH, RENDER_HEIGHT);
        self.framebuffer.bind_draw();

        // Enable or disable wireframe mode
        let polygon_mode = match self.wireframe_enabled {
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
        self.sky_texture.bind(bindings::TextureSlot::BaseColor);
        self.sky_shader.use_program();
        self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);

        // Draw glfw models
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
        }

        let main_shader = match self.ps1_mode {
            true => &self.ps1_shader,
            false => &self.pbr_shader
        };
        main_shader.use_program();

        // Update and bind lights
        if let Some(ubo_lights) = &mut self.ubo_lights {
            ubo_lights.set_lights(0, &Light {
                enabled: true.to_std140(),
                light_pos: game_state.camera.pos().to_std140(),
                light_dir: vec3(0.0, 0.0, 0.0).to_std140(),
                light_type: (LightType::PointLight as i32).to_std140(),
                color: vec3(1.0, 0.89, 0.8).to_std140(),
                intensity: (1000.0).to_std140(),
                range: (100.0).to_std140(),
                inner_cone_angle: (0.0).to_std140(),
                outer_cone_angle: (0.0).to_std140()
            });
            ubo_lights.bind(bindings::UniformBlockBinding::LightParams);
        }

        self.demo_scene_model.render(&mut self.ubo_global, true);
        self.fire_orb_model.set_transform(&Matrix4::from_translation(game_state.ball_pos));
        self.fire_orb_model.render(&mut self.ubo_global, true);

        // Unbind framebuffer
        self.framebuffer.unbind();

        // Render framebuffer to screen
        let (window_width, window_height) = self.window_viewport;
        self.set_gl_viewport(window_width, window_height);

        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Disable(gl::DEPTH_TEST)
        }

        self.framebuffer.bind_color_tex(bindings::TextureSlot::BaseColor);
        self.blit_shader.use_program();
        self.full_screen_rect.draw_indexed(gl::TRIANGLES, 6);
    }

    /// Toggle ps1 graphics mode
    pub fn toggle_graphics_mode(&mut self) {
        self.ps1_mode = !self.ps1_mode;
        println!("ps1 shader {}", if self.ps1_mode { "enabled" } else { "disabled "});
    }

    /// Update the window viewport
    pub fn set_window_viewport(&mut self, width: i32, height: i32) {
        println!("Setting viewport to {width} * {height}");
        self.window_viewport = (width, height);
        self.ubo_global.set_window_aspect(&(width as f32 / height as f32));
        self.ubo_global.upload_changed();
    }

    /// Toggle wireframe mode
    pub fn toggle_wireframe_mode(&mut self) {
        self.wireframe_enabled = !self.wireframe_enabled;
    }

    /// Update the gl viewport
    fn set_gl_viewport(&mut self, width: i32, height: i32) {
        unsafe { gl::Viewport(0, 0, width, height) };
    }

    /// Build a light ubo from the gltf light list
    fn _lights_from_models(lights: &[GltfLight]) -> Option<UniformBuffer<LightParams>> {
        match lights.len() {
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
        }
    }
}

