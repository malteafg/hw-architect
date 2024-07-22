use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::Spine;

use super::CurveUnique;

/// A quadratic bezier curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quadratic;

impl CurveUnique for Quadratic {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }

    fn reverse(&mut self) {}

    fn contains_pos(&self, _pos: Vec3, _width: f32) -> bool {
        true
    }
}
