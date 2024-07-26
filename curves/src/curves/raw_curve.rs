use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::math::Loc;

/// Represents a curve as a vector of positions, with no restrictions on the distance between the
/// points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosCurve {
    positions: Vec<Vec3>,
}

impl core::ops::Deref for PosCurve {
    type Target = Vec<Vec3>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.positions
    }
}

impl core::ops::DerefMut for PosCurve {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.positions
    }
}

impl PosCurve {
    pub fn from_vec(positions: Vec<Vec3>) -> Self {
        Self { positions }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
        }
    }

    pub fn empty() -> Self {
        Self { positions: vec![] }
    }

    pub fn compute_length(&self) -> f32 {
        let mut result = 0.;
        for i in 0..(self.len() - 1) {
            result += (self[i] - self[i + 1]).length();
        }
        result
    }
}

/// Represents a curve as a vector of locations, with no restrictions on the distance between the
/// points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocCurve {
    locations: Vec<Loc>,
}

impl core::ops::Deref for LocCurve {
    type Target = Vec<Loc>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.locations
    }
}

impl core::ops::DerefMut for LocCurve {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.locations
    }
}

impl LocCurve {
    pub fn from_vec(locations: Vec<Loc>) -> Self {
        Self { locations }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            locations: Vec::with_capacity(capacity),
        }
    }

    pub fn empty() -> Self {
        Self { locations: vec![] }
    }

    pub fn compute_length(&self) -> f32 {
        let mut result = 0.;
        for i in 0..(self.len() - 1) {
            result += (self[i].pos - self[i + 1].pos).length();
        }
        result
    }
}
