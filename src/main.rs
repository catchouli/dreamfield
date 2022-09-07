mod sim;
mod resources;

use cgmath::vec3;
use bevy_ecs::prelude::*;
use bevy_ecs::world::World;

use dreamfield_renderer::renderer;
use dreamfield_renderer::components::{PlayerCamera, Visual, Animation, Position, ScreenEffect, RunTime};
use dreamfield_system::{GameHost, WindowSettings};
use dreamfield_system::resources::{InputState, SimTime};

use sim::{PlayerMovement, Ball, TestSphere};
use sim::level_collision::LevelCollision;

/// The width of the window
const WINDOW_WIDTH: i32 = 1024 * 2;

/// The height of the window
const WINDOW_HEIGHT: i32 = 768 * 2;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 15;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

/// Initialise sim, returning the update bevy schedule
fn init_sim(world: &mut World) -> Schedule {
    // Sim resources
    world.insert_resource(InputState::new());
    world.insert_resource(SimTime::new(0.0, FIXED_UPDATE_TIME));
    world.init_resource::<LevelCollision>();

    // Create update schedule
    let mut update_schedule = Schedule::default();

    update_schedule.add_stage("sim", SystemStage::parallel()
        .with_system_set(sim::systems())
    );

    update_schedule
}

/// Initialise renderer, returning the render bevy schedule
fn init_renderer(world: &mut World) -> Schedule {
    // Renderer resources
    world.insert_resource(WindowSettings::with_window_size((WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)));
    world.insert_resource(resources::create_shader_manager());
    world.insert_resource(resources::create_texture_manager());
    world.insert_resource(resources::create_model_manager());
    world.insert_resource(resources::create_world_chunk_manager());

    // Create render schedule
    let mut render_schedule = Schedule::default();

    render_schedule.add_stage("render", SystemStage::single_threaded()
        .with_system_set(renderer::systems())
    );

    render_schedule
}

/// Initialize bevy world
fn init_entities(world: &mut World) {
    // Create sky
    world.spawn()
        .insert(ScreenEffect::new(RunTime::PreScene, "sky", Some("sky")));

    // Create player
    world.spawn()
        // Entrance to village
        //.insert(PlayerCamera::new(vec3(-125.1, 5.8, 123.8), 0.063, 0.099))
        //.insert(PlayerMovement::new(vec3(-125.1, 5.8, 123.8), vec3(0.0, 0.0, 0.0)));
        // Entrance to cathedral
        //.insert(PlayerCamera::new(vec3(-99.988, 6.567, 75.533), -0.0367, 0.8334))
        // In corridor, going out
        //.insert(PlayerCamera::new(vec3(-45.99, 5.75, 17.37), 0.163, 1.7323))
        // Looking at torch
        //.insert(PlayerCamera::new(vec3(-33.04357, 4.42999, 15.564), 0.563, 2.499))
        // Looking at corridor
        //.insert(PlayerCamera::new(vec3(5.2, 0.8, 12.8), 0.03, 2.0))
        // Default dungeon pos
        // TODO: split the orientation out into PlayerMovement, and then make this be initialized
        // by that. Add a PlayerCamera::from_player_movement() or something so the logic isn't
        // duplicated.
        //.insert(PlayerCamera::new(vec3(0.0, 0.5 + 1.8 - 0.1, 10.0), -0.17, 0.0))
        //.insert(PlayerMovement::new(vec3(0.0, 0.5, 10.0), vec3(0.0, 0.0, 0.0)));
        // Going outside
        //.insert(PlayerCamera::new(vec3(-53.925, 5.8, 19.56), 0.097, 1.57))
        // Test spherecast
        .insert(PlayerCamera::new(vec3(-45.885677, 6.7, 20.211102), -0.50333333, 0.7666667))
        .insert(PlayerMovement::new(vec3(-45.885677, 6.7 - 1.7, 20.211102), vec3(0.0, 0.0, 0.0)));

    // Create fire orb
    //world.spawn()
    //    .insert(Ball::default())
    //    .insert(Position::new(vec3(-9.0, 0.0, 9.0)))
    //    .insert(Visual::new_with_anim("fire_orb", false, Animation::Loop("Orb".to_string())));

    world.spawn()
        .insert(TestSphere {})
        .insert(Position::new(vec3(-9.0, 0.5, 9.0)))
        .insert(Visual::new("white_sphere", false));
}

/// Entry point
fn main() {
    // Initialise logging
    env_logger::init();
    log::info!("Welcome to Dreamfield!");

    // Create game host
    let mut host = GameHost::new(WINDOW_WIDTH, WINDOW_HEIGHT, FIXED_UPDATE_TIME);

    // Create bevy world
    let mut world = World::default();

    // Initialise renderer
    let render_schedule = init_renderer(&mut world);

    // Initialise sim
    let update_schedule = init_sim(&mut world);

    // Initialise entities
    init_entities(&mut world);

    // Run game
    host.run(world, update_schedule, render_schedule);
}
