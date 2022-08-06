use cgmath::*;

pub fn circle_curve(pos1: Vector3<f32>, dir1: Vector3<f32>, pos2: Vector3<f32>) -> [Vector3<f32>; 4] {
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    [   pos1,
        pos1 + r,
        pos2 + r - diff * (2.0 * diff.dot(r) / diff.dot(diff)),
        pos2,]
}

pub fn circle_curve_fudged(pos1: Vector3<f32>, dir1: Vector3<f32>, pos2: Vector3<f32>, dir2: Vector3<f32>) -> [Vector3<f32>; 4] {
    let diff = pos2 - pos1;
    let r = dir1 * circle_scale(diff, dir1);

    [   pos1,
        pos1 + r,
        pos2 + dir2.normalize() * r.magnitude(),
        pos2,]
}

fn circle_scale(diff: Vector3<f32>, dir: Vector3<f32>) -> f32 {
    let dot = diff.normalize().dot(dir.normalize());
    2.0 / 3.0 * diff.magnitude() * (1.0 - dot) / (dir.magnitude() * (1.0 - dot * dot))
}

pub fn calc_bezier_point(guide_points: Vec<Vector3<f32>>, t: f32) -> Vector3<f32> {
    let mut v = Vector3::new(0.0,0.0,0.0);
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
    };
    v
}

pub fn calc_bezier_dir(guide_points: Vec<Vector3<f32>>, t: f32) -> Vector3<f32> {
    let mut v = Vector3::new(0.0,0.0,0.0);
    let mut r = (1.0 - t).powi(guide_points.len() as i32 - 2);
    let mut l = 1.0;
    let mut i: i32 = 0;
    for p in 0..(guide_points.len() - 2) {
        v = v + guide_points[p + 1] - guide_points[p] * l * r;
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
    v * guide_points.len() as f32
}

//dir2.rescale(R.length)

