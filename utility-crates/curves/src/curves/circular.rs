use serde::{Deserialize, Serialize};

use crate::{GuidePoints, Spine};

use super::CurveUnique;

/// A circular curve approximated using cubic bezier curves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circular {
    guide_points: GuidePoints,
}

impl Circular {
    pub fn from_guide_points(guide_points: GuidePoints) -> Self {
        Self { guide_points }
    }
}

impl CurveUnique for Circular {
    fn compute_spine(&self) -> Spine {
        Spine::from_guide_points(&self.guide_points)
    }
}
