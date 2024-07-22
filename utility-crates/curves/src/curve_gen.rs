use crate::GuidePoints;
use utils::{DirXZ, Loc, VecUtils};

use glam::Vec3;

const PRETTY_CLOSE: f32 = 0.97;
const CLOSE_ENOUGH: f32 = 0.95;
const COS_45: f32 = std::f32::consts::FRAC_1_SQRT_2;

const MIN_SEGMENT_LENGTH: f32 = 10.0;
const MAX_CIRCLE_SIZE: f32 = 400000.0;

// Notable functions:
// - free_three_quarter_circle_curve
// - snap_three_quarter_circle_curve
// - double_snap_curve_case
// - match_double_snap_curve_case
// - guide_points_and_direction
//
// for double snap call "double_snap_curve_case" to get the enum
// then call "match_double_snap_curve_case" with the enum
// to get the guidepoints
//
// for both three quarter circle and double snap, call
// "guide_points_and_direction" with the guidepoints
// to get a vec of tuples with both guidepoints and the direction
// for the new nodes
//
// snap_three_quarter_circle_curve makes circular curves but snaps to
// 90 degree intervals of road curvature

pub fn guide_points_and_direction(
    guide_points: Vec<GuidePoints>,
) -> (Vec<(GuidePoints, Vec3)>, Vec3) {
    let mut result: Vec<(GuidePoints, Vec3)> = Vec::new();
    for curve in guide_points.iter() {
        result.push((
            GuidePoints::from_vec(curve.to_vec()),
            (curve[curve.len() - 1] - curve[curve.len() - 2]).normalize(),
        ));
    }

    (
        result,
        (guide_points[0][1] - guide_points[0][0]).normalize(),
    )
}

fn snap_circle_projection(pos1: Vec3, dir1: DirXZ, pos2: Vec3, line_angle: f32) -> Vec3 {
    let dir1: Vec3 = dir1.into();
    let diff = pos2 - pos1;
    let tau = std::f32::consts::PI * 2.0;
    let no_lines = tau / line_angle;
    let a = diff.angle_between(dir1) / tau;
    let angle = (a * no_lines).round().min((no_lines * 3.0 / 8.0).floor()) * tau / no_lines;
    let (sin, cos) = angle.sin_cos();

    let line = dir1 * cos + dir1.right_hand() * diff.side(dir1) * sin;

    pos1 + line.normalize() * diff.length()
}

/// Best approximation of circular curve when constrained by directions at both points
pub fn circle_curve_fudged(pos1: Vec3, dir1: DirXZ, pos2: Vec3, dir2: DirXZ) -> GuidePoints {
    let dir1: Vec3 = dir1.into();
    let dir2: Vec3 = dir2.into();
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    GuidePoints::from_vec(vec![
        pos1,
        pos1 + r,
        pos2 + dir2.normalize() * r.length(),
        pos2,
    ])
}

#[derive(Debug, Clone)]
pub enum DoubleSnapCurveCase {
    SingleCircle,
    DoubleCircle,
    Ellipse,
}

#[derive(Debug, Clone)]
pub enum DoubleSnapError {
    TooSmall,
    TooBig,
    SegmentAngle,
    CurveAngle,
}

pub fn double_snap_curve_case(
    pos1: Vec3,
    dir1: Vec3,
    pos2: Vec3,
    dir2: Vec3,
    no_lanes: u8,
) -> Result<DoubleSnapCurveCase, DoubleSnapError> {
    use DoubleSnapCurveCase::*;
    use DoubleSnapError::*;

    let dir2 = -dir2;
    let diff = pos2 - pos1;

    if dir1.mirror(diff).ndot(dir2) > PRETTY_CLOSE
        && (-diff).dot(dir2) >= PRETTY_CLOSE - 1.0
        && diff.dot(dir1) >= PRETTY_CLOSE - 1.0
    {
        Ok(SingleCircle)
    } else {
        let ndir1 = dir1.normalize();
        let ndir2 = dir2.normalize();
        let t = s_curve_segment_length(pos1, ndir1, pos2, dir2);
        let center = (pos1 + pos2 + ndir1 * t + ndir2 * t) / 2.0;

        if (center - pos1).length_squared() > MAX_CIRCLE_SIZE {
            return Err(TooBig);
        }
        if is_curve_too_small(dir1, center - pos1, no_lanes)
            || is_curve_too_small(dir2, center - pos2, no_lanes)
        {
            return Err(TooSmall);
        }
        if dir1.dot(center - pos1) <= 0.0 || dir2.dot(center - pos2) <= 0.0 {
            return Err(SegmentAngle);
        }
        if (pos2 - pos1).dot(center - pos1) <= 0.0 || (pos1 - pos2).dot(center - pos2) <= 0.0 {
            return Err(CurveAngle);
        }
        if is_elliptical(pos1, dir1, pos2, dir2) {
            Ok(Ellipse)
        } else {
            Ok(DoubleCircle)
        }
    }
}

