use super::LANE_WIDTH;
use crate::math_utils::*;
use anyhow::Ok;
use cgmath::*;

const PRETTY_CLOSE: f32 = 0.97;
const CLOSE_ENOUGH: f32 = 0.95;
const COS_45: f32 = 0.7071067812; //aka sqrt(0.5)

const MIN_SEGMENT_LENGTH: f32 = 10.0;

pub fn three_quarter_circle_curve(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
) -> Vec<(Vec<Vector3<f32>>, Vector3<f32>)> {
    let projected_point = three_quarter_projection(pos1, dir1, pos2);
    if dot(pos2 - pos1, dir1) > 0.0 {
        vec![circle(pos1, dir1, projected_point)]
    } else {
        let modPoint = curve_mid_point(pos1, dir1, projected_point);
        vec![circle(pos1, dir1, modPoint), circle(modPoint, pos2 - pos1, projected_point)]
    }
}

fn three_quarter_projection(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
) -> Vector3<f32> {
    let diff = pos2 - pos1;
    let proj_length = dot(diff, dir1) / dir1.magnitude();
    if proj_length >= - COS_45 * diff.magnitude() {
        pos2
    } else {
        proj(diff, dir1) + anti_proj(diff, dir1).normalize() * proj_length
    }
}

fn curve_mid_point(pos1: Vector3<f32>, dir: Vector3<f32>, pos2: Vector3<f32>) -> Vector3<f32> {
    let diff = pos2 - pos1;
    let dir2 = dir.normalize() + diff.normalize();
    pos1 + (dir2 * (diff.magnitude2() / 2.0 / dot(dir2, diff)))
}

pub fn circle(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
) -> (Vec<Vector3<f32>>, Vector3<f32>) {
    let c_points = circle_curve(pos1, dir1, pos2);
    let dir = c_points[3] - c_points[2];
    (c_points, dir)
}

/// The guidepoints for a curve as circular as posible with four guide points, up to half a circle
pub fn circle_curve(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
) -> Vec<Vector3<f32>> {
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    vec![
        pos1,
        pos1 + r,
        pos2 + r - diff * (2.0 * diff.dot(r) / diff.dot(diff)),
        pos2,
    ]
}

/// Best aproximation of circular curve when constrained by directions at both points
pub fn circle_curve_fudged(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
    dir2: Vector3<f32>,
) -> Vec<Vector3<f32>> {
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    vec![
        pos1,
        pos1 + r,
        pos2 + dir2.normalize() * r.magnitude(),
        pos2,
    ]
}

fn circle_scale(diff: Vector3<f32>, dir: Vector3<f32>) -> f32 {
    let dot = diff.normalize().dot(dir.normalize());
    2.0 / 3.0 * diff.magnitude() * (1.0 - dot) / (dir.magnitude() * (1.0 - dot * dot))
}

/// The guidenodes for a curve given both endpoints and both directions.
/// If two circular segments are required their guidepoints will be seperate entries in the outer Vec
pub fn double_snap_curve(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
    dir2: Vector3<f32>,
    lane_count: i32,
) -> anyhow::Result<Vec<Vec<Vector3<f32>>>> {
    let mut points = Vec::new();
    let diff = pos2 - pos1;

    if ndot(mirror(dir1, diff), dir2) > PRETTY_CLOSE
        && (-diff).dot(dir2) >= PRETTY_CLOSE - 1.0
        && diff.dot(dir1) >= PRETTY_CLOSE - 1.0
    {
        points.push(circle_curve_fudged(pos1, dir1, pos2, dir2));
    } else {
        let ndir1 = dir1.normalize();
        let ndir2 = dir2.normalize();
        let t = s_curve_segment_length(pos1, ndir1, pos2, dir2);
        let center = (pos1 + pos2 + ndir1 * t + ndir2 * t) / 2.0;
        if is_curve_too_small(dir1, center - pos1, lane_count)
            || is_curve_too_small(dir2, center - pos2, lane_count)
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
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
    dir2: Vector3<f32>,
) -> anyhow::Result<Vec<Vector3<f32>>> {
    if intersects_in_xz(dir1, dir2) {
        Ok(vec![pos1, intersection_in_xz(pos1, dir1, pos2, dir2), pos2])
    } else {
        Err(anyhow::anyhow!("No intersection in simple curve"))
    }
}

fn s_curve_segment_length(
    v1: Vector3<f32>,
    r1: Vector3<f32>,
    v2: Vector3<f32>,
    r2: Vector3<f32>,
) -> f32 {
    let v = v2 - v1;
    let r = r2 - r1;
    let k = v.dot(r) / (4.0 - r.magnitude2());
    k + (v.magnitude2() / (4.0 - r.magnitude2()) + k * k).sqrt()
}

fn min_road_length(d1: Vector3<f32>, d2: Vector3<f32>, lane_count: i32) -> f32 {
    MIN_SEGMENT_LENGTH.max(
        LANE_WIDTH * lane_count as f32 * 3.0 * d1.cross(d2).magnitude()
            / d1.magnitude()
            / d2.magnitude(),
    )
}

fn is_curve_too_small(d1: Vector3<f32>, d2: Vector3<f32>, lane_count: i32) -> bool {
    d2.magnitude() < min_road_length(d1, d2, lane_count)
}

fn is_eliptical(
    pos1: Vector3<f32>,
    dir1: Vector3<f32>,
    pos2: Vector3<f32>,
    dir2: Vector3<f32>,
) -> bool {
    let delta_pos = pos2 - pos1;
    if dir1.dot(dir2) > 0.0 {
        return false;
    }
    if ndot(-delta_pos, dir2) < PRETTY_CLOSE - 1.0 || ndot(delta_pos, dir1) < PRETTY_CLOSE - 1.0 {
        return false;
    }
    if anti_proj(dir1, delta_pos).dot(anti_proj(dir2, delta_pos)) < 0.0 {
        return false;
    }
    if !intersects_in_xz(dir1, dir2) {
        return false;
    }
    let intersection = intersection_in_xz(pos1, dir1, pos2, dir2);
    let f1 = (intersection - pos1).magnitude();
    let f2 = (intersection - pos2).magnitude();
    let rel = f1.min(f2) / f1.max(f2);
    if f1 * rel < MIN_SEGMENT_LENGTH || f2 * rel < MIN_SEGMENT_LENGTH {
        return false;
    }
    rel <= CLOSE_ENOUGH
}

pub fn calc_bezier_pos(guide_points: Vec<Vector3<f32>>, t: f32) -> Vector3<f32> {
    let mut v = Vector3::new(0.0, 0.0, 0.0);
    let mut r = (1.0 - t).powi(guide_points.len() as i32 - 1);
    let mut l = 1.0;
    let mut i: i32 = 0;
    for p in guide_points.iter() {
        let f = l * r;
        v = v + p * f;
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

pub fn calc_bezier_dir(guide_points: Vec<Vector3<f32>>, t: f32) -> Vector3<f32> {
    let mut v = Vector3::new(0.0, 0.0, 0.0);
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
