use glam::*;
use std::f32::consts::PI;
// pub fn quart<A: Into<Rad<f32>>>(angle: A, dir: Vector3<f32>) -> Quaternion<f32> {
//     let angle = angle.into().0 / 2.0;
//     let (sin, cos) = angle.sin_cos();
//     let dir = dir.normalize();
//     Quaternion::new(cos, dir.x * sin, dir.y * sin, dir.z * sin)
// }

// TODO write trait that encapsulate vec functions
pub fn proj(vector: Vec3, target: Vec3) -> Vec3 {
    target * (vector.dot(target) / target.length_squared())
}

pub fn anti_proj(vector: Vec3, target: Vec3) -> Vec3 {
    vector - proj(vector, target)
}

pub fn mirror(vector: Vec3, mirror_normal: Vec3) -> Vec3 {
    vector - proj(vector, mirror_normal) * 2.0
}

pub fn ndot(a: Vec3, b: Vec3) -> f32 {
    a.normalize().dot(b.normalize())
}

pub fn intersects_in_xz(d1: Vec3, d2: Vec3) -> bool {
    d2.x * d1.z - d2.z * d1.x != 0.0
}

pub fn intersection_in_xz(v1: Vec3, d1: Vec3, v2: Vec3, d2: Vec3) -> Vec3 {
    v2 + (d2 * ((v2.z - v1.z) * d1.x - (v2.x - v1.x) * d1.z) / (d2.x * d1.z - d2.z * d1.x))
}

// TODO write trait that encapsulate mat functions
pub fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
    let f = dir.normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);

    let x = Vec4::new(s.x, u.x, -f.x, 0.0);
    let y = Vec4::new(s.y, u.y, -f.y, 0.0);
    let z = Vec4::new(s.z, u.z, -f.z, 0.0);
    let w = Vec4::new(-eye.dot(s), -eye.dot(u), eye.dot(f), 1.0);

    Mat4::from_cols(x, y, z, w)
}

pub fn to_4x4(mat: Mat4) -> [[f32; 4]; 4] {
    [
        mat.x_axis.into(),
        mat.y_axis.into(),
        mat.z_axis.into(),
        mat.w_axis.into(),
    ]
}

// TODO write trait that encapsulate angle functions
pub fn rad_normalize(a: f32) -> f32 {
    a % (2.0 * PI)
}
