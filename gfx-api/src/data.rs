pub struct RawCameraData {
    pub pos: [f32; 3],
    pub pitch: f32,
    pub yaw: f32,
}

// in the future this should probably work in chunks
#[derive(Clone, Debug, Default)]
pub struct RoadMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub lane_vertices: Vec<[f32; 3]>,
    pub lane_indices: Vec<u32>,
}

impl RoadMesh {
    pub fn empty() -> RoadMesh {
        RoadMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            lane_vertices: Vec::new(),
            lane_indices: Vec::new(),
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.vertices.len(), self.lane_vertices.len())
    }
}
