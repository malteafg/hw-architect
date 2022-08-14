use super::LANE_WIDTH;
use crate::math_utils::{VecUtils, Round};
use anyhow::Ok;
use glam::*;

const PRETTY_CLOSE: f32 = 0.97;
const CLOSE_ENOUGH: f32 = 0.95;
const COS_45: f32 = std::f32::consts::FRAC_1_SQRT_2;

const COS_THREE_SIXTEENTH: f32 = 0.3826834322;
const COS_SIXTEENTH: f32 = 0.9238795324;

const MIN_SEGMENT_LENGTH: f32 = 10.0;

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

pub fn guide_points_and_direction(guide_points: Vec<Vec<Vec3>>) -> Vec<(Vec<Vec3>, Vec3)> {
    let mut result: Vec<(Vec<Vec3>, Vec3)> = Vec::new();
    for curve in guide_points.iter() {
        result.push((
            curve.to_vec(),
            curve[curve.len() - 1] - curve[curve.len() - 2],
        ));
    }

    result
}

pub fn free_three_quarter_circle_curve(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec<Vec<Vec3>> {
    three_quarter_circle_curve(pos1, dir1, three_quarter_projection(pos1, dir1, pos2))
}

pub fn snap_three_quarter_circle_curve(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec<Vec<Vec3>> {
    three_quarter_circle_curve(pos1, dir1, snap_circle_projection(pos1, dir1, pos2))
}

fn three_quarter_circle_curve(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec<Vec<Vec3>> {
    if (pos2 - pos1).dot(dir1) >= 0.0 {
        vec![circle_curve(pos1, dir1, pos2)]
    } else {
        let mid_point = curve_mid_point(pos1, dir1, pos2);
        vec![
            circle_curve(pos1, dir1, mid_point),
            circle_curve(mid_point, pos2 - pos1, pos2),
        ]
    }
}

fn three_quarter_projection(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec3 {
    let diff = pos2 - pos1;
    let proj_length = diff.dot(dir1) / dir1.length();
    if proj_length >= -COS_45 * diff.length() {
        pos2
    } else {
        pos1 + diff.proj(dir1) + diff.anti_proj(dir1).normalize() * proj_length.abs()
    }
}

fn snap_circle_projection(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec3 { //, no_lines: u32 
    let diff = pos2 - pos1;
    let deg = 22.5;
    let no_lines = 360.0 / deg as f32;
    let tau = std::f32::consts::PI * 2.0;
    let a = diff.angle_between(dir1) / tau;
    dbg!(a);
    let angle = (a * no_lines).round().min((no_lines * 3.0 / 8.0).floor()) * tau / no_lines;
    let (sin, cos) = angle.sin_cos();

    let line = dir1 * cos + dir1.right_hand() * diff.side(dir1) * sin;

    pos1 + line.normalize() * diff.length()
}

fn old_snap_circle_projection(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec3 {
    let diff = pos2 - pos1;
    let proj_length = diff.dot(dir1) / dir1.length();
    let diff_length = diff.length();

    if proj_length >= COS_SIXTEENTH * diff_length {
        pos1 + diff.proj(dir1)
    } else if proj_length.abs() <= COS_THREE_SIXTEENTH * diff_length {
        pos1 - dir1.normalize() * proj_length.abs() * 0.002 + diff.anti_proj(dir1)
    } else {
        pos1 + diff.proj(dir1) + diff.anti_proj(dir1).normalize() * proj_length.abs()
    }
}

fn curve_mid_point(pos1: Vec3, dir: Vec3, pos2: Vec3) -> Vec3 {
    let diff = pos2 - pos1;
    let dir2 = dir.normalize() + diff.normalize();
    let result = pos1 + (dir2 * (diff.length_squared() / 2.0 / dir2.dot(diff)));
    result
}

/// The guidepoints for a curve as circular as posible with four guide points, up to half a circle
pub fn circle_curve(pos1: Vec3, dir1: Vec3, pos2: Vec3) -> Vec<Vec3> {
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    vec![
        pos1,
        pos1 + r,
        pos2 + r - diff * (2.0 * diff.dot(r) / diff.length_squared()),
        pos2,
    ]
}

/// Best aproximation of circular curve when constrained by directions at both points
pub fn circle_curve_fudged(pos1: Vec3, dir1: Vec3, pos2: Vec3, dir2: Vec3) -> Vec<Vec3> {
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    vec![pos1, pos1 + r, pos2 + dir2.normalize() * r.length(), pos2]
}

fn circle_scale(diff: Vec3, dir: Vec3) -> f32 {
    let dot = diff.normalize().dot(dir.normalize());
    if dot == 1.0 {
        // Makes it so that straight curves have intermidiary guidepoints and 1/3 and 2/3
        diff.length() / (3.0 * dir.length())
    } else {
        2.0 / 3.0 * diff.length() * (1.0 - dot) / (dir.length() * (1.0 - dot * dot))
    }
}

pub enum DoubleSnapCurveCase {
    SingleCircle,
    DoubleCircle,
    Elipse,
    ErrorTooSmall,
    ErrorSegmentAngle,
    ErrorCurveAngle,
    ErrorUnhandled,
}

pub fn double_snap_curve_case(
    pos1: Vec3,
    dir1: Vec3,
    pos2: Vec3,
    dir2: Vec3,
    no_lanes: u8,
) -> DoubleSnapCurveCase {
    let dir2 = -dir2;
    let diff = pos2 - pos1;

    if dir1.mirror(diff).ndot(dir2) > PRETTY_CLOSE
        && (-diff).dot(dir2) >= PRETTY_CLOSE - 1.0
        && diff.dot(dir1) >= PRETTY_CLOSE - 1.0
    {
        return DoubleSnapCurveCase::SingleCircle;
    } else {
        let ndir1 = dir1.normalize();
        let ndir2 = dir2.normalize();
        let t = s_curve_segment_length(pos1, ndir1, pos2, dir2);
        let center = (pos1 + pos2 + ndir1 * t + ndir2 * t) / 2.0;
        if is_curve_too_small(dir1, center - pos1, no_lanes)
            || is_curve_too_small(dir2, center - pos2, no_lanes)
        {
            return DoubleSnapCurveCase::ErrorTooSmall;
        }
        if dir1.dot(center - pos1) < 0.0 || dir2.dot(center - pos2) < 0.0 {
            return DoubleSnapCurveCase::ErrorSegmentAngle;
        }
        if (pos2 - pos1).dot(center - pos1) < 0.0 || (pos1 - pos2).dot(center - pos2) < 0.0 {
            return DoubleSnapCurveCase::ErrorCurveAngle;
        }
        if is_eliptical(pos1, dir1, pos2, dir2) {
            return DoubleSnapCurveCase::Elipse;
        } else {
            return DoubleSnapCurveCase::DoubleCircle;
        }
    }
}

pub fn match_double_snap_curve_case(
    pos1: Vec3,
    dir1: Vec3,
    pos2: Vec3,
    dir2: Vec3,
    case: DoubleSnapCurveCase,
) -> Vec<Vec<Vec3>> {
    let dir2 = -dir2;
    match case {
        DoubleSnapCurveCase::SingleCircle => vec![circle_curve_fudged(pos1, dir1, pos2, dir2)],
        DoubleSnapCurveCase::DoubleCircle => {
            vec![simple_curve_points(pos1, dir1, pos2, dir2).expect("Simple curve fuck up!")]
        }
        DoubleSnapCurveCase::Elipse => double_curve(pos1, dir1, pos2, dir2),
        _ => vec![],
    }
}

fn double_curve(pos1: Vec3, dir1: Vec3, pos2: Vec3, dir2: Vec3) -> Vec<Vec<Vec3>> {
    let mut points = Vec::new();
    let ndir1 = dir1.normalize();
    let ndir2 = dir2.normalize();
    let t = s_curve_segment_length(pos1, ndir1, pos2, dir2);
    let center = (pos1 + pos2 + ndir1 * t + ndir2 * t) / 2.0;

    points.push(circle_curve(pos1, dir1, center));
    let mut second_half = circle_curve(pos2, dir2, center);
    second_half.reverse();
    points.push(second_half);

    points
}

/// The complete double snap function before case-determination and guidepoint generateion was seperated
/// The guidenodes for a curve given both endpoints and both directions.
/// If two circular segments are required their guidepoints will be seperate entries in the outer Vec
pub fn double_snap_curve_debricated(
    pos1: Vec3,
    dir1: Vec3,
    pos2: Vec3,
    dir2: Vec3,
    no_lanes: u8,
) -> anyhow::Result<Vec<Vec<Vec3>>> {
    let mut points = Vec::new();
    let diff = pos2 - pos1;

    if dir1.mirror(diff).ndot(dir2) > PRETTY_CLOSE
        && (-diff).dot(dir2) >= PRETTY_CLOSE - 1.0
        && diff.dot(dir1) >= PRETTY_CLOSE - 1.0
    {
        points.push(circle_curve_fudged(pos1, dir1, pos2, dir2));
    } else {
        let ndir1 = dir1.normalize();
        let ndir2 = dir2.normalize();
        let t = s_curve_segment_length(pos1, ndir1, pos2, dir2);
        let center = (pos1 + pos2 + ndir1 * t + ndir2 * t) / 2.0;
        if is_curve_too_small(dir1, center - pos1, no_lanes)
            || is_curve_too_small(dir2, center - pos2, no_lanes)
        {
            return Err(anyhow::anyhow!("Curve too small"));
        }
        if dir1.dot(center - pos1) < 0.0 || dir2.dot(center - pos2) < 0.0 {
            return Err(anyhow::anyhow!("Unsupported angle"));
        }
        if (pos2 - pos1).dot(center - pos1) < 0.0 || (pos1 - pos2).dot(center - pos2) < 0.0 {
            return Err(anyhow::anyhow!("Another unsupported angle"));
        }
        if is_eliptical(pos1, dir1, pos2, dir2) {
            let spoints = simple_curve_points(pos1, dir1, pos2, dir2).expect("");
            points.push(spoints);
        } else {
            points.push(circle_curve(pos1, dir1, center));
            let mut second_half = circle_curve(pos2, dir2, center);
            second_half.reverse();
            points.push(second_half);
        }
    }

    if points.len() > 0 {
        Ok(points)
    } else {
        Err(anyhow::anyhow!("Unhandled edge case"))
    }
}

fn simple_curve_points(
    pos1: Vec3,
    dir1: Vec3,
    pos2: Vec3,
    dir2: Vec3,
) -> anyhow::Result<Vec<Vec3>> {
    if dir1.intersects_in_xz(dir2) {
        Ok(vec![pos1, pos1.intersection_in_xz(dir1, pos2, dir2), pos2])
    } else {
        Err(anyhow::anyhow!("No intersection in simple curve"))
    }
}

fn s_curve_segment_length(v1: Vec3, r1: Vec3, v2: Vec3, r2: Vec3) -> f32 {
    let v = v2 - v1;
    let r = r2 - r1;
    let k = v.dot(r) / (4.0 - r.length_squared());
    k + (v.length_squared() / (4.0 - r.length_squared()) + k * k).sqrt()
}

fn min_road_length(d1: Vec3, d2: Vec3, no_lanes: u8) -> f32 {
    MIN_SEGMENT_LENGTH
        .max(LANE_WIDTH * no_lanes as f32 * 3.0 * d1.cross(d2).length() / d1.length() / d2.length())
}

fn is_curve_too_small(d1: Vec3, d2: Vec3, no_lanes: u8) -> bool {
    d2.length() < min_road_length(d1, d2, no_lanes)
}

fn is_eliptical(pos1: Vec3, dir1: Vec3, pos2: Vec3, dir2: Vec3) -> bool {
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

pub fn calc_bezier_pos(guide_points: Vec<Vec3>, t: f32) -> Vec3 {
    let mut v = Vec3::new(0.0, 0.0, 0.0);
    let mut r = (1.0 - t).powi(guide_points.len() as i32 - 1);
    let mut l = 1.0;
    let mut i: i32 = 0;
    for p in guide_points.iter() {
        let f = l * r;
        v = v + *p * f;
        if t == 1.0 {
            if i == guide_points.len() as i32 - 2 {
                r = 1.0;
            } else {
                r = 0.0;
            }
        } else {
            r *= t / (1.0 - t);
        }
        l *= guide_points.len() as f32 / (1.0 + i as f32) - 1.0;
        i += 1;
    }
    v
}

pub fn calc_bezier_dir(guide_points: Vec<Vec3>, t: f32) -> Vec3 {
    let mut v = Vec3::new(0.0, 0.0, 0.0);
    let mut r = (1.0 - t).powi(guide_points.len() as i32 - 2);
    let mut l = 1.0;
    let mut i: i32 = 0;
    for p in 0..(guide_points.len() - 1) {
        v = v + (guide_points[p + 1] - guide_points[p]) * l * r;
        if t == 1.0 {
            if i == guide_points.len() as i32 - 3 {
                r = 1.0;
            } else {
                r = 0.0;
            }
        } else {
            r *= t / (1.0 - t);
        }
        l *= (guide_points.len() as f32 - 1.0) / (1.0 + i as f32) - 1.0;
        i += 1;
    }
    let result = v * guide_points.len() as f32;
    result
}
