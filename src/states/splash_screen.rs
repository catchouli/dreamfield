use bevy_ecs::prelude::*;
use cgmath::{vec2, Vector2, perspective, Deg, Matrix4, vec3, Matrix3, SquareMatrix};
use dreamfield_renderer::components::{PlayerCamera, Visual, Animation, TextBox};
use dreamfield_system::components::Transform;
use std::time::{Instant, Duration};
use crate::app_state::AppState;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SplashScreenState {
    Start,
    NoSignal,
    Samy,
    CatStation,
}

/// The splash screen resource
pub struct SplashScreenResource {
    splash_screen_start: Instant,
    splash_screen_state: SplashScreenState,
    current_state_entities: Vec<Entity>,
}

/// A tag component we use to identify entities that were created as part of the splash screen
#[derive(Component)]
struct SplashScreenEntity;

/// The stages of the splash screen
const SPLASH_SCREEN_STAGES: [(Duration, SplashScreenState); 3] = [
    (Duration::from_millis(1500), SplashScreenState::NoSignal),
    (Duration::from_millis(3500), SplashScreenState::Samy),
    (Duration::from_millis(3000), SplashScreenState::CatStation)
];

/// Add splash screen systems to stage
pub fn init_splash_screen(stage: &mut SystemStage) {
    stage.add_system_set(SystemSet::on_enter(AppState::SplashScreen)
        .with_system(enter_splash_screen));

    stage.add_system_set(SystemSet::on_update(AppState::SplashScreen)
        .with_system(splash_screen_system));

    stage.add_system_set(SystemSet::on_exit(AppState::SplashScreen)
        .with_system(leave_splash_screen));
}

/// Create splash screen entities when we enter the splash screen
fn enter_splash_screen(mut commands: Commands) {
    log::info!("Entering splash screen");

    // Add timer resource
    commands.insert_resource(SplashScreenResource {
        splash_screen_start: Instant::now(),
        splash_screen_state: SplashScreenState::Start,
        current_state_entities: Vec::new(),
    });

    // Create camera
    const RENDER_RES: Vector2<f32> = vec2(1280.0, 960.0);
    const RENDER_ASPECT: f32 = RENDER_RES.x / RENDER_RES.y;
    const CLIP_RANGE: Vector2<f32> = vec2(0.1, 35.0);
    const FOV: f32 = 60.0;

    commands.spawn()
        .insert(SplashScreenEntity)
        .insert(PlayerCamera {
            proj: perspective(Deg(FOV), RENDER_ASPECT, CLIP_RANGE.x, CLIP_RANGE.y),
            view: Matrix4::identity(),
            clear_color: vec3(0.0, 0.0, 0.0),
            render_res: vec2(RENDER_RES.x, RENDER_RES.y),
            render_aspect: RENDER_ASPECT,
            render_fov_rad: FOV * std::f32::consts::PI / 180.0,
            clip_range: CLIP_RANGE,
            fog_color: vec3(0.0, 0.0, 0.0),
            fog_range: vec2(1000.0, 1000.0),
            render_world: false,
            simulate_composite: false,
        });
}

/// Remove splash screen entities when we leave the splash screen
fn leave_splash_screen(mut commands: Commands, query: Query<Entity, With<SplashScreenEntity>>) {
    log::info!("Leaving splash screen");
    
    commands.remove_resource::<SplashScreenResource>();

    query.for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

/// Update the splash screen
fn splash_screen_system(mut splash_screen: ResMut<SplashScreenResource>, mut app_state: ResMut<State<AppState>>,
    mut commands: Commands)
{
    // Figure out current splash screen state
    let elapsed = splash_screen.splash_screen_start.elapsed();
    let current_splash_state = {
        let mut current_state = None;

        let mut stage_start = Duration::ZERO;
        for (duration, state) in SPLASH_SCREEN_STAGES.iter() {
            let stage_end = stage_start + *duration;
            if elapsed >= stage_start && elapsed < stage_end {
                current_state = Some(state);
            }
            stage_start = stage_end;
        }

        current_state
    };

    if let Some(current_splash_state) = current_splash_state {
        // If this is different to the current state, switch to the new state
        if splash_screen.splash_screen_state != *current_splash_state {
            log::info!("Switching to splash screen state: {current_splash_state:?}");
            splash_screen.splash_screen_state = current_splash_state.clone();

            // Clear entities from previous stages
            splash_screen.current_state_entities.iter().for_each(|entity| {
                commands.entity(*entity).despawn();
            });
            splash_screen.current_state_entities.clear();

            // Initialise each state
            // TODO: Figure out how to fade parts in and out
            match current_splash_state {
                SplashScreenState::Start => {},
                SplashScreenState::NoSignal => {
                    splash_screen.current_state_entities.push(commands.spawn()
                        .insert(SplashScreenEntity)
                        .insert(TextBox::new("text", "medieval_4x", "Vx32", "No Signal", None, vec2(10.0, 10.0), None))
                        .id());
                },
                SplashScreenState::Samy => {
                    splash_screen.current_state_entities.push(commands.spawn()
                        .insert(SplashScreenEntity)
                        .insert(Transform::new(vec3(0.0, 0.0, -5.0), Matrix3::identity()))
                        .insert(Visual::new_with_anim("samy", false, Animation::Once("Samy".to_string())))
                        .id());
                },
                SplashScreenState::CatStation => {
                    splash_screen.current_state_entities.push(commands.spawn()
                        .insert(SplashScreenEntity)
                        .insert(TextBox::new("text", "medieval_4x", "Vx32", "CatStation", None, vec2(10.0, 10.0), None))
                        .id());
                },
            }
        }
    }
    else {
        // If we got to the end of our list of stages, continue to the main menu
        log::info!("Splash screen done after {elapsed:?}");
        app_state.set(AppState::MainMenu).unwrap();
    }
}

