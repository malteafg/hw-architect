mod circular;
mod cubic;
mod quadratic;
mod straight;

pub use circular::Circular;
pub use cubic::Cubic;
pub use quadratic::Quadratic;
pub use straight::Straight;

use crate::Spine;

use enum_dispatch::enum_dispatch;
use glam::Vec3;

#[enum_dispatch]
pub trait CurveSpec {
    /// Returns the spine of this curve segment
    fn get_spine(&self) -> &Spine;

    /// Returns the length in meters of this curve segment
    fn get_length(&self) -> f32;

    /// Checks if the given position is contained within the curve given a width
    fn contains_pos(&self, pos: Vec3) -> bool;
}

#[enum_dispatch]
pub trait RawCurveSpec {
    fn compute_spine(&self) -> Spine;
}

#[enum_dispatch(CurveSpec)]
#[derive(Debug, Clone)]
pub enum CurveType {
    Straight(Curve<Straight>),
    Circular(Curve<Circular>),
    Quadratic(Curve<Quadratic>),
    Cubic(Curve<Cubic>),
}

#[enum_dispatch(RawCurveSpec)]
#[derive(Debug, Clone)]
pub enum RawCurveType {
    Straight(Straight),
    Circular(Circular),
    Quadratic(Quadratic),
    Cubic(Cubic),
}

#[derive(Debug, Clone)]
pub struct Curve<C: RawCurveSpec> {
    instance: C,
    length: f32,
    spine: Spine,
}

impl<C: RawCurveSpec> From<C> for Curve<C> {
    fn from(value: C) -> Self {
        let spine = value.compute_spine();

        Self {
            instance: value,
            length: 0.0,
            spine,
        }
    }
}

impl<C: RawCurveSpec> From<C> for CurveType {
    fn from(value: C) -> Self {
        // let curve: Curve<C> = value.into();
        let res: CurveType = value.into();
        res
    }
}

impl<C: RawCurveSpec> CurveSpec for Curve<C> {
    fn get_spine(&self) -> &Spine {
        &self.spine
    }

    fn get_length(&self) -> f32 {
        self.length
    }

    fn contains_pos(&self, pos: Vec3) -> bool {
        true
    }
}
