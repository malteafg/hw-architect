use crate::Spine;

use super::RawCurveSpec;

/// A circular curve approximated using cubic bezier curves
#[derive(Debug, Clone)]
pub struct Circular;

impl RawCurveSpec for Circular {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
