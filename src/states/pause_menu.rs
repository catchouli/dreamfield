use bevy_ecs::prelude::*;
use cgmath::vec2;
use dreamfield_renderer::components::TextBox;
use dreamfield_system::resources::{InputState, InputName};
use crate::app_state::AppState;

/// A tag component for entities we create as part of the pause menu
#[derive(Component)]
struct PauseMenuEntity;

/// Initialize pause menu state
pub fn init_pause_menu(stage: &mut SystemStage) {
    stage.add_system_set(SystemSet::on_enter(AppState::Paused)
        .with_system(enter_pause_menu));

    stage.add_system_set(SystemSet::on_update(AppState::Paused)
        .with_system(update_pause_menu));

    stage.add_system_set(SystemSet::on_exit(AppState::Paused)
        .with_system(leave_pause_menu));
}

/// Create entities when entering the pause menu
fn enter_pause_menu(mut commands: Commands) {
    log::info!("Entering pause menu");

    // Create text
    commands.spawn()
        .insert(PauseMenuEntity)
        .insert(TextBox::new("text", "medieval", "Vx8", "Paused", None, vec2(10.0, 60.0), None));
}

/// Create entities when leaving the pause menu
fn leave_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuEntity>>) {
    log::info!("Leaving pause menu");

    query.for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

/// Update the pause menu
fn update_pause_menu(mut input: ResMut<InputState>, mut app_state: ResMut<State<AppState>>) {
    if input.is_just_pressed(InputName::Pause) {
        input.clear_just_pressed(InputName::Pause);
        app_state.pop().unwrap();
    }
}
