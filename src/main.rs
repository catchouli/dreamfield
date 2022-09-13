pub mod sim;
pub mod resources;

use cgmath::{vec3, Quaternion, vec2, Vector3, perspective, Deg, Matrix4, SquareMatrix, vec4, Vector2};
use bevy_ecs::prelude::*;
use bevy_ecs::world::World;

use dreamfield_renderer::renderer;
use dreamfield_renderer::components::{PlayerCamera, Visual, ScreenEffect, RunTime, Animation, TextBox, DiagnosticsTextBox};
use dreamfield_system::GameHost;
use dreamfield_system::components::Transform;

use dreamfield_system::systems::entity_spawner::EntitySpawnRadius;
use sim::{PlayerMovement, TestSphere, PlayerMovementMode, Ball};

/// The fixed update frequency
const FIXED_UPDATE: i32 = 15;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

/// The player position entering the village
const _VILLAGE_ENTRANCE: (Vector3<f32>, Vector2<f32>) = (vec3(-125.1, 5.8, 123.8), vec2(0.063, 0.099));

// Entrance to cathedral
const _CATHEDRAL_ENTRANCE: (Vector3<f32>, Vector2<f32>) = (vec3(-99.988, 6.567, 75.533), vec2(-0.0367, 0.8334));

// In corridor, going out
const _LEAVING_DUNGEON: (Vector3<f32>, Vector2<f32>) = (vec3(-53.925, 5.8, 19.56), vec2(0.097, 1.57));

// Looking at torch
const _LOOKING_AT_TORCH: (Vector3<f32>, Vector2<f32>) = (vec3(-33.04357, 4.42999, 15.564), vec2(0.563, 2.499));

// Looking at corridor
const _LOOKING_AT_CORRIDOR: (Vector3<f32>, Vector2<f32>) = (vec3(5.2, 0.8, 12.8), vec2(0.03, 2.0));

/// Initialize bevy world
fn init_entities(world: &mut World) {
    // Diagnostics
    let stats_bounds = vec4(10.0, 10.0, 310.0, 230.0);
    world.spawn()
        .insert(DiagnosticsTextBox)
        .insert(TextBox::new("text", "medieval", "Vx8", "", None, Some(stats_bounds)));

    // Create sky
    world.spawn()
        .insert(ScreenEffect::new(RunTime::PreScene, "sky", Some("sky")));

    // Create player
    let (initial_pos, initial_rot) = _VILLAGE_ENTRANCE;
    world.spawn()
        // Entrance to village
        .insert(EntitySpawnRadius::new(10.0))
        .insert(Transform::new(initial_pos, Quaternion::new(1.0, 0.0, 0.0, 0.0)))
        .insert(create_player_camera())
        .insert(PlayerMovement::new_pos_look(PlayerMovementMode::Normal, initial_rot));

    // Create fire orb
    world.spawn()
        .insert(Ball::default())
        .insert(Transform::new(vec3(-9.0, 0.0, 9.0), Quaternion::new(1.0, 0.0, 0.0, 0.0)))
        .insert(Visual::new_with_anim("fire_orb", false, Animation::Loop("Orb".to_string())));

    // Test sphere
    world.spawn()
        .insert(TestSphere {})
        .insert(Transform::new(vec3(-9.0, 0.5, 9.0), Quaternion::new(1.0, 0.0, 0.0, 0.0)))
        .insert(Visual::new("white_sphere", false));

    //world.spawn()
    //    .insert(Transform::new(vec3(8.0, 2.5, -2.85), Quaternion::new(1.0, 0.0, 0.0, 0.0)))
    //    .insert(Visual::new_with_anim("samy", false, Animation::Loop("Samy".to_string())));
    //world.spawn()
    //    .insert(Transform::new(vec3(0.0, 2.5, -2.85), Quaternion::new(1.0, 0.0, 0.0, 0.0)))
    //    .insert(Visual::new_with_anim("samy", false, Animation::Loop("Samy".to_string())));
    //world.spawn()
    //    .insert(Transform::new(vec3(-8.0, 2.5, -2.85), Quaternion::new(1.0, 0.0, 0.0, 0.0)))
    //    .insert(Visual::new_with_anim("samy", false, Animation::Loop("Samy".to_string())));
}

/// Create the PlayerCamera with all our renderer params
fn create_player_camera() -> PlayerCamera {
    const RENDER_WIDTH: i32 = 320;
    const RENDER_HEIGHT: i32 = 240;

    const RENDER_ASPECT: f32 = 4.0 / 3.0;

    const FOV: f32 = 60.0;
    const NEAR_CLIP: f32 = 0.1;
    const FAR_CLIP: f32 = 35.0;

    const FOG_START: f32 = FAR_CLIP - 10.0;
    const FOG_END: f32 = FAR_CLIP - 5.0;

    const FOG_COLOR: Vector3<f32> = vec3(0.0, 0.0, 0.0);

    let proj = perspective(Deg(FOV), RENDER_ASPECT, NEAR_CLIP, FAR_CLIP);
    let view = Matrix4::identity();

    PlayerCamera {
        proj,
        view,
        render_res: vec2(RENDER_WIDTH as f32, RENDER_HEIGHT as f32),
        render_aspect: RENDER_ASPECT,
        render_fov_rad: FOV * std::f32::consts::PI / 180.0,
        fog_color: FOG_COLOR,
        fog_range: vec2(FOG_START, FOG_END)
    }
}

/// Entry point
fn main() {
    // Initialise logging
    env_logger::init();
    log::info!("Welcome to Dreamfield!");

    // Create game host
    let mut host = GameHost::new(None, FIXED_UPDATE_TIME);

    // Create bevy world
    let mut world = World::default();

    // Initialise resources
    resources::add_resources(&mut world);

    // Create update schedule
    let mut update_schedule = Schedule::default();

    update_schedule.add_stage("sim", SystemStage::parallel()
        .with_system_set(dreamfield_system::systems())
        .with_system_set(sim::systems())
    );

    // Create render schedule
    let mut render_schedule = Schedule::default();

    render_schedule.add_stage("render", SystemStage::single_threaded()
        .with_system_set(renderer::systems())
    );

    // Initialise entities
    init_entities(&mut world);

    // Run game
    host.run(world, update_schedule, render_schedule);
}
