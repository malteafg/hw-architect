//! Contains utils for math as traits that are implemented for different math
//! types.

use glam::*;
use std::f32::consts::PI;

mod cur;
mod dir;
mod loc;
mod mat;
mod vec;

pub use cur::Cur;
pub use dir::DirXZ;
pub use loc::{Loc, PosOrLoc};
pub use mat::{Mat3Utils, Mat4Utils};
pub use vec::VecUtils;

/// Defines functions associated with angle computations.
pub trait Angle {
    fn rad_normalize(self) -> Self;
}

impl Angle for f32 {
    fn rad_normalize(self) -> Self {
        self % (2.0 * PI)
    }
}

pub trait Round {
    fn round_half_down(self) -> Self;
}

impl Round for f32 {
    fn round_half_down(self) -> Self {
        let remainder = self % 1.0;
        self - remainder + if remainder <= 0.5 { 0.0 } else { 1.0 }
    }
}

/// Represents a 3 dimensional ray.
#[derive(Copy, Clone, Default, Debug)]
pub struct Ray {
    pub pos: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        Ray { pos, dir }
    }
}
