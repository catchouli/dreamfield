use std::collections::HashMap;

use dreamfield_system::world::{world_chunk::{ChunkIndex, VERTEX_STRIDE, INDEX_STRIDE, WorldChunk}, WorldChunkManager, aabb::Aabb};
use cgmath::{Vector3, vec2, InnerSpace, vec3};

use ncollide3d::{bounding_volume::BoundingVolume, interpolation::ConstantLinearVelocityRigidMotion};
use ncollide3d::na::{RealField, Unit};
use ncollide3d::shape::{TriMesh, Ball, FeatureId, CompositeShape, Shape};
use ncollide3d::math::{Point, Isometry, Translation};
use ncollide3d::query::Contact;
use ncollide3d::query::visitors::BoundingVolumeInterferencesCollector;

use super::intersection;

/// A struct for storing spherecast hits
pub struct SpherecastResult {
    hit_toi: f32,
    hit_normal: Vector3<f32>
}

impl SpherecastResult {
    pub fn new(hit_toi: f32, hit_normal: Vector3<f32>) -> Self {
        Self {
            hit_toi,
            hit_normal
        }
    }

    pub fn toi(&self) -> f32 {
        self.hit_toi
    }

    pub fn normal(&self) -> &Vector3<f32> {
        &self.hit_normal
    }
}

/// The level collision service
pub struct LevelCollision {
    chunk_meshes: HashMap<ChunkIndex, Option<(Aabb, Vec<(Aabb, TriMesh<f32>)>)>>,
}

impl Default for LevelCollision {
    fn default() -> Self {
        Self {
            chunk_meshes: HashMap::new()
        }
    }
}

impl LevelCollision {
    /// Sweep a sphere out from start to end, returning an intersection result along with a time of impact
    pub fn sweep_sphere(&mut self, world: &mut WorldChunkManager, start: &Vector3<f32>, end: &Vector3<f32>,
        radius: f32) -> Option<SpherecastResult>
    {
        // Calculate ray direction
        let ray = end - start;
        let ray_length = ray.magnitude();
        let ray_dir = ray / ray_length;

        // Construct an aabb for the sphere's path
        let mut sphere_path_aabb = Aabb::new();
        sphere_path_aabb.expand_with_point(&(start - vec3(radius, radius, radius)));
        sphere_path_aabb.expand_with_point(&(start + vec3(radius, radius, radius)));
        sphere_path_aabb.expand_with_point(&(end - vec3(radius, radius, radius)));
        sphere_path_aabb.expand_with_point(&(end + vec3(radius, radius, radius)));

        // Walk aabb bounds and find all chunks that intersect the spherecast
        let (min, max) = sphere_path_aabb.min_max().unwrap();
        let (chunk_min_x, chunk_min_z) = WorldChunk::point_to_chunk_index(min);
        let (chunk_max_x, chunk_max_z) = WorldChunk::point_to_chunk_index(max);

        //// We clip this toi by each intersection until we end up with no more intersections
        let mut clipped_toi = ray_length;
        let mut closest_normal: Option<Vector3<f32>> = None;

        for x in chunk_min_x..=chunk_max_x {
            for z in chunk_min_z..=chunk_max_z {
                if let Some((aabb, meshes)) = self.get_chunk_meshes(world, (x, z)) {
                    let res = Self::sweep_sphere_chunk(aabb, meshes, start, &ray_dir, clipped_toi, radius);
                    if let Some((toi, normal)) = res {
                        clipped_toi = toi;
                        closest_normal = Some(normal);
                    }
                }
            }
        }

        // If we have a closest_normal that means there was at least one intersection, otherwise there was none
        closest_normal.map(|normal| SpherecastResult::new(clipped_toi, normal))
    }

