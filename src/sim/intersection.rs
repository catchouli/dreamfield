use cgmath::{Vector3, vec3, InnerSpace};
use dreamfield_system::world::aabb::Aabb;

/// A plane primitive
pub struct Plane {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32
}

impl Plane {
    /// Construct a new plane from the given point and normal
    fn new_from_point_and_normal(point: Vector3<f32>, normal: Vector3<f32>) -> Self {
        let a = normal.x;
        let b = normal.y;
        let c = normal.z;
        let d = -point.x * a - point.y * b - point.z * c;

        Self { a, b, c, d }
    }

    /// Get the distance of the plane from a point
    fn dist_from_point(&self, point: &Vector3<f32>) -> f32 {
        self.a * point.x + self.b * point.y + self.c * point.z + self.d
    }

    /// Project a point onto the sphere
    fn project(&self, point: &Vector3<f32>) -> Vector3<f32> {
        let dist = self.dist_from_point(point);
        vec3(point.x - self.a * dist, point.y - self.b * dist, point.z - self.c * dist)
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

/// Time of intersection between a moving sphere and an aabb
pub fn toi_moving_sphere_aabb(_sphere: &Sphere, _aabb: &Aabb, _sphere_velocity: &Vector3<f32>) -> Option<f32> {
    // Sweep along the ray testing at each 'radius' interval to see if it ever intersects
    Some(0.0)
}

/// Time of intersection between a moving sphere and a triangle. We handle this by clipping
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
/// back out, so we don't have to do the weird 'bump' thing. 
pub fn toi_moving_sphere_triangle(sphere: &Sphere, triangle: &Triangle, move_dir: &Vector3<f32>, move_dist: f32)
    -> Option<f32>
{
    // One: Check the normal dot product. A positive value means we're moving away from the
    // triangle and can't possibly intersect it.
    let normal = triangle.normal();
    let normal_dot_move_dir = normal.dot(*move_dir);
    // TODO: check epsilon needed
    if normal_dot_move_dir > -0.001 {
        return None;
    }

    // Two: Construct plane of triangle and test against it. This will leave us intersecting with
    // the triangle if we approach it at any angle but straight on, but is a good start, and will
    // leave us either touching or intersecting with the triangle.
    let plane = Plane::new_from_point_and_normal(triangle.a, normal);
    let dist_from_plane = plane.dist_from_point(&sphere.center);

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

    // First, however, we handle the case where the intersection point between the plane and the
    // sphere is inside the triangle, and return that intersection straight away, both as an
    // early-out, and because if we didn't handle it here, you might walk straight through a
    // triangle that's bigger than the sphere, because it won't end up intersecting with the
    // vertices or edges.
    if false && normal_dot_move_dir != 0.0 {
        let toi = (dist_from_plane - sphere.radius) / -normal_dot_move_dir;
        let point_on_plane = sphere.center + move_dir * toi - normal * sphere.radius;
        // TODO: check that this logic is right. I don't know if we need to keep going and check the
        // vertices and edges of the triangle in this case, in case there's a closer intersection.
        // I can't think of any cases where that would happen, but if anything funny happens around
        // more complex models, that may be what happened.
        if point_in_triangle(triangle, &point_on_plane) {
            return Some(toi);
        }
    }

    // At this point, we need to clip the movement against all of the vertices and edges of the
    // triangle, to look for intersections that we've missed, and find the closest one.
    let mut nearest_toi = move_dist;

    // Three: Intersect the sphere with the triangle's vertices
    for i in 0..0 {
        let tri_vertex = triangle.vertex_at(i);

        // To intersect a moving sphere with a point, we reverse the test and intersect a moving
        // point (ray) with a sphere at the point instead, which should give the same result
        let vertex_sphere = Sphere::new(*tri_vertex, sphere.radius);
        if let Some(toi) = toi_ray_sphere(&vertex_sphere, &sphere.center, move_dir) {
            if toi < nearest_toi {
                nearest_toi = toi;
            }
        }
    }

    // Four: Intersect the sphere with the triangle's edges
    for i in 0..3 {
        let a = triangle.vertex_at(i);
        let b = triangle.vertex_at((i + 1) % 3);

        // To intersect a moving sphere with a line segment, we expand the line out by the sphere's
        // radius to make a an infinite cylinder, and then we can just perform a raycast to get the
        // toi where the sphere intersects the line. We can then check that the point of intersection
        // of the sphere and the line is actually on the line segment.
        //
        // Technically, I think this misses the cases where the side of a sphere brushes past the
        // segment, but we've already tested the time of impact of the sphere and the vertices of
        // the triangle, which handles that case.
        //if let Some(toi) = toi_ray_infinite_cylinder(&sphere.center, move_dir, a, b, sphere.radius) {
            //a.cross(b)
        //}

        let c = b - move_dir;

        let v0 = a - b;
        let v1 = c - b;
        let n = v1.cross(v0).normalize();

        let plane = Plane::new_from_point_and_normal(*a, n);

        let d = plane.dist_from_point(&sphere.center);
        if d > sphere.radius || d < -sphere.radius {
            continue;
        }

        let srr = sphere.radius * sphere.radius;
        let r = f32::sqrt(srr - d * d);

        let pt0 = plane.project(&sphere.center);

        // point to line segment
        let v = pt0 - a;
        let s = pt0 - b;
        let length_square = s.magnitude2();
        let dot = v.dot(s) / length_square;
        let disp = s * dot;
        let on_line = pt0 + disp;

        let v = on_line - pt0;
        v.normalize();

        // Point on the sphere which will maybe collide with the edge
        let pt1 = v * r + pt0;

        // Figure out the point on the line
        // avoid sqrt
        let seg = b - a;
        let a_to_point = pt1 - a;
        let sign = if seg.dot(a_to_point) >= 0.0 { 1.0 } else { -1.0 };
        let dist_from_a = (pt1 - a).magnitude() * sign;
        let seg_length = (b - a).magnitude();

        if dist_from_a < seg_length {
            let point_on_ray = pt1 - normal * sphere.radius;
            let toi = (point_on_ray - sphere.center).magnitude();
            println!("got intersection at {toi}");
            if toi < nearest_toi {
                nearest_toi = toi;
                println!("found new intersection with segment {i}: {toi} (from {:?})", sphere.center);
            }
        }
    }

    // If the nearest toi we've found is less than the movement distance, we have a valid
    // intersection with this triangle.
    if nearest_toi < move_dist {
        println!("intersection: {normal:?}, distance: {nearest_toi}"); 
        Some(nearest_toi)
    }
    else {
        None
    }
}

/// Find the time of impact of a ray and an infinite cylinder along a line segment
//fn toi_ray_infinite_cylinder(ray_start: &Vector3<f32>, ray_dir: &Vector3<f32>, a: &Vector3<f32>, b: &Vector3<f32>,
//    radius: f32) -> Option<f32>
//{
//    None
//}

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
    let v0 = triangle.b - triangle.a;
    let v1 = triangle.c - triangle.a;
    let v2 = point - triangle.a;

    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);

    let denom = d00 * d11 - d01 * d01;

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;

    0.0 <= v && v <= 1.0 &&
    0.0 <= w && w <= 1.0 &&
    (v + w) <= 1.0
}
