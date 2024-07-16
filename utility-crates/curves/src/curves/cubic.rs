use crate::Spine;

use super::RawCurveSpec;

/// A cubic bezier curve
pub struct Cubic;

impl RawCurveSpec for Cubic {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
