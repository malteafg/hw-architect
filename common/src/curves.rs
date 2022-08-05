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

//dir2.rescale(R.length)

