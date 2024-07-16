use crate::Spine;

use super::RawCurveSpec;

/// A quadratic bezier curve
pub struct Quadratic;

impl RawCurveSpec for Quadratic {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
