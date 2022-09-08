use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, InnerSpace, Vector2, Zero, Quaternion, Rad, Rotation3, Matrix4, SquareMatrix};
use dreamfield_system::world::WorldChunkManager;

use super::TestSphere;
use super::level_collision::{LevelCollision, SpherecastResult};
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

const PM_AIR_ACCELERATE: f32 = 100.0;
const PM_ACCELERATE: f32 = 10.0;
const STEPSIZE: f32 = 0.1;

/// The PlayerMovement component
#[derive(Component)]
pub struct PlayerMovement {
    pub movement_mode: PlayerMovementMode,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub pitch_yaw: Vector2<f32>,
    pub ground_normal: Option<Vector3<f32>>,
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
            ground_normal: None,
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

    pub fn up(&self) -> Vector3<f32> {
        self.orientation() * WORLD_UP
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
        // Get camera movement input
        let move_input = input_state.get_movement_input();

        // Now do pmove
        pmove(level_collision.as_mut(), world.as_mut(), &mut player_movement, &input_state, time_delta);

        // Update camera
        let cam_pos = player_movement.position + vec3(0.0, CHAR_EYE_LEVEL, 0.0);
        let cam_transform = Matrix4::from_translation(cam_pos) * Matrix4::from(player_movement.orientation());
        cam.view = cam_transform.invert().unwrap();

        // Debug spherecast
        if input_state.inputs[InputName::Rewind as usize] {
            let (_, mut pos) = test_sphere.single_mut();

            let start = player_movement.position;
            let dir = player_movement.forward();

            let res = level_collision.sweep_sphere(world.as_mut(), &start, &dir, 10.0, 0.5);
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
    // https://github.com/id-Software/Quake-III-Arena/blob/master/code/game/bg_pmove.c#L1903
    // Save old origin in case we get stuck
    let previous_origin = player_movement.position;

    // Save old movement for crashlanding
    let previous_velocity = player_movement.velocity;

    // Update view direction
    pm_update_view_angles(player_movement, input_state, time_delta);

    // Noclip
    if player_movement.movement_mode == PlayerMovementMode::Noclip {
        pm_noclip(player_movement, input_state, time_delta);
        return;
    }

    // Update ground state
    pm_ground_trace(level, world, player_movement);

    // Walking or air movement
    if player_movement.walking {
        pm_walk_move(level, world, player_movement, input_state, time_delta);
    }
    else {
        pm_air_move(level, world, player_movement, input_state, time_delta);
    }

    // Set ground normal
    pm_ground_trace(level, world, player_movement);
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

// Trace through the scene, does a spherecast instead of whatever quake does..
fn pm_trace(level: &mut LevelCollision, world: &mut WorldChunkManager, from: Vector3<f32>, to: Vector3<f32>)
    -> Option<SpherecastResult>
{
    let start = from + vec3(0.0, COLLIDER_RADIUS, 0.0);
    let ray = to - from;
    let length = ray.magnitude();
    let dir = ray / length;
    let radius = COLLIDER_RADIUS;

    level.sweep_sphere(world, &start, &dir, length, radius)
}

fn pm_ground_trace(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement) {
    let from = player_movement.position;
    let to = from + vec3(0.0, -0.25, 0.0);

    let mut trace = pm_trace(level, world, from, to);

    // Do something corrective if the trace starts in a solid
    if let Some(res) = &trace {
        if res.toi() == 0.0 {
            trace = pm_correct_all_solid(level, world, player_movement);
        }
    }

    // If the trace didn't hit anything we are in free fall
    if trace.is_none() {
        pm_ground_trace_missed(level, world, player_movement);
        player_movement.ground_normal = None;
        player_movement.walking = false;
        return;
    }

    let trace = trace.unwrap();

    // Check if getting thrown off the ground
    if player_movement.velocity.y > 0.0 {
        player_movement.ground_normal = None;
        player_movement.walking = false;
        return;
    }

    // Slopes that are too steep will not be considered on ground
    if trace.normal().y < MIN_WALK_NORMAL {
        player_movement.ground_normal = Some(*trace.normal());
        player_movement.walking = false;
        return;
    }

    // Otherwise
    player_movement.ground_normal = Some(*trace.normal());
    player_movement.walking = true;

    // TODO: some entity stuff
}

fn pm_ground_trace_missed(_level: &mut LevelCollision, _world: &mut WorldChunkManager, _player_movement: &mut PlayerMovement) {
    //if player_movement.ground_normal.is_some() {
        // TODO: just does some animation stuff, not really relevant
    //}
}

fn pm_correct_all_solid(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement)
    -> Option<SpherecastResult>
{
    let mut trace = None;
    for i in -1..=1 {
        for k in -1..=1 {
            for j in -1..=1 {
                let point = player_movement.position + vec3(i as f32, j as f32, k as f32);
                trace = pm_trace(level, world, player_movement.position, point);
                if trace.is_none() {
                    return None;
                }
            }
        }
    }

    trace
}

fn pm_walk_move(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement,
    input_state: &InputState, time_delta: f32)
{
    let (forward_input, right_input) = input_state.get_movement_input();

    let speed = match input_state.inputs[InputName::CamSpeed as usize] {
        false => CAM_MOVE_SPEED,
        true => CAM_MOVE_SPEED_FAST,
    };

    // water move, and then something about jumping which we don't support yet

    pm_friction(level, world, player_movement, time_delta);

    let mut forward_lateral = player_movement.forward();
    forward_lateral.y = 0.0;
    forward_lateral = forward_lateral.normalize();

    let mut right_lateral = player_movement.right();
    right_lateral.y = 0.0;
    right_lateral = right_lateral.normalize();

    let ground_normal = player_movement.ground_normal.unwrap();

    const OVERCLIP: f32 = 1.001;
    forward_lateral = pm_clip_velocity(forward_lateral, ground_normal, OVERCLIP).normalize();
    right_lateral = pm_clip_velocity(right_lateral, ground_normal, OVERCLIP).normalize();

    let wishvel = forward_lateral * forward_input * speed + right_lateral * right_input * speed;
    let wishspeed = wishvel.magnitude();
    let wishdir = if wishspeed == 0.0 { vec3(0.0, 0.0, 0.0) } else { wishvel / wishspeed };

    // the speed is clamped lower if ducking

    // and if wading or walking on the bottom

    // TODO: it changes this to PM_AIR_ACCELERATE if the player gets knocked back because they lose control
    let accelerate = PM_AIR_ACCELERATE;

    pm_accelerate(player_movement, wishdir, wishspeed, accelerate, time_delta);

    // here if it's slick or knockback they do something with gravity??

    let vel = player_movement.velocity.magnitude();

    // Slide along the ground plane
    player_movement.velocity = pm_clip_velocity(player_movement.velocity, ground_normal, OVERCLIP);

    // don't decrease velocity when going up or down a slope
    let vel_scale = player_movement.velocity.magnitude();
    let vel_nrm = if vel_scale == 0.0 { vec3(0.0, 0.0, 0.0) } else { player_movement.velocity / vel_scale };
    player_movement.velocity = vel_nrm * vel;

    // Don't do anything if standing still
    if player_movement.velocity.x == 0.0 && player_movement.velocity.z == 0.0 {
        return;
    }

    pm_step_slide_move(level, world, player_movement, time_delta, false);
}

fn pm_air_move(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement,
    input_state: &InputState, time_delta: f32)
{
    let (forward_input, right_input) = input_state.get_movement_input();

    let speed = match input_state.inputs[InputName::CamSpeed as usize] {
        false => CAM_MOVE_SPEED,
        true => CAM_MOVE_SPEED_FAST,
    };

    pm_friction(level, world, player_movement, time_delta);

    let mut forward_lateral = player_movement.forward();
    forward_lateral.y = 0.0;
    forward_lateral = forward_lateral.normalize();

    let mut right_lateral = player_movement.right();
    right_lateral.y = 0.0;
    right_lateral = right_lateral.normalize();

    let mut wishvel = forward_lateral * forward_input * speed +
        right_lateral * right_input * speed;
    wishvel.y = 0.0;

    let wishspeed = wishvel.magnitude(); 
    let wishdir = if wishspeed == 0.0 { vec3(0.0, 0.0, 0.0) } else { wishvel / wishspeed };

    pm_accelerate(player_movement, wishdir, wishspeed, PM_AIR_ACCELERATE, time_delta);

    if let Some(ground_normal) = player_movement.ground_normal {
        const OVERCLIP: f32 = 1.001;
        player_movement.velocity = pm_clip_velocity(player_movement.velocity, ground_normal, OVERCLIP);
    }

    pm_step_slide_move(level, world, player_movement, time_delta, true);
}

fn pm_clip_velocity(velocity: Vector3<f32>, ground_normal: Vector3<f32>, overclip: f32) -> Vector3<f32> {
    let mut backoff = velocity.dot(ground_normal);

    if backoff < 0.0 {
        backoff *= overclip;
    }
    else {
        backoff /= overclip;
    }

    let change = ground_normal * backoff;

    velocity - change
}

fn pm_step_slide_move(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement,
    time_delta: f32, gravity: bool)
{
    //player_movement.position += player_movement.velocity * time_delta;
    //println!("updated position: {:?}, vel: {:?}", player_movement.position, player_movement.velocity);

    let start_o = player_movement.position;
    let start_v = player_movement.velocity;

    if !pm_slide_move(level, world, player_movement, time_delta, gravity) {
        // we got to exactly where we wanted to go on the first try
        return;
    }

    let mut down = start_o;
    down.y -= STEPSIZE;
    let trace = pm_trace(level, world, start_o, down);
    let mut up = vec3(0.0, 1.0, 0.0);

    if player_movement.velocity.y > 0.0 && (trace.is_none() || trace.unwrap().normal().dot(up) < 0.7) {
        return;
    }

    let down_o = player_movement.position;
    let down_v = player_movement.velocity;

    up = start_o;
    up.y += STEPSIZE;

    let dir = (up - start_o).normalize();
    let trace = pm_trace(level, world, start_o, up);
    if let Some(trace) = trace.as_ref() {
        if trace.toi() == 0.0 {
            return; // can't step up
        }
    }

    let end_pos = trace.as_ref().map(|res| start_o + dir * res.toi()).unwrap_or(up);

    let step_size = end_pos.y - start_o.y;

    // try slidemove from this position
    player_movement.position = end_pos;
    player_movement.velocity = start_v;

    pm_slide_move(level, world, player_movement, time_delta, gravity);

    // push down the final amount
    let mut down = player_movement.position;
    down.y -= step_size;
    let trace = pm_trace(level, world, player_movement.position, down);
    let all_solid = trace.as_ref().map(|trace| trace.toi() != 0.0).unwrap_or(false);
    if !all_solid {
        let dir = (down - player_movement.position).normalize();
        let end_pos = trace.as_ref().map(|res| start_o + dir * res.toi()).unwrap_or(up);
        player_movement.position = end_pos;
    }
    let dist = (down - player_movement.position).magnitude();
    if let Some(trace) = trace {
        if trace.toi() < dist {
            const OVERCLIP: f32 = 1.001;
            pm_clip_velocity(player_movement.velocity, *trace.normal(), OVERCLIP);
        }
    }
}

// returns true if the velocity was clipped in some way
fn pm_slide_move(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement,
    time_delta: f32, gravity: bool) -> bool
{
    const MAX_CLIP_PLANES: usize = 5;
    let mut planes: Vec<Vector3<f32>> = Vec::new();

    let numbumps = 4;

    let mut primal_velocity = player_movement.velocity;

    let mut end_velocity = Vector3::zero();
    if gravity {
        end_velocity = player_movement.velocity;
        end_velocity.y -= GRAVITY_ACCELERATION * time_delta;
        player_movement.velocity.y = (player_movement.velocity.y + end_velocity.y) * 0.5;
        primal_velocity.y = end_velocity.y;

        if let Some(ground_normal) = player_movement.ground_normal {
            const OVERCLIP: f32 = 1.001;
            player_movement.velocity = pm_clip_velocity(player_movement.velocity, ground_normal, OVERCLIP);
        }
    }

    let mut time_left = time_delta;

    // never turn against the ground plane
    if let Some(ground_normal) = player_movement.ground_normal {
        planes.push(ground_normal);
    }

    // never turn against original velocity
    planes.push(player_movement.velocity.normalize());

    let mut bumpcount = 0;
    for bumpcount2 in 0..numbumps {
        bumpcount = bumpcount2;
        let end = player_movement.position + player_movement.velocity * time_left;
        let length = (end - player_movement.position).magnitude();

        let trace = pm_trace(level, world, player_movement.position, end);

        if trace.is_none() {
            player_movement.position = end;
            break;
        }

        let trace = trace.unwrap();

        // entity is completely trapped in another solid
        if trace.toi() == 0.0 {
            // Don't build up falling damage
            player_movement.velocity.y = 0.0;
            return true;
        }

        if trace.toi() > 0.0 {
            // actually covered some distance
            player_movement.position = player_movement.position + player_movement.velocity * trace.toi();
        }

        time_left -= time_left * (trace.toi() / length);

        if planes.len() >= MAX_CLIP_PLANES {
            // apparently tihs shouldn't really happen
            player_movement.velocity = vec3(0.0, 0.0, 0.0);
            return true;
        }

        // "if this is the same plane we hit before, nudge velocity out along it, which fixes some
        // espilon issues with non-axial planes"
        let mut i = 0;
        while i < planes.len() {
            if trace.normal().dot(planes[i]) > 0.99 {
                player_movement.velocity += *trace.normal();
                break;
            }

            i += 1;
        }

        if i < planes.len() {
            continue;
        }

        planes.push(*trace.normal());

        // modify velocity so it "parallels all of the clip planes"
        for i in 0..planes.len() {
            let into = player_movement.velocity.dot(planes[i]);
            if into >= 0.1 {
                continue; // move doesn't interact with teh plane
            }

            // slide along the plane
            const OVERCLIP: f32 = 1.001;
            let mut clip_velocity = pm_clip_velocity(player_movement.velocity, planes[i], OVERCLIP);
            let mut end_clip_velocity = pm_clip_velocity(end_velocity, planes[i], OVERCLIP);

            // snee if there's a second plane that the new move enters
            for j in 0..planes.len() {
                if j == i {
                    continue;
                }

                if clip_velocity.dot(planes[j]) >= 0.1 {
                    continue; // move doesn' tintersect with the plane
                }

                clip_velocity = pm_clip_velocity(player_movement.velocity, planes[i], OVERCLIP);
                end_clip_velocity = pm_clip_velocity(end_velocity, planes[i], OVERCLIP);

                // see if it goes back into the first clip plane
                if clip_velocity.dot(planes[i]) >= 0.0 {
                    continue;
                }

                // slide the original velocity along the crease
                let dir = planes[i].cross(planes[j]).normalize();
                let d = dir.dot(player_movement.velocity);
                clip_velocity = dir * d;

                let d = dir.dot(end_velocity);
                end_clip_velocity = dir * d;

                for k in 0..planes.len() {
                    if k == i || k == j {
                        continue;
                    }

                    if clip_velocity.dot(planes[k]) >= 0.1 {
                        continue; // move doesn't interact with the plane
                    }

                    // stop dead at a triple plane interaction
                    player_movement.velocity = vec3(0.0, 0.0, 0.0);
                    return true;
                }
            }

            player_movement.velocity = clip_velocity;
            end_velocity = end_clip_velocity;
            break;
        }

        if gravity {
            
        }
    }

    if gravity {
        player_movement.velocity = end_velocity;
    }

    bumpcount != 0
}

fn pm_friction(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement, time_delta: f32) {
    let mut vel = player_movement.velocity;

    if player_movement.walking {
        // Ignore slope movement
        vel.y = 0.0;
    }

    let speed = vel.magnitude();
    // TODO: 1.0 is probably too high
    if speed < 1.0 {
        vel.x = 0.0;
        vel.z = 0.0;
        return;
        // there was a fixme here that says "still have z friction underwater?"
    }

    let mut drop = 0.0;

    // here it had support for being partially in water or on a slick surface
    // TODO: this porbably too high too
    let pm_stopspeed = 100.0;
    let pm_friction = 6.0;
    if player_movement.walking {
        let control = f32::max(pm_stopspeed, speed);
        drop += control * pm_friction * time_delta;
    }

    // Scale the velocity
    let newspeed = f32::max(0.0, speed - drop);

    vel *= newspeed / speed;

    player_movement.velocity = vel;
}

fn pm_accelerate(player_movement: &mut PlayerMovement, wishdir: Vector3<f32>, wishspeed: f32, accel: f32, time_delta: f32) {
    let cur_speed = player_movement.velocity.dot(wishdir);
    let add_speed = wishspeed - cur_speed;

    if add_speed <= 0.0 {
        return;
    }

    let accel_speed = f32::min(add_speed, accel * time_delta * wishspeed);

    player_movement.velocity += accel_speed * wishdir;

    // after this it mentioned a proper way that avoided a strafe jump maxspeed bug, but feels bad.
    // this probably means air strafe bhopping like in counter strike, which we definitely want tbh
}
