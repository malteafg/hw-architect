use glam::{Mat3, Mat4, Vec3, Vec4};

/// Defines utility functions intended for 4x4 matrices
pub trait Mat4Utils {
    fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Self;
    fn to_4x4(self) -> [[f32; 4]; 4];
}

impl Mat4Utils for Mat4 {
    fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Self {
        let f = dir.normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        let x = Vec4::new(s.x, u.x, -f.x, 0.0);
        let y = Vec4::new(s.y, u.y, -f.y, 0.0);
        let z = Vec4::new(s.z, u.z, -f.z, 0.0);
        let w = Vec4::new(-eye.dot(s), -eye.dot(u), eye.dot(f), 1.0);

        Self::from_cols(x, y, z, w)
    }

    fn to_4x4(self) -> [[f32; 4]; 4] {
        [
            self.x_axis.into(),
            self.y_axis.into(),
            self.z_axis.into(),
            self.w_axis.into(),
        ]
    }
}

/// Defines utility functions intended for 3x3 matrices
pub trait Mat3Utils {
    fn to_3x3(self) -> [[f32; 3]; 3];
}

impl Mat3Utils for Mat3 {
    fn to_3x3(self) -> [[f32; 3]; 3] {
        [self.x_axis.into(), self.y_axis.into(), self.z_axis.into()]
    }
}
