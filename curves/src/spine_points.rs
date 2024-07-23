use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::GuidePoints;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinePoints(Vec<Vec3>);

impl core::ops::Deref for SpinePoints {
    type Target = Vec<Vec3>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for SpinePoints {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

impl SpinePoints {
    pub fn from_vec(vec: Vec<Vec3>) -> Self {
        Self(vec)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn _from_guide_points(guide_points: &GuidePoints, dt: f32) -> Self {
        let mut spine_points = SpinePoints::empty();
        let mut t = 0.0;
        for _ in 0..((1. / dt) as u32) {
            spine_points.push(guide_points.calc_bezier_pos(t));
            t += dt;
        }
        spine_points
    }

    pub fn compute_length(&self) -> f32 {
        let mut result = 0.;
        for i in 0..(self.len() - 1) {
            result += (self[i] - self[i + 1]).length();
        }
        result
    }
}