pub fn match_double_snap_curve_case(
    pos1: Vec3,
    dir1: DirXZ,
    pos2: Vec3,
    dir2: DirXZ,
    case: DoubleSnapCurveCase,
) -> Vec<GuidePoints> {
    let dir2 = -dir2;
    match case {
        DoubleSnapCurveCase::SingleCircle => vec![circle_curve_fudged(pos1, dir1, pos2, dir2)],
        DoubleSnapCurveCase::Ellipse => {
            vec![simple_curve_points(pos1, dir1, pos2, dir2).expect("Simple curve fuck up!")]
        }
        DoubleSnapCurveCase::DoubleCircle => double_curve(pos1, dir1, pos2, dir2),
    }
}

fn double_curve(pos1: Vec3, dir1: DirXZ, pos2: Vec3, dir2: DirXZ) -> Vec<GuidePoints> {
    let mut points = Vec::new();
    let t = s_curve_segment_length(pos1, dir1.into(), pos2, dir2.into());
    let center = (pos1 + pos2 + dir1 * t + dir2 * t) / 2.0;

    points.push(circle_curve(pos1, dir1, center));
    let mut second_half = circle_curve(pos2, dir2, center);
    second_half.reverse();
    points.push(second_half);

    points
}

fn simple_curve_points(
    pos1: Vec3,
    dir1: DirXZ,
    pos2: Vec3,
    dir2: DirXZ,
) -> anyhow::Result<GuidePoints> {
    if Vec3::from(dir1).intersects_in_xz(Vec3::from(dir2)) {
        Ok(GuidePoints::from_vec(vec![
            pos1,
            pos1.intersection_in_xz(Vec3::from(dir1), pos2, Vec3::from(dir2)),
            pos2,
        ]))
    } else {
        Err(anyhow::anyhow!("No intersection in simple curve"))
    }
}

fn s_curve_segment_length(v1: Vec3, r1: Vec3, v2: Vec3, r2: Vec3) -> f32 {
    let v = v2 - v1;
    let r = r2 - r1;
    if r.length_squared() == 4.0 {
        return 0.0;
    }
    let k = v.dot(r) / (4.0 - r.length_squared());
    k + (v.length_squared() / (4.0 - r.length_squared()) + k * k).sqrt()
}

fn min_road_length(d1: Vec3, d2: Vec3, no_lanes: u8) -> f32 {
    // TODO SHOULD DEPEND ON LANEWIDTH instead of 3.5
    MIN_SEGMENT_LENGTH
        .max(3.5 * no_lanes as f32 * 3.0 * d1.cross(d2).length() / d1.length() / d2.length())
}

fn is_curve_too_small(d1: Vec3, d2: Vec3, no_lanes: u8) -> bool {
    d2.length() < min_road_length(d1, d2, no_lanes)
}

fn is_elliptical(pos1: Vec3, dir1: Vec3, pos2: Vec3, dir2: Vec3) -> bool {
    let delta_pos = pos2 - pos1;
    if dir1.dot(dir2) > 0.0 {
        return false;
    }
    if (-delta_pos).ndot(dir2) < PRETTY_CLOSE - 1.0 || delta_pos.ndot(dir1) < PRETTY_CLOSE - 1.0 {
        return false;
    }
    if dir1.anti_proj(delta_pos).dot(dir2.anti_proj(delta_pos)) < 0.0 {
        return false;
    }
    if !dir1.intersects_in_xz(dir2) {
        return false;
    }
    let intersection = pos1.intersection_in_xz(dir1, pos2, dir2);
    let f1 = (intersection - pos1).length();
    let f2 = (intersection - pos2).length();
    let rel = f1.min(f2) / f1.max(f2);
    if f1 * rel < MIN_SEGMENT_LENGTH || f2 * rel < MIN_SEGMENT_LENGTH {
        return false;
    }
    rel <= CLOSE_ENOUGH
}

pub fn _spiral_curve(pos1: Vec3, dir1: Vec3, pos2: Vec3, radius: Vec3) -> GuidePoints {
    let diff = pos2 - pos1;
    let dir = dir1.normalize();

    let d = ((radius - 4.0 / 3.0 * diff).cross(dir)).length();
    let a = d * d - radius.length_squared();
    let b = 2.0 * (diff.dot(dir) * radius.length_squared() - radius.cross(diff).length() * d);
    let c = -radius.dot(diff).powi(2);

    let s = -(b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a);

    return GuidePoints::from_vec(vec![pos1, pos1 + dir * s, pos2]);
}
