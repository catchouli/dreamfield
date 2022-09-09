use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, Vector2, Zero, Quaternion, Rad, Rotation3, Matrix4, SquareMatrix, InnerSpace};
use dreamfield_system::world::WorldChunkManager;

use super::TestSphere;
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

    //player_movement.velocity += (right_cmd + forward_cmd) * PM_ACCELERATE * time_delta;

    //// friction
    //let speed = player_movement.velocity.magnitude();
    //if speed < 1.0 {
    //    player_movement.velocity.x = 0.0;
    //    player_movement.velocity.z = 0.0;
    //}
    //else if speed > 0.0 {
    //    let pm_stopspeed = 1.0;
    //    let pm_friction = 6.0;
    //    let control = if speed < pm_stopspeed { pm_stopspeed } else { speed };
    //    let drop = control * pm_friction * time_delta;

    //    let mut newspeed = speed - drop;
    //    if newspeed < 0.0 {
    //        newspeed = 0.0;
    //    }
    //    newspeed /= speed;

    //    player_movement.velocity *= newspeed;
    //}

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
        println!("cancelling gravity");
        }
        else {
            player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;
        println!("adding gravity acceleration");
        }
    }
    else {
        player_movement.ground_plane = None;
        player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;
        println!("adding gravity acceleration");
    }

    let movement_xz = time_delta * vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z);
    player_movement.position = recursive_slide(level, world, player_movement.position, movement_xz, 0);

    if player_movement.velocity.y != 0.0 {
        let movement_y = time_delta * vec3(0.0, player_movement.velocity.y, 0.0);
        player_movement.position = recursive_slide(level, world, player_movement.position, movement_y, 0);
    }

    // bump out of walls
    //let collider_pos = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    //if let Some(contact) = level._sphere_contact_any(world, &collider_pos, COLLIDER_RADIUS) {
    //    if contact.depth > 0.0 {
    //        let n = contact.normal;
    //        player_movement.position += contact.depth * vec3(n.x, n.y, n.z);
    //    }
    //}
}

fn recursive_slide(level: &mut LevelCollision, world: &mut WorldChunkManager, position: Vector3<f32>,
    velocity: Vector3<f32>, depth: i32) -> Vector3<f32>
{
    println!("sliding from {position:?} with velocity {velocity:?}");
    if depth > 10 {
        println!("hit 10 recursions");
        return position;
    }

    // Find the closest intersection point
    let sweep_start = position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    let length = velocity.magnitude();

    if length < f32::EPSILON {
        return position;
    }

    let dir = velocity / length;

    // TODO: make sweep_sphere have two versions, one which works in ellipsoid space, and one which
    // just does the transformation from world and back for other cases. otherwise we end up
    // transforming back and forth unnecessarily.
    if let Some(hit) = level.sweep_sphere(world, &sweep_start, &dir, length, COLLIDER_RADIUS) {
        let toi = f32::max(0.0, hit.toi() - 0.01);
        let new_pos = position + toi * dir;

        // Calculate sliding normal
        let sliding_plane = {
            // TODO: it needs to be in ellipsoid space
            let plane_origin = *hit.point();
            // TODO: if this is an ellipsoid we need to scale it by the radius vector
            let plane_normal = (new_pos - hit.normal()).normalize();
            println!("got sliding plane at {plane_origin:?} and normal {plane_normal:?}");
            Plane::new_from_point_and_normal(plane_origin, plane_normal)
        };

        let remaining_dist = length - toi;
        let movement_to_subtract = hit.normal() * velocity.dot(*hit.normal());
        println!("moved {toi}, remaining dist: {remaining_dist}, subtracting movement: {movement_to_subtract:?}");
        let mut remaining_movement = velocity - movement_to_subtract;
        if remaining_movement.magnitude2() < f32::EPSILON {
            return new_pos;
        }
        remaining_movement = remaining_movement.normalize() * remaining_dist;

        //return new_pos;

        //let destination_point = (position/COLLIDER_RADIUS) + (velocity/COLLIDER_RADIUS);
        //let new_destination_point = sliding_plane.project(velocity/COLLIDER_RADIUS);
        //let new_velocity = destination_point - sliding_plane.dist_from_point(destination_point) * sliding_plane.normal();
        //let new_destination_point = destination_point -
            //sliding_plane.dist_from_point(destination_point) *
            //sliding_plane.normal();

        //let new_vel = new_destination_point - hit.point();

        if remaining_dist < 0.01 {
            new_pos
        }
        else {
            println!("sliding {} with remaining movement: {remaining_movement:?}", depth + 1);
            recursive_slide(level, world, new_pos, remaining_movement, depth + 1)
        }
    }
    else {
        position + velocity
    }
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
