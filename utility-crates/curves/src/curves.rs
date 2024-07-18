mod circular;
mod cubic;
mod quadratic;
mod straight;

pub use circular::Circular;
pub use cubic::Cubic;
pub use quadratic::Quadratic;
pub use straight::Straight;
use utils::Loc;

use crate::Spine;

use thiserror::Error;

use enum_dispatch::enum_dispatch;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[enum_dispatch]
pub trait CurveSpec: CurveShared + CurveBuilder {}

#[enum_dispatch]
pub trait CurveShared {
    /// Returns the spine of this curve segment
    fn get_spine(&self) -> &Spine;

    /// Returns the first element of the spine. The direction of this element must coincide with
    /// the node the segment is built from.
    fn first(&self) -> Loc;

    /// Returns the last element of the spine. The direction of this element must coincide with
    /// the node the segment is built to.
    fn last(&self) -> Loc;

    /// Returns the length in meters of this curve segment
    fn get_length(&self) -> f32;

    /// Checks if the given position is contained within the curve given a width
    fn contains_pos(&self, pos: Vec3) -> bool;
}

#[enum_dispatch]
pub trait CurveUnique {
    fn compute_spine(&self) -> Spine;
}

#[derive(Error, Debug)]
pub enum CurveError {
    #[error("The curve has points for which the curvature is too extreme")]
    TooTight,
    #[error("The curve is impossible to construct with the given constraints")]
    Impossible,
}

pub type CurveResult<C> = std::result::Result<C, CurveError>;

pub trait CurveBuilder
where
    Self: Sized + CurveShared,
{
    /// Generates the curve between two position with no direction constraints.
    fn from_free(first_pos: Vec3, last_pos: Vec3) -> CurveResult<Self>;

    /// Generates the curve between two positions where the first position is locked by a
    /// direction.
    fn from_start_locked(first: Loc, last_pos: Vec3) -> CurveResult<Self>;

    /// Generates the curve between two positions where the last position is locked by a
    /// direction.
    fn from_end_locked(first_pos: Vec3, last: Loc) -> CurveResult<Self>;

    /// Generates the curve between two positions where both positions are locked by a direction.
    fn from_both_locked(first: Loc, last: Loc) -> CurveResult<Self>;
}

#[enum_dispatch(CurveShared)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurveSum {
    Straight(Curve<Straight>),
    Circular(Curve<Circular>),
    Quadratic(Curve<Quadratic>),
    Cubic(Curve<Cubic>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curve<C> {
    instance: C,
    length: f32,
    spine: Spine,
}

impl<C: CurveUnique> From<C> for Curve<C> {
    fn from(value: C) -> Self {
        let spine = value.compute_spine();

        Self {
            instance: value,
            length: 0.0,
            spine,
        }
    }
}

impl<C: CurveUnique> From<C> for CurveSum
where
    Curve<C>: Into<CurveSum>,
{
    fn from(value: C) -> Self {
        let curve: Curve<C> = value.into();
        let res: CurveSum = curve.into();
        res
    }
}

impl<C> CurveShared for Curve<C> {
    fn get_spine(&self) -> &Spine {
        &self.spine
    }

    fn first(&self) -> Loc {
        self.spine[0]
    }

    fn last(&self) -> Loc {
        self.spine[self.spine.len() - 1]
    }

    fn get_length(&self) -> f32 {
        self.length
    }

    fn contains_pos(&self, pos: Vec3) -> bool {
        true
    }
}
