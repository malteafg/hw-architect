use glam::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidePoints(Vec<Vec3>);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinePoints(Vec<Vec3>);

impl core::ops::Deref for GuidePoints {
    type Target = Vec<Vec3>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for GuidePoints {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

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

impl GuidePoints {
    pub fn from_vec(vec: Vec<Vec3>) -> Self {
        GuidePoints(vec)
    }

    pub fn empty() -> Self {
        GuidePoints(vec![])
    }

    pub fn calc_bezier_pos(&self, t: f32) -> Vec3 {
        let mut v = Vec3::new(0.0, 0.0, 0.0);
        let mut r = (1.0 - t).powi(self.len() as i32 - 1);
        let mut l = 1.0;
        for (i, p) in self.iter().enumerate() {
            let f = l * r;
            v += *p * f;
            if t == 1.0 {
                if i == self.len() - 2 {
                    r = 1.0;
                } else {
                    r = 0.0;
                }
            } else {
                r *= t / (1.0 - t);
            }
            l *= self.len() as f32 / (1.0 + i as f32) - 1.0;
        }
        v
    }

    pub fn calc_bezier_dir(&self, t: f32) -> Vec3 {
        let mut v = Vec3::new(0.0, 0.0, 0.0);
        let mut r = (1.0 - t).powi(self.len() as i32 - 2);
        let mut l = 1.0;
        for p in 0..(self.len() - 1) {
            v += (self[p + 1] - self[p]) * l * r;
            if t == 1.0 {
                if p == self.len() - 3 {
                    r = 1.0;
                } else {
                    r = 0.0;
                }
            } else {
                r *= t / (1.0 - t);
            }
            l *= (self.len() as f32 - 1.0) / (1.0 + p as f32) - 1.0;
        }
        v * self.len() as f32
    }

    pub fn get_spine_points(&self, dt: f32) -> SpinePoints {
        let mut spine_points = SpinePoints::empty();
        let mut t = 0.0;
        for _ in 0..((1. / t) as u32) {
            spine_points.push(self.calc_bezier_pos(t));
            t += dt;
        }
        spine_points
    }

    pub fn is_inside(&self, ground_pos: Vec3, width: f32) -> bool {
        let direct_dist = (self[self.len() - 1] - self[0]).length_squared();

        let mut close = false;
        let mut distance_squared = f32::MAX;
        for &point in self.iter() {
            let dist = (point - ground_pos).length_squared();
            if dist < direct_dist {
                close = true;
                if dist < distance_squared {
                    distance_squared = dist;
                }
            }
        }
        if !close {
            return false;
        } else if distance_squared < width * width {
            return true;
        }

        let mut a = 0.0;
        let mut c = 1.0;
        let mut point_a = self.calc_bezier_pos(a);
        let mut point_c = self.calc_bezier_pos(c);
        for _ in 0..10 {
            let point_b = self.calc_bezier_pos((a + c) / 2.0);
            if (point_b - ground_pos).length_squared() < width * width {
                return true;
            }

            if (point_a - ground_pos).length_squared() < (point_c - ground_pos).length_squared() {
                point_c = point_b;
                c = (a + c) / 2.0;
            } else {
                point_a = point_b;
                a = (a + c) / 2.0;
            }
        }
        false
    }
}

impl SpinePoints {
    pub fn from_vec(vec: Vec<Vec3>) -> Self {
        SpinePoints(vec)
    }

    pub fn empty() -> Self {
        SpinePoints(vec![])
    }
}
