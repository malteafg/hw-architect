//! This crate defines the api for the graphics engine that hw-architect uses. The only interaction
//! that other crates are allowed to have to a graphics engine must go through this api, to keep
//! things modular.
//! Dependency on wgpu in Gfx.render, on winit in Gfx.render and on glam in OPENGL_TO_WGPU_MATRIX
//! should be removed

mod camera;
pub use camera::Camera;
use std::collections::HashMap;
use utils::id::SegmentId;

/// This trait defines how a gpu engine should be interacted with
pub trait Gfx {
    // render should contain error handling as well
    // fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    /// This method should be changed to a generic way of handling errors, such that this crate
    /// does not depend on wgpu
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;

    /// Dependency on winit should be removed
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);

    fn update(&mut self, dt: instant::Duration);

    fn add_instance(&mut self, position: glam::Vec3);

    fn remove_instance(&mut self);

    // some function that loads gfx data from a file on startup. This should be coded in such a way
    // that Gfx can be used without being dependent on GfxData
    // fn load_gfx();
}

/// This trait defines how tool is allowed to interact with the data associated with roads,
/// that is needed by the gpu.
pub trait GfxRoadData {
    /// Adds a set of road meshes to the renderer such that they are now rendered. Fewer calls
    /// to this is strongly preferred, for performance reasons.
    fn add_road_meshes(&mut self, meshes: HashMap<SegmentId, RoadMesh>);

    /// Removes a set of road meshes given by their ids, such that their are no longer rendered
    /// and stored by the renderer. Fewer calls to this is strongly preferred, for performance
    /// reasons.
    fn remove_road_meshes(&mut self, ids: Vec<SegmentId>);

    /// Sets the mesh for the road tool. None is intended to signal that no mesh should be
    /// rendered.
    fn set_road_tool_mesh(&mut self, road_mesh: Option<RoadMesh>);

    // Used to mark a road segment. Pass {`None`} to signal that no segment shall be marked.
    // fn mark_road_segment(&mut self, segment: Option<SegmentId>);
}

/// This trait defines how tool is allowed to interact with the data associated with the camera,
/// that is needed by the gpu.
pub trait GfxCameraData {
    /// Updates the camera and computes new view and projection matrices.
    fn update_camera(&mut self, camera: &Camera);

    /// Given a the position of the mouse on the screen and the camera, the ray is computed and
    /// returned.
    fn compute_ray(&self, mouse_pos: glam::Vec2, camera: &Camera) -> utils::Ray;
}

// Legacy code from gfx_bridge
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

// Represents a cameras position and projection view matrix in raw form. It cannot be computed
// without the projection from the gpu side
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

