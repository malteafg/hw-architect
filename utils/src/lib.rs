//! This crate defines utilities that any other crate can utilize. This crate
//! should not depend on any other crates.

// for use in input enums
extern crate strum;
#[macro_use]
extern crate strum_macros;

pub mod consts;
pub mod id;
pub mod input;
pub mod loader;
pub mod math;
pub mod time;
