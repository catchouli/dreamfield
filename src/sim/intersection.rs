use cgmath::{Vector3, vec3, InnerSpace};
use dreamfield_system::world::aabb::Aabb;

/// A plane primitive
#[derive(Copy, Clone)]
pub struct Plane {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32
}

impl Plane {
    /// Construct a new plane from the given point and normal
    pub fn new_from_point_and_normal(point: Vector3<f32>, normal: Vector3<f32>) -> Self {
        let a = normal.x;
        let b = normal.y;
        let c = normal.z;
        let d = -point.x * a - point.y * b - point.z * c;

        Self { a, b, c, d }
    }

    /// Get the distance of the plane from a point
    pub fn dist_from_point(&self, point: Vector3<f32>) -> f32 {
        self.a * point.x + self.b * point.y + self.c * point.z + self.d
    }

    /// Project a point onto the plane
    pub fn _project_point(&self, point: Vector3<f32>) -> Vector3<f32> {
        let dist = self.dist_from_point(point);
        vec3(point.x - self.a * dist, point.y - self.b * dist, point.z - self.c * dist)
    }

    pub fn normal(&self) -> Vector3<f32> {
        vec3(self.a, self.b, self.c)
    }
}

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
    pub fn normal(&self) -> Vector3<f32> {
        let ab = self.b - self.a;
        let ac = self.c - self.a;
        ab.cross(ac).normalize()
    }

    pub fn vertex_at(&self, i: usize) -> &Vector3<f32> {
        match i {
            0 => &self.a,
            1 => &self.b,
            2 => &self.c,
            _ => panic!("vertex_at: i must be 0 <= i <= 2")
        }
    }
}

impl From<ncollide3d::shape::Triangle<f32>> for Triangle {
    fn from(triangle: ncollide3d::shape::Triangle<f32>) -> Self {
        Self {
            a: vec3(triangle.a.x, triangle.a.y, triangle.a.z),
            b: vec3(triangle.b.x, triangle.b.y, triangle.b.z),
            c: vec3(triangle.c.x, triangle.c.y, triangle.c.z)
        }
    }
}

/// Time of intersection between a swept sphere and an AABB
/// TODO: implement this
pub fn toi_moving_sphere_aabb(_sphere: &Sphere, _aabb: &Aabb, _move_dir: &Vector3<f32>, move_dist: f32) -> Option<f32> {
    Some(move_dist)
}

pub fn toi_moving_sphere_plane(sphere: &Sphere, plane: &Plane, move_dir: &Vector3<f32>, move_dist: f32)
    -> Option<f32>
{
    let plane_normal = plane.normal();
    let normal_dot_move_dir = plane_normal.dot(*move_dir);
    if normal_dot_move_dir >= -0.001 {
        return None;
    }

    let dist_from_plane = plane.dist_from_point(sphere.center);

    if dist_from_plane < -sphere.radius {
        return None;
    }

    if dist_from_plane - sphere.radius > move_dist {
        return None;
    }

    let toi = (dist_from_plane - sphere.radius) / -normal_dot_move_dir;

    Some(f32::max(toi, 0.0))
}

