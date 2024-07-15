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
pub trait CurveSpec {
    /// Returns the spine of this curve segment. Maybe this should compute the spine, which can
    /// then be stored elsewhere.
    fn get_spine(&self) -> Spine;

    /// Returns the length in meters of this curve segment
    fn get_length(&self) -> f32;
}

#[enum_dispatch(CurveSpec)]
pub enum Curve {
    Straight,
    Circular,
    Quadratic,
    Cubic,
}
