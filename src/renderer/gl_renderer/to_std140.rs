pub trait ToStd140<T> {
    fn to_std140(&self) -> T;
}

impl ToStd140<std140::boolean> for bool {
    fn to_std140(&self) -> std140::boolean {
        std140::boolean::from(*self)
    }
}

impl ToStd140<std140::float> for f32 {
    fn to_std140(&self) -> std140::float {
        std140::float(*self)
    }
}

impl ToStd140<std140::vec3> for cgmath::Vector3<f32> {
    fn to_std140(&self) -> std140::vec3 {
        std140::vec3(self.x, self.y, self.z)
    }
}

impl ToStd140<std140::vec4> for cgmath::Vector4<f32> {
    fn to_std140(&self) -> std140::vec4 {
        std140::vec4(self.x, self.y, self.z, self.w)
    }
}

impl ToStd140<std140::mat3x3> for cgmath::Matrix3<f32> {
    fn to_std140(&self) -> std140::mat3x3 {
        std140::mat3x3(self.x.to_std140(), self.y.to_std140(), self.z.to_std140())
    }
}

impl ToStd140<std140::mat4x4> for cgmath::Matrix4<f32> {
    fn to_std140(&self) -> std140::mat4x4 {
        std140::mat4x4(self.x.to_std140(), self.y.to_std140(), self.z.to_std140(), self.w.to_std140())
    }
}
