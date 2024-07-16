use crate::Spine;

use super::{Curve, CurveUnique};

/// A quadratic bezier curve
pub struct Quadratic;

impl CurveUnique for Curve<Quadratic> {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
