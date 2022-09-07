use cgmath::{Vector3, vec3, InnerSpace, vec2};

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
    let dist_from_plane = plane.dist_from_point(&sphere.center);

    // If the sphere is on the far side of the plane, we can't intersect with this triangle
    if dist_from_plane < -sphere.radius {
        return None;
    }

    // If the plane is too far away to intersect with our movement, then we can't intersect
    // with the triangle either
    //if dist_from_plane - sphere.radius > move_dist {
        //return None;
    //}

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
    //if normal_dot_move_dir != 0.0 {
    //    let toi = (dist_from_plane - sphere.radius) / -normal_dot_move_dir;
    //    let point_on_plane = sphere.center + move_dir * toi - normal * sphere.radius;

    //    // TODO: what case does it solve that this keeps executing?
    //    if point_in_triangle(triangle, &point_on_plane) {
    //        nearest_toi = toi;
    //    }
    //}
    //let mut h = dist_from_plane;
    //if h > sphere.radius {
    //    h -= sphere.radius;
    //    let dot = normal.dot(move_dir);
    //    if dot != 0.0 {
    //        let t = -h / dot;
    //        let on_plane = sphere.center + move_dir * t;
    //        if point_in_triangle(&triangle, &on_plane) {
    //            if t < nearest_toi {
    //                println!("got direct intersect");
    //                nearest_toi = t;
    //            }
    //        }
    //    }
    //}
    //if normal_dot_move_dir != 0.0 {
    //    let toi = (dist_from_plane - sphere.radius) / -normal_dot_move_dir;
    //    let point = sphere.center + move_dir * toi;
    //    let plane_point = plane.project(&point);
    //    if point_in_triangle(&triangle, &plane_point) {
    //        nearest_toi = toi;
    //    }
    //}

    // Three: Intersect the sphere with the triangle's vertices
    for i in 0..0 {
        let seg_pt0 = triangle.vertex_at(i);
        let seg_pt1 = seg_pt0 - move_dir;

        let inter1;
        let inter2;
        let res = {
            let square = |x: f32| x * x;

            let a = square(seg_pt1.x - seg_pt0.x) + square(seg_pt1.y - seg_pt0.y) + square(seg_pt1.z - seg_pt0.z);
            let b = 2.0 * ((seg_pt1.x - seg_pt0.x) * (seg_pt0.x - sphere.center.x) +
                           (seg_pt1.y - seg_pt0.y) * (seg_pt0.y - sphere.center.y) +
                           (seg_pt1.z - seg_pt0.z) * (seg_pt0.z - sphere.center.z));
            let c = square(sphere.center.x) + square(sphere.center.y) + square(sphere.center.z) + square(seg_pt0.x) +
                square(seg_pt0.y) + square(seg_pt0.z) -
                2.0 * (sphere.center.x * seg_pt0.x + sphere.center.y * seg_pt0.y + sphere.center.z * seg_pt0.z)
                - square(sphere.radius);
            let i = b * b - 4.0 * a * c;

            if i < 0.0 {
                inter1 = 0.0;
                inter2 = 0.0;
                false
            }
            else if i == 0.0 {
                inter1 = -b / (2.0 * a);
                inter2 = -b / (2.0 * a);
                true
            }
            else {
                inter1 = (-b + f32::sqrt(square(b) - 4.0 * a * c)) / (2.0 * a);
                inter2 = (-b - f32::sqrt(square(b) - 4.0 * a * c)) / (2.0 * a);
                true
            }
        };

        if !res {
            continue;
        }

        let t = f32::min(inter1, inter2);

        if t < 0.0 {
            continue;
        }

        if t < nearest_toi {
            nearest_toi = t;
        }
    }

    // Four: Intersect the sphere with the triangle's edges
    for i in 0..3 {
        let edge0 = triangle.vertex_at(i);
        let edge1 = triangle.vertex_at((i + 1) % 3);
        let edge2 = edge1 - move_dir;

        let plane = {
            let v0 = edge0 - edge1;
            let v1 = edge2 - edge1;
            let n = v1.cross(v0).normalize();
            Plane::new_from_point_and_normal(*edge0, n)
        };

        let d = plane.dist_from_point(&sphere.center);
        if d > sphere.radius || d < -sphere.radius {
            continue;
        }

        let srr = sphere.radius * sphere.radius;
        let r = f32::sqrt(srr - d * d);

        let pt0 = plane.project(&sphere.center);

        let on_line = {
            let v = pt0 - edge0;
            let s = edge1 - edge0;
            let len_sq = s.magnitude2();
            let dot = v.dot(s) / len_sq;
            let disp = s * dot;
            pt0 + disp
        };
        let v = (on_line - pt0).normalize();
        let pt1 = v * r + pt0;

        let mut a0 = 0;
        let mut a1 = 1;
        let pl_x = f32::abs(plane.a);
        let pl_y = f32::abs(plane.b);
        let pl_z = f32::abs(plane.c);
        if pl_x > pl_y && pl_x > pl_z {
            a0 = 1;
            a1 = 2;
        }
        else if pl_y > pl_z {
            a0 = 0;
            a1 = 2;
        }

        let vv = pt1 + move_dir;

        let res = {
            let _p1 = vec2(pt1[a0], pt1[a1]);
            let _p2 = vec2(vv[a0], vv[a1]);
            let _p3 = vec2(edge0[a0], edge0[a1]);
            let _p4 = vec2(edge1[a0], edge1[a1]);

            let d1 = _p2 - _p1;
            let d2 = _p3 - _p4;

            let denom = d2.y * d1.x - d2.x * d1.y;
            if denom == 0.0 {
                None
            }
            else {
                let dist = (d2.x * (_p1.y - _p3.y) - d2.y * (_p1.x - _p3.x)) / denom;
                //println!("edge {i}: {edge0:?}, {edge1:?}, {t}, nearest: {nearest_toi}");
                Some(dist)
            }
        };

        if res.filter(|t| *t >= 0.0).is_none() {
            continue;
        }

        let t = res.unwrap();
        let inter = pt1 + move_dir * t;

        let r1 = edge0 - inter;
        let r2 = edge1 - inter;

        if r1.dot(r2) > 0.0 {
            continue;
        }

        if t > nearest_toi {
            continue;
        }

        println!("edge {i}: {edge0:?}, {edge1:?}, {t}, nearest: {nearest_toi}");

        nearest_toi = t;
    }

    // If the nearest toi we've found is less than the movement distance, we have a valid
    // intersection with this triangle.
    if nearest_toi < move_dist {
        //println!("intersection: {normal:?}, distance: {nearest_toi}"); 
        Some((nearest_toi, normal))
    }
    else {
        None
    }
}

