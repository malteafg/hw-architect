use glam::Vec3;

pub const LANE_MARKINGS_WIDTH: f32 = 0.2;
pub const LANE_MARKINGS_LENGTH: f32 = 5.0;
pub const ROAD_HEIGHT: f32 = 0.2;
pub const ROAD_MIN_LENGTH: f32 = 10.0;
pub const DEFAULT_DIR: Vec3 = Vec3::new(1.0, 0.0, 0.0);

// Figure out what these two do
pub const VERTEX_DENSITY: f32 = 0.05;
pub const CUT_LENGTH: f32 = 5.0;

pub const MAX_NO_LANES: u8 = 6;

/// For now we only have one model, but change this in the future and not use const. Maybe compute
/// hash of models.
pub const TREE_MODEL_ID: u128 = 0;