/// Time of intersection between a swept sphere and a triangle. We handle this by clipping
/// the motion against, in order:
/// * The plane of the triangle. If the intersection point is in the triangle, we can report that
/// intersection right away. This also handles the case where the sphere intersects with a triangle
/// much larger than itself, and would pass through in between the vertices and edges. If the sphere
/// never intersects with the plane, it can't intersect with the triangle either. In any other case,
/// we need to test against the vertices and edges of the triangle.
/// * Each vertex of the triangle, for cases where the sphere intersects the plane outside the
/// triangle, but one of the sphere's sides still intersects one of the vertices of the triangle.
/// * Finally, each edge of the triangle, as there may be a case where one of the sphere's sides
/// slips through between two vertices without intersecting either of them.
///
/// We now transform all points into 'ellipsoid space', the space where the sphere is a unit
/// sphere. This allows us to support ellpsoids in theory, and also simplifies a lot of the math.
/// http://www.peroxide.dk/papers/collision/collision.pdf
///
/// TODO: if we are intersecting, we might want to return a small negative toi to push us
/// back out, so we don't have to do the weird 'bump' thing. update: it has that but it was causing
/// weird up and down jumps when going over hills, so something's funny. for now I clamped it to 0
pub fn toi_moving_sphere_triangle(sphere: &Sphere, triangle: &Triangle, move_dir: &Vector3<f32>, move_dist: f32)
    -> Option<(f32, Vector3<f32>, Vector3<f32>)>
{
    // The change of basis matrix for R3 (world coordinates) to ellipsoid space simplifies to
    // dividing by the radius (or the radius vector for an ellipsoid)
    let cbm = 1.0 / sphere.radius;

    let center = sphere.center * cbm;
    let velocity = move_dir * move_dist * cbm;

    let normal = triangle.normal();

    let v0 = triangle.a * cbm;
    let v1 = triangle.b * cbm;
    let v2 = triangle.c * cbm;

    let plane_constant = -v0.x * normal.x - v0.y * normal.y - v0.z * normal.z;

    // First check - that the sphere intersects the plane of the triangle

    // Check triangle is front facing
    // TODO: if this still isn't accurate enough, let's just make the collision data doubles. This
    // would allow us to have separate collision and render meshes anyway.
    if normal.dot(*move_dir) > 0.0 {
        return None;
    }

    // SignedDistance(p) = normal.dot(p) + plane_constant
    // t0 = 1 - SignedDistance(center) / normal.dot(velocity)
    // There's a special case when normal.dot(velocity) where the sphere is already intersecting
    // the plane, either the absolute distance is less than 1 (in which case the sphere intersects
    // the plane from t0 = 0 and t1 = 1), or the distance is greater than 1 and the collision can
    // not happen. If it's equal to 1, I guess the sphere is moving parallel and not intersecting.
    let dist_to_center = normal.dot(center) + plane_constant;
    let normal_dot_velocity = normal.dot(velocity);

    // Calculate the points t0 and t1, between which the swept sphere intersects with the triangle
    // plane
    let t0;
    let embedded_in_plane;

    // If we're not moving parallel to the plane, calculate t0 and t1
    if normal_dot_velocity != 0.0 {
        embedded_in_plane = false;

        // Calculate intersection points
        let mut intersection1 = (1.0 - dist_to_center) / normal_dot_velocity;
        let mut intersection2 = (-1.0 - dist_to_center) / normal_dot_velocity;

        // Swap intersection points so t0 is the closest
        if intersection1 > intersection2 {
            (intersection1, intersection2) = (intersection2, intersection1);
        }

        // If the range is outside (0..1) then there's no intersection
        if intersection1 > 1.0 || intersection2 < 0.0 {
            return None;
        }

        // Discard t1, it's no longer needed, and clamp t0 to 0..1
        t0 = f32::clamp(intersection1, 0.0, 1.0);
    }
    else if f32::abs(dist_to_center) < 1.0 {
        embedded_in_plane = true;
        t0 = 0.0;
    }
    else {
        return None;
    }

    // Now there are three cases:
    // * Collision inside the triangle (at t0, which is the closest possible intersection point)
    // * Collision with one of the vertices of the triangle
    // * Collision with one of the edges of the triangle
    let mut collision_point: Option<Vector3<f32>> = None;
    let mut t = 1.0;

    // Check if the intersection point is in the triangle
    if !embedded_in_plane {
        let intersection_point = center + t0 * velocity - normal;
        let triangle = Triangle { a: v0, b: v1, c: v2 };

        if point_in_triangle(&triangle, &intersection_point) {
            collision_point = Some(intersection_point);
            t = t0;
        }
    }

    // If we haven't found a collision yet we'll need to sweep the sphere against the vertices and
    // edges of the triangle. We don't need to do this if we already found a collision, because if
    // we found a collision inside the triangle above it will always be the first.
    if collision_point.is_none() {
        let velocity_magnitude2 = velocity.magnitude2();

        // Intersect each vertex
        let vertices = [v0, v1, v2];
        for v in vertices.iter() {
            let a = velocity_magnitude2;
            let b = 2.0 * velocity.dot(center - v);
            let c = (v - center).magnitude2() - 1.0;
            if let Some(new_t) = lowest_root(a, b, c, t) {
                t = new_t;
                collision_point = Some(*v);
            }
        }

        // Intersect each edge
        for i in 0..3 {
            let p1 = vertices[i];
            let p2 = vertices[(i+1) % 3];

            let edge = p2 - p1;
            let center_to_vertex = p1 - center;

            let edge_magnitude2 = edge.magnitude2();
            let edge_dot_velocity = edge.dot(velocity);
            let edge_dot_center_to_vertex = edge.dot(center_to_vertex);

            let a = edge_magnitude2 * -velocity_magnitude2
                + edge_dot_velocity * edge_dot_velocity;
            let b = edge_magnitude2 * (2.0 * velocity.dot(center_to_vertex))
                - 2.0 * edge_dot_velocity * edge_dot_center_to_vertex;
            let c = edge_magnitude2 * (1.0 - center_to_vertex.magnitude2())
                + edge_dot_center_to_vertex * edge_dot_center_to_vertex;

            if let Some(new_t) = lowest_root(a, b, c, t) {
                // Check if intersection is within line segment
                let f = (edge.dot(velocity) * new_t - edge.dot(center_to_vertex)) / edge.magnitude2();
                if f >= 0.0 && f <= 1.0 {
                    t = new_t;
                    collision_point = Some(p1 + f * edge);
                }
            }
        }
    }

    // If we found a collision point, return the distance and point
    collision_point.map(|point| {
        (t * move_dist, point / cbm, normal)
    })
}

