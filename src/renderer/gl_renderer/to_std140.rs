pub trait ToStd140<T> {
    fn to_std140(&self) -> T;
}

pub trait FromStd140<T> {
    fn from_std140(&self) -> T;
}

impl ToStd140<std140::boolean> for bool {
    fn to_std140(&self) -> std140::boolean {
        std140::boolean::from(*self)
    }
}

impl FromStd140<bool> for std140::boolean {
    fn from_std140(&self) -> bool {
        match self {
            std140::boolean::True => true,
            std140::boolean::False => false
        }
    }
}

impl ToStd140<std140::float> for f32 {
    fn to_std140(&self) -> std140::float {
        std140::float(*self)
    }
}

impl FromStd140<f32> for std140::float {
    fn from_std140(&self) -> f32 {
        match self {
            std140::float(val) => *val
        }
    }
}

impl ToStd140<std140::vec3> for cgmath::Vector3<f32> {
    fn to_std140(&self) -> std140::vec3 {
        std140::vec3(self.x, self.y, self.z)
    }
}

impl FromStd140<cgmath::Vector3<f32>> for std140::vec3 {
    fn from_std140(&self) -> cgmath::Vector3<f32> {
        match self {
            std140::vec3(x, y, z) => cgmath::vec3(*x, *y, *z)
        }
    }
}

impl ToStd140<std140::vec4> for cgmath::Vector4<f32> {
    fn to_std140(&self) -> std140::vec4 {
        std140::vec4(self.x, self.y, self.z, self.w)
    }
}

impl FromStd140<cgmath::Vector4<f32>> for std140::vec4 {
    fn from_std140(&self) -> cgmath::Vector4<f32> {
        match self {
            std140::vec4(x, y, z, w) => cgmath::vec4(*x, *y, *z, *w)
        }
    }
}

impl ToStd140<std140::mat3x3> for cgmath::Matrix3<f32> {
    fn to_std140(&self) -> std140::mat3x3 {
        std140::mat3x3(self.x.to_std140(), self.y.to_std140(), self.z.to_std140())
    }
}

impl FromStd140<cgmath::Matrix3<f32>> for std140::mat3x3 {
    fn from_std140(&self) -> cgmath::Matrix3<f32> {
        unsafe {
            // Bit ugly and unsafe, we can't get the actual values because they're private
            // annoyingly so we have to transmute it to a regular array type.
            let arr: [std140::ArrayElementWrapper<std140::vec3>; 3] = std::mem::transmute(*self);
            cgmath::Matrix3::from_cols(
                arr[0].element.from_std140(),
                arr[1].element.from_std140(),
                arr[2].element.from_std140())
        }
    }
}

impl ToStd140<std140::mat4x4> for cgmath::Matrix4<f32> {
    fn to_std140(&self) -> std140::mat4x4 {
        std140::mat4x4(self.x.to_std140(), self.y.to_std140(), self.z.to_std140(), self.w.to_std140())
    }
}

impl FromStd140<cgmath::Matrix4<f32>> for std140::mat4x4 {
    fn from_std140(&self) -> cgmath::Matrix4<f32> {
        unsafe {
            // Bit ugly and unsafe, we can't get the actual values because they're private
            // annoyingly so we have to transmute it to a regular array type.
            let arr: [std140::ArrayElementWrapper<std140::vec4>; 4] = std::mem::transmute(*self);
            cgmath::Matrix4::from_cols(
                arr[0].element.from_std140(),
                arr[1].element.from_std140(),
                arr[2].element.from_std140(),
                arr[3].element.from_std140())
        }
    }
}

