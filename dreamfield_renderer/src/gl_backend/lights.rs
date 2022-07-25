pub const LIGHT_COUNT: usize = 20;

#[derive(Debug)]
pub enum LightType {
    PointLight = 0,
    DirectionalLight = 1,
    SpotLight = 2
}
