use bevy_ecs::{system::{Res, Query, ParamSet}, prelude::Component};
use cgmath::{Vector3, InnerSpace, vec3, Matrix3, SquareMatrix};
use dreamfield_renderer::components::PlayerCamera;
use dreamfield_system::resources::{SimTime, InputState, InputName};
use dreamfield_system::components::Transform as TransformComponent;

use super::PlayerMovement;

/// Minecart component
#[derive(Component)]
pub struct Minecart {
    track_segments: Vec<TrackSegment>,
    pos: f32,
    velocity: f32,
}

struct TrackSegment {
    a: Vector3<f32>,
    b: Vector3<f32>,
    segment_length: f32,
    segment_start: f32,
    segment_end: f32,
}

impl Minecart {
    pub fn new(track_points: Vec<Vector3<f32>>) -> Self {
        let mut track_segments = Vec::new();

        let mut segment_start = 0.0;
        for (a, b) in track_points.iter().zip(track_points.iter().skip(1)) {
            let segment_length = (a - b).magnitude();
            let segment_end = segment_start + segment_length;

            track_segments.push(TrackSegment {
                a: *a,
                b: *b,
                segment_length,
                segment_start,
                segment_end,
            });

            segment_start += segment_length;
        }

        Minecart {
            track_segments,
            pos: 0.0,
            velocity: 0.0,
        }
    }

    fn get_segment(&self, pos: f32) -> Option<&TrackSegment> {
        for segment in self.track_segments.iter() {
            if pos < segment.segment_end {
                return Some(segment);
            }
        }

        self.track_segments.last()
    }

    fn get_pos(&self, pos: f32) -> Option<Vector3<f32>> {
        self.get_segment(pos)
            .map(|segment| {
                let pos_in_segment = f32::clamp((f32::min(pos, segment.segment_end) - segment.segment_start) / segment.segment_length, 0.0, 1.0);
                let seg_diff = segment.b - segment.a;
                segment.a + seg_diff * pos_in_segment
            })
    }
}

pub fn update_minecart(sim_time: Res<SimTime>,
                       input: Res<InputState>,
                       mut param_set: ParamSet<(
                           Query<(&mut Minecart, &mut TransformComponent)>,
                           Query<(&PlayerCamera, &mut TransformComponent, &mut PlayerMovement)>)>)
{
    const MAX_SPEED: f32 = 5.0;
    const SPEED_LOSS_PER_SECOND: f32 = 2.5;
    const SPEED_LOSS_PER_SECOND_RIDING: f32 = 0.1;
    const STOP_SPEED: f32 = 1.0;
    
    let (player_in_minecart, player_pos) = {
        let query = param_set.p1();
        let (_, transform, movement) = query.single();
        (!movement.enabled, transform.pos)
    };

    let mut player_in_minecart_pos = None;

    for (mut minecart, mut transform) in param_set.p0().iter_mut() {
        let forward_dir = transform.rot * vec3(0.0, 0.0, -1.0);
        let to_player = player_pos - transform.pos;
        let dist_to_player = f32::max(0.1, to_player.magnitude());
        if dist_to_player < 1.5 && !player_in_minecart {
            minecart.velocity += 2.5 * -forward_dir.dot(to_player / dist_to_player);
        }

        let speed_loss_per_second = if player_in_minecart {
            SPEED_LOSS_PER_SECOND_RIDING
        }
        else {
            SPEED_LOSS_PER_SECOND
        };

        let cur_speed = f32::abs(minecart.velocity);
        let speed_loss = speed_loss_per_second * sim_time.sim_time_delta as f32;
        let mut new_speed = f32::clamp(cur_speed - speed_loss, 0.0, MAX_SPEED);
        if new_speed < STOP_SPEED {
            new_speed = 0.0;
        }
        minecart.velocity = minecart.velocity.signum() * new_speed;

        let new_pos_track = minecart.pos + minecart.velocity * sim_time.sim_time_delta as f32;

        if let Some(new_segment) = minecart.get_segment(new_pos_track) {
            let new_pos = minecart.get_pos(new_pos_track).unwrap();
            let movement_dir = (new_segment.b - new_segment.a).normalize();

            let up = vec3(0.0, 1.0, 0.0);
            let forward = -movement_dir;
            let right = up.cross(forward);
            let look_at = Matrix3::new(right.x, up.x, forward.x, right.y, up.y, forward.y, right.z, up.z, forward.z);

            // I'm not really sure why we have to invert this...
            // TODO: make sure coordinate system is correct
            transform.rot = look_at.invert().unwrap();
            transform.pos = new_pos;
            minecart.pos = new_pos_track;
        }

        if player_in_minecart {
            player_in_minecart_pos = Some(transform.pos);
        }

        if input.is_just_pressed(InputName::Use) && dist_to_player < 3.0 {
            if player_in_minecart {
                player_in_minecart_pos = None;
            }
            else {
                player_in_minecart_pos = Some(transform.pos);
            }
        }
    }

    if let Some(player_in_minecart_pos) = player_in_minecart_pos {
        let mut query = param_set.p1();
        let (_, mut transform, mut movement) = query.single_mut();
        movement.enabled = false;
        transform.pos = player_in_minecart_pos;
    }
    else {
        let mut query = param_set.p1();
        let (_, _, mut movement) = query.single_mut();
        movement.enabled = true;
    }
}
