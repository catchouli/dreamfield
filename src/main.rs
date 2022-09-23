mod sim;
mod resources;
mod states;
mod app_state;

use bevy_ecs::prelude::*;
use bevy_ecs::world::World;
use dreamfield_system::GameHost;
use app_state::AppState;

/// The initial app state
const INITIAL_STATE: AppState = AppState::SplashScreen;

/// The fixed update frequency
const FIXED_UPDATE: i32 = 15;

/// The fixed update target time
const FIXED_UPDATE_TIME: f64 = 1.0 / (FIXED_UPDATE as f64);

// Create update schedule
fn create_update_schedule(world: &mut World) -> Schedule {
    // Add app state with initial value SplashScreen
    world.insert_resource(State::new(INITIAL_STATE));

    // Create main update stage, right now this has to be one big stage, because the app state
    // can't be shared between stages
    let mut update_stage = SystemStage::parallel()
        .with_system_set(State::<AppState>::get_driver());

    states::splash_screen::init_splash_screen(&mut update_stage);
    states::title_screen::init_title_screen(&mut update_stage);
    states::main_game::init_main_game(&mut update_stage);
    states::pause_menu::init_pause_menu(&mut update_stage);

    Schedule::default().with_stage("main_update", update_stage)
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

    // Initialise system and renderer
    dreamfield_system::init(&mut world);
    dreamfield_renderer::init(&mut world,
        resources::create_model_manager(),
        resources::create_shader_manager(),
        resources::create_texture_manager(),
        resources::create_font_manager(),
        resources::create_world_chunk_manager());

    // Create update schedule
    let update_schedule = create_update_schedule(&mut world);

    // Create render schedule
    let render_schedule = Schedule::default()
        .with_stage("render", SystemStage::single_threaded()
            .with_system_set(dreamfield_renderer::systems()));

    // Run game
    host.run(world, update_schedule, render_schedule);
}
