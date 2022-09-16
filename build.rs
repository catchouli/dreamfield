use dreamfield_system::{include_world_model, build_log};
use dreamfield_system::world::world_builder::*;

/// Directory to output chunks to
pub const CHUNK_OUTPUT_DIR: &'static str = "target/world_chunks";

/// World models
const WORLD_MODELS: &'static [WorldModel] = &[
    include_world_model!("resources/models/village.glb"),
    include_world_model!("resources/models/dungeon.glb"),
    //include_world_model!("resources/models/scene_a.glb"),
    //include_world_model!("resources/models/scene_b.glb"),
    //include_world_model!("resources/models/scene_c.glb"),
    //include_world_model!("resources/models/scene_d.glb"),
    //include_world_model!("resources/models/scene_e.glb"),
    //include_world_model!("resources/models/triangle.glb"),
];

/// TODO: output files to update on when changed
fn main() {
    build_log!("Building world models");
    // A hack because otherwise it tries to delete it later and fails
    std::fs::create_dir_all(CHUNK_OUTPUT_DIR).unwrap();
    WorldBuilder::new(CHUNK_OUTPUT_DIR, WORLD_MODELS).build_world_models();
}
