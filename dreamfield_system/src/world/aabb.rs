use cgmath::{Vector3, vec3};
use speedy::{Readable, Writable, Context};

/// A wrapper for Vector3<f32> that's serializable
#[derive(Clone)]
struct WrappedVector3(Vector3<f32>);

impl WrappedVector3 {
    pub fn as_vec(&self) -> &Vector3<f32> {
        match self {
            WrappedVector3(v) => &v
        }
    }
}

impl<'a, C: Context> Writable<C> for WrappedVector3 {
    fn write_to< T: ?Sized + speedy::Writer< C > >( &self, writer: &mut T ) -> Result<(), C::Error > {
        let v = self.as_vec();
        writer.write_f32(v.x)?;
        writer.write_f32(v.y)?;
        writer.write_f32(v.z)?;
        Ok(())
    }
}

impl<'a, C: Context> Readable<'a, C> for WrappedVector3 {
    fn read_from< R: speedy::Reader< 'a, C > >( reader: &mut R ) -> Result< Self, <C as Context>::Error > {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_f32()?;
        Ok(WrappedVector3(vec3(x, y, z)))
    }
}

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
}
