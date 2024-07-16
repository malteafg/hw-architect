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
pub trait CurveShared {
    /// Returns the spine of this curve segment
    fn get_spine(&self) -> &Spine;

    /// Returns the length in meters of this curve segment
    fn get_length(&self) -> f32;
}

#[enum_dispatch]
pub trait CurveUnique {
    fn compute_spine(&self) -> Spine;
}

pub trait CurveSpec: CurveUnique + CurveShared {}

#[enum_dispatch(CurveUnique, CurveShared)]
pub enum CurveType {
    Straight(Curve<Straight>),
    Circular(Curve<Circular>),
    Quadratic(Curve<Quadratic>),
    Cubic(Curve<Cubic>),
}

pub struct Curve<C> {
    instance: C,
    length: f32,
    spine: Spine,
}

impl<C> CurveShared for Curve<C> {
    fn get_spine(&self) -> &Spine {
        &self.spine
    }

    fn get_length(&self) -> f32 {
        self.length
    }
}
