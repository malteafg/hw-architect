use crate::Spine;

use super::CurveSpec;

/// A cubic bezier curve
pub struct Cubic;

impl CurveSpec for Cubic {
    fn get_spine(&self) -> Spine {
        Spine::empty()
    }

    fn get_length(&self) -> f32 {
        0.0
    }
}
