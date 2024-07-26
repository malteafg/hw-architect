use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::math::Loc;

use crate::{LocCurve, PosCurve};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrlPoints(Vec<Vec3>);

impl core::ops::Deref for CtrlPoints {
    type Target = Vec<Vec3>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for CtrlPoints {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

impl CtrlPoints {
    pub fn from_vec(vec: Vec<Vec3>) -> Self {
        Self(vec)
    }

    /// Creates four points on a straight line between the two given points (containing them).
    /// # Panics
    /// Panics if the points are the same.
    pub fn from_two_points(p1: Vec3, p2: Vec3) -> Self {
        let dir = (p2 - p1).normalize();
        let dist = (p2 - p1).length();
        let dist_dir = dir * (dist / 3.);
        CtrlPoints::from_vec(vec![p1, p1 + dist_dir, p1 + (2. * dist_dir), p2])
    }

    /// Computes and returns the summed distance from the path starting in the first point to the
    /// last point, visiting all points.
    pub fn dist(&self) -> f32 {
        let mut sum = 0.0;
        for i in 0..self.len() - 1 {
            sum += (self[i] - self[i + 1]).length()
        }
        sum
    }

    pub fn gen_pos_curve(&self) -> PosCurve {
        let mut pos_curve = PosCurve::empty();

        let num_of_cuts = (utils::consts::VERTEX_DENSITY * (1000.0 + self.dist())) as u32;
        let dt = 1.0 / (num_of_cuts as f32 - 1.0);
        let mut t = 0.0;

        for _ in 0..num_of_cuts {
            let pos = self.calc_bezier_pos(t);
            pos_curve.push(pos);
            t += dt;
        }
        pos_curve
    }

    pub fn gen_loc_curve(&self) -> LocCurve {
        let mut loc_curve = LocCurve::empty();

        let num_of_cuts = (utils::consts::VERTEX_DENSITY * (1000.0 + self.dist())) as u32;
        let dt = 1.0 / (num_of_cuts as f32 - 1.0);
        let mut t = 0.0;

        for _ in 0..num_of_cuts {
            let pos = self.calc_bezier_pos(t);
            let dir = self.calc_bezier_dir(t);
            loc_curve.push(Loc::new(pos, dir.into()));
            t += dt;
        }
        loc_curve
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
        v.normalize()
    }

    pub fn reverse(vec: &mut Vec<Self>) {
        vec.reverse();
        for guide_points in vec.iter_mut() {
            guide_points.reverse();
        }
    }

    pub fn contains_pos(&self, ground_pos: Vec3, width: f32) -> bool {
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
