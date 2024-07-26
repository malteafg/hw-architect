use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// Curvatures with values greater than or equal to this are considered straight.
const CUR_STRAIGHT: f64 = 10_000.0;

/// Represents curvature of a point as the radius of the tangent circle in meters.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Cur(f64);

impl Deref for Cur {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Cur {
    pub fn is_straight(&self) -> bool {
        self.0 >= CUR_STRAIGHT
    }
}
