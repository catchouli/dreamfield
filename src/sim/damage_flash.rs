use bevy_ecs::system::{Res, ResMut};
use dreamfield_system::resources::{Diagnostics, SimTime};

/// The intensity the damage flash starts at when the player takes damage
pub const FLASH_TRIGGER_INTENSITY: f32 = 1.0;

/// How fast the damage flash intensity decays, per second
const FLASH_DECAY_RATE: f32 = 2.5;

/// Decay the damage flash intensity after the player takes damage
pub fn damage_flash_update(sim_time: Res<SimTime>, mut diagnostics: ResMut<Diagnostics>) {
    diagnostics.damage_flash = f32::max(0.0,
        diagnostics.damage_flash - FLASH_DECAY_RATE * sim_time.sim_time_delta as f32);
}