    /// Sweep a sphere against a chunk
    fn sweep_sphere_chunk(chunk_aabb: &Aabb, chunk_meshes: &Vec<(Aabb, TriMesh<f32>)>, start: &Vector3<f32>,
        dir: &Vector3<f32>, length: f32, radius: f32) -> Option<(f32, Vector3<f32>)>
    {
        // Construct the moving sphere
        let sphere = intersection::Sphere::new(*start, radius);

        let mut clipped_toi = length;
        let mut sphere_velocity = dir * clipped_toi;
        let mut clipped_normal = None;

        // Check if the sphere is going to intersect the abbb at all and return None if not
        if intersection::toi_moving_sphere_aabb(&sphere, chunk_aabb, &sphere_velocity).is_none() {
            println!("sphere didn't intersect with chunk aabb");
            return None;
        }

        // Check each mesh in the chunk for intersections
        for (mesh_aabb, mesh) in chunk_meshes.iter() {
            if intersection::toi_moving_sphere_aabb(&sphere, mesh_aabb, &sphere_velocity).is_none() {
                println!("sphere didn't intersect with mesh aabb");
                continue;
            }

            println!("testing triangles of mesh");
            for i in 0..mesh.nparts() {
                let triangle = intersection::Triangle::from(mesh.triangle_at(i));
                let tri_normal = triangle.normal();

                if tri_normal.dot(*dir) > -0.001 {
                    continue;
                }

                let dispatcher = ncollide3d::query::DefaultTOIDispatcher {};
                let sphere_translation = Isometry::translation(sphere.center.x, sphere.center.y, sphere.center.z);
                let velocity = dir * clipped_toi;
                let sphere_velocity_2 = ncollide3d::na::Vector3::new(velocity.x, velocity.y, velocity.z);
                let motion = ConstantLinearVelocityRigidMotion::new(0.0, sphere_translation, sphere_velocity_2);
                let ball = Ball::new(sphere.radius);
                let no_motion = ConstantLinearVelocityRigidMotion::new(0.0, Isometry::identity(), ncollide3d::na::Vector3::new(0.0, 0.0, 0.0));
                let nc = ncollide3d::query::nonlinear_time_of_impact(&dispatcher, &motion, &ball, &no_motion, &mesh.triangle_at(i), clipped_toi, 0.0).unwrap();
                if let Some(nc) = nc {
                    if nc.toi < clipped_toi {
                        clipped_toi = nc.toi;
                        println!("got new toi: {clipped_toi}");
                        sphere_velocity = dir * clipped_toi;
                        // TODO: could probably save the triangle instead, and get the normal later, to
                        // prevent doing it multiple times
                        // Actualy, we need to calculate the normal anyway to intersection test against
                        // it, so maybe we can just return it instead once we're done with it
                        clipped_normal = Some(tri_normal);
                    }
                }
                //if let Some(toi) = intersection::toi_moving_sphere_triangle(&sphere, &triangle, dir, clipped_toi) {
                //    clipped_toi = toi;
                //    sphere_velocity = dir * clipped_toi;
                //    clipped_normal = Some(triangle.normal());
                //}
            }
        }

        // If we got a clipped_normal, that means at least one intersection was resolved
        clipped_normal.map(|normal| (clipped_toi, normal))
    }

    //// Get an upper bound for the spherecast hit distance by stepping along the ray by the diameter of the sphere
    //fn get_spherecast_upper_bound(&mut self, world: &mut WorldChunkManager, start: &Vector3<f32>, end: &Vector3<f32>,
    //    radius: f32) -> Option<(f32, Contact<f32>)>
    //{
    //    println!("finding spherecast upper bound");

    //    // We step by the diameter of the sphere to find an initial intersection as this should
    //    // mean we can get an upper bound on the distance we need to step without missing any
    //    // intersections. We want to start stepping from the radius (we're assuming the current
    //    // position isn't blocked, because we already tested that) which puts the spheres extents
    //    // from 0.0 to its diameter. We want to then step by the diameter until the center of the
    //    // sphere is at the target position.
    //    let ray = end - start;
    //    let ray_dist = ray.magnitude();
    //    let ray_dir = ray / ray_dist;

    //    let step_size = radius;
    //    let step_count = f32::ceil(ray_dist / step_size) as usize;

    //    for i in 0..=step_count {
    //        let cur_dist = f32::min(ray_dist, step_size * (i as f32));
    //        let center = start + ray_dir * cur_dist;

    //        let mut found_contact = None;

    //        self.sphere_contact_all(world, &center, radius, |c| {
    //            //let n = &c.normal;
    //            //let normal = vec3(n.x, n.y, n.z);

    //            //let dot = normal.dot(ray_dir);
    //            //if dot < 0.0 {
    //                found_contact = Some(c);
    //                false
    //            //}
    //            //else {
    //                //true
    //            //}
    //        });

    //        if let Some(contact) = found_contact {
    //            return Some((cur_dist, contact));
    //        }
    //    }

    //    None
    //}

    /// Sphere contact with the level, returning true as soon as it finds an object that intersects with the sphere.
    /// Note that this isn't guaranteed to be the closest contact, as we stop when we find a single contact point.
    pub fn sphere_contact_any(&mut self, world: &mut WorldChunkManager, center: &Vector3<f32>, radius: f32)
        -> Option<Contact<f32>>
    {
        let mut found_contact = None;

        self.sphere_contact_all(world, &center, radius, |c| {
            found_contact = Some(c);
            false
        });

        found_contact
    }

