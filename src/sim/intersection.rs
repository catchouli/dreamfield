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
/// TODO: if we are intersecting, we might want to return a small negative toi to push us
/// back out, so we don't have to do the weird 'bump' thing. update: it has that but it was causing
/// weird up and down jumps when going over hills, so something's funny. for now I clamped it to 0
pub fn toi_moving_sphere_triangle(sphere: &Sphere, triangle: &Triangle, move_dir: &Vector3<f32>, move_dist: f32)
    -> Option<(f32, Vector3<f32>)>
{
    let move_dir = move_dir.normalize();
    // One: Check the normal dot product. A positive value means we're moving away from the
    // triangle and can't possibly intersect it.
    let normal = triangle.normal();
    let normal_dot_move_dir = normal.dot(move_dir);
    // TODO: check epsilon needed
    if normal_dot_move_dir >= -0.001 {
        return None;
    }

    // Two: Construct plane of triangle and test against it. This will leave us intersecting with
    // the triangle if we approach it at any angle but straight on, but is a good start, and will
    // leave us either touching or intersecting with the triangle.
    let plane = Plane::new_from_point_and_normal(triangle.a, normal);
    let dist_from_plane = plane.dist_from_point(sphere.center);

    // If the sphere is on the far side of the plane, we can't intersect with this triangle
    if dist_from_plane < -sphere.radius {
        return None;
    }

    // If the plane is too far away to intersect with our movement, then we can't intersect
    // with the triangle either
    if dist_from_plane - sphere.radius > move_dist {
        return None;
    }

    // At this point, we know that there's either a valid intersection with the plane up ahead, or
    // we're already touching (dist = 0) or in contact with the plane. At this point, we need to
    // intersect with the triangle vertices and edges below to check that there's actually an
    // intersection with the triangle, and that we didn't just intersect with the plane at some
    // position away from it.

    // At this point, we need to clip the movement against all of the vertices and edges of the
    // triangle, to look for intersections that we've missed, and find the closest one.
    let mut nearest_toi = move_dist;

    // First, however, we handle the case where the intersection point between the plane and the
    // sphere is inside the triangle, and return that intersection straight away, both as an
    // early-out, and because if we didn't handle it here, you might walk straight through a
    // triangle that's bigger than the sphere, because it won't end up intersecting with the
    // vertices or edges.
    if normal_dot_move_dir != 0.0 {
        let toi = (dist_from_plane - sphere.radius) / -normal_dot_move_dir;
        let point_on_plane = sphere.center + move_dir * toi - normal * sphere.radius;

        // TODO: what case does it solve that this keeps executing?
        if point_in_triangle(triangle, &point_on_plane) {
            nearest_toi = toi;
        }
    }

    // Three: Intersect the sphere with the triangle's vertices. This is basically the sphere caps
    // from the test below on the edges, thinking about it, but if we just test them here then we
    // dedpulicate the work.
    for i in 0..3 {
        let v = triangle.vertex_at(i);
        let v_sphere = Sphere::new(*v, sphere.radius);

        if let Some(hit) = toi_ray_sphere(&v_sphere, &sphere.center, &move_dir) {
            if hit < nearest_toi {
                nearest_toi = hit;
            }
        }
    }

    // Four: Intersect the sphere with the triangle's edges
    for i in 0..3 {
        let edge0 = triangle.vertex_at(i);
        let edge1 = triangle.vertex_at((i + 1) % 3);
        let edge = edge1 - edge0;
        let edge_len = edge.magnitude();
        let edge_dir = edge / edge_len;

        // Iteratively figure out sphere sweep toi with triangle edge, slow and not accurate
        // TODO: implement a proper version of this, I suspect the algorithm will be like:
        //
        // * Transform the line segment of the triangle's edge, and the sphere's movement
        //   direction, so that the line segment is aligned to the y-axis.
        // * As the cylinder is axis aligned, we can then intersect a line with a circle in 2D to
        //   get the time of impact.
        // * Calculate the point of impact, and check if it's on the line segment, to determine if
        //   the sphere intersects with it.
        //
        // Note that this won't fully resolve the swept sphere/line segment intersection, as we're
        // ignoring the caps of the cylinder. To do it fully we'd need to test against spheres at
        // the ends of the line segment (e.g. a minowski sum - like a capsule). However, we're
        // already testing against the vertices of the triangle above, which I *think* is
        // equivalent.
        let iters = 5;
        for i in 1..iters {
            let dist = (i as f32) / (iters as f32);
            let point = edge0 + edge_dir * dist * edge_len;

            let point_sphere = Sphere::new(point, sphere.radius);
            if let Some(hit) = toi_ray_sphere(&point_sphere, &sphere.center, &move_dir) {
                if hit < nearest_toi {
                    nearest_toi = hit;
                }
            }
        }
    }

    // If the nearest toi we've found is less than the movement distance, we have a valid
    // intersection with this triangle.
    if nearest_toi < move_dist {
        Some((nearest_toi, normal))
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
