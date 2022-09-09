use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, Vector2, Zero, Quaternion, Rad, Rotation3, Matrix4, SquareMatrix, InnerSpace};
use dreamfield_system::world::WorldChunkManager;

use super::{TestSphere, intersection};
use super::intersection::{Plane, Sphere};
use super::level_collision::LevelCollision;
use dreamfield_renderer::components::{PlayerCamera, Position};
use dreamfield_system::resources::{SimTime, InputName, InputState};

/// The character height
const CHAR_HEIGHT: f32 = 1.8;

/// The character's collider radius
const COLLIDER_RADIUS: f32 = 0.5;

/// Height that we can step up onto objects
const _STEP_OFFSET: f32 = 0.2;

/// The minimum ground_normal y value to stop you from walking on steep slopes
const MIN_WALK_NORMAL: f32 = 0.7;

/// The camera look speed
const CAM_LOOK_SPEED: f32 = 0.5;

/// The camera fast look speed
const CAM_LOOK_SPEED_FAST: f32 = 1.5;

/// The camera move speed
const CAM_MOVE_SPEED: f32 = 4.0;

/// The camera fast move speed
const CAM_MOVE_SPEED_FAST: f32 = 12.0;

/// The gravity acceleration
const GRAVITY_ACCELERATION: f32 = 9.8;
//const GRAVITY_ACCELERATION: f32 = 0.0;

/// The character eye level. According to a cursory google, this is on average 4.5" below the top
/// of your head, which is just over 10cm
const CHAR_EYE_LEVEL: f32 = CHAR_HEIGHT - 0.10;

/// The world forward direction
const WORLD_FORWARD: Vector3<f32> = vec3(0.0, 0.0, -1.0);

/// The world up direction
const WORLD_UP: Vector3<f32> = vec3(0.0, 1.0, 0.0);

/// The world right direction
const WORLD_RIGHT: Vector3<f32> = vec3(1.0, 0.0, 0.0);

//const OVER_CLIP: f32 = 1.001;

//const PM_AIR_ACCELERATE: f32 = 100.0;
const PM_ACCELERATE: f32 = 10.0;
//const STEPSIZE: f32 = 0.1;

/// The PlayerMovement component
#[derive(Component)]
pub struct PlayerMovement {
    pub movement_mode: PlayerMovementMode,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub pitch_yaw: Vector2<f32>,
    pub ground_plane: Option<Plane>,
    pub walking: bool
}

#[derive(PartialEq)]
pub enum PlayerMovementMode {
    Noclip,
    Clip
}

impl PlayerMovement {
    pub fn new_pos_look(movement_mode: PlayerMovementMode, position: Vector3<f32>, pitch_yaw: Vector2<f32>) -> Self {
        PlayerMovement {
            movement_mode,
            position,
            pitch_yaw,
            velocity: Vector3::zero(),
            ground_plane: None,
            walking: false
        }
    }

    // TODO: could cache these
    pub fn orientation(&self) -> Quaternion<f32> {
        let pitch = Quaternion::from_axis_angle(WORLD_RIGHT, Rad(self.pitch_yaw.x));
        let yaw = Quaternion::from_axis_angle(WORLD_UP, Rad(self.pitch_yaw.y));
        yaw * pitch
    }

    pub fn forward(&self) -> Vector3<f32> {
        self.orientation() * WORLD_FORWARD
    }

    pub fn right(&self) -> Vector3<f32> {
        self.orientation() * WORLD_RIGHT
    }
}

/// The player update system
pub fn player_update(mut level_collision: ResMut<LevelCollision>, mut world: ResMut<WorldChunkManager>,
    input_state: Res<InputState>, sim_time: Res<SimTime>, mut query: Query<(&mut PlayerCamera, &mut PlayerMovement)>,
    mut test_sphere: Query<(&TestSphere, &mut Position)>)
{
    let time_delta = sim_time.sim_time_delta as f32;

    for (mut cam, mut player_movement) in query.iter_mut() {
        // Now do pmove
        pmove(level_collision.as_mut(), world.as_mut(), &mut player_movement, &input_state, time_delta);

        // Update camera
        let cam_pos = player_movement.position + vec3(0.0, CHAR_EYE_LEVEL, 0.0);

        // Add debug sphere
        {
            //cam_pos -= player_movement.forward() * 3.0;
            //let (_, mut pos) = test_sphere.single_mut();
            //pos.pos = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
        }

        let cam_transform = Matrix4::from_translation(cam_pos) * Matrix4::from(player_movement.orientation());
        cam.view = cam_transform.invert().unwrap();


        // Debug spherecast
        if input_state.inputs[InputName::Jump as usize] {
            let (_, mut pos) = test_sphere.single_mut();

            let start = player_movement.position + vec3(0.0, CHAR_EYE_LEVEL, 0.0);
            let dir = player_movement.forward();

            let res = level_collision.sweep_sphere(world.as_mut(), &start, &dir, 15.0, 0.5);
            if let Some(res) = res {
                pos.pos = start + dir * res.toi();
            }
            else {
                pos.pos = vec3(9.0, 0.0, -9.0);
            }
        }
    }
}

