use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::id::SegmentId;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct VehicleLoc {
    pos: Vec3,
    yrot: f32,
}

impl VehicleLoc {
    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn yrot(&self) -> f32 {
        self.yrot
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vehicle {
    curr_segment: SegmentId,
}

impl Vehicle {
    pub fn get_segment(&self) -> SegmentId {
        self.curr_segment
    }
}
