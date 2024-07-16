use crate::Spine;

use super::{Curve, CurveUnique};

/// A cubic bezier curve
pub struct Cubic;

impl CurveUnique for Curve<Cubic> {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
