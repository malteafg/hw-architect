use crate::Spine;

use super::RawCurveSpec;

/// Represent a completely straight line
pub struct Straight;

impl RawCurveSpec for Straight {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
