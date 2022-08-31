pub mod renderer;
pub mod sim;
mod fixed_timestep;
mod bevy_ecs_game_host;

use cgmath::vec3;
use sim::{SimTime, PlayerCamera, PlayerMovement, Ball};
use sim::input::InputState;
use sim::level_collision::LevelCollision;
use renderer::RendererSettings;
use bevy_ecs::prelude::*;
use bevy_ecs::world::World;

use crate::bevy_ecs_game_host::BevyEcsGameHost;

/// The width of the window
const WINDOW_WIDTH: i32 = 1024 * 2;

/// The height of the window
const WINDOW_HEIGHT: i32 = 768 * 2;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 15;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

/// Initialize bevy world
fn init_world() -> World {
    // Create bevy world
    let mut world = World::default();

    // Initialise renderer settings
    world.insert_resource(RendererSettings::with_window_size((WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)));

    // Register other resources
    world.insert_resource(InputState::new());
    world.insert_resource(SimTime::new(0.0, FIXED_UPDATE_TIME));
    world.insert_resource(LevelCollision::new(renderer::resources::MODEL_DEMO_SCENE));

    // Create player entity
    world.spawn()
        // Entrance to village
        .insert(PlayerCamera::new(vec3(-125.1, 5.8, 123.8), 0.063, 0.099))
        // Entrance to cathedral
        //.insert(PlayerCamera::new(vec3(-99.988, 6.567, 75.533), -0.0367, 0.8334))
        // In corridor, going out
        //.insert(PlayerCamera::new(vec3(-45.99, 5.75, 17.37), 0.163, 1.7323))
        // Looking at torch
        //.insert(PlayerCamera::new(vec3(-33.04357, 4.42999, 15.564), 0.563, 2.499))
        // Looking at corridor
        //.insert(PlayerCamera::new(vec3(5.2, 0.8, 12.8), 0.03, 2.0))
        // Default dungeon pos
        //.insert(PlayerCamera::new(vec3(0.0, 1.0, 10.0), -0.17, 0.0))
        // Going outside
        //.insert(PlayerCamera::new(vec3(-53.925, 5.8, 19.56), 0.097, 1.57))
        .insert(PlayerMovement::default());

    // Create ball entity
    world.spawn()
        .insert(Ball::default());

    world
}

/// Create update schedule
fn create_update_schedule() -> Schedule {
    let mut update_schedule = Schedule::default();

    update_schedule.add_stage("sim", SystemStage::parallel()
        .with_system_set(sim::systems())
    );

    update_schedule
}

/// Create render schedule
fn create_render_schedule() -> Schedule {
    let mut render_schedule = Schedule::default();

    render_schedule.add_stage("render", SystemStage::single_threaded()
        .with_system_set(renderer::systems())
    );

    render_schedule
}


/// Entry point
fn main() {
    // Initialise logging
    env_logger::init();
    log::info!("Welcome to Dreamfield!");

    // Create game host
    let mut host = BevyEcsGameHost::new(WINDOW_WIDTH, WINDOW_HEIGHT, FIXED_UPDATE_TIME);

    // Initialise bevy ecs world
    let world = init_world();

    // Create update schedule
    let update_schedule = create_update_schedule();

    // Create render schedule
    // (this is separate from the update schedule, as we update at a fixed rate which is separate from the render)
    let render_schedule = create_render_schedule();

    // Run game
    host.run(world, update_schedule, render_schedule);
}