// Solve a quadratic equation
fn lowest_root(a: f32, b: f32, c: f32, max: f32) -> Option<f32> {
    let determinant = b * b - 4.0 * a * c;

    if determinant < 0.0 {
        return None;
    }

    // possible optimization when determinant = 0 that r1 = r2
    let sqrt_d = f32::sqrt(determinant);
    let mut r1 = (-b - sqrt_d) / (2.0 * a);
    let mut r2 = (-b + sqrt_d) / (2.0 * a);

    if r1 > r2 {
        (r1, r2) = (r2, r1);
    }

    if r1 > 0.0 && r1 < max {
        Some(r1)
    }
    else if r2 > 0.0 && r2 < max {
        Some(r2)
    }
    else {
        None
    }
}

/// Find the time of impact between a line segment and a sphere
fn toi_ray_sphere(sphere: &Sphere, ray_start: &Vector3<f32>, ray_dir: &Vector3<f32>) -> Option<f32> {
    let offset = ray_start - sphere.center;

    // Ray dir is normalised so a = 0.0
    let a = 1.0;
    let b = 2.0 * ray_dir.dot(offset);
    let c = offset.dot(offset) - (sphere.radius * sphere.radius);

    if b * b - 4.0 * a * c >= 0.0 {
        Some((-b - f32::sqrt((b * b) - 4.0 * a * c)) / (2.0 * a))
    }
    else {
        None
    }
}

/// Test whether a point is in a triangle, by calculating the barycentric coordinates and then
/// checking that 0 <= v <= 1.0, 0 <= w <= 1.0, and v + v <= 1.0
/// https://gamedev.stackexchange.com/a/23745
fn point_in_triangle(triangle: &Triangle, point: &Vector3<f32>) -> bool {
    let u = triangle.b - triangle.a;
    let v = triangle.c - triangle.a;
    let w = point - triangle.a;

    let uu = u.dot(u);
    let uv = u.dot(v);
    let vv = v.dot(v);
    let wu = w.dot(u);
    let wv = w.dot(v);
    let d = uv * uv - uu * vv;

    let inv_d = 1.0 / d;
    let s = (uv * wv - vv * wu) * inv_d;
    if s < 0.0 || s > 1.0 {
        return false;
    }

    let t = (uv * wu - uu * wv) * inv_d;
    if t < 0.0 || (s + t > 1.0) {
        return false;
    }

    true
}
