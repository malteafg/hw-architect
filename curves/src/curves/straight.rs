use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::consts::ROAD_MIN_LENGTH;
use utils::math::{DirXZ, Loc, VecUtils};

use crate::curves::CurveUnique;
use crate::{CtrlPoints, Curve, CurveError, CurveInfo, CurveResult, Spine};

/// Represents a completely straight line. Should not use guide_points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Straight {
    guide_points: CtrlPoints,
}

impl Straight {
    fn new(first_pos: Vec3, last_pos: Vec3) -> Self {
        let guide_points = CtrlPoints::from_two_points(first_pos, last_pos);
        Self { guide_points }
    }
}

impl CurveUnique for Straight {
    fn compute_spine(&self) -> Spine {
        self.guide_points.gen_loc_curve().into()
    }

    fn reverse(&mut self) {
        self.guide_points.reverse()
    }

    fn contains_pos(&self, pos: Vec3, width: f32) -> bool {
        self.guide_points.contains_pos(pos, width)
    }
}

impl Curve<Straight> {
    pub fn from_free(first_pos: Vec3, last_pos: Vec3) -> (Self, CurveInfo) {
        let dir = DirXZ::from(last_pos - first_pos);
        let (last_pos, info) = proj_straight_too_short(first_pos, last_pos, dir);
        let curve = Straight::new(first_pos, last_pos);
        (curve.into(), info)
    }

    pub fn from_first_locked(first: Loc, last_pos: Vec3) -> (Self, CurveInfo) {
        let first_pos = first.pos;
        let first_dir = first.dir;
        let first_to_last = last_pos - first_pos;
        let proj_pos = if first_to_last.dot(*first_dir) > ROAD_MIN_LENGTH {
            // The projection will yield a long enough segment
            first_to_last.proj(*first_dir) + first_pos
        } else {
            // The projection will be to short and therefore we set proj_pos to min road length
            first_pos + first_dir * ROAD_MIN_LENGTH
        };
        let curve = Straight::new(first_pos, proj_pos);
        (curve.into(), CurveInfo::Projection(last_pos))
    }

    pub fn from_last_locked(first_pos: Vec3, last: Loc) -> CurveResult<Self> {
        let dir: DirXZ = (last.pos - first_pos).into();
        if dir != last.dir {
            return Err(CurveError::Impossible);
        }

        let curve = Straight::new(first_pos, last.pos);
        let curve: Curve<Straight> = curve.into();
        if (last.pos - first_pos).length() < ROAD_MIN_LENGTH {
            Err(CurveError::TooShort(curve.into()))
        } else {
            Ok(curve.into())
        }
    }

    pub fn from_both_locked(first: Loc, last: Loc) -> CurveResult<Self> {
        if first.dir == last.dir {
            Curve::<Straight>::from_last_locked(first.pos, last)
        } else {
            Err(CurveError::Impossible)
        }
    }
}

fn proj_straight_too_short(start_pos: Vec3, pref_pos: Vec3, proj_dir: DirXZ) -> (Vec3, CurveInfo) {
    if (pref_pos - start_pos).length() < ROAD_MIN_LENGTH {
        let dir = (pref_pos - start_pos).try_normalize().unwrap_or(*proj_dir);
        (
            start_pos + dir * ROAD_MIN_LENGTH,
            CurveInfo::Projection(pref_pos),
        )
    } else {
        (pref_pos, CurveInfo::Satisfied)
    }
}