fn pmove(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement,
    input_state: &InputState, time_delta: f32)
{
    // Update view direction
    pm_update_view_angles(player_movement, input_state, time_delta);

    // Noclip
    if player_movement.movement_mode == PlayerMovementMode::Noclip {
        pm_noclip(player_movement, input_state, time_delta);
        return;
    }

    // Do horizontal movement
    let (forward_input, right_input) = input_state.get_movement_input();

    let speed = match input_state.inputs[InputName::CamSpeed as usize] {
        false => CAM_MOVE_SPEED,
        true => CAM_MOVE_SPEED_FAST,
    };

    let mut forward_cmd = forward_input * speed * player_movement.forward();
    forward_cmd.y = 0.0;
    let mut right_cmd = right_input * speed * player_movement.right();
    right_cmd.y = 0.0;

    player_movement.velocity += (right_cmd + forward_cmd) * PM_ACCELERATE * time_delta;

    // friction
    let speed = player_movement.velocity.magnitude();
    if speed < 1.0 {
        player_movement.velocity.x = 0.0;
        player_movement.velocity.z = 0.0;
    }
    else if speed > 0.0 {
        let pm_stopspeed = 1.0;
        let pm_friction = 6.0;
        let control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
        let drop = control * pm_friction * time_delta;

        let mut newspeed = speed - drop;
        if newspeed < 0.0 {
            newspeed = 0.0;
        }
        newspeed /= speed;

        player_movement.velocity *= newspeed;
    }

    let lat_vel = right_cmd + forward_cmd;
    player_movement.velocity.x = lat_vel.x;
    player_movement.velocity.z = lat_vel.z;

    // Find ground plane
    let ground_start = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    if let Some(hit) = level.sweep_sphere(world, &ground_start, &vec3(0.0, -1.0, 0.0), 0.02, COLLIDER_RADIUS) {
        let point = ground_start + vec3(0.0, -1.0, 0.0) * hit.toi() - hit.normal() * COLLIDER_RADIUS;
        let ground_plane = Plane::new_from_point_and_normal(point, *hit.normal());
        player_movement.ground_plane = Some(ground_plane);

        if hit.normal().y >= MIN_WALK_NORMAL {
            player_movement.velocity.y = 0.0;
        }
        else {
            player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;
        }
    }
    else {
        player_movement.ground_plane = None;
        player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;
    }

    // Recursive slide
    let mut clip_planes = Vec::new();
    if let Some(ground_plane) = player_movement.ground_plane {
        clip_planes.push(ground_plane);
    }
    // Add original velocity as plane
    let vel_plane_nrm = player_movement.velocity.normalize();
    let vel_plane_pos = player_movement.position - vel_plane_nrm * COLLIDER_RADIUS;
    clip_planes.push(Plane::new_from_point_and_normal(vel_plane_pos, vel_plane_nrm));

    recursive_slide(level, world, &mut player_movement.position, &mut player_movement.velocity, 0, time_delta, &mut clip_planes);

    // bump out of walls
    let collider_pos = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    if let Some(contact) = level._sphere_contact_any(world, &collider_pos, COLLIDER_RADIUS) {
        if contact.depth > 0.0 {
            let n = contact.normal;
            player_movement.position += contact.depth * vec3(n.x, n.y, n.z);
        }
    }
}

