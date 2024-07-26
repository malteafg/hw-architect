use gfx_api::colors::RGBAColor;
use utils::math::{Mat3Utils, Mat4Utils};

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

pub struct ColoredInstance {
    instance: Instance,
    color: RGBAColor,
}

impl ColoredInstance {
    pub fn new(position: Vec3, rotation: Quat, color: RGBAColor) -> Self {
        let instance = Instance::new(position, rotation);
        Self { instance, color }
    }

    pub fn to_raw(&self) -> ColoredInstanceRaw {
        let instance_raw = self.instance.to_raw();
        ColoredInstanceRaw {
            model: instance_raw.model,
            normal: Mat3::from_quat(self.instance.rotation).to_3x3(),
            color: self.color,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredInstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
    pub color: [f32; 4],
}
