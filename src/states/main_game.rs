use bevy_ecs::prelude::*;
use cgmath::{vec2, perspective, Deg, Matrix4, vec3, Matrix3, SquareMatrix, Vector3, Vector2};
use dreamfield_renderer::components::{PlayerCamera, Visual, Animation, DiagnosticsTextBox, TextBox, ScreenEffect, RunTime};
use dreamfield_system::{components::{Transform, EntityName}, systems::entity_spawner::EntitySpawnRadius, resources::{InputState, InputName}};
use crate::{app_state::AppState, sim::{PlayerMovement, PlayerMovementMode, Ball}};

/// The player position entering the village
const _VILLAGE_ENTRANCE: (Vector3<f32>, Vector2<f32>) = (vec3(-125.1, 5.8, 123.8), vec2(0.063, -0.5));

/// Entrance to cathedral
const _CATHEDRAL_ENTRANCE: (Vector3<f32>, Vector2<f32>) = (vec3(-99.988, 6.567, 75.533), vec2(-0.0367, 0.8334));

/// In corridor, going out
const _LEAVING_DUNGEON: (Vector3<f32>, Vector2<f32>) = (vec3(-53.925, 5.8, 19.56), vec2(0.097, 1.57));

/// Looking at torch
const _LOOKING_AT_TORCH: (Vector3<f32>, Vector2<f32>) = (vec3(-33.04357, 4.42999, 15.564), vec2(0.563, 2.499));

/// Looking at corridor
const _LOOKING_AT_CORRIDOR: (Vector3<f32>, Vector2<f32>) = (vec3(5.2, 0.8, 12.8), vec2(0.03, 2.0));

/// Main game resource
pub struct MainGameResource {
    diagnostics_entity: Option<Entity>,
}

/// Initialise main game state
pub fn init_main_game(stage: &mut SystemStage) {
    stage.add_system_set(SystemSet::on_enter(AppState::MainGame)
        .with_system(enter_main_game));
    stage.add_system_set(SystemSet::on_update(AppState::MainGame)
        .with_system(update_main_game));
    stage.add_system_set(dreamfield_system::systems()
        .with_run_criteria(State::<AppState>::on_update(AppState::MainGame)));
    stage.add_system_set(crate::sim::systems()
        .with_run_criteria(State::<AppState>::on_update(AppState::MainGame)));
}

/// Create main game entities when entering the main game state
fn enter_main_game(mut commands: Commands) {
    log::info!("Entering main game");

    // Add resource
    commands.insert_resource(MainGameResource {
        diagnostics_entity: None,
    });

    // Create sky pre-scene effect
    commands.spawn()
        .insert(ScreenEffect::new(RunTime::PreScene, "sky", Some("sky")));

    // Create player
    let (initial_pos, initial_rot) = _LOOKING_AT_CORRIDOR;
    commands.spawn()
        .insert(EntityName::new("Player"))
        // Entrance to village
        .insert(Transform::new(initial_pos, Matrix3::identity()))
        .insert(PlayerMovement::new_pos_look(PlayerMovementMode::Normal, initial_rot))
        .insert(PlayerMovement::collider())
        .insert(create_player_camera())
        .insert(EntitySpawnRadius::new(10.0));

    // Create fire orb
    commands.spawn()
        .insert(Ball::default())
        .insert(Transform::new(vec3(-9.0, 0.0, 9.0), Matrix3::identity()))
        .insert(Visual::new("fire_orb", "ps1", false, Some(Animation::Loop("Orb".to_string()))));
}

/// Update the main game
fn update_main_game(
    mut local: ResMut<MainGameResource>,
    mut input: ResMut<InputState>,
    mut app_state: ResMut<State<AppState>>,
    mut commands: Commands)
{
    // Pause when pause is pressed
    if input.is_just_pressed(InputName::Pause) {
        // Clear the input, because otherwise the pause system will run during the same update, and
        // might switch back to this state.
        input.clear_just_pressed(InputName::Pause);
        app_state.push(AppState::Paused).unwrap();
    }

    // Enable or disable diagnostics
    if input.is_just_pressed(InputName::EnableDiagnostics) {
        if let Some(entity_id) = local.diagnostics_entity.take() {
            commands.entity(entity_id).despawn();
        }
        else {
            local.diagnostics_entity = Some(commands
                .spawn()
                .insert(DiagnosticsTextBox)
                .insert(TextBox::new("text", "medieval", "Vx8", "", None, vec2(10.0, 10.0), None))
                .id());
        }
    }
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
        clear_color: vec3(0.0, 0.0, 0.0),
        render_res: vec2(RENDER_WIDTH as f32, RENDER_HEIGHT as f32),
        render_aspect: RENDER_ASPECT,
        render_fov_rad: FOV * std::f32::consts::PI / 180.0,
        clip_range: vec2(NEAR_CLIP, FAR_CLIP),
        fog_color: FOG_COLOR,
        fog_range: vec2(FOG_START, FOG_END),
        render_world: true,
        simulate_composite: true,
    }
}


