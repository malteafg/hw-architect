use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::{DirXZ, Loc, VecUtils};

use crate::{Curve, CurveError, CurveInfo, CurveResult, GuidePoints, Spine};

use super::{CompositeCurve, CurveUnique};

const COS_45: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// A circular curve approximated using cubic bezier curves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circular {
    guide_points: GuidePoints,
}

impl Circular {
    fn new(first: Loc, last_pos: Vec3) -> Self {
        Circular {
            guide_points: circle_curve(first, last_pos),
        }
    }
}

impl CurveUnique for Circular {
    fn compute_spine(&self) -> Spine {
        Spine::from_guide_points(&self.guide_points)
    }

    fn reverse(&mut self) {
        self.guide_points.reverse()
    }

    fn contains_pos(&self, pos: Vec3, width: f32) -> bool {
        self.guide_points.is_inside(pos, width)
    }
}

impl Curve<Circular> {
    /// Direction has been set either by tool or a snapped node
    pub fn from_first_locked(first: Loc, last_pos: Vec3) -> (CompositeCurve<Self>, CurveInfo) {
        let (last_pos, curve_info) = match three_quarter_projection(first, last_pos) {
            Some(proj_pos) => (proj_pos, CurveInfo::Projection(last_pos)),
            None => (last_pos, CurveInfo::Satisfied),
        };

        let diff = last_pos - first.pos;
        if diff.dot(Vec3::from(first.dir)) >= 0.0 {
            let curve = Circular::new(first, last_pos);
            (CompositeCurve::Single(curve.into()), curve_info)
        } else {
            let mid = curve_mid(first, last_pos);
            let curve1 = Circular::new(first, mid.pos);
            let curve2 = Circular::new(mid, last_pos);
            (
                CompositeCurve::Double(curve1.into(), curve2.into()),
                curve_info,
            )
        }
    }

    /// Only position has been set and we are snapping to another node
    pub fn from_last_locked(first_pos: Vec3, last: Loc) -> CurveResult<CompositeCurve<Self>> {
        let diff = last.pos - first_pos;
        let first_dir = -last.dir.mirror(diff);
        let first = Loc::new(first_pos, first_dir);

        if three_quarter_projection(first, last.pos).is_some() {
            return Err(CurveError::Impossible);
        };

        if diff.dot(Vec3::from(first.dir)) >= 0.0 {
            let curve = Circular::new(first, last.pos);
            Ok(CompositeCurve::Single(curve.into()))
        } else {
            let mid = curve_mid(first, last.pos);
            let curve1 = Circular::new(first, mid.pos);
            let curve2 = Circular::new(mid, last.pos);
            Ok(CompositeCurve::Double(curve2.into(), curve1.into()))
        }
    }

    /// A double snap
    pub fn from_both_locked(first: Loc, last: Loc) -> (CompositeCurve<Self>, CurveInfo) {

        unimplemented!()
    }
}

/// Checks if the circle can be created with less than 270 degrees, otherwise returns the projected
/// point that yields a 270 degree circle.
fn three_quarter_projection(first: Loc, last_pos: Vec3) -> Option<Vec3> {
    let diff = last_pos - first.pos;
    let proj_length = diff.dot(Vec3::from(first.dir));
    if proj_length >= -COS_45 * diff.length() {
        None
    } else {
        let proj = diff.proj(Vec3::from(first.dir));
        let anti_proj = diff.anti_proj(Vec3::from(first.dir));
        Some(first.pos + proj + anti_proj.rescale(proj_length))
    }
}

/// Computes the mid point off the circular curve.
fn curve_mid(first: Loc, last_pos: Vec3) -> Loc {
    let diff = last_pos - first.pos;
    let dir_to_mid = diff.normalize() + first.dir;
    let mid_len = diff.length_squared() / (dir_to_mid.dot(diff) * 2.0);
    let mid_pos = first.pos + (dir_to_mid * mid_len);
    let mid_dir = diff.into();
    Loc::new(mid_pos, mid_dir)
}

/// The guidepoints for a curve as circular as possible with four guide points, up to half a circle
fn circle_curve(first: Loc, last_pos: Vec3) -> GuidePoints {
    let diff = last_pos - first.pos;
    let r = first.dir * circle_scale(diff, first.dir);

    GuidePoints::from_vec(vec![
        first.pos,
        first.pos + r,
        last_pos + r - diff * (2.0 * diff.dot(r) / diff.length_squared()),
        last_pos,
    ])
}

/// Computes the side length of the trapezoid that defines the guide points of the circular curve.
fn circle_scale(diff: Vec3, dir: DirXZ) -> f32 {
    let dot = diff.normalize().dot(dir.into());
    if dot == 1.0 {
        // Makes it so that straight curves have intermidiary guidepoints and 1/3 and 2/3
        diff.length() / 3.0
    } else {
        2.0 / 3.0 * diff.length() * (1.0 - dot) / (1.0 - dot * dot)
    }
}
