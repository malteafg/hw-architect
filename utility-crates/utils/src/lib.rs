//! This crate defines utilities that any other crate can utilize. This crate
//! should not depend on any other crates.

// for use in input enums
extern crate strum;
#[macro_use]
extern crate strum_macros;

mod math_utils;
pub use math_utils::Angle;
pub use math_utils::DirXZ;
pub use math_utils::Loc;
pub use math_utils::Mat3Utils;
pub use math_utils::Mat4Utils;
pub use math_utils::Ray;
pub use math_utils::Round;
pub use math_utils::VecUtils;

pub mod consts;
pub mod id;
pub mod input;
pub mod loader;
pub mod time;
