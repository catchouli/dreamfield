use cgmath::{Vector3, vec3, Vector4, vec4, Matrix4};
use speedy::{Readable, Writable, Context};

/// A wrapper for Vector3<f32> that's serializable
#[derive(Clone, Debug)]
pub struct WrappedVector3(pub Vector3<f32>);

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

impl From<Vector3<f32>> for WrappedVector3 {
    fn from(v: Vector3<f32>) -> Self {
        WrappedVector3(v)
    }
}

/// A wrapper for Vector4<f32> that's serializable
#[derive(Clone, Debug)]
pub struct WrappedVector4(pub Vector4<f32>);

impl WrappedVector4 {
    pub fn as_vec(&self) -> &Vector4<f32> {
        match self {
            WrappedVector4(v) => &v
        }
    }
}

impl<'a, C: Context> Writable<C> for WrappedVector4 {
    fn write_to< T: ?Sized + speedy::Writer< C > >( &self, writer: &mut T ) -> Result<(), C::Error > {
        let v = self.as_vec();
        writer.write_f32(v.x)?;
        writer.write_f32(v.y)?;
        writer.write_f32(v.z)?;
        writer.write_f32(v.w)?;
        Ok(())
    }
}

impl<'a, C: Context> Readable<'a, C> for WrappedVector4 {
    fn read_from< R: speedy::Reader< 'a, C > >( reader: &mut R ) -> Result< Self, <C as Context>::Error > {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_f32()?;
        let w = reader.read_f32()?;
        Ok(WrappedVector4(vec4(x, y, z, w)))
    }
}


impl From<Vector4<f32>> for WrappedVector4 {
    fn from(v: Vector4<f32>) -> Self {
        WrappedVector4(v)
    }
}

/// A wrapped for Matrix4<f32> that's serializable
#[derive(Clone, Debug)]
pub struct WrappedMatrix4(pub Matrix4<f32>);

impl WrappedMatrix4 {
    pub fn as_mat(&self) -> &Matrix4<f32> {
        match self {
            WrappedMatrix4(v) => &v
        }
    }
}

impl<'a, C: Context> Writable<C> for WrappedMatrix4 {
    fn write_to< T: ?Sized + speedy::Writer< C > >( &self, writer: &mut T ) -> Result<(), C::Error > {
        let m = self.as_mat();
        Writable::write_to(&WrappedVector4(m.x), writer)?;
        Writable::write_to(&WrappedVector4(m.y), writer)?;
        Writable::write_to(&WrappedVector4(m.z), writer)?;
        Writable::write_to(&WrappedVector4(m.w), writer)?;
        Ok(())
    }
}

impl<'a, C: Context> Readable<'a, C> for WrappedMatrix4 {
    fn read_from< R: speedy::Reader< 'a, C > >( reader: &mut R ) -> Result< Self, <C as Context>::Error > {
        let x = *WrappedVector4::read_from(reader)?.as_vec();
        let y = *WrappedVector4::read_from(reader)?.as_vec();
        let z = *WrappedVector4::read_from(reader)?.as_vec();
        let w = *WrappedVector4::read_from(reader)?.as_vec();
        Ok(WrappedMatrix4(Matrix4::from_cols(x, y, z, w)))
    }
}

impl From<Matrix4<f32>> for WrappedMatrix4 {
    fn from(m: Matrix4<f32>) -> Self {
        WrappedMatrix4(m)
    }
}
