use bevy_ecs::prelude::*;
use cgmath::{vec2, Vector2, perspective, Deg, Matrix4, vec3, Matrix3, SquareMatrix};
use dreamfield_renderer::components::{PlayerCamera, Visual, Animation};
use dreamfield_system::components::Transform;
use std::time::{Instant, Duration};
use crate::app_state::AppState;

const SPLASH_SCREEN_TIME: Duration = Duration::from_millis(3000);

/// The resource we use to track the splash screen state
struct SplashScreenState {
    start_time: Instant
}

/// A tag component we use to identify entities that were created as part of the splash screen
#[derive(Component)]
struct SplashScreenEntity;

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

    // Start splash screen timer
    commands.insert_resource(SplashScreenState { start_time: Instant::now() });

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

    // Samy
    commands.spawn()
        .insert(SplashScreenEntity)
        .insert(Transform::new(vec3(0.0, 0.0, -5.0), Matrix3::identity()))
        .insert(Visual::new_with_anim("samy", false, Animation::Once("Samy".to_string())));
}

/// Remove splash screen entities when we leave the splash screen
fn leave_splash_screen(mut commands: Commands, query: Query<Entity, With<SplashScreenEntity>>) {
    log::info!("Leaving splash screen");
    
    commands.remove_resource::<SplashScreenState>();

    query.for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

/// Update the splash screen
fn splash_screen_system(splash_screen: Res<SplashScreenState>, mut app_state: ResMut<State<AppState>>) {
    let elapsed = splash_screen.start_time.elapsed();
    if elapsed > SPLASH_SCREEN_TIME {
        log::info!("Splash screen done after {elapsed:?}");
        app_state.set(AppState::MainMenu).unwrap();
    }
}

