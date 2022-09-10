use bevy_ecs::component::Component;
use bevy_ecs::system::{Res, ResMut, Query};
use cgmath::{Vector3, vec3, Vector2, Zero, Quaternion, Rad, Rotation3, Matrix4, SquareMatrix, InnerSpace, vec2, ElementWise};
use dreamfield_system::world::WorldChunkManager;

use super::TestSphere;
use super::intersection::Plane;
use super::level_collision::{LevelCollision, SpherecastResult};
use dreamfield_renderer::components::{PlayerCamera, Position};
use dreamfield_system::resources::{SimTime, InputName, InputState};

/// The character height
const CHAR_HEIGHT: f32 = 1.8;

/// The character's collider radius
const COLLIDER_RADIUS: Vector3<f32> = vec3(0.5, CHAR_HEIGHT * 0.5, 0.5);

/// The character's collider change of basis matrix (to convert world coordinates to e-space, where
/// the collider is a unit sphere)
const COLLIDER_CBM: Vector3<f32> = vec3(1.0 / COLLIDER_RADIUS.x, 1.0 / COLLIDER_RADIUS.y, 1.0 / COLLIDER_RADIUS.z);

/// The minimum ground_normal y value to stop you from walking on steep slopes
const MIN_WALK_NORMAL: f32 = 0.85;

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
const ACCELERATE: f32 = 20.0;

/// The acceleration when in the air
const AIR_ACCELERATE: f32 = 20.0;

/// The ground friction as percentage of speed to lose per second
const GROUND_FRICTION: f32 = 20.0;

/// Maximum walking speed on the ground
const GROUND_MAX_SPEED: f32 = 3.0;

/// Amount the running speed increases the speed and acceleration
const RUNNING_MULTIPLIER: f32 = 2.0;

/// Jump initial acceleration (instant)
const INSTANT_JUMP_ACCELERATION: f32 = 3.0;

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

            let radius = 0.5;

            let start = player_movement.position + vec3(0.0, CHAR_EYE_LEVEL, 0.0);
            let velocity = 15.0 * player_movement.forward();

            let res = level_collision.sweep_sphere(world.as_mut(), start, velocity, radius);
            if let Some(res) = res {
                pos.pos = start + velocity * res.toi();
            }
            else {
                pos.pos = vec3(9.0, 0.0, -9.0);
            }
        }
    }
}

// TODO: make sweep work for single point, so we can use it to check if we're stuck and escape
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

    // Find ground plane, converting to ellipsoid space first
    let position_e_space = player_movement.position.mul_element_wise(COLLIDER_CBM);
    player_movement.ground_plane = sweep_unit(level, world, position_e_space, vec3(0.0, -0.02, 0.0))
        .map(|hit| {
            let hit_point_world = hit.point().div_element_wise(COLLIDER_CBM);
            let hit_normal_world = hit.normal().div_element_wise(COLLIDER_CBM).normalize();
            Plane::new_from_point_and_normal(hit_point_world, hit_normal_world)
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

    // Convert position and velocity to e-space for unit sphere sweep
    let mut position_es = player_movement.position
        .mul_element_wise(COLLIDER_CBM);
    let velocity_es = player_movement.velocity
        .mul_element_wise(COLLIDER_CBM);

    // Update lateral movement
    let movement_xz_es = time_delta * vec3(velocity_es.x, 0.0, velocity_es.z);
    position_es = recursive_slide(level, world, position_es, movement_xz_es, 0);

    // Add gravity
    if player_movement.velocity.y != 0.0 {
        let movement_y_es = time_delta * vec3(0.0, velocity_es.y, 0.0);
        position_es = recursive_slide(level, world, position_es, movement_y_es, 0);
    }

    // Convert player position back to R3 (world space)
    player_movement.position = position_es
        .div_element_wise(COLLIDER_CBM);

    // TODO: might want to reimplement the 'bump' behavior for if we get stuck, now that we've
    // removed ncollide we don't have it anymore
}

/// Sweep a unit sphere through the world from the start with a given velocity. Start and velocity
/// must be converted to e-space first by multiplying by COLLIDER_CBM, and then the results must
/// eventually be converted back to world space by doing the opposite.
fn sweep_unit(level: &mut LevelCollision, world: &mut WorldChunkManager, start: Vector3<f32>, velocity: Vector3<f32>)
    -> Option<SpherecastResult>
{
    // Add 1 radius to the starting y coordinate so the base of the sphere is at the player's feet
    let start = start + vec3(0.0, 1.0, 0.0);

    // Sweep the scene
    let result = level.sweep_unit_sphere(world, start, velocity, COLLIDER_CBM);

    // Subtract radius back from intersection point if there was a hit
    result.map(|result| {
        let toi = result.toi();
        let point = result.point() - vec3(0.0, 1.0, 0.0);
        let normal = *result.normal();
        SpherecastResult::new(toi, point, normal)
    })
}

/// Move through the world sliding on surfaces we collide with
fn recursive_slide(level: &mut LevelCollision, world: &mut WorldChunkManager, position: Vector3<f32>,
    velocity: Vector3<f32>, depth: i32) -> Vector3<f32>
{
    const MAX_RECURSION_DEPTH: i32 = 5;
    const VERY_CLOSE_DISTANCE: f32 = 0.01;

    // If we hit the maximum recursion, just return the current position and don't advance anymore
    if depth >= MAX_RECURSION_DEPTH {
        return position;
    }

    // If the velocity is 0, stop advancing too
    let velocity_length = velocity.magnitude();
    if velocity_length == 0.0 {
        return position;
    }

    // Sphere sweep and find next intersection point
    let hit = sweep_unit(level, world, position, velocity);
    if hit.is_none() {
        return position + velocity;
    }
    let hit = hit.unwrap();

    // Only update position if we aren't already very close
    let hit_distance = hit.toi() * velocity_length;
    let (new_position, hit_point) = match hit_distance > VERY_CLOSE_DISTANCE {
        true => {
            let velocity_dir = velocity / velocity_length;

            // Update position to just before the hit point so we don't move into it
            let new_position = position + velocity_dir * (hit_distance - VERY_CLOSE_DISTANCE);

            // Update the hit point too so that it doesn't throw off the plane calculation
            let hit_point = hit.point() - VERY_CLOSE_DISTANCE * velocity_dir;

            (new_position, hit_point)
        },
        false => (position, *hit.point())
    };

    // Calculate sliding normal using clever math from triangle soup paper
    let slide_plane_origin = hit_point;
    let slide_plane_normal = (new_position - hit_point).normalize();
    let slide_plane = Plane::new_from_point_and_normal(slide_plane_origin, slide_plane_normal);

    // Project original destination onto plane, and subtract it the intersection point from it to
    // get a new velocity
    let original_destination = position + velocity;
    let new_destination_point = slide_plane.project(original_destination);
    let new_velocity_vector = new_destination_point - hit_point;

    // If the new velocity is too low, just return the new position and stop moving
    if new_velocity_vector.magnitude2() < (VERY_CLOSE_DISTANCE * VERY_CLOSE_DISTANCE) {
        return new_position;
    }

    recursive_slide(level, world, new_position, new_velocity_vector, depth + 1)
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
