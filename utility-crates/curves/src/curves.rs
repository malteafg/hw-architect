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

#[enum_dispatch]
pub trait CurveSpec<C: RawCurveSpec> {
    fn from_raw_curve(raw_curve: C) -> Self;

    /// Returns the spine of this curve segment
    fn get_spine(&self) -> &Spine;

    /// Returns the length in meters of this curve segment
    fn get_length(&self) -> f32;
}

#[enum_dispatch]
pub trait RawCurveSpec {
    fn compute_spine(&self) -> Spine;
}

#[enum_dispatch(CurveSpec)]
pub enum CurveType {
    Straight(Curve<Straight>),
    Circular(Curve<Circular>),
    Quadratic(Curve<Quadratic>),
    Cubic(Curve<Cubic>),
}

#[enum_dispatch(RawCurveSpec)]
pub enum RawCurveType {
    Straight(Straight),
    Circular(Circular),
    Quadratic(Quadratic),
    Cubic(Cubic),
}

pub struct Curve<C: RawCurveSpec> {
    instance: C,
    length: f32,
    spine: Spine,
}

impl<C: RawCurveSpec> CurveSpec<C> for Curve<C> {
    fn from_raw_curve(raw_curve: C) -> Self {
        let spine = raw_curve.compute_spine();

        Self {
            instance: raw_curve,
            length: 0.0,
            spine,
        }
    }

    fn get_spine(&self) -> &Spine {
        &self.spine
    }

    fn get_length(&self) -> f32 {
        self.length
    }
}
