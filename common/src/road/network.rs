use cgmath::*;
use std::collections::HashMap;

pub type NodeId = u32;
pub type SegmentId = u32;

// pub struct NodeGenerator {
//     pos: Vector3<f32>,
//     dir: Vector3<f32>,
// }

#[derive(Clone)]
pub enum NodeDescriptor {
    EXISTING(NodeId),
    NEW(Node),
}

#[derive(PartialEq, Eq, Hash)]
pub enum RoadElementId {
    NODE(NodeId),
    SEGMENT(SegmentId),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RoadVertex {
    pub position: [f32; 3],
}

// in the future this should probably work in chunks
#[derive(Clone, Debug)]
pub struct RoadMesh {
    pub vertices: Vec<RoadVertex>,
    pub indices: Vec<u32>,
}

impl RoadMesh {
    fn new() -> Self {
        RoadMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// RoadGenerator should always generate the vec in direction of the cars, and there should be one more node than segment
#[derive(Clone)]
pub struct RoadGenerator {
    start_node: NodeDescriptor,
    pub start_pos: Vector3<f32>,
    segments: Vec<(Segment, NodeDescriptor, RoadMesh)>,
}

impl RoadGenerator {
    pub fn new(
        start_node: NodeDescriptor,
        start_pos: Vector3<f32>,
        end_node: NodeDescriptor,
        mesh: RoadMesh,
    ) -> Self {
        RoadGenerator {
            start_node,
            start_pos,
            segments: vec![(Segment { curve_type: 1 }, end_node, mesh)],
        }
    }

    pub fn update(&mut self, end_node: NodeDescriptor, mesh: RoadMesh) {
        self.segments = vec![(Segment { curve_type: 1 }, end_node, mesh)];
    }
}

type LeadingPair = (NodeId, SegmentId);

#[derive(Clone)]
pub struct Node {
    pos: Vector3<f32>,
    dir: Vector3<f32>,
}

impl Node {
    pub fn new(pos: Vector3<f32>, dir: Vector3<f32>) -> Self {
        Node { pos, dir }
    }
}

#[derive(Clone)]
pub struct Segment {
    curve_type: u32,
}

pub struct RoadGraph {
    node_map: HashMap<NodeId, Node>,
    segment_map: HashMap<SegmentId, Segment>,
    forward_refs: HashMap<NodeId, Vec<LeadingPair>>,
    backward_refs: HashMap<NodeId, Vec<LeadingPair>>,
    node_id_count: u32,
    segment_id_count: u32,
    road_meshes: HashMap<RoadElementId, RoadMesh>,
}

impl RoadGraph {
    pub fn new() -> Self {
        let node_map = HashMap::new();
        let segment_map = HashMap::new();
        let forward_refs = HashMap::new();
        let backward_refs = HashMap::new();
        let node_id_count = 0;
        let segment_id_count = 0;
        let road_meshes = HashMap::new();

        Self {
            node_map,
            segment_map,
            forward_refs,
            backward_refs,
            node_id_count,
            segment_id_count,
            road_meshes,
        }
    }

    pub fn add_road(&mut self, road: RoadGenerator) -> RoadMesh {
        use NodeDescriptor::*;
        use RoadElementId::*;

        // the order of ids follow the order of driving
        let segment_ids = vec![self.generate_segment_id(); road.segments.len()];
        let mut node_ids = Vec::new();

        let mut nodes = vec![road.start_node];
        road.segments
            .into_iter()
            .enumerate()
            .for_each(|(i, (segment, node, mesh))| {
                self.segment_map.insert(segment_ids[i], segment);
                self.road_meshes.insert(SEGMENT(segment_ids[i]), mesh);
                nodes.push(node);
            });

        nodes.into_iter().for_each(|node| match node {
            EXISTING(node_id) => node_ids.push(node_id),
            NEW(node) => {
                let node_id = self.generate_node_id();
                node_ids.push(node_id);
                self.node_map.insert(node_id, node);
                self.forward_refs.insert(node_id, Vec::new());
                self.backward_refs.insert(node_id, Vec::new());
            }
        });

        for i in 0..(node_ids.len() - 1) {
            self.forward_refs
                .get_mut(&(i as u32))
                .unwrap()
                .push((node_ids[i + 1], segment_ids[i]));
            self.backward_refs
                .get_mut(&(i as u32 + 1))
                .unwrap()
                .push((node_ids[i], segment_ids[i]));
        }

        // recompute meshes for affected nodes
        self.combine_road_meshes()
    }

    pub fn remove_road(&self, segment: SegmentId) {
        // remove segment and update affected nodes
    }

    // iterate over road_meshes and return vec of RoadVertex
    // in the future separate road_meshes into "chunks"
    fn combine_road_meshes(&self) -> RoadMesh {
        let mut indices_count = 0;
        let mut road_mesh: RoadMesh = RoadMesh::new();

        for (_, mesh) in self.road_meshes.iter() {
            road_mesh.vertices.append(&mut mesh.vertices.clone());
            road_mesh.indices.append(
                &mut mesh
                    .indices
                    .clone()
                    .into_iter()
                    .map(|i| i + indices_count)
                    .collect(),
            );
            indices_count += mesh.vertices.len() as u32;
        }

        road_mesh
    }

    pub fn select_road_element(&self, ray: Vector3<f32>) -> RoadElementId {
        // check nodes first with their radius
        // check segments based on the curve?
        RoadElementId::NODE(1)
    }

    fn generate_node_id(&mut self) -> NodeId {
        let node_id = self.node_id_count;
        self.node_id_count += 1;
        node_id
    }

    fn generate_segment_id(&mut self) -> SegmentId {
        let segment_id = self.segment_id_count;
        self.segment_id_count += 1;
        segment_id
    }
}
