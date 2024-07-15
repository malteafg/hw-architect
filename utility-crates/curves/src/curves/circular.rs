use crate::Spine;

use super::CurveSpec;

/// A circular curve approximated using cubic bezier curves
pub struct Circular;

impl CurveSpec for Circular {
    fn get_spine(&self) -> Spine {
        Spine::empty()
    }

    fn get_length(&self) -> f32 {
        0.0
    }
}
