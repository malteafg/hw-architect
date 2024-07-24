use std::ops::{Add, AddAssign, Deref, Mul, MulAssign, Neg, Sub};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::consts::DEFAULT_DIR;

use super::vec::VecUtils;

/// Represents a direction in the xz plane that is always guaranteed to be normalized.
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct DirXZ(Vec3);

impl PartialEq for DirXZ {
    fn eq(&self, other: &Self) -> bool {
        let this = format!("{:.3}", self.0);
        let other = format!("{:.3}", other.0);
        // dbg!(this.clone());
        // dbg!(other.clone());
        this == other
    }
}

impl From<Vec3> for DirXZ {
    fn from(mut value: Vec3) -> Self {
        value.y = 0.0;
        let vec = value.normalize_or(DEFAULT_DIR);
        Self(vec)
    }
}

impl From<DirXZ> for Vec3 {
    fn from(value: DirXZ) -> Self {
        value.0
    }
}

impl From<DirXZ> for [f32; 3] {
    fn from(value: DirXZ) -> Self {
        let vec: Vec3 = value.into();
        vec.into()
    }
}

impl Deref for DirXZ {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DirXZ {
    pub fn new() -> Self {
        Self(DEFAULT_DIR)
    }

    pub fn dot(self, other: DirXZ) -> f32 {
        (*self).dot(*other)
    }

    pub fn flip(self, flip: bool) -> Self {
        let vec = self.0.flip(flip);
        vec.into()
    }

    pub fn mirror(self, normal: Vec3) -> Self {
        let vec: Vec3 = self.into();
        let res = vec - vec.proj(normal) * 2.0;
        res.into()
    }

    pub fn left_hand(self) -> Self {
        Self(Vec3::new(self.0.z, self.0.y, -self.0.x))
    }

    pub fn right_hand(self) -> Self {
        Self(Vec3::new(-self.0.z, self.0.y, self.0.x))
    }

    pub fn length_squared(self) -> f32 {
        self.0.length_squared()
    }
}

impl Add<DirXZ> for DirXZ {
    type Output = Self;
    fn add(self, rhs: DirXZ) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Add<DirXZ> for Vec3 {
    type Output = Self;
    fn add(self, rhs: DirXZ) -> Self::Output {
        self + rhs.0
    }
}

impl Sub<DirXZ> for DirXZ {
    type Output = Self;
    fn sub(self, rhs: DirXZ) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl Sub<DirXZ> for Vec3 {
    type Output = Self;
    fn sub(self, rhs: DirXZ) -> Self::Output {
        self - rhs.0
    }
}

impl AddAssign<DirXZ> for Vec3 {
    fn add_assign(&mut self, rhs: DirXZ) {
        *self += rhs.0;
    }
}

impl Mul<DirXZ> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: DirXZ) -> Self::Output {
        self * rhs.0
    }
}

impl MulAssign<DirXZ> for Vec3 {
    fn mul_assign(&mut self, rhs: DirXZ) {
        *self *= rhs.0;
    }
}

impl Mul<f32> for DirXZ {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Self::Output {
        self.0 * rhs
    }
}

impl Neg for DirXZ {
    type Output = DirXZ;

    fn neg(self) -> DirXZ {
        self.flip(true)
    }
}