fn recursive_slide(level: &mut LevelCollision, world: &mut WorldChunkManager, position: &mut Vector3<f32>,
    velocity: &mut Vector3<f32>, depth: i32, time_delta: f32, clip_planes: &mut Vec<Plane>)
{
    const MAX_DEPTH: i32 = 2;

    let mut move_start = *position;
    let mut remaining_movement = *velocity * time_delta;

    for _ in 0..3 {
        let move_dist = remaining_movement.magnitude();
        if move_dist < f32::EPSILON {
            break;
        }

        let move_dir = remaining_movement / move_dist;

        // Spherecast to find the next plane to clip against
        let sphere_start = move_start + vec3(0.0, COLLIDER_RADIUS, 0.0);
        if let Some(hit) = level.sweep_sphere(world, &sphere_start, &move_dir, move_dist, COLLIDER_RADIUS) {
            // Check the plane isn't already in there
            let mut found_plane = false;
            for plane in clip_planes.iter() {
                if hit.normal().dot(plane.normal()) > 0.99 {
                    found_plane = true;
                }
            }

            // TODO: should we do something if it is found?
            if !found_plane {
                let point = sphere_start + move_dir * hit.toi() - hit.normal() * COLLIDER_RADIUS;
                let plane = Plane::new_from_point_and_normal(point, *hit.normal());
                clip_planes.push(plane);
            }
        }

        // Clip movement to clip planes
        let sphere = Sphere::new(move_start + vec3(0.0, COLLIDER_RADIUS, 0.0), COLLIDER_RADIUS);
        let mut clip_plane = None;
        let mut clip_toi = move_dist;
        for (_, plane) in clip_planes.iter().enumerate() {
            // works ok in some places but was falling through ramp
            if let Some(toi) = intersection::toi_moving_sphere_plane(&sphere, plane, &move_dir, clip_toi) {
                if toi < clip_toi {
                    clip_plane = Some(*plane);
                    clip_toi = toi;
                    if clip_toi < 0.01 {
                        clip_toi = 0.0;
                    }
                }
            }
            //
            // Algo idea (based on quake): try and resolve the movement into other clip planes so
            // that it doesn't go into any of the other clip planes. This can be done recursively.
            // If we find any solution that doesn't end up intersecting any of the other clip
            // planes then we can move onto the next iteration and look for more clip planes to
            // add.
        }

        let step_move_dist = clip_toi - 0.01;
        let step_move_time = (step_move_dist / move_dist) * time_delta;
        // Recursively check this movement is ok
        //let rec_pos = move_start;
        //let rec_vel = *velocity;
        if depth < MAX_DEPTH {
            //let old_pos = *position;
            // TODO: make sure we account for time remaining and movement remaining properly. tbh
            // might be worth just using one or the other, or changing how this recursion works
            recursive_slide(level, world, position, velocity, depth + 1, step_move_time, clip_planes);
            //let diff = *position - old_pos;
            //let dist_moved = diff.magnitude();
            //let remaining_dist = move_dist - dist_moved;
            //if remaining_dist > 0.0 {
            //    remaining_movement = velocity.normalize() * remaining_dist;
            //}
        }

        if let Some(plane) = clip_plane {
            remaining_movement -= plane.normal() * plane.normal().dot(remaining_movement);
            if remaining_movement.magnitude2() > 0.0 {
                remaining_movement = remaining_movement.normalize() * (move_dist - step_move_dist);
            }
        }
        else {
            remaining_movement = vec3(0.0, 0.0, 0.0);
        }

        //if let Some(plane) = clip_plane {
        //    remaining_movement -= plane.normal() * plane.normal().dot(remaining_movement);
        //}
        //else {
        //    let step_move_dist = clip_toi - 0.01;
        //    move_start += move_dir * f32::max(0.0, step_move_dist);
        //    remaining_movement = vec3(0.0, 0.0, 0.0);
        //    if remaining_movement.magnitude2() > 0.0 {
        //        remaining_movement = remaining_movement.normalize() * (move_dist - step_move_dist);
        //    }
        //}

        move_start += move_dir * f32::max(0.0, step_move_dist);
    }

    *position = move_start;
}

fn pm_update_view_angles(player_movement: &mut PlayerMovement, input_state: &InputState, time_delta: f32) {
    let (horz_input, vert_input) = input_state.get_look_input();

    let look_speed = match input_state.inputs[InputName::CamSpeed as usize] {
        false => CAM_LOOK_SPEED,
        true => CAM_LOOK_SPEED_FAST,
    };

    let pitch_yaw = &mut player_movement.pitch_yaw;

    pitch_yaw.x += vert_input * look_speed * time_delta;
    pitch_yaw.y += horz_input * look_speed * time_delta;
}

// The simplest movement mode: noclip
fn pm_noclip(player_movement: &mut PlayerMovement, input_state: &InputState, time_delta: f32) {
    let (forward_input, right_input) = input_state.get_movement_input();

    let speed = match input_state.inputs[InputName::CamSpeed as usize] {
        false => CAM_MOVE_SPEED,
        true => CAM_MOVE_SPEED_FAST,
    };

    // Update velocity
    player_movement.velocity = forward_input * speed * player_movement.forward() +
        right_input * speed * player_movement.right();

    // Update position
    player_movement.position += player_movement.velocity * time_delta;
}
