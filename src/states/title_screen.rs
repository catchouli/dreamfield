use dreamfield_renderer::components::{ScreenEffect, RunTime};
use bevy_ecs::prelude::*;
use cgmath::{vec2, Vector2, perspective, Deg, Matrix4, vec3, SquareMatrix, Quaternion, Rad, Rotation3};
use dreamfield_renderer::components::{PlayerCamera, TextBox};
use dreamfield_system::resources::{InputState, InputName};
use crate::app_state::AppState;

/// A tag component for entities we create as part of the title screen
#[derive(Component)]
pub struct TitleScreenEntity;

/// Add title screen systems to the stage
pub fn init_title_screen(stage: &mut SystemStage) {
    stage.add_system_set(SystemSet::on_enter(AppState::TitleScreen)
        .with_system(enter_title_screen));

    stage.add_system_set(SystemSet::on_update(AppState::TitleScreen)
        .with_system(title_screen_system));

    stage.add_system_set(SystemSet::on_exit(AppState::TitleScreen)
        .with_system(leave_title_screen));
}

/// Create title screen entities when we enter the title screen
fn enter_title_screen(mut commands: Commands) {
    // Create camera
    const RENDER_RES: Vector2<f32> = vec2(640.0, 480.0);
    const RENDER_ASPECT: f32 = RENDER_RES.x / RENDER_RES.y;
    const CLIP_RANGE: Vector2<f32> = vec2(0.1, 100.0);
    const FOV: f32 = 60.0;

    let pitch = Quaternion::from_axis_angle(vec3(1.0, 0.0, 0.0), Rad(-0.3));
    let yaw = Quaternion::from_axis_angle(vec3(0.0, 1.0, 0.0), Rad(-0.6));
    let orientation = pitch * yaw;
    let view = (Matrix4::from_translation(vec3(-139.9, 20.7, 70.2)) * Matrix4::from(orientation)).invert().unwrap();
    commands.spawn()
        .insert(TitleScreenEntity)
        .insert(PlayerCamera {
            proj: perspective(Deg(FOV), RENDER_ASPECT, CLIP_RANGE.x, CLIP_RANGE.y),
            view,
            clear_color: vec3(0.0, 0.0, 0.0),
            render_res: vec2(RENDER_RES.x, RENDER_RES.y),
            render_aspect: RENDER_ASPECT,
            render_fov_rad: FOV * std::f32::consts::PI / 180.0,
            clip_range: CLIP_RANGE,
            fog_color: vec3(0.0, 0.0, 0.0),
            fog_range: vec2(1000.0, 1000.0),
            render_world: true,
            simulate_composite: false,
        });

    // Create sky pre-scene effect
    commands.spawn()
        .insert(ScreenEffect::new(RunTime::PreScene, "sky", Some("sky")));

    // Create text
    commands.spawn()
        .insert(TitleScreenEntity)
        .insert(TextBox::new("text", "medieval_4x", "Vx32", "Dreamfield", None, vec2(250.0, 240.0), None));

    commands.spawn()
        .insert(TitleScreenEntity)
        .insert(TextBox::new("text", "medieval_2x", "Vx16", "Press jump to start", None, vec2(250.0, 280.0), None));
}

/// Remove title screen entities when we leave the title screen
fn leave_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenEntity>>) {
    log::info!("Leaving title screen");

    query.for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

/// Update the title screen
fn title_screen_system(input_state: Res<InputState>, mut app_state: ResMut<State<AppState>>) {
    if input_state.is_just_pressed(InputName::Jump) {
        log::info!("Main menu pressed");
        app_state.set(AppState::MainGame).unwrap();
    }
}

