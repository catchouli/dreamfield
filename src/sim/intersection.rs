use cgmath::Vector3;
use dreamfield_system::world::aabb::Aabb;

/// A sphere primitive
pub struct Sphere {
    pub center: Vector3<f32>,
    pub radius: f32
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32) -> Self {
        Self {
            center,
            radius
        }
    }
}

/// A triangle primitive
pub struct Triangle {
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
    pub c: Vector3<f32>
}

impl Triangle {
    pub fn new(a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>) -> Self {
        Self {
            a,
            b,
            c
        }
    }
}

/// Time of intersection between a moving sphere and an aabb
pub fn toi_moving_sphere_aabb(sphere: &Sphere, aabb: &Aabb, sphere_velocity: &Vector3<f32>) -> Option<f32> {
    None
}

/// Time of intersection between a moving sphere and a triangle
pub fn toi_moving_sphere_triangle(sphere: &Sphere, triangle: &Triangle, sphere_velocity: &Vector3<f32>) -> Option<f32> {
    None
}
