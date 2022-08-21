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

