use utils::{Mat3Utils, Mat4Utils};

use glam::{Mat3, Mat4, Quat, Vec3};

pub struct Instance {
    position: Vec3,
    rotation: Quat,
}

impl Instance {
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        Self { position, rotation }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let model = Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation);
        InstanceRaw {
            model: model.to_4x4(),
            normal: Mat3::from_quat(self.rotation).to_3x3(),
        }
    }

    pub fn to_raw_with_scale(&self, scale: f32) -> InstanceRaw {
        let model = Mat4::from_scale_rotation_translation(
            Vec3::new(scale, scale, scale),
            self.rotation,
            self.position,
        );
        InstanceRaw {
            model: model.to_4x4(),
            normal: Mat3::from_quat(self.rotation).to_3x3(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}
