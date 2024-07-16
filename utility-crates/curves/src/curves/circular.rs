use crate::Spine;

use super::{Curve, CurveUnique};

/// A circular curve approximated using cubic bezier curves
pub struct Circular;

impl CurveUnique for Curve<Circular> {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
