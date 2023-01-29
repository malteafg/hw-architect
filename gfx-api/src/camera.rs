//! probably remove glam dependency, or move code to tool
use glam::*;

#[derive(Debug)]
pub struct Camera {
    /// The point that the camera is tracking. For now this is a point on the terrain.
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist_to_target: f32,
}

impl Camera {
    pub fn new(target: Vec3, yaw: f32, pitch: f32, dist_to_target: f32) -> Self {
        Self {
            target,
            yaw,
            pitch,
            dist_to_target,
        }
    }

    /// Computes and returns the camera's current position
    pub fn calc_pos(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        self.target
            + (Vec3::new(-cos_yaw, 0.0, -sin_yaw) * cos_pitch + Vec3::new(0.0, sin_pitch, 0.0))
                * self.dist_to_target
    }

    /// Computes and returns the camera's current view matrix
    pub fn compute_view_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_to_rh(
            self.calc_pos(),
            Vec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }
}

