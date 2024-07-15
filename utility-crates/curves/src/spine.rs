use crate::{GuidePoints, SpinePoints};

use utils::VecUtils;

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Spines always have a uniform distribution of their points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spine(Vec<(Vec3, Vec3)>);

impl core::ops::Deref for Spine {
    type Target = Vec<(Vec3, Vec3)>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Spine {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

impl Spine {
    fn from_vec(vec: Vec<(Vec3, Vec3)>) -> Self {
        Self(vec)
    }

    /// Should be made not public
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Will make the returned spine uniform.
    pub fn from_guide_points(guide_points: &GuidePoints) -> Self {
        let mut spine = Spine::empty();

        let num_of_cuts = (utils::consts::VERTEX_DENSITY * (1000.0 + guide_points.dist())) as u32;
        let dt = 1.0 / (num_of_cuts as f32 - 1.0);
        let mut t = 0.0;

        for _ in 0..num_of_cuts {
            let pos = guide_points.calc_bezier_pos(t);
            let dir = guide_points.calc_bezier_dir(t);
            spine.push((pos, dir));
            t += dt;
        }

        spine.make_uniform()
    }

    fn make_uniform(self) -> Self {
        // Note that .0 is pos and .1 is dir. Probably not ideal.

        let mut segment_length = 0.0;
        for i in 0..(self.len() - 1) {
            segment_length += (self[i + 1].0 - self[i].0).length();
        }

        let num_of_subsegements = (segment_length / utils::consts::CUT_LENGTH / 3.0).round() * 3.0;
        let uniform_dist = segment_length / (num_of_subsegements as f32);

        let mut uniform_spine: Spine = Spine::from_vec(vec![self[0]]);
        let mut oldpoint = 0;
        let mut track_pos = 0.0;
        let mut within_subsegment = true;

        while oldpoint < self.len() - 2 || within_subsegment {
            let old_subsegment = self[oldpoint + 1].0 - self[oldpoint].0;
            let oss_length = old_subsegment.length();

            within_subsegment = track_pos < oss_length - uniform_dist;

            if !within_subsegment && oldpoint < self.len() - 2 {
                track_pos -= oss_length;
                oldpoint += 1;
            } else {
                let interpolation_factor = (track_pos + uniform_dist) / oss_length;
                let pos = self[oldpoint].0 + old_subsegment * interpolation_factor;
                let dir = self[oldpoint + 1].1 * interpolation_factor
                    + self[oldpoint].1 * (1.0 - interpolation_factor);
                uniform_spine.push((pos, dir.normalize()));
                track_pos += uniform_dist;
            }
        }
        oldpoint += 1;
        uniform_spine.push((self[oldpoint].0, self[oldpoint].1));

        uniform_spine
    }

    /// Generates a set of parallel spine_points ordered from left to right.
    pub fn gen_parallel(&self, path_width: f32, no_paths: u8) -> Vec<SpinePoints> {
        let mut paths = Vec::with_capacity(no_paths.into());
        for _ in 0..no_paths {
            paths.push(SpinePoints::with_capacity(self.len()));
        }

        for (pos, dir) in self.iter() {
            let space = dir.right_hand() * path_width;
            let left_most = *pos - (no_paths as f32 / 2.) * space;
            for (i, path) in paths.iter_mut().enumerate() {
                let p = left_most + space * i as f32;
                path.push(p)
            }
        }
        paths
    }
}
