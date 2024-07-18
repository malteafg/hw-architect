use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::{consts::ROAD_MIN_LENGTH, DirXZ, Loc, VecUtils};

use crate::{Curve, GuidePoints, Spine};

use super::{CurveBuilder, CurveResult, CurveUnique};

/// Represents a completely straight line. Should not use guide_points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Straight {
    guide_points: GuidePoints,
}

impl Straight {
    fn new(start: Vec3, end: Vec3) -> Self {
        let guide_points = GuidePoints::from_two_points(start, end);
        Self { guide_points }
    }
}

impl CurveUnique for Straight {
    fn compute_spine(&self) -> Spine {
        Spine::from_guide_points(&self.guide_points)
    }
}

impl CurveBuilder for Curve<Straight> {
    fn from_free(first_pos: Vec3, last_pos: Vec3) -> CurveResult<Self> {
        let dir = DirXZ::from(last_pos - first_pos);
        let end = proj_straight_too_short(first_pos, last_pos, dir);
        let curve = Straight::new(first_pos, end);
        CurveResult::Ok(curve.into())
    }

    fn from_start_locked(first: Loc, last_pos: Vec3) -> CurveResult<Self> {
        let first_pos = first.pos;
        let first_dir = first.dir;
        let first_to_last = last_pos - first_pos;
        let proj_pos = if first_to_last.dot(first_dir.into()) > ROAD_MIN_LENGTH {
            // The projection will yield a long enough segment
            first_to_last.proj(first_dir.into()) + first_pos
        } else {
            // The projection will be to short and therefore we set proj_pos to min road length
            first_pos + Vec3::from(first_dir) * ROAD_MIN_LENGTH
        };
        let curve = Straight::new(first_pos, proj_pos);
        CurveResult::Ok(curve.into())
    }

    fn from_end_locked(first_pos: Vec3, last: Loc) -> CurveResult<Self> {
        unimplemented!()
    }

    fn from_both_locked(first: Loc, last: Loc) -> CurveResult<Self> {
        unimplemented!()
    }
}

fn proj_straight_too_short(start_pos: Vec3, pref_pos: Vec3, proj_dir: DirXZ) -> Vec3 {
    if (pref_pos - start_pos).length() < ROAD_MIN_LENGTH {
        start_pos
            + (pref_pos - start_pos)
                .try_normalize()
                .unwrap_or(proj_dir.into())
                * ROAD_MIN_LENGTH
    } else {
        pref_pos
    }
}
