use strum_macros::{EnumIter, Display};

#[derive(EnumIter, Display)]
pub enum UniformBlockBinding {
    GlobalParams = 0,
    ModelParams = 1,
    MaterialParams = 2,
    LightParams = 3,
    JointParams = 4
}

pub enum AttribBinding {
    Positions = 0,
    Normals = 1,
    TexCoords = 3,
    Tangents = 4,
    Colors = 5,
    Joints = 6,
    Weights = 7
}

pub enum TextureSlot {
    BaseColor = 0
}
