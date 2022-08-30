use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, Query};
use cgmath::{Vector3, vec3};

use super::sim_time::SimTime;

/// The ball component
#[derive(Component)]
pub struct Ball {
    pub pos: Vector3<f32>
}

impl Default for Ball {
    fn default() -> Self {
        Ball {
            pos: vec3(0.0, 0.0, 0.0)
        }
    }
}

/// The ball update system
pub fn ball_update(sim_time: Res<SimTime>, mut query: Query<&mut Ball>)
{
    for mut ball in query.iter_mut() {
        let ball_height = sim_time.sim_time.sin() as f32 + 2.0;
        ball.pos = vec3(-9.0, ball_height, 9.0);
    }
}
