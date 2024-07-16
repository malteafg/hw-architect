use glam::Vec3;

use crate::{GuidePoints, Spine};

use super::RawCurveSpec;

/// Represent a completely straight line. Should not use guide_points
#[derive(Debug, Clone)]
pub struct Straight {
    guide_points: GuidePoints,
}

impl Straight {
    pub fn new(start: Vec3, end: Vec3) -> Self {
        let guide_points = GuidePoints::from_two_points(start, end);
        Self { guide_points }
    }
}

impl RawCurveSpec for Straight {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
