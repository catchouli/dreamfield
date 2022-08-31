use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, Query};

use dreamfield_renderer::components::Position;
use dreamfield_system::resources::SimTime;

/// The ball component
#[derive(Component)]
pub struct Ball {
}

impl Default for Ball {
    fn default() -> Self {
        Ball {}
    }
}

/// The ball update system
pub fn ball_update(sim_time: Res<SimTime>, mut query: Query<(&mut Ball, &mut Position)>)
{
    for (_, mut pos) in query.iter_mut() {
        let ball_height = sim_time.sim_time.sin() as f32 + 2.0;
        pos.pos.y = ball_height;
    }
}
