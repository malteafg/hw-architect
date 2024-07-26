use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::consts::ROAD_MIN_LENGTH;
use utils::math::{DirXZ, Loc, VecUtils};

use crate::{CtrlPoints, Curve, CurveError, CurveInfo, CurveResult, Spine};

use super::{CompositeCurve, CurveUnique};

const PRETTY_CLOSE: f32 = 0.97;
const CLOSE_ENOUGH: f32 = 0.95;
const COS_45: f32 = std::f32::consts::FRAC_1_SQRT_2;
const MIN_SEGMENT_LENGTH: f32 = 10.0;

/// A circular curve approximated using cubic bezier curves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circular {
    guide_points: CtrlPoints,
}

impl Circular {
    fn new(first: Loc, last_pos: Vec3) -> Self {
        Circular {
            guide_points: circle_curve(first, last_pos),
        }
    }

    fn from_guide_points(guide_points: CtrlPoints) -> Self {
        Circular { guide_points }
    }
}

impl CurveUnique for Circular {
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

impl Curve<Circular> {
    /// Direction has been set either by tool or a snapped node
    pub fn from_first_locked(first: Loc, last_pos: Vec3) -> (CompositeCurve<Self>, CurveInfo) {
        let (last_pos, curve_info) = if (last_pos - first.pos).length() < ROAD_MIN_LENGTH {
            (
                first.pos
                    + (last_pos - first.pos).try_normalize().unwrap_or(*first.dir)
                        * ROAD_MIN_LENGTH,
                CurveInfo::Projection(last_pos),
            )
        } else {
            (last_pos, CurveInfo::Satisfied)
        };

        let (last_pos, curve_info) = match three_quarter_projection(first, last_pos) {
            Some(proj_pos) => (proj_pos, CurveInfo::Projection(last_pos)),
            None => (last_pos, curve_info),
        };

        let diff = last_pos - first.pos;
        if diff.dot(*first.dir) >= 0.0 {
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

        if diff.dot(*first.dir) >= 0.0 {
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
    pub fn from_both_locked(first: Loc, last: Loc) -> CurveResult<CompositeCurve<Self>> {
        let diff = last.pos - first.pos;
        let last = last.flip(true);

        if (*first.dir).mirror(diff).ndot(*last.dir) > PRETTY_CLOSE
            && (-diff).dot(*last.dir) >= PRETTY_CLOSE - 1.0
            && diff.dot(*first.dir) >= PRETTY_CLOSE - 1.0
        {
            let guide_points = circle_curve_fudged(first, last);
            let curve = Circular::from_guide_points(guide_points);
            return Ok(CompositeCurve::Single(curve.into()));
        }

        let t = s_curve_segment_length(first, last);
        let center = (first.pos + last.pos + first.dir * t + last.dir * t) / 2.0;

        if is_curve_too_small(*first.dir, center - first.pos, 6)
            || is_curve_too_small(*last.dir, center - last.pos, 6)
        {
            return Err(CurveError::Impossible);
        }

        // Segment angle. The center must be in front of the two end points.
        if (*first.dir).dot(center - first.pos) <= 0.0 || (*last.dir).dot(center - last.pos) <= 0.0
        {
            return Err(CurveError::Impossible);
        }

        // Curve angle. The direction towards center should be approximately the same as the
        // direction towards the other end point, for both end points.
        if (last.pos - first.pos).dot(center - first.pos) <= 0.0
            || (first.pos - last.pos).dot(center - last.pos) <= 0.0
        {
            return Err(CurveError::Impossible);
        }

        if is_elliptical(first, last) {
            simple_curve_points(first, last).map(|guide_points| {
                let curve = Circular::from_guide_points(guide_points);
                CompositeCurve::Single(curve.into())
            })
        } else {
            let (g1, g2) = double_curve(first, center, last);
            let curve1 = Circular::from_guide_points(g1);
            let curve2 = Circular::from_guide_points(g2);
            Ok(CompositeCurve::Double(curve1.into(), curve2.into()))
        }
    }
}

/// Checks if the circle can be created with less than 270 degrees, otherwise returns the projected
/// point that yields a 270 degree circle.
fn three_quarter_projection(first: Loc, last_pos: Vec3) -> Option<Vec3> {
    let diff = last_pos - first.pos;
    let proj_length = diff.dot(*first.dir);
    if proj_length >= -COS_45 * diff.length() {
        None
    } else {
        let proj = diff.proj(*first.dir);
        let anti_proj = diff.anti_proj(*first.dir);
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
fn circle_curve(first: Loc, last_pos: Vec3) -> CtrlPoints {
    let diff = last_pos - first.pos;
    let r = first.dir * circle_scale(diff, first.dir);

    CtrlPoints::from_vec(vec![
        first.pos,
        first.pos + r,
        last_pos + r - diff * (2.0 * diff.dot(r) / diff.length_squared()),
        last_pos,
    ])
}

/// Best approximation of circular curve when constrained by directions at both points
fn circle_curve_fudged(first: Loc, last: Loc) -> CtrlPoints {
    let diff = last.pos - first.pos;
    let r = first.dir * circle_scale(diff, first.dir);

    CtrlPoints::from_vec(vec![
        first.pos,
        first.pos + r,
        last.pos + last.dir * r.length(),
        last.pos,
    ])
}

/// Only used if the double snap is elliptical. Simply creates a 3 point bezier curve.
fn simple_curve_points(first: Loc, last: Loc) -> CurveResult<CtrlPoints> {
    if (*first.dir).intersects_in_xz(*last.dir) {
        Ok(CtrlPoints::from_vec(vec![
            first.pos,
            first
                .pos
                .intersection_in_xz(*first.dir, last.pos, *last.dir),
            last.pos,
        ]))
    } else {
        Err(CurveError::Impossible)
    }
}

fn double_curve(first: Loc, center: Vec3, last: Loc) -> (CtrlPoints, CtrlPoints) {
    let first_half = circle_curve(first, center);
    let mut second_half = circle_curve(last, center);
    second_half.reverse();
    (first_half, second_half)
}

/// Computes the side length of the trapezoid that defines the guide points of the circular curve.
fn circle_scale(diff: Vec3, dir: DirXZ) -> f32 {
    let dot = diff.normalize().dot(*dir);
    if dot == 1.0 {
        // Makes it so that straight curves have intermidiary guidepoints and 1/3 and 2/3
        diff.length() / 3.0
    } else {
        2.0 / 3.0 * diff.length() * (1.0 - dot) / (1.0 - dot * dot)
    }
}

/// Used for double snap only. No clue what happens here
fn s_curve_segment_length(first: Loc, last: Loc) -> f32 {
    let diff_pos = last.pos - first.pos;
    let diff_dir = *last.dir - first.dir;
    if diff_dir.length_squared() == 4.0 {
        return 0.0;
    }
    let k = diff_pos.dot(diff_dir) / (4.0 - diff_dir.length_squared());
    k + (diff_pos.length_squared() / (4.0 - diff_dir.length_squared()) + k * k).sqrt()
}

/// Used for double snap only. No clue what happens here
fn is_elliptical(first: Loc, last: Loc) -> bool {
    let diff = last.pos - first.pos;
    if first.dir.dot(last.dir) > 0.0 {
        return false;
    }
    if (-diff).ndot(*last.dir) < PRETTY_CLOSE - 1.0 || diff.ndot(*first.dir) < PRETTY_CLOSE - 1.0 {
        return false;
    }
    if (*first.dir)
        .anti_proj(diff)
        .dot((*last.dir).anti_proj(diff))
        < 0.0
    {
        return false;
    }
    if !(*first.dir).intersects_in_xz(*last.dir) {
        return false;
    }
    let intersection = first
        .pos
        .intersection_in_xz(*first.dir, last.pos, *last.dir);
    let f1 = (intersection - first.pos).length();
    let f2 = (intersection - last.pos).length();
    let rel = f1.min(f2) / f1.max(f2);
    if f1 * rel < MIN_SEGMENT_LENGTH || f2 * rel < MIN_SEGMENT_LENGTH {
        return false;
    }
    rel <= CLOSE_ENOUGH
}

fn min_road_length(d1: Vec3, d2: Vec3, no_lanes: u8) -> f32 {
    MIN_SEGMENT_LENGTH
        .max(3.5 * no_lanes as f32 * 3.0 * d1.cross(d2).length() / d1.length() / d2.length())
}

fn is_curve_too_small(d1: Vec3, d2: Vec3, no_lanes: u8) -> bool {
    d2.length() < min_road_length(d1, d2, no_lanes)
}
