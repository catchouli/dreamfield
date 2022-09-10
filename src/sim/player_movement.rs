use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, Vector2, Zero, Quaternion, Rad, Rotation3, Matrix4, SquareMatrix, InnerSpace, vec2};
use dreamfield_system::world::WorldChunkManager;

use super::TestSphere;
use super::intersection::Plane;
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

/// The gravity acceleration
const GRAVITY_ACCELERATION: f32 = 9.8;

/// The character eye level. According to a cursory google, this is on average 4.5" below the top
/// of your head, which is just over 10cm
const CHAR_EYE_LEVEL: f32 = CHAR_HEIGHT - 0.10;

/// The world forward direction
const WORLD_FORWARD: Vector3<f32> = vec3(0.0, 0.0, -1.0);

/// The world up direction
const WORLD_UP: Vector3<f32> = vec3(0.0, 1.0, 0.0);

/// The world right direction
const WORLD_RIGHT: Vector3<f32> = vec3(1.0, 0.0, 0.0);

/// The acceleration when on the ground
const ACCELERATE: f32 = 15.0;

/// The acceleration when in the air
const AIR_ACCELERATE: f32 = 1.0;

/// The ground friction as percentage of speed to lose per second
const GROUND_FRICTION: f32 = 15.0;

/// Maximum walking speed on the ground
const GROUND_MAX_SPEED: f32 = 3.0;

/// Amount the running speed increases the speed and acceleration
const RUNNING_MULTIPLIER: f32 = 2.0;

/// Jump initial acceleration (instant)
const INSTANT_JUMP_ACCELERATION: f32 = 2.0;

/// Jump acceleration per second thereafter
const CONTINUED_JUMP_ACCELERATION: f32 = 3.0;

/// Number of seconds jump can be held for
const JUMP_TIME_LIMIT: f32 = 1.5;

/// The PlayerMovement component
#[derive(Component)]
pub struct PlayerMovement {
    pub movement_mode: PlayerMovementMode,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub pitch_yaw: Vector2<f32>,
    pub ground_plane: Option<Plane>,
    pub walking: bool,
    /// Seconds since player started holding the jump button
    pub jump_timer: f32,
    pub jump_held: bool
}

#[derive(PartialEq)]
pub enum PlayerMovementMode {
    Noclip,
    Normal
}

