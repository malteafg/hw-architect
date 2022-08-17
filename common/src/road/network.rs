use super::{generator, LANE_WIDTH};
use glam::*;
use std::collections::HashMap;

pub const MAX_LANES: usize = 6;

#[derive(Debug, Default, Clone, Copy)]
pub enum CurveType {
    #[default]
    Straight,
    Curved,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RoadType {
    pub no_lanes: u8,
    pub curve_type: CurveType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapConfig {
    pub node_id: NodeId,
    // TODO update to actually match lanes
    pub lane_config: (Option<SegmentId>, Option<SegmentId>),
    // Reverse means that outgoing lanes exist, and incoming does not
    pub reverse: bool,
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

#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub pos: Vec3,
    pub dir: Vec3,
    pub no_lanes: u8,
    // pub lane_map: [(SegmentId, SegmentId); MAX_LANES],
    pub lane_map: (Option<SegmentId>, Option<SegmentId>),
}

impl Node {
    pub fn new(
        pos: Vec3,
        dir: Vec3,
        no_lanes: u8,
        lane_map: (Option<SegmentId>, Option<SegmentId>),
    ) -> Self {
        Node {
            pos,
            dir,
            no_lanes,
            // lane_map: [(SegmentId(0), SegmentId(0)); MAX_LANES],
            lane_map,
        }
    }

    // TODO compute and return possible snap configs for lanes
    pub fn is_snappable(self) -> bool {
        self.lane_map.0.is_none() || self.lane_map.1.is_none()
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

        // id 0 is reserved as a none option (see lane_map in node)
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
        selected_node: Option<SnapConfig>,
        snapped_node: Option<SnapConfig>,
    ) -> (RoadMesh, SnapConfig) {
        let road_type = road.get_road_type();

        let segment_list = road.get_segments();
        let node_list = road.get_nodes();

        let mut nodes = vec![];
        if road.is_reverse() {
            nodes.push(snapped_node);
            for _ in 0..node_list.len() - 2 {
                nodes.push(None);
            }
            nodes.push(selected_node);
        } else {
            nodes.push(selected_node);
            for _ in 0..node_list.len() - 2 {
                nodes.push(None);
            }
            nodes.push(snapped_node);
        }

        let segment_ids: Vec<SegmentId> = segment_list
            .iter()
            .map(|_| self.generate_segment_id())
            .collect();

        segment_list
            .iter()
            .enumerate()
            .for_each(|(i, (segment, mesh))| {
                let id = segment_ids[i];
                self.segment_map.insert(id, *segment);
                self.road_meshes
                    .insert(RoadElementId::Segment(id), mesh.clone());
                self.forward_refs.insert(id, Vec::new());
                self.backward_refs.insert(id, Vec::new());
            });

        let mut node_ids = vec![];
        nodes.iter().enumerate().for_each(|(i, node)| {
            let node_id = match node {
                Some(node) => {
                    let lane_map = match node.lane_config {
                        (Some(id), None) => (Some(id), Some(segment_ids[0])),
                        (None, Some(id)) => (Some(segment_ids[segment_ids.len() - 1]), Some(id)),
                        _ => panic!("lane config broke"),
                    };
                    self.node_map.get_mut(&node.node_id).unwrap().lane_map = lane_map;
                    node.node_id
                }
                None => {
                    let node_id = self.generate_node_id();
                    let (pos, dir) = node_list[i];
                    self.node_map.insert(
                        node_id,
                        Node::new(
                            pos,
                            dir,
                            road_type.no_lanes,
                            // TODO hacky solution
                            (
                                segment_ids.get(((i as i32 - 1) % 10) as usize).copied(),
                                segment_ids.get(i).copied(),
                            ),
                        ),
                    );
                    node_id
                }
            };
            node_ids.push(node_id);
        });

        // dbg!(self.forward_refs.clone());

        // TODO add connections to segments on opposite side of snapped and selected node
        // for i in 0..(segment_ids.len() - 1) {
        //     self.forward_refs
        //         .get_mut(&segment_ids[i])
        //         .unwrap()
        //         .push((node_ids[i + 1], segment_ids[i + 1]));
        //     self.backward_refs
        //         .get_mut(&segment_ids[i + 1])
        //         .unwrap()
        //         .push((node_ids[i - 1], segment_ids[i]));
        // }

        // recompute meshes for affected nodes
        let (new_id, new_lane) = if road.is_reverse() {
            let id = node_ids[0];
            (id, self.get_node(id).lane_map)
        } else {
            let id = node_ids[node_ids.len() - 1];
            (id, self.get_node(id).lane_map)
        };
        let new_sel = SnapConfig {
            node_id: new_id,
            lane_config: new_lane,
            reverse: road.is_reverse(),
        };
        (self.combine_road_meshes(), new_sel)
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

    pub fn select_road_element(&self) -> RoadElementId {
        // check nodes first with their radius
        // check segments based on the curve?
        RoadElementId::Node(NodeId(1))
    }

    // TODO update to work with lanes
    pub fn get_node_from_pos(&self, pos: Vec3) -> Option<SnapConfig> {
        for (i, n) in self.node_map.iter() {
            if !n.is_snappable() {
                continue;
            }
            if (n.pos - pos).length() < n.no_lanes as f32 * LANE_WIDTH {
                return Some(SnapConfig {
                    node_id: *i,
                    lane_config: n.lane_map,
                    reverse: n.lane_map.0.is_none(),
                });
            }
        }
        None
    }

    pub fn get_node_debug(&self, pos: Vec3) -> Option<SnapConfig> {
        for (i, n) in self.node_map.iter() {
            if (n.pos - pos).length() < n.no_lanes as f32 * LANE_WIDTH {
                return Some(SnapConfig {
                    node_id: *i,
                    lane_config: n.lane_map,
                    reverse: n.lane_map.0.is_none(),
                });
            }
        }
        None
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(u32);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SegmentId(u32);

#[derive(PartialEq, Eq, Hash)]
pub enum RoadElementId {
    Node(NodeId),
    Segment(SegmentId),
}
