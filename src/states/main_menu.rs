use bevy_ecs::prelude::*;
use cgmath::{vec2, Vector2, perspective, Deg, Matrix4, vec3, SquareMatrix, Quaternion, Rad, vec4, Rotation3};
use dreamfield_renderer::components::{PlayerCamera, TextBox};
use dreamfield_system::resources::{InputState, InputName};
use crate::app_state::AppState;

/// A tag component for entities we create as part of the main menu
#[derive(Component)]
pub struct MainMenuEntity;

/// Add main menu systems to the stage
pub fn init_main_menu(stage: &mut SystemStage) {
    stage.add_system_set(SystemSet::on_enter(AppState::MainMenu)
        .with_system(enter_main_menu));

    stage.add_system_set(SystemSet::on_update(AppState::MainMenu)
        .with_system(main_menu_system));

    stage.add_system_set(SystemSet::on_exit(AppState::MainMenu)
        .with_system(leave_main_menu));
}

/// Create main menu entities when we enter the main menu
fn enter_main_menu(mut commands: Commands) {
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
        .insert(MainMenuEntity)
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
        });

    // Create text
    commands.spawn()
        .insert(MainMenuEntity)
        .insert(TextBox::new("text", "medieval", "Vx8", "Dreamfield", None, Some(vec4(100.0, 10.0, 200.0, 200.0))));

    commands.spawn()
        .insert(MainMenuEntity)
        .insert(TextBox::new("text", "medieval", "Vx8", "Press jump to start", None, Some(vec4(10.0, 50.0, 100.0, 100.0))));
}

/// Remove main menu entities when we leave the main menu
fn leave_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuEntity>>) {
    log::info!("Leaving main menu");

    query.for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

/// Update the main menu
fn main_menu_system(input_state: Res<InputState>, mut app_state: ResMut<State<AppState>>) {
    if input_state.is_just_pressed(InputName::Jump) {
        log::info!("Main menu pressed");
        app_state.set(AppState::InGame).unwrap();
    }
}

