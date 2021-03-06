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

impl ToStd140<std140::int> for i32 {
    fn to_std140(&self) -> std140::int {
        std140::int(*self)
    }
}

impl FromStd140<i32> for std140::int {
    fn from_std140(&self) -> i32 {
        match self {
            std140::int(val) => *val
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

impl ToStd140<std140::vec2> for cgmath::Vector2<f32> {
    fn to_std140(&self) -> std140::vec2 {
        std140::vec2(self.x, self.y)
    }
}

impl FromStd140<cgmath::Vector2<f32>> for std140::vec2 {
    fn from_std140(&self) -> cgmath::Vector2<f32> {
        match self {
            std140::vec2(x, y) => cgmath::vec2(*x, *y)
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
        cgmath::Matrix3::from_cols(
            self.internal[0].element.from_std140(),
            self.internal[1].element.from_std140(),
            self.internal[2].element.from_std140())
    }
}

impl ToStd140<std140::mat4x4> for cgmath::Matrix4<f32> {
    fn to_std140(&self) -> std140::mat4x4 {
        std140::mat4x4(self.x.to_std140(), self.y.to_std140(), self.z.to_std140(), self.w.to_std140())
    }
}

impl FromStd140<cgmath::Matrix4<f32>> for std140::mat4x4 {
    fn from_std140(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_cols(
            self.internal[0].element.from_std140(),
            self.internal[1].element.from_std140(),
            self.internal[2].element.from_std140(),
            self.internal[3].element.from_std140())
    }
}

impl<T: std140::Std140Struct + Copy, const N: usize> ToStd140<std140::array<T, N>> for [T; N] {
    fn to_std140(&self) -> std140::array<T, N> {
        std140::array::from_wrapped(self.map(|ele| std140::ArrayElementWrapper { element: ele }))
    }
}
