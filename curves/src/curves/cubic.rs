use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::Spine;

use super::CurveUnique;

/// A cubic bezier curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cubic;

impl CurveUnique for Cubic {
    fn compute_spine(&self) -> Spine {
        unimplemented!()
    }

    fn reverse(&mut self) {}

    fn contains_pos(&self, _pos: Vec3, _width: f32) -> bool {
        true
    }
}
