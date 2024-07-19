//! This crate defines utilities that any other crate can utilize. This crate
//! should not depend on any other crates.

// for use in input enums
extern crate strum;
#[macro_use]
extern crate strum_macros;

mod math_utils;
pub use math_utils::{Angle, DirXZ, Loc, Mat3Utils, Mat4Utils, PosOrLoc, Ray, Round, VecUtils};

pub mod consts;
pub mod id;
pub mod input;
pub mod loader;
pub mod time;
