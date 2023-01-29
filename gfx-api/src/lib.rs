//! This crate defines the api for the graphics engine that hw-architect uses. The only interaction
//! that other crates are allowed to have to a graphics engine must go through this api, to keep
//! things modular.

pub trait Gfx {
    // render should contain error handling as well
    // fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    // depends on winit
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn update(
        &mut self,
        gfx_data: &mut GfxData,
        dt: instant::Duration,
        camera_view: CameraView,
    );

    fn add_instance(&mut self, position: glam::Vec3);

    fn remove_instance(&mut self);
}

// Legacy code from gfx_bridge
use glam::{Mat4, Vec4};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

#[derive(Default)]
pub struct GfxData {
    pub road_mesh: Option<RoadMesh>,
    pub road_tool_mesh: Option<RoadMesh>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RoadVertex {
    pub position: [f32; 3],
}

// in the future this should probably work in chunks
#[derive(Clone, Debug, Default)]
pub struct RoadMesh {
    pub vertices: Vec<RoadVertex>,
    pub indices: Vec<u32>,
    pub lane_vertices: Vec<RoadVertex>,
    pub lane_indices: Vec<u32>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraView {
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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

