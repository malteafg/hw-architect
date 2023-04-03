use gfx_api::RawCameraData;
use utils::Mat4Utils;

use glam::*;
use wgpu::util::DeviceExt;

use std::rc::Rc;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    window_width: f32,
    window_height: f32,
}

impl Projection {
    fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
            window_width: width as f32,
            window_height: height as f32,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

// Represents a cameras position and projection view matrix in raw form. It cannot be computed
// without the projection from the gpu side
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraView {
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Default for CameraView {
    fn default() -> Self {
        Self {
            view_pos: [0.0; 4],
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl CameraView {
    pub fn new(view_pos: [f32; 4], view_proj: [[f32; 4]; 4]) -> Self {
        Self {
            view_pos,
            view_proj,
        }
    }
}

/// Computes and returns the camera's current view matrix
fn compute_view_matrix(camera: RawCameraData) -> Mat4 {
    let (sin_pitch, cos_pitch) = camera.pitch.sin_cos();
    let (sin_yaw, cos_yaw) = camera.yaw.sin_cos();

    Mat4::look_to_rh(
        Vec3::from_array(camera.pos),
        Vec3::new(cos_pitch * cos_yaw, -sin_pitch, cos_pitch * sin_yaw).normalize(),
        Vec3::Y,
    )
}

pub struct Camera {
    projection: Projection,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: Rc<wgpu::BindGroup>,
}

impl Camera {
    pub fn new(
        device: &wgpu::Device,
        window_width: u32,
        window_height: u32,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let projection = Projection::new(
            window_width,
            window_height,
            45.0f32.to_radians(),
            5.0,
            2000.0,
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraView::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = Rc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        }));

        Self {
            projection,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn get_bind_group(&self) -> &Rc<wgpu::BindGroup> {
        &self.camera_bind_group
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.projection.resize(width, height);
    }

    pub fn update_camera(&mut self, camera: RawCameraData, queue: &wgpu::Queue) {
        let view_pos = Vec3::from_array(camera.pos).extend(1.0).into();
        let view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.projection.calc_matrix() * compute_view_matrix(camera))
                .to_4x4();
        let camera_view = CameraView::new(view_pos, view_proj);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_view]));
    }

    pub fn compute_ray(&self, mouse_pos: [f32; 2], camera: RawCameraData) -> [f32; 3] {
        let screen_vec = Vec4::new(
            2.0 * mouse_pos[0] as f32 / self.projection.window_width - 1.0,
            1.0 - 2.0 * mouse_pos[1] as f32 / self.projection.window_height,
            1.0,
            1.0,
        );
        let eye_vec = self.projection.calc_matrix().inverse() * screen_vec;
        let full_vec =
            compute_view_matrix(camera).inverse() * Vec4::new(eye_vec.x, eye_vec.y, -1.0, 0.0);
        let processed_vec = Vec3::new(full_vec.x, full_vec.y, full_vec.z).normalize();

        processed_vec.into()
    }
}
