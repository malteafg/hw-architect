//! This crate defines the api for the graphics engine that hw-architect uses. The only interaction
//! that other crates are allowed to have to a graphics engine must go through this api, to keep
//! things modular.
//! Dependency on wgpu in Gfx.render should be removed
mod data;
mod error;

pub use data::*;
pub use error::*;

use std::collections::HashMap;
use utils::id::SegmentId;

/// This trait defines all the behavior that a gpu backend must implement to render all of
/// hw-architect.
pub trait GfxSuper: Gfx + GfxWorldData + GfxCameraData {}
impl<T: Gfx + GfxWorldData + GfxCameraData> GfxSuper for T {}

/// This trait defines how a gpu backend should be interacted with
pub trait Gfx {
    /// This method should be changed to a generic way of handling errors, such that this crate
    /// does not depend on wgpu
    fn render(&mut self) -> Result<(), GfxFrameError>;

    /// Resizes the window. The unit of the parameters are in pixels.
    fn resize(&mut self, width: u32, height: u32);

    fn update(&mut self, dt: instant::Duration);
}

/// This trait defines all the data that a gpu backend must implement in order to render the world.
pub trait GfxWorldData: GfxRoadData + GfxTreeData {}
impl<T: GfxRoadData + GfxTreeData> GfxWorldData for T {}

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

    /// Used to mark a road segment. Pass {`None`} to signal that no segment shall be marked.
    fn mark_road_segments(&mut self, segments: Vec<SegmentId>);

    /// Sets the mesh for the road tool. None is intended to signal that no mesh should be
    /// rendered.
    fn set_road_tool_mesh(&mut self, road_mesh: Option<RoadMesh>);

    /// Renders the positions for nodes given by markers. Pass an empty list to stop rendering node
    /// markers.
    fn set_node_markers(&mut self, markers: Vec<[f32; 3]>);
}

/// This trait defines how tool is allowed to interact with the data associated with the camera,
/// that is needed by the gpu.
pub trait GfxCameraData {
    /// Updates the camera and computes new view and projection matrices.
    fn update_camera(&mut self, camera: RawCameraData);

    /// Given a the position of the mouse on the screen and the camera, the ray is computed and
    /// returned as the direction of the ray.
    fn compute_ray(&self, mouse_pos: [f32; 2], camera: RawCameraData) -> [f32; 3];
}

/// This trait defines how tool is allowed to interact with the data associated with trees.
pub trait GfxTreeData {
    /// Sets the trees that should be rendered by the gpu.
    fn set_trees(&mut self, pos_with_yrot: Vec<([f32; 3], f32)>);
}
