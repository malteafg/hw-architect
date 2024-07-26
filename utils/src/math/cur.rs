use std::ops::Deref;

use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::DirXZ;

/// Curvatures with values greater than or equal to this are considered straight.
const CUR_STRAIGHT: f32 = 10_000.0;

/// Represents curvature of a point as the radius of the tangent circle in meters.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Cur(f32);

impl Deref for Cur {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Cur {
    pub fn is_straight(&self) -> bool {
        self.0 >= CUR_STRAIGHT
    }

    pub fn from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let dir1: DirXZ = (p2 - p1).into();
        let dir2: DirXZ = (p3 - p2).into();
        if dir1 == dir2 {
            return Cur(CUR_STRAIGHT);
        }

        let x12 = p1.x - p2.x;
        let x13 = p1.x - p3.x;

        let z12 = p1.z - p2.z;
        let z13 = p1.z - p3.z;

        let z31 = p3.z - p1.z;
        let z21 = p2.z - p1.z;

        let x31 = p3.x - p1.x;
        let x21 = p2.x - p1.x;

        let sx13 = p1.x.powi(2) - p3.x.powi(2);
        let sz13 = p1.z.powi(2) - p3.z.powi(2);
        let sx21 = p2.x.powi(2) - p1.x.powi(2);
        let sz21 = p2.y.powi(2) - p1.y.powi(2);

        let f = ((sx13) * (x12) + (sz13) * (x12) + (sx21) * (x13) + (sz21) * (x13))
            / (2. * ((z31) * (x12) - (z21) * (x13)));
        let g = ((sx13) * (z12) + (sz13) * (z12) + (sx21) * (z13) + (sz21) * (z13))
            / (2. * ((x31) * (z12) - (x21) * (z13)));

        let c = -p1.x.powi(2) - p1.z.powi(2) - 2. * g * p1.x - 2. * f * p1.z;

        let c_x = -g;
        let c_z = -f;
        let r_squared = c_x.powi(2) + c_z.powi(2) - c;

        Cur(r_squared.sqrt())
    }
}
