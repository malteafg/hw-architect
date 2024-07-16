use crate::Spine;

use super::RawCurveSpec;

/// A quadratic bezier curve
#[derive(Debug, Clone)]
pub struct Quadratic;

impl RawCurveSpec for Quadratic {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
