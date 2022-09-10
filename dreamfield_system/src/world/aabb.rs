use cgmath::{Vector3, vec3, ElementWise};
use speedy::{Readable, Writable};
use super::wrapped_vectors::WrappedVector3;

/// Axis aligned bounding box
#[derive(Clone, Readable, Writable)]
pub struct Aabb {
    min_max: Option<(WrappedVector3, WrappedVector3)>,
}

impl Aabb {
    pub fn new() -> Self {
        Self {
            min_max: None,
        }
    }

    /// Get min and max as a cgmath vector
    pub fn min_max(&self) -> Option<(&Vector3<f32>, &Vector3<f32>)> {
        self.min_max.as_ref().map(|(min, max)| (min.as_vec(), max.as_vec()))
    }

    pub fn set_min_max(&mut self, min: &Vector3<f32>, max: &Vector3<f32>) {
        self.min_max = Some((WrappedVector3(*min), WrappedVector3(*max)));
    }

    pub fn expand_with_point(&mut self, p: &Vector3<f32>) {
        if let Some((min, max)) = self.min_max() {
            let new_min = Self::vec_min(&min, p);
            let new_max = Self::vec_max(&max, p);
            self.set_min_max(&new_min, &new_max);
        }
        else {
            self.set_min_max(p, p);
        }
    }

    pub fn expand_with_aabb(&mut self, other: &Aabb) {
        if let Some((min, max)) = self.min_max() {
            if let Some((other_min, other_max)) = other.min_max() {
                let new_min = Self::vec_min(&min, &other_min);
                let new_max = Self::vec_max(&max, &other_max);
                self.set_min_max(&new_min, &new_max);
            }
        }
        else {
            self.min_max = other.min_max.clone();
        }
    }

    pub fn vec_min(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
        vec3(
            f32::min(a.x, b.x),
            f32::min(a.y, b.y),
            f32::min(a.z, b.z),
        )
    }

    pub fn vec_max(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
        vec3(
            f32::max(a.x, b.x),
            f32::max(a.y, b.y),
            f32::max(a.z, b.z),
        )
    }

    pub fn intersects_sphere(&self, center: &Vector3<f32>, radius: f32) -> bool {
        if let Some((WrappedVector3(min), WrappedVector3(max))) = &self.min_max {
            // https://stackoverflow.com/a/4579192
            let mut dmin = 0.0;

            for i in 0..3 {
                if center[i] < min[i] {
                    dmin += (center[i] - min[i]) * (center[i] - min[i]);
                }
                else if center[i] > max[i] {
                    dmin += (center[i] - max[i]) * (center[i] - max[i]);
                }
            }

            dmin <= radius * radius
        }
        else {
            false
        }
    }

    pub fn intersects_aabb(&self, other: &Aabb) -> bool {
        if let Some((a_min, a_max)) = &self.min_max {
            let a_min = a_min.as_vec();
            let a_max = a_max.as_vec();
            if let Some((b_min, b_max)) = &other.min_max {
                let b_min = b_min.as_vec();
                let b_max = b_max.as_vec();

                return a_min.x <= b_max.x && a_max.x >= b_min.x &&
                       a_min.y <= b_max.y && a_max.y >= b_min.y &&
                       a_min.z <= b_max.z && a_max.z >= b_min.z;
            }
        }

        false
    }

    /// Apply a change of basis matrix (scale only, for ellipsoid space) to an aabb
    pub fn apply_cbm(&self, cbm: Vector3<f32>) -> Self {
        let mut aabb = Aabb::new();

        if let Some((min, max)) = &self.min_max {
            aabb.set_min_max(&min.as_vec().mul_element_wise(cbm), &max.as_vec().mul_element_wise(cbm));
        }

        aabb
    }
}
