use strum_macros::{EnumIter, Display};

#[derive(EnumIter, Display)]
pub enum UniformBlockBinding {
    GlobalRenderParams = 0,
    ModelRenderParams = 1
}
