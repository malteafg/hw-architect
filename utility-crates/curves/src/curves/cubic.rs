use serde::{Deserialize, Serialize};

use crate::Spine;

use super::CurveUnique;

/// A cubic bezier curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cubic;

impl CurveUnique for Cubic {
    fn compute_spine(&self) -> Spine {
        Spine::empty()
    }

    fn reverse(&mut self) {}
}
