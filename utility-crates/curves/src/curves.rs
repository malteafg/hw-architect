mod circular;
mod cubic;
mod quadratic;
mod straight;

pub use circular::Circular;
pub use cubic::Cubic;
pub use quadratic::Quadratic;
pub use straight::Straight;
use utils::math::Loc;

use crate::Spine;

use thiserror::Error;

use enum_dispatch::enum_dispatch;
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[enum_dispatch]
pub trait CurveSpec: CurveShared {}

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
    fn contains_pos(&self, pos: Vec3, width: f32) -> bool;

    /// Reverses this curve
    fn reverse(&mut self);
}

trait CurveUnique {
    fn compute_spine(&self) -> Spine;
    fn reverse(&mut self);
    fn contains_pos(&self, pos: Vec3, width: f32) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub enum CurveInfo {
    /// The curve was built to exactly the specification.
    Satisfied,

    /// The last point of the curve was projected. The target position that was not satisfied is
    /// returned.
    Projection(Vec3),
}

#[derive(Debug, Clone)]
pub enum CompositeCurveSum {
    Single(CurveSum),
    Double(CurveSum, CurveSum),
}

impl<C: Into<CurveSum>> From<C> for CompositeCurveSum {
    fn from(value: C) -> Self {
        CompositeCurveSum::Single(value.into())
    }
}

#[derive(Debug, Clone)]
pub enum CompositeCurve<C: CurveShared> {
    Single(C),
    Double(C, C),
}

impl<C: Into<CurveSum> + CurveShared> From<CompositeCurve<C>> for CompositeCurveSum {
    fn from(value: CompositeCurve<C>) -> Self {
        match value {
            CompositeCurve::Single(curve) => CompositeCurveSum::Single(curve.into()),
            CompositeCurve::Double(c1, c2) => CompositeCurveSum::Double(c1.into(), c2.into()),
        }
    }
}

#[derive(Error, Debug)]
pub enum CurveError {
    /// A curve can be constructed but the curve is too tight.
    #[error("The curve has points for which the curvature is too extreme")]
    TooTight(CompositeCurveSum),

    /// A curve can be constructed but the curve is too short.
    #[error("The curve is too short")]
    TooShort(CompositeCurveSum),

    /// The curve cannot be created given the current parameters.
    #[error("The curve is impossible to construct with the given constraints")]
    Impossible,
}

pub type CurveResult<C> = std::result::Result<C, CurveError>;

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

impl<C: CurveUnique> CurveSpec for Curve<C> {}

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

impl<C: CurveUnique> CurveShared for Curve<C> {
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

    fn contains_pos(&self, pos: Vec3, width: f32) -> bool {
        self.instance.contains_pos(pos, width)
    }

    fn reverse(&mut self) {
        self.instance.reverse();
        self.spine = self.instance.compute_spine();
    }
}
