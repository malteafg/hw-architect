use crate::{LocCurve, PosCurve};

use utils::math::{Cur, DirXZ, Loc, VecUtils};

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Spines always have a uniform distribution of their points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spine(Vec<Loc>);

impl core::ops::Deref for Spine {
    type Target = Vec<Loc>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Spine {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

impl From<LocCurve> for Spine {
    fn from(curve: LocCurve) -> Self {
        let mut segment_length = 0.0;
        for i in 0..(curve.len() - 1) {
            segment_length += (curve[i + 1].pos - curve[i].pos).length();
        }

        let num_of_subsegements = (segment_length / utils::consts::CUT_LENGTH / 3.0).round() * 3.0;
        let uniform_dist = segment_length / (num_of_subsegements as f32);

        let mut uniform_spine: Spine = Spine::from_vec(vec![curve[0]]);
        let mut oldpoint = 0;
        let mut track_pos = 0.0;
        let mut within_subsegment = true;

        while oldpoint < curve.len() - 2 || within_subsegment {
            let old_subsegment = curve[oldpoint + 1].pos - curve[oldpoint].pos;
            let oss_length = old_subsegment.length();

            within_subsegment = track_pos < oss_length - uniform_dist;

            if !within_subsegment && oldpoint < curve.len() - 2 {
                track_pos -= oss_length;
                oldpoint += 1;
            } else {
                let interpolation_factor = (track_pos + uniform_dist) / oss_length;
                let pos = curve[oldpoint].pos + old_subsegment * interpolation_factor;
                let dir = Vec3::from(curve[oldpoint + 1].dir) * interpolation_factor
                    + Vec3::from(curve[oldpoint].dir) * (1.0 - interpolation_factor);
                uniform_spine.push(Loc::new(pos, dir.into()));
                track_pos += uniform_dist;
            }
        }
        oldpoint += 1;
        uniform_spine.push(Loc::new(curve[oldpoint].pos, curve[oldpoint].dir.into()));

        uniform_spine
    }
}

impl Spine {
    fn from_vec(vec: Vec<Loc>) -> Self {
        Self(vec)
    }

    /// Generates a set of `RawCurve`s that are parallel to this curve, ordered from left to right.
    pub fn gen_parallel(&self, path_width: f32, no_paths: u8) -> Vec<PosCurve> {
        let mut paths = Vec::with_capacity(no_paths.into());
        for _ in 0..no_paths {
            paths.push(PosCurve::with_capacity(self.len()));
        }

        for loc in self.iter() {
            let space = Vec3::from(loc.dir).right_hand() * path_width;
            let left_most = loc.pos - (no_paths as f32 / 2.) * space;
            for (i, path) in paths.iter_mut().enumerate() {
                let p = left_most + space * i as f32;
                path.push(p)
            }
        }
        paths
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SpinePoint {
    pos: Vec3,
    dir: DirXZ,
    cur: Cur,
}
