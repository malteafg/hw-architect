use crate::Spine;

use super::CurveSpec;

/// Represent a completely straight line
pub struct Straight;

impl CurveSpec for Straight {
    fn get_spine(&self) -> Spine {
        Spine::empty()
    }

    fn get_length(&self) -> f32 {
        0.0
    }
}
