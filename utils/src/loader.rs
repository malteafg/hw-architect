//! This module handles I/O

use std::io;

/// Loads a file relative to the res directory as a [`String`] object. This is
/// useful for loading plaintext such as yaml files.
pub fn load_string(file_name: &str) -> io::Result<String> {
    let path = std::path::Path::new("res").join(file_name);
    let txt = std::fs::read_to_string(path)?;
    Ok(txt)
}

/// Loads a file relative to the res directoly as a [`Vec`] of bytes ([`u8`]).
/// This is useful for loading textures, models and such.
pub fn load_binary(file_name: &str) -> io::Result<Vec<u8>> {
    let path = std::path::Path::new("res").join(file_name);
    let data = std::fs::read(path)?;
    Ok(data)
}
