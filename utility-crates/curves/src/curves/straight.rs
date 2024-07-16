use crate::Spine;

use super::{Curve, CurveUnique};

/// Represent a completely straight line
pub struct Straight;

impl CurveUnique for Curve<Straight> {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }
}
