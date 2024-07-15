use crate::Spine;

use super::CurveSpec;

/// A quadratic bezier curve
pub struct Quadratic;

impl CurveSpec for Quadratic {
    fn get_spine(&self) -> Spine {
        Spine::empty()
    }

    fn get_length(&self) -> f32 {
        0.0
    }
}