impl PlayerMovement {
    pub fn new_pos_look(movement_mode: PlayerMovementMode, position: Vector3<f32>, pitch_yaw: Vector2<f32>) -> Self {
        PlayerMovement {
            movement_mode,
            position,
            pitch_yaw,
            velocity: Vector3::zero(),
            ground_plane: None,
            walking: false,
            jump_timer: 0.0,
            jump_held: false
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
        // Now move the player
        player_move(level_collision.as_mut(), world.as_mut(), &mut player_movement, &input_state, time_delta);

        // Update camera
        let cam_pos = player_movement.position + vec3(0.0, CHAR_EYE_LEVEL, 0.0);

        let cam_transform = Matrix4::from_translation(cam_pos) * Matrix4::from(player_movement.orientation());
        cam.view = cam_transform.invert().unwrap();

        // Debug spherecast
        if input_state.inputs[InputName::Debug as usize] {
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

fn player_move(level: &mut LevelCollision, world: &mut WorldChunkManager, player_movement: &mut PlayerMovement,
    input_state: &InputState, time_delta: f32)
{
    // Update view direction
    update_view_angles(player_movement, input_state, time_delta);

    // Noclip movement
    if player_movement.movement_mode == PlayerMovementMode::Noclip {
        player_move_noclip(player_movement, input_state, time_delta);
        return;
    }

    // Find ground plane
    let ground_start = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    player_movement.ground_plane = level
        .sweep_sphere(world, &ground_start, &vec3(0.0, -1.0, 0.0), 0.05, COLLIDER_RADIUS)
        .map(|hit| {
            let point = ground_start + vec3(0.0, -1.0, 0.0) * hit.toi() - hit.normal() * COLLIDER_RADIUS;
            Plane::new_from_point_and_normal(point, *hit.normal())
        });

    // Apply gravity acceleration
    player_movement.velocity.y -= GRAVITY_ACCELERATION * time_delta;

    // Cancel out gravity if we're standing on the ground and it's not too steep
    let mut steep_slope = true;
    let mut acceleration = AIR_ACCELERATE;
    let mut max_speed = GROUND_MAX_SPEED;
    if let Some(ground_plane) = player_movement.ground_plane {
        if ground_plane.normal().y >= MIN_WALK_NORMAL {
            if player_movement.velocity.y < 0.0 {
                player_movement.velocity.y = 0.0;
            }
            steep_slope = false;
            acceleration = ACCELERATE;

            if !input_state.inputs[InputName::Jump as usize] {
                player_movement.jump_timer = 0.0;
            }
        }
    }

    // Apply jump acceleration
    println!("jump time: {}, jump held: {}", player_movement.jump_timer, player_movement.jump_held);
    if input_state.inputs[InputName::Jump as usize] {
        // Start jump if we're on the ground
        if !steep_slope && !player_movement.jump_held && player_movement.jump_timer == 0.0 {
            player_movement.velocity.y += INSTANT_JUMP_ACCELERATION;
        }
        else {
            let jump_time_remaining = f32::max(0.0, JUMP_TIME_LIMIT - player_movement.jump_timer);
            let jump_acceleration_frame = f32::min(jump_time_remaining, time_delta) * CONTINUED_JUMP_ACCELERATION;
            player_movement.velocity.y += jump_acceleration_frame;
        }
        player_movement.jump_held = true;
        player_movement.jump_timer += time_delta;
    }
    else {
        player_movement.jump_held = false;
    }

    // Increase max speed and acceleration if the hax button is pressed
    if input_state.inputs[InputName::Run as usize] {
        acceleration *= RUNNING_MULTIPLIER;
        max_speed *= RUNNING_MULTIPLIER;
    }

    // Update velocity with movement acceleration
    let input_vector = get_movement_vector(player_movement, input_state);
    player_movement.velocity += vec3(input_vector.x, 0.0, input_vector.z) * acceleration * time_delta;

    // Friction (only apply it if there's no directional input). It's modelled as a constant
    // deceleration of GROUND_FRICTION, and the max speed is capped at GROUND_MAX_SPEED.
    let speed = vec2(player_movement.velocity.x, player_movement.velocity.z).magnitude();
    if speed > 0.0 {
        // Only apply friction if there's no movement input
        let apply_friction = input_vector.x == 0.0 && input_vector.z == 0.0;
        let friction = if apply_friction { GROUND_FRICTION } else { 0.0 };
        let frame_friction = friction * time_delta;

        // Apply friction (if necessary) and clamp speed
        let new_speed = f32::clamp(speed - frame_friction, 0.0, max_speed);

        // Update velocity
        let speed_ratio = new_speed / speed;
        player_movement.velocity.x *= speed_ratio;
        player_movement.velocity.z *= speed_ratio;
    }

    // Update lateral movement
    let movement_xz = time_delta * vec3(player_movement.velocity.x, 0.0, player_movement.velocity.z);
    let collider_pos = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    let final_collider_pos = recursive_slide(level, world, collider_pos, movement_xz, 0, steep_slope);
    player_movement.position = final_collider_pos - vec3(0.0, COLLIDER_RADIUS, 0.0);

    // Add gravity
    if player_movement.velocity.y != 0.0 {
        let movement_y = time_delta * vec3(0.0, player_movement.velocity.y, 0.0);
        // TODO: make a function that takes into account this collider pos for us so we don't have
        // to always remember. even better, make it use an ellipse too.
        let collider_pos = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
        // TODO: why doesn't this slide down slopes? hmm
        let final_collider_pos = recursive_slide(level, world, collider_pos, movement_y, 0, steep_slope);
        player_movement.position = final_collider_pos - vec3(0.0, COLLIDER_RADIUS, 0.0);
    }

    // bump out of walls
    let collider_pos = player_movement.position + vec3(0.0, COLLIDER_RADIUS, 0.0);
    if let Some(contact) = level._sphere_contact_any(world, &collider_pos, COLLIDER_RADIUS) {
        if contact.depth > 0.0 {
            log::warn!("Got stuck somehow, bumping out");
            let n = contact.normal;
            player_movement.position += contact.depth * vec3(n.x, n.y, n.z);
            player_movement.velocity = vec3(0.0, 0.0, 0.0);
        }
    }
}

fn recursive_slide(level: &mut LevelCollision, world: &mut WorldChunkManager, position: Vector3<f32>,
    velocity: Vector3<f32>, depth: i32, steep_slope: bool) -> Vector3<f32>
{
    const MAX_RECURSION_DEPTH: i32 = 5;
    const VERY_CLOSE_DISTANCE: f32 = 0.01;

    if depth >= MAX_RECURSION_DEPTH {
        return position;
    }

    // Find the closest intersection point
    let sweep_start = position;
    let length = velocity.magnitude();

    if length < f32::EPSILON {
        return position;
    }

    let dir = velocity / length;

    // TODO: make sweep_sphere have two versions, one which works in ellipsoid space, and one which
    // just does the transformation from world and back for other cases. otherwise we end up
    // transforming back and forth unnecessarily.
    // TODO: actually, support ellipses, is probably more smarter
    let hit = level.sweep_sphere(world, &sweep_start, &dir, length, COLLIDER_RADIUS);
    if hit.is_none() {
        return position + velocity;
    }
    let hit = hit.unwrap();

    // The original destination point
    let destination_point = position + velocity;
    let mut new_base_point = position;

    let mut hit_point = *hit.point();

    if hit.toi() > VERY_CLOSE_DISTANCE {
        let v = if velocity.x != 0.0 || velocity.y != 0.0 || velocity.z != 0.0 {
            let nearest_distance = hit.toi();
            velocity.normalize() * (nearest_distance - VERY_CLOSE_DISTANCE)
        } else {
            vec3(0.0, 0.0, 0.0)
        };

        new_base_point = position + v;

        if v.x != 0.0 || v.y != 0.0 || v.z != 0.0 {
            let v_dir = v.normalize();
            hit_point -= VERY_CLOSE_DISTANCE * v_dir;
        }
    }

    // Convert
    let cbm = 1.0 / COLLIDER_RADIUS;
    let point_es = hit_point * cbm;
    let destination_point_es = destination_point  * cbm;
    let new_base_point_es = new_base_point  * cbm;

    let slide_plane_origin_es = point_es;
    let slide_plane_normal_es = (new_base_point_es - point_es).normalize();
    let slide_plane_es = Plane::new_from_point_and_normal(slide_plane_origin_es, slide_plane_normal_es);

    let new_destination_point_es = destination_point_es - slide_plane_es.signed_distance_to(destination_point_es) * slide_plane_normal_es;
    let new_velocity_vector_es = new_destination_point_es - point_es;

    if new_velocity_vector_es.magnitude() < VERY_CLOSE_DISTANCE {
        return new_base_point;
    }

    let mut new_velocity_vector = new_velocity_vector_es / cbm;

    if steep_slope && new_velocity_vector.y > 0.0 {
        if new_velocity_vector.x != 0.0 || new_velocity_vector.z != 0.0 {
            let vel_length = new_velocity_vector.magnitude();
            new_velocity_vector.y = 0.0;
            new_velocity_vector = new_velocity_vector.normalize() * vel_length;
        }
        else {
            new_velocity_vector = vec3(0.0, 0.0, 0.0);
        }
    }

    recursive_slide(level, world, new_base_point, new_velocity_vector, depth + 1, steep_slope)
}

/// Update the view direction
fn update_view_angles(player_movement: &mut PlayerMovement, input_state: &InputState, time_delta: f32) {
    let (horz_input, vert_input) = input_state.get_look_input();

    let look_speed = match input_state.inputs[InputName::Run as usize] {
        false => CAM_LOOK_SPEED,
        true => CAM_LOOK_SPEED_FAST,
    };

    let pitch_yaw = &mut player_movement.pitch_yaw;

    pitch_yaw.x += vert_input * look_speed * time_delta;
    pitch_yaw.y += horz_input * look_speed * time_delta;
}

/// Get the movement vector based on the player's input
fn get_movement_vector(player_movement: &PlayerMovement, input_state: &InputState) -> Vector3<f32> {
    let (forward_input, right_input) = input_state.get_movement_input();
    forward_input * player_movement.forward() + right_input * player_movement.right()
}

/// The simplest movement mode: noclip
fn player_move_noclip(player_movement: &mut PlayerMovement, input_state: &InputState, time_delta: f32) {
    let (forward_input, right_input) = input_state.get_movement_input();

    let speed = match input_state.inputs[InputName::Run as usize] {
        false => GROUND_MAX_SPEED,
        true => GROUND_MAX_SPEED * 2.0,
    };

    // Update velocity
    player_movement.velocity = forward_input * speed * player_movement.forward() +
        right_input * speed * player_movement.right();

    // Update position
    player_movement.position += player_movement.velocity * time_delta;
}
