//! Contains utils for math as traits that are implemented for different math
//! types.

use glam::*;
use serde::{Deserialize, Serialize};
use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub},
};

use crate::consts::DEFAULT_DIR;

/// Defines utility functions intended for vector types
pub trait VecUtils {
    /// Projects self on to target
    fn proj(self, target: Self) -> Self;

    /// Anti projects self on to target
    fn anti_proj(self, target: Self) -> Self;

    /// Normalizes self and gives it the specified length
    fn rescale(self, length: f32) -> Self;

    /// Mirrors self on the given normal
    fn mirror(self, normal: Vec3) -> Self;

    fn ndot(self, other: Self) -> f32;

    // perhaps move these to Vec3Utils
    fn intersects_in_xz(self, other: Self) -> bool;
    fn intersection_in_xz(self, self_dir: Self, other: Self, other_dir: Self) -> Self;
    fn side(self, other: Self) -> f32;
    fn right_hand(self) -> Self;
    fn left_hand(self) -> Self;
    fn flip(self, flip: bool) -> Self;
}

impl VecUtils for Vec3 {
    fn proj(self, target: Self) -> Self {
        target * (self.dot(target) / target.length_squared())
    }

    fn anti_proj(self, target: Self) -> Self {
        self - self.proj(target)
    }

    fn rescale(self, length: f32) -> Self {
        self.normalize() * length
    }

    fn mirror(self, normal: Self) -> Self {
        self - self.proj(normal) * 2.0
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

    /// Should be removed and only be in dir2
    fn left_hand(self) -> Self {
        Self::new(self.z, self.y, -self.x)
    }

    /// Should be removed and only be in dir2
    fn right_hand(self) -> Self {
        Self::new(-self.z, self.y, self.x)
    }

    fn flip(self, flip: bool) -> Self {
        if flip {
            self * -1.
        } else {
            self
        }
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

/// Defines utility functions intended for 3x3 matrices
pub trait Mat3Utils {
    fn to_3x3(self) -> [[f32; 3]; 3];
}

impl Mat3Utils for Mat3 {
    fn to_3x3(self) -> [[f32; 3]; 3] {
        [self.x_axis.into(), self.y_axis.into(), self.z_axis.into()]
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

/// Represents a direction in the xz plane that is always guaranteed to be normalized.
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct DirXZ(Vec3);

impl PartialEq for DirXZ {
    fn eq(&self, other: &Self) -> bool {
        let this = format!("{:.4}", self.0);
        let other = format!("{:.4}", other.0);
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

impl DirXZ {
    pub fn new() -> Self {
        Self(DEFAULT_DIR)
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

/// Represents a position in xyz and a direction in xz. Maybe rename to Loc2 to reflect dir only
/// being in xz
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Loc {
    pub pos: Vec3,
    pub dir: DirXZ,
}

impl Loc {
    pub fn new(pos: Vec3, dir: DirXZ) -> Self {
        Self { pos, dir }
    }

    pub fn flip(self, flip: bool) -> Self {
        Loc::new(self.pos, self.dir.flip(flip))
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PosOrLoc {
    Pos(Vec3),
    Loc(Loc),
}

impl PosOrLoc {
    pub fn flip(self, flip: bool) -> Self {
        match self {
            PosOrLoc::Pos(_) => self,
            PosOrLoc::Loc(loc) => PosOrLoc::Loc(loc.flip(flip)),
        }
    }

    pub fn pos(self) -> Vec3 {
        match self {
            PosOrLoc::Pos(pos) => pos,
            PosOrLoc::Loc(loc) => loc.pos,
        }
    }

    pub fn is_pos(self) -> bool {
        match self {
            PosOrLoc::Pos(_) => true,
            PosOrLoc::Loc(_) => false,
        }
    }

    pub fn is_loc(self) -> bool {
        match self {
            PosOrLoc::Pos(_) => false,
            PosOrLoc::Loc(_) => true,
        }
    }

    pub fn to_pos(self) -> Self {
        match self {
            PosOrLoc::Pos(_) => self,
            PosOrLoc::Loc(loc) => PosOrLoc::Pos(loc.pos),
        }
    }
}

impl From<Vec3> for PosOrLoc {
    fn from(value: Vec3) -> Self {
        PosOrLoc::Pos(value)
    }
}

impl From<Loc> for PosOrLoc {
    fn from(value: Loc) -> Self {
        PosOrLoc::Loc(value)
    }
}
