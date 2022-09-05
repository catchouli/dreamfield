use std::collections::HashMap;

use dreamfield_system::world::{world_chunk::{ChunkIndex, VERTEX_STRIDE, INDEX_STRIDE, WorldChunk}, WorldChunkManager, aabb::Aabb};
use cgmath::{Vector3, vec2, InnerSpace, vec3};

use ncollide3d::bounding_volume::BoundingVolume;
use ncollide3d::na::{RealField, Unit};
use ncollide3d::shape::{TriMesh, Ball, FeatureId, CompositeShape, Shape};
use ncollide3d::math::{Point, Isometry, Translation};
use ncollide3d::query::{Ray, RayCast, RayIntersection, Contact};
use ncollide3d::query::visitors::BoundingVolumeInterferencesCollector;

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
    /// Raycast into the level
    pub fn _raycast(&mut self, world: &mut WorldChunkManager, origin: &Vector3<f32>, direction: &Vector3<f32>,
        max_dist: f32) -> Option<f32>
    {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        // Raycast the current chunk's trimeshes
        // TODO: we probably need to check every chunk along the ray
        let chunk_index = WorldChunk::point_to_chunk_index(origin);
        if let Some((_, meshes)) = self.get_chunk_meshes(world, chunk_index) {
            let mut nearest: Option<f32> = None;

            for (_, mesh) in meshes.iter() {
                let mesh_hit = mesh.toi_with_ray(&Isometry::identity(), &ray, max_dist, true);
                if let Some(hit) = mesh_hit {
                    if nearest.is_none() || hit < nearest.unwrap() {
                        nearest = mesh_hit;
                    }
                }
            }

            nearest
        }
        else {
            None
        }
    }

    /// Raycast into the level, and obtain a normal
    pub fn _raycast_normal(&mut self, world: &mut WorldChunkManager, origin: &Vector3<f32>, direction: &Vector3<f32>,
        max_dist: f32) -> Option<RayIntersection<f32>>
    {
        let ray = Ray::new(
            Point::new(origin.x, origin.y, origin.z),
            ncollide3d::math::Vector::new(direction.x, direction.y, direction.z));

        // Raycast the current chunk's trimeshes
        // TODO: we probably need to check every chunk along the ray
        let chunk_index = WorldChunk::point_to_chunk_index(origin);
        if let Some((_, meshes)) = self.get_chunk_meshes(world, chunk_index) {
            let mut nearest: Option<RayIntersection<f32>> = None;

            for (_, mesh) in meshes.iter() {
                let hit = mesh.toi_and_normal_with_ray(&Isometry::identity(), &ray, max_dist, true);
                if let Some(new_hit) = hit {
                    if nearest.is_none() || new_hit.toi < nearest.unwrap().toi {
                        nearest = hit;
                    }
                }
            }

            nearest
        }
        else {
            None
        }
    }

    pub fn sweep_sphere(&mut self, world: &mut WorldChunkManager, start: &Vector3<f32>, end: &Vector3<f32>,
        radius: f32) -> Option<SpherecastResult>
    {
        let far_contact = self.get_spherecast_upper_bound(world, &start, &end, radius)?;
        println!("sweep_sphere far contact: {}", far_contact.0);
        //let far_contact = self.sphere_contact_any(world, &end, radius)?;

        let ray_dir = (end - start).normalize();

        // Binary search to find better candidate
        // TODO: this works good, but we probably have to:
        // * Sweep the radiuses still to make sure we don't jump through walls
        // * Check against every returned face to find the actual closest one, maybe, or just
        // resolve the collision against all of them...
        // Then it'll probably be really solid. Otherwise, back to the qauke source code
        const STEPS: i32 = 5;

        let mut nearest_contact = far_contact;

        let mut max = 1.0;
        let mut min = 0.0;

        for _ in 0..5 {
            let toi = 0.5 * (min + max);
            let mid = start + ray_dir * toi;

            if let Some((hit_toi, mid_contact)) = self.sphere_contact_nearest(world, &mid, radius, &start, &ray_dir) {
                max = hit_toi;
                nearest_contact = (hit_toi, mid_contact);
            }
            else {
                min = toi;
            }
        }

        let (toi, contact) = nearest_contact;
        let normal = vec3(contact.normal.x, contact.normal.y, contact.normal.z);
        Some(SpherecastResult::new(toi, normal))
    }

    /// Spherecast into the level, from the current point up to the maximum number of steps
    pub fn spherecast(&mut self, world: &mut WorldChunkManager, start: &Vector3<f32>, end: &Vector3<f32>,
        radius: f32) -> Option<SpherecastResult>
    {
        // Do an initial sweep to determine the upper bound of the collision distance. If there
        // isn't one, it means the whole sweep was unobstructed, and we return None.
        let upper_bound = self.get_spherecast_upper_bound(world, &start, &end, radius)?;

        //println!("spherecast upper bound: {upper_bound}");

        // Now do a binary search to figure out the first toi that doesn't intersect
        const STEPS: i32 = 10;

        let ray = end - start;
        let ray_length = ray.magnitude();
        let ray_dir = ray / ray_length;

        let mut nearest_intersection: Option<(f32, Contact<f32>)> = None;

        // TODO: make this actually a binary search
        for i in (0..=STEPS).rev() {
            let toi = (i as f32) / (STEPS as f32);
            let center = start + ray * toi;
            //println!("spherecast contact {toi}, {center:?}");

            let contact = self.sphere_contact_nearest(world, &center, radius, &start, &ray_dir);
            if contact.is_some() {
                nearest_intersection = contact;
            }
            else if let Some((toi, contact)) = nearest_intersection {
                let normal = vec3(contact.normal.x, contact.normal.y, contact.normal.z);
                return Some(SpherecastResult::new(toi, normal));
            }
        }

        //panic!("shouldnt get here");

        // If we get here, then we didn't find any points that it was safe to move to, so as a
        // backup we return the last hit along with the toi 0.0. We should have at least one
        // nearest_intersection by now or something went wrong.
        if let Some((_, contact)) = nearest_intersection {
            let normal = vec3(contact.normal.x, contact.normal.y, contact.normal.z);
            Some(SpherecastResult::new(0.0, normal))
        }
        else {
            log::warn!("Got to end of spherecast without finding a single hit");
            Some(SpherecastResult::new(0.0, vec3(0.0, 0.0, 0.0)))
        }
    }

    // Get an upper bound for the spherecast hit distance by stepping along the ray by the diameter of the sphere
    fn get_spherecast_upper_bound(&mut self, world: &mut WorldChunkManager, start: &Vector3<f32>, end: &Vector3<f32>,
        radius: f32) -> Option<(f32, Contact<f32>)>
    {
        // We step by the diameter of the sphere to find an initial intersection as this should
        // mean we can get an upper bound on the distance we need to step without missing any
        // intersections. We want to start stepping from the radius (we're assuming the current
        // position isn't blocked, because we already tested that) which puts the spheres extents
        // from 0.0 to its diameter. We want to then step by the diameter until the center of the
        // sphere is at the target position.
        let ray = end - start;
        let ray_dist = ray.magnitude();
        let ray_dir = ray / ray_dist;

        let step_size = radius;
        let step_count = f32::ceil(ray_dist / step_size) as usize;

        for i in 0..=step_count {
            let cur_dist = f32::min(ray_dist, step_size * (i as f32));
            let center = start + ray_dir * cur_dist;

            if let Some(contact) = self.sphere_contact_nearest(world, &center, radius, &start, &ray_dir) {
                //return Some((cur_dist / ray_dist, contact));
                return Some(contact);
            }
        }

        None
    }

    /// Sphere contact with the level, returning true as soon as it finds an object that intersects with the sphere.
    /// Note that this isn't guaranteed to be the closest contact, as we stop when we find a single contact point.
    pub fn sphere_contact_any(&mut self, world: &mut WorldChunkManager, center: &Vector3<f32>, radius: f32)
        -> Option<Contact<f32>>
    {
        let chunk_index = WorldChunk::point_to_chunk_index_2d(&vec2(center.x, center.z));
        if let Some((chunk_aabb, meshes)) = self.get_chunk_meshes(world, chunk_index).as_ref() {
            // Check if the chunk aabb intersects the sphere
            if !chunk_aabb.intersects_sphere(&center, radius) {
                return None;
            }

            // Calculate query parameters
            let ball = Ball::new(radius);
            let ball_transform = Isometry::from(Translation::new(center.x, center.y, center.z));
            let level_transform = Isometry::identity();

            for (aabb, mesh) in meshes.iter() {
                if !aabb.intersects_sphere(&center, radius) {
                    continue;
                }

                let mut contact: Option<Contact<f32>> = None;
                Self::contact_trimesh_ball(&level_transform, mesh, &ball_transform, &ball, 0.0, |c| {
                    contact = Some(c);
                    return false;
                });

                if contact.is_some() {
                    return contact;
                }
            }
        }

        None
    }

    /// Sphere contact with the level, returning the closest intersection found to the ray origin
    fn sphere_contact_nearest(&mut self, world: &mut WorldChunkManager, center: &Vector3<f32>, radius: f32, ray_start: &Vector3<f32>,
        ray_dir: &Vector3<f32>) -> Option<(f32, Contact<f32>)>
    {
        let chunk_index = WorldChunk::point_to_chunk_index_2d(&vec2(center.x, center.z));
        if let Some((chunk_aabb, meshes)) = self.get_chunk_meshes(world, chunk_index).as_ref() {
            // Check if the chunk aabb intersects the sphere
            if !chunk_aabb.intersects_sphere(&center, radius) {
                return None;
            }

            // Calculate query parameters
            let ball = Ball::new(radius);
            let ball_transform = Isometry::from(Translation::new(center.x, center.y, center.z));
            let level_transform = Isometry::identity();
            let initial_point_on_ray = (center - ray_start).magnitude();

            let mut nearest_contact: Option<(f32, Contact<f32>)> = None;

            for (aabb, mesh) in meshes.iter() {
                if !aabb.intersects_sphere(&center, radius) {
                    continue;
                }

                // TODO: one annoying thing about this is that it might return the contact for a
                // face with a normal that isn't pointing opposite the direction of our motion so
                // we can't intersect with it, like two walls at 90 degrees, it might end up
                // returning the wrong one and letting us walk through the other. Ideally we'd be
                // able to reject intersections with the wrong normal when doing the intersection
                // with each triangle instead of at this point.
                const ITERATIONS: i32 = 5;
                for _ in 0..1 {
                    let cur_ball_transform = ball_transform;
                    Self::contact_trimesh_ball(&level_transform, mesh, &cur_ball_transform, &ball, 0.0, |contact| {
                        let normal = vec3(contact.normal.x, contact.normal.y, contact.normal.z);

                        // Skip faces the ray can't intersect with
                        if normal.dot(*ray_dir) >= 0.0 {
                            return true;
                        }

                        let point = vec3(contact.world1.x, contact.world1.y, contact.world1.z);
                        let point_on_ray = point + normal * radius;
                        // TODO: dunno if this is accurate tbh
                        // TODO: let's just try this using *_any and make the spherecast find the
                        // nearest non-intersecting point instead...
                        let dist = (point_on_ray - ray_start).magnitude();

                        if let Some((nearest_dist, _)) = nearest_contact {
                            if dist < nearest_dist {
                                println!("found closer dist: {dist}");
                                nearest_contact = Some((dist, contact));
                            }
                        }
                        else {
                            println!("found initial contact, dist: {dist}");
                            nearest_contact = Some((dist, contact));
                        }

                        true

                        //if dist < initial_point_on_ray {
                        //    println!("new closer point on ray: {dist} < {initial_point_on_ray}");
                        //    //ball_transform = Isometry::from(Translation::new(point_on_ray.x, point_on_ray.y, point_on_ray.z));
                        //    return false;
                        //}
                        //else {
                        //    return true;
                        //}

                        //if let Some((old_dist, _)) = nearest_contact {
                        //    if dist < old_dist {
                        //        //println!("found closer intersection {dist} < {old_dist}");
                        //        nearest_contact = Some((dist, contact));
                        //    }
                        //}
                        //else {
                        //    //println!("found initial intersection {dist}");
                        //    nearest_contact = Some((dist, contact));
                        //}

                        //true
                    });
                }
            }

            nearest_contact
        }
        else {
            None
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

        let p2 = Point::from(m2.translation.vector);
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
    pub fn contact_trimesh_ball_dir(
        m1: &Isometry<f32>,
        g1: &TriMesh<f32>,
        m2: &Isometry<f32>,
        g2: &Ball<f32>,
        prediction: f32,
        move_dir: Option<ncollide3d::na::Vector3<f32>>,
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

        let p2 = Point::from(m2.translation.vector);

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
    pub fn contact_ball_convex_polyhedron<N: RealField + Copy>(
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
    pub fn contact_convex_polyhedron_ball<N: RealField + Copy>(
        m1: &Isometry<N>,
        poly1: &(impl Shape<N> + ?Sized),
        ball_center2: &Point<N>,
        ball2: &Ball<N>,
        prediction: N,
        move_dir: Option<ncollide3d::na::Vector3<N>>,
    ) -> Option<Contact<N>> {
        let mut res = Self::contact_ball_convex_polyhedron(ball_center2, ball2, m1, poly1, prediction);

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