    /// Sphere contact with the level, calling a closure for every intersected triangle. Returning
    /// false from that closure will cause the intersection test to end without calling the
    /// closure again.
    pub fn sphere_contact_all<F>(&mut self, world: &mut WorldChunkManager, center: &Vector3<f32>, radius: f32, mut f: F)
    where
        F: FnMut(Contact<f32>) -> bool
    {
        let chunk_index = WorldChunk::point_to_chunk_index_2d(&vec2(center.x, center.z));
        if let Some((chunk_aabb, meshes)) = self.get_chunk_meshes(world, chunk_index).as_ref() {
            // Check if the chunk aabb intersects the sphere
            if !chunk_aabb.intersects_sphere(&center, radius) {
                return;
            }

            // Calculate query parameters
            let ball = Ball::new(radius);
            let ball_transform = Isometry::from(Translation::new(center.x, center.y, center.z));
            let level_transform = Isometry::identity();

            for (aabb, mesh) in meshes.iter() {
                if !aabb.intersects_sphere(&center, radius) {
                    continue;
                }

                Self::contact_trimesh_ball(&level_transform, mesh, &ball_transform, &ball, 0.0, |c| {
                    f(c)
                });
            }
        }
    }

    /// Get the meshes for a chunk, loading them if necessary from the world chunk manager
    fn get_chunk_meshes(&mut self, world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> &Option<(Aabb, Vec<(Aabb, TriMesh<f32>)>)>
    {
        self.chunk_meshes
            .entry(chunk_index)
            .or_insert_with(|| { Self::load_chunk_meshes(world, chunk_index) })
    }

    /// Load the meshes for a chunk from the world chunk manager
    fn load_chunk_meshes(world: &mut WorldChunkManager, chunk_index: ChunkIndex)
        -> Option<(Aabb, Vec<(Aabb, TriMesh<f32>)>)>
    {
        world.get_or_load_chunk(chunk_index)
            .as_ref()
            .map(|chunk| {
                log::info!("Loading {} chunk meshes for chunk {}, {}", chunk.meshes().len(), chunk_index.0, chunk_index.1);

                let meshes = chunk.meshes().iter().map(|mesh| {
                    let mut points = Vec::with_capacity(mesh.vertices().len() / VERTEX_STRIDE);
                    let mut indices = Vec::with_capacity(mesh.indices().len() / INDEX_STRIDE);

                    assert!(mesh.vertices().len() % VERTEX_STRIDE == 0);
                    for v in mesh.vertices().chunks_exact(VERTEX_STRIDE) {
                        points.push(Point::new(v[0], v[1], v[2]));
                    }

                    assert!(mesh.indices().len() % INDEX_STRIDE == 0);
                    for i in mesh.indices().chunks(INDEX_STRIDE) {
                        indices.push(Point::new(i[0] as usize, i[1] as usize, i[2] as usize));
                    }

                    (mesh.aabb().clone(), TriMesh::new(points, indices, None))
                }).collect();

                (chunk.aabb().clone(), meshes)
            })
    }

    /// Best contact between a composite shape (`Mesh`, `Compound`) and any other shape.
    pub fn contact_trimesh_ball<F: FnMut(Contact<f32>) -> bool> (
        m1: &Isometry<f32>,
        g1: &TriMesh<f32>,
        m2: &Isometry<f32>,
        g2: &Ball<f32>,
        prediction: f32,
        mut f: F,
    ) -> Option<Contact<f32>>
    {
        // Find new collisions
        let ls_m2 = m1.inverse() * m2.clone();
        let ls_aabb2 = g2.aabb(&ls_m2).loosened(prediction);

        let mut interferences = Vec::new();
        {
            let mut visitor = BoundingVolumeInterferencesCollector::new(&ls_aabb2, &mut interferences);
            g1.bvh().visit(&mut visitor);
        }

        //let p2 = Point::from(m2.translation.vector);
        for i in interferences.into_iter() {
            if let Some(c) = ncollide3d::query::contact(&m1, &g1.triangle_at(i), &m2, g2, prediction) {
                if !f(c) {
                    return Some(c);
                }
            }
        }

        None
    }
    
    /// Best contact between a composite shape (`Mesh`, `Compound`) and any other shape.
    ///
    /// This version is copy pasted from the ncollide3d source code, but modified to support
    /// providing a movement direction to test against when considering whether collisions are
    /// valid or not.
    pub fn _contact_trimesh_ball_dir(
        m1: &Isometry<f32>,
        g1: &TriMesh<f32>,
        m2: &Isometry<f32>,
        g2: &Ball<f32>,
        prediction: f32,
        _move_dir: Option<ncollide3d::na::Vector3<f32>>,
    ) -> Option<Contact<f32>>
    {
        // Find new collisions
        let ls_m2 = m1.inverse() * m2.clone();
        let ls_aabb2 = g2.aabb(&ls_m2).loosened(prediction);

        let mut interferences = Vec::new();

        {
            let mut visitor = BoundingVolumeInterferencesCollector::new(&ls_aabb2, &mut interferences);
            g1.bvh().visit(&mut visitor);
        }

        let mut res = None::<Contact<f32>>;

        let _p2 = Point::from(m2.translation.vector);

        for i in interferences.into_iter() {
            g1.map_part_at(i, m1, &mut |m, part| {
                if let Some(c) = ncollide3d::query::contact(&m, part, &m2, g2, prediction) {
                //if let Some(c) = Self::contact_convex_polyhedron_ball(&m, part, &p2, &g2, prediction, move_dir) {
                    let replace = res.map_or(true, |cbest| c.depth > cbest.depth);

                    if replace {
                        res = Some(c)
                    }
                }
            });
        }

        res
    }
    
    /// Contact between a ball and a convex polyhedron.
    ///
    /// This function panics if the input shape does not implement
    /// both the ConvexPolyhedron and PointQuery traits.
    ///
    /// This version is copy pasted from the ncollide3d source code, but modified to support
    /// providing a movement direction to test against when considering whether collisions are
    /// valid or not.
    pub fn _contact_ball_convex_polyhedron<N: RealField + Copy>(
        ball_center1: &Point<N>,
        ball1: &Ball<N>,
        m2: &Isometry<N>,
        shape2: &(impl Shape<N> + ?Sized),
        prediction: N,
    ) -> Option<Contact<N>> {
        // NOTE: this code is mostly taken from the narrow-phase's BallConvexPolyhedronManifoldGenerator
        // after removal of all the code related to contact kinematics because it is not needed here
        // TODE: is there a way to refactor this to avoid duplication?.
        let poly2 = shape2
            .as_convex_polyhedron()
            .expect("The input shape does not implement the ConvexPolyhedron trait.");
        let pt_query2 = shape2
            .as_point_query()
            .expect("The input shape does not implement the PointQuery trait.");

        let (proj, f2) = pt_query2.project_point_with_feature(m2, &ball_center1);
        let world2 = proj.point;
        let dpt = world2 - ball_center1;

        let depth;
        let normal;
        if let Some((dir, dist)) = Unit::try_new_and_get(dpt, N::default_epsilon()) {
            if proj.is_inside {
                depth = dist + ball1.radius;
                normal = -dir;
            } else {
                depth = -dist + ball1.radius;
                normal = dir;
            }
        } else {
            if f2 == FeatureId::Unknown {
                // We cant do anything more at this point.
                return None;
            }

            depth = ball1.radius;
            normal = -poly2.feature_normal(f2);
        }

        if depth >= -prediction {
            let world1 = ball_center1 + normal.into_inner() * ball1.radius;
            return Some(Contact::new(world1, world2, normal, depth));
        }

        None
    }

    /// Contact between a convex polyhedron and a ball.
    ///
    /// This function panics if the input shape does not implement
    /// both the ConvexPolyhedron and PointQuery traits.
    ///
    /// This version is copy pasted from the ncollide3d source code, but modified to support
    /// providing a movement direction to test against when considering whether collisions are
    /// valid or not.
    pub fn _contact_convex_polyhedron_ball<N: RealField + Copy>(
        m1: &Isometry<N>,
        poly1: &(impl Shape<N> + ?Sized),
        ball_center2: &Point<N>,
        ball2: &Ball<N>,
        prediction: N,
        move_dir: Option<ncollide3d::na::Vector3<N>>,
    ) -> Option<Contact<N>> {
        let mut res = Self::_contact_ball_convex_polyhedron(ball_center2, ball2, m1, poly1, prediction);

        if let Some(c) = &mut res {
            c.flip();

            let valid_normal = move_dir
                .map(|move_dir| c.normal.dot(&move_dir) < -N::default_epsilon())
                .unwrap_or(true);

            if !valid_normal {
                return None;
            }
        }

        res
    }
}
