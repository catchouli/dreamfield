use strum_macros::{EnumIter, Display};

#[derive(EnumIter, Display)]
pub enum UniformBlockBinding {
    GlobalParams = 0,
    ModelParams = 1,
    MaterialParams = 2,
    LightParams = 3
}

pub enum TextureSlot {
    BaseColor = 0
}
