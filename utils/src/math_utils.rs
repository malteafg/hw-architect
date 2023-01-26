//! Contains utils for math as traits that are implemented for different math 
//! types.

use glam::*;
use std::f32::consts::PI;

/// Defines utility functions intended for vector types
pub trait VecUtils {
    fn proj(self, target: Self) -> Self;
    fn anti_proj(self, target: Self) -> Self;
    fn mirror(self, mirror_normal: Vec3) -> Self;
    fn ndot(self, other: Self) -> f32;
    // perhaps move these to Vec3Utils
    fn intersects_in_xz(self, other: Self) -> bool;
    fn intersection_in_xz(self, self_dir: Self, other: Self, other_dir: Self) -> Self;
    fn side(self, other: Self) -> f32;
    fn right_hand(self) -> Self;
    fn left_hand(self) -> Self;
}

impl VecUtils for Vec3 {
    fn proj(self, target: Self) -> Self {
        target * (self.dot(target) / target.length_squared())
    }

    fn anti_proj(self, target: Self) -> Self {
        self - self.proj(target)
    }

    fn mirror(self, mirror_normal: Self) -> Self {
        self - self.proj(mirror_normal) * 2.0
    }

    fn ndot(self, other: Self) -> f32 {
        self.normalize().dot(other.normalize())
    }

    fn intersects_in_xz(self, other: Self) -> bool {
        // TODO use .xz()? and dot?
        other.x * self.z - other.z * self.x != 0.0
    }

    fn side(self, other: Self) -> f32 {
        (self.z * other.x - self.x * other.z).signum()
    }

    fn intersection_in_xz(self, self_dir: Self, other: Self, other_dir: Self) -> Self {
        other
            + (other_dir * ((other.z - self.z) * self_dir.x - (other.x - self.x) * self_dir.z)
                / (other_dir.x * self_dir.z - other_dir.z * self_dir.x))
    }

    fn left_hand(self) -> Self {
        Self::new(self.z, self.y, -self.x)
    }

    fn right_hand(self) -> Self {
        Self::new(-self.z, self.y, self.x)
    }
}

/// Defines utility functions intended for 4x4 matrices
pub trait Mat4Utils {
    fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Self;
    fn to_4x4(self) -> [[f32; 4]; 4];
}

impl Mat4Utils for Mat4 {
    fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Self {
        let f = dir.normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        let x = Vec4::new(s.x, u.x, -f.x, 0.0);
        let y = Vec4::new(s.y, u.y, -f.y, 0.0);
        let z = Vec4::new(s.z, u.z, -f.z, 0.0);
        let w = Vec4::new(-eye.dot(s), -eye.dot(u), eye.dot(f), 1.0);

        Self::from_cols(x, y, z, w)
    }

    fn to_4x4(self) -> [[f32; 4]; 4] {
        [
            self.x_axis.into(),
            self.y_axis.into(),
            self.z_axis.into(),
            self.w_axis.into(),
        ]
    }
}

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
        self - remainder + if remainder <= 0.5 {0.0} else {1.0}
    }
}