/// Test whether a point is in a triangle, by calculating the barycentric coordinates and then
/// checking that 0 <= v <= 1.0, 0 <= w <= 1.0, and v + v <= 1.0
/// https://gamedev.stackexchange.com/a/23745
fn point_in_triangle(triangle: &Triangle, point: &Vector3<f32>) -> bool {
    //return true;
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

    //let a = vec3(triangle.a.x as f64, triangle.a.y as f64, triangle.a.z as f64);
    //let b = vec3(triangle.b.x as f64, triangle.b.y as f64, triangle.b.z as f64);
    //let c = vec3(triangle.c.x as f64, triangle.c.y as f64, triangle.c.z as f64);
    //let point = vec3(point.x as f64, point.y as f64, point.z as f64);

    //let v0 = b - a;
    //let v1 = c - a;
    //let v2 = point - a;

    //let d00 = v0.dot(v0);
    //let d01 = v0.dot(v1);
    //let d11 = v1.dot(v1);
    //let d20 = v2.dot(v0);
    //let d21 = v2.dot(v1);

    //let denom = d00 * d11 - d01 * d01;

    //let v = (d11 * d20 - d01 * d21) / denom;
    //let w = (d00 * d21 - d01 * d20) / denom;

    //0.0 <= v && v <= 1.0 &&
    //0.0 <= w && w <= 1.0 &&
    //(v + w) <= 1.0

    //let a = triangle.a - point;
    //let b = triangle.b - point;
    //let c = triangle.c - point;

    //let u = b.cross(c);
    //let v = c.cross(a);
    //let w = a.cross(b);

    //if u.dot(v) < 0.0 {
    //    false
    //}
    //else if u.dot(w) < 0.0 {
    //    false
    //}
    //else {
    //    true
    //}

    //let same_side = |p1: Vector3<f32>, p2: Vector3<f32>, a: Vector3<f32>, b: Vector3<f32>| {
    //    let cp1 = (b - a).cross(p1 - a);
    //    let cp2 = (b - a).cross(p2 - a);
    //    cp1.dot(cp2) >= 0.0
    //};

    //let a = triangle.a;
    //let b = triangle.b;
    //let c = triangle.c;
    //same_side(*point, a, b, c) && same_side(*point, b, a, c) && same_side(*point, c, a, b)
}
