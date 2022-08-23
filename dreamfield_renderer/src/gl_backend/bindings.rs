use strum_macros::{EnumIter, Display};

#[derive(EnumIter, Display)]
pub enum UniformBlockBinding {
    GlobalParams = 0,
    ModelParams = 1,
    MaterialParams = 2,
    LightParams = 3,
    JointParams = 4
}

pub enum TextureSlot {
    BaseColor = 0
}
