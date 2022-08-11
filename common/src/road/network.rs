use super::generator;
use cgmath::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(u32);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SegmentId(u32);

#[derive(Debug, Clone, Copy)]
pub enum CurveType {
    Straight,
    Curved,
}

#[derive(Debug, Clone, Copy)]
pub struct RoadType {
    pub no_lanes: u32,
    pub curve_type: CurveType,
}

#[derive(PartialEq, Eq, Hash)]
pub enum RoadElementId {
    Node(NodeId),
    Segment(SegmentId),
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
    pub fn new() -> Self {
        RoadMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

type LeadingPair = (NodeId, SegmentId);

#[derive(Clone, Copy)]
pub struct Node {
    pub pos: Vector3<f32>,
    pub dir: Vector3<f32>,
}

impl Node {
    pub fn new(pos: Vector3<f32>, dir: Vector3<f32>) -> Self {
        Node { pos, dir }
    }
}

#[derive(Clone, Copy)]
pub struct Segment {
    pub curve_type: CurveType,
}

impl Segment {
    pub fn new(curve_type: CurveType) -> Self {
        Segment { curve_type }
    }
}

pub struct RoadGraph {
    node_map: HashMap<NodeId, Node>,
    segment_map: HashMap<SegmentId, Segment>,
    forward_refs: HashMap<SegmentId, Vec<LeadingPair>>,
    backward_refs: HashMap<SegmentId, Vec<LeadingPair>>,
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

    pub fn add_road(
        &mut self,
        road: generator::RoadGenerator,
        selected_node: Option<NodeId>,
        snapped_node: Option<NodeId>,
    ) -> RoadMesh {
        let segment_list = road.get_segments();
        let segment_ids = vec![self.generate_segment_id(); segment_list.len()];

        segment_list
            .into_iter()
            .enumerate()
            .for_each(|(i, (segment, mesh))| {
                let id = segment_ids[i];
                self.segment_map.insert(id, *segment);
                self.road_meshes.insert(RoadElementId::Segment(id), mesh.clone());
                self.forward_refs.insert(id, Vec::new());
                self.backward_refs.insert(id, Vec::new());
            });

        // TODO change behavior when selected and snapped node are set
        let node_list = road.get_nodes();
        let mut node_ids = vec![];
        node_list.iter().for_each(|(pos, dir)| {
            let node_id = self.generate_node_id();
            node_ids.push(node_id);
            self.node_map.insert(node_id, Node::new(*pos, *dir));
        });

        // TODO add connections to segments on opposite side of snapped and selected node
        for i in 0..(segment_ids.len() - 1) {
            self.forward_refs
                .get_mut(&(SegmentId(i as u32)))
                .unwrap()
                .push((node_ids[i + 1], segment_ids[i + 1]));
            self.backward_refs
                .get_mut(&(SegmentId(i as u32 + 1)))
                .unwrap()
                .push((node_ids[i - 1], segment_ids[i]));
        }

        // recompute meshes for affected nodes
        self.combine_road_meshes()
    }

    pub fn remove_road(&self, segment: SegmentId) {
        // remove segment and update affected nodes
    }

    pub fn get_node(&self, node: NodeId) -> Node {
        *self
            .node_map
            .get(&node)
            .expect("Node does not exist in node map")
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
        RoadElementId::Node(NodeId(1))
    }

    fn generate_node_id(&mut self) -> NodeId {
        let node_id = self.node_id_count;
        self.node_id_count += 1;
        NodeId(node_id)
    }

    fn generate_segment_id(&mut self) -> SegmentId {
        let segment_id = self.segment_id_count;
        self.segment_id_count += 1;
        SegmentId(segment_id)
    }
}
