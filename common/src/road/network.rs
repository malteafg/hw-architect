use crate::math_utils::VecUtils;

use super::{curves, generator, LANE_WIDTH};
use glam::*;
use std::collections::{HashMap, VecDeque};

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

impl RoadMesh {
    pub fn new() -> Self {
        RoadMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

// #[derive(Debug, Clone, PartialEq)]
pub type SnapRange = Vec<i8>;

trait SnapRangeTrait {
    fn create(start: i8, end: i8) -> Self;
    fn reduce_size(&self, end: u8) -> Self;
}

impl SnapRangeTrait for SnapRange {
    fn create(start: i8, end: i8) -> Self {
        let mut snap_range = vec![];
        for i in 0..end - start {
            snap_range.push(i as i8 + start)
        }
        snap_range
    }

    fn reduce_size(&self, end: u8) -> Self {
        let mut snap_range = vec![];
        for i in self.iter() {
            if *i >= 0 && *i < end as i8 {
                snap_range.push(*i)
            }
        }
        snap_range
    }
}

#[derive(Debug, Clone)]
pub struct SnapConfig {
    pub node_id: NodeId,
    pub pos: Vec3,
    pub dir: Vec3,
    pub snap_range: SnapRange,
    // Reverse means that outgoing lanes exist, and incoming does not
    pub reverse: bool,
}

impl PartialEq for SnapConfig {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.snap_range == other.snap_range
            && self.reverse == other.reverse
    }
}

type LaneMap = VecDeque<Option<SegmentId>>;

trait LaneMapUtils {
    fn contains_none(&self) -> bool;
    fn contains_some(&self) -> bool;
    fn contains_different_somes(&self) -> bool;
    fn is_same(&self) -> bool;
    fn create(no_lanes: u8, id: Option<SegmentId>) -> Self;
    fn update(&mut self, snap_range: &SnapRange, segment_id: SegmentId);
    fn expand(&mut self, snap_range: &SnapRange, segment_id: Option<SegmentId>);
}

impl LaneMapUtils for LaneMap {
    fn contains_none(&self) -> bool {
        let mut contains_none = false;
        for seg in self {
            if seg.is_none() {
                contains_none = true;
            }
        }
        contains_none
    }

    fn contains_some(&self) -> bool {
        let mut contains_some = false;
        for seg in self {
            if seg.is_some() {
                contains_some = true;
            }
        }
        contains_some
    }

    fn contains_different_somes(&self) -> bool {
        let mut temp: Option<SegmentId> = None;
        for ele in self {
            match (temp, ele) {
                (Some(a), Some(b)) => {
                    if a != *b {
                        return true;
                    }
                }
                (None, _) => temp = *ele,
                _ => {}
            }
        }
        false
    }

    fn is_same(&self) -> bool {
        let temp = self[0];
        for seg in self {
            if *seg != temp {
                return false;
            }
        }
        true
    }

    fn create(no_lanes: u8, id: Option<SegmentId>) -> Self {
        let mut vec = VecDeque::new();
        for _ in 0..no_lanes {
            vec.push_back(id)
        }
        vec
    }

    fn update(&mut self, snap_range: &SnapRange, segment_id: SegmentId) {
        for i in snap_range.iter() {
            if self[*i as usize].replace(segment_id).is_some() {
                panic!("Some segment was overriden in an update of a nodes lane map")
            }
        }
    }

    fn expand(&mut self, snap_range: &SnapRange, segment_id: Option<SegmentId>) {
        let len = self.len() as i8;
        for i in snap_range.iter() {
            if *i < 0 {
                self.push_front(segment_id);
            }
            if *i >= len {
                self.push_back(segment_id);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub pos: Vec3,
    pub dir: Vec3,
    incoming_lanes: LaneMap,
    outgoing_lanes: LaneMap,
}

impl Node {
    fn new(
        pos: Vec3,
        dir: Vec3,
        no_lanes: u8,
        lane_map: (Option<SegmentId>, Option<SegmentId>),
    ) -> Self {
        Node {
            pos,
            dir,
            incoming_lanes: LaneMap::create(no_lanes, lane_map.0),
            outgoing_lanes: LaneMap::create(no_lanes, lane_map.1),
        }
    }

    fn no_lanes(&self) -> u8 {
        self.incoming_lanes.len() as u8
    }

    fn has_snappable_lane(&self) -> bool {
        self.outgoing_lanes.contains_none() || self.incoming_lanes.contains_none()
    }

    fn expand_node(&mut self, snap_config: SnapConfig, segment_id: SegmentId) {
        self.pos = snap_config.pos;
        if snap_config.reverse {
            self.incoming_lanes
                .expand(&snap_config.snap_range, Some(segment_id));
            self.outgoing_lanes.expand(&snap_config.snap_range, None);
        } else {
            self.incoming_lanes.expand(&snap_config.snap_range, None);
            self.outgoing_lanes
                .expand(&snap_config.snap_range, Some(segment_id));
        }
    }

    fn update_lane_map(&mut self, snap_config: SnapConfig, segment_id: SegmentId) {
        let sized_snap_range = snap_config.snap_range.reduce_size(self.no_lanes());
        if snap_config.reverse {
            self.incoming_lanes.update(&sized_snap_range, segment_id)
        } else {
            self.outgoing_lanes.update(&sized_snap_range, segment_id)
        }
        if snap_config.snap_range.len() as u8 > self.no_lanes() {
            self.expand_node(snap_config, segment_id);
        }
    }

    fn get_snap_configs_from_map(
        &self,
        lane_map: &VecDeque<Option<SegmentId>>,
        reverse: bool,
        no_lanes: u8,
        node_id: NodeId,
        opposite_same: bool,
    ) -> Vec<SnapConfig> {
        let lane_width_dir = self.dir.right_hand() * LANE_WIDTH;
        if !lane_map.contains_some() {
            if no_lanes == self.no_lanes() {
                vec![SnapConfig {
                    node_id,
                    pos: self.pos,
                    dir: self.dir,
                    reverse,
                    snap_range: SnapRange::create(0, self.no_lanes() as i8),
                }]
            } else if opposite_same {
                if no_lanes < self.no_lanes() {
                    let mut snap_configs = vec![];
                    let diff = self.no_lanes() - no_lanes;
                    for i in 0..(diff + 1) {
                        snap_configs.push(SnapConfig {
                            node_id,
                            pos: self.pos + (i as f32 - diff as f32 / 2.0) * lane_width_dir,
                            dir: self.dir,
                            reverse,
                            snap_range: SnapRange::create(i as i8, (i + no_lanes) as i8),
                        });
                    }
                    snap_configs
                } else {
                    let mut snap_configs = vec![];
                    let diff = no_lanes - self.no_lanes();
                    for i in 0..(diff + 1) {
                        snap_configs.push(SnapConfig {
                            node_id,
                            pos: self.pos + (i as f32 - diff as f32 / 2.0) * lane_width_dir,
                            dir: self.dir,
                            reverse,
                            snap_range: SnapRange::create(
                                i as i8 - diff as i8,
                                (i + no_lanes) as i8 - diff as i8,
            ),
                        });
                    }
                    snap_configs
                }
            } else {
                // cannot snap as the opposite is not the same segment, and this side no_lanes is
                // not the same size
                vec![]
            }
        } else {
            let mut snap_configs = vec![];
            let mut possible_snaps: Vec<SnapRange> = vec![];
            let diff = self.no_lanes() as i8 - no_lanes as i8;
            let start_pos = self.pos - lane_width_dir * diff as f32 / 2.0;
            for (i, l) in lane_map.iter().enumerate() {
                if l.is_none() {
                    possible_snaps.push(vec![]);
                    possible_snaps.iter_mut().for_each(|s| s.push(i as i8));
                    possible_snaps.retain_mut(|s| {
                        if s.len() as u8 == no_lanes {
                            snap_configs.push(SnapConfig {
                                node_id,
                                pos: start_pos
                                    + (i as i8 - (no_lanes as i8 - 1)) as f32 * lane_width_dir,
                                dir: self.dir,
                                reverse,
                                snap_range: s.clone(),
                            });
                            false
                        } else {
                            true
                        }
                    });
                } else {
                    possible_snaps = vec![];
                }
            }
            snap_configs
        }
    }

    fn get_snap_configs(&self, no_lanes: u8, node_id: NodeId) -> Vec<SnapConfig> {
        if self.outgoing_lanes.contains_none() {
            self.get_snap_configs_from_map(
                &self.outgoing_lanes,
                false,
                no_lanes,
                node_id,
                self.incoming_lanes.is_same(),
            )
        } else if self.incoming_lanes.contains_none() {
            self.get_snap_configs_from_map(
                &self.incoming_lanes,
                true,
                no_lanes,
                node_id,
                self.outgoing_lanes.is_same(),
            )
        } else {
            // TODO possibly implement such that one can snap in same dir when one side is all None
            // this should only be possible if the total no_lanes is less that MAX_LANES
            vec![]
        }
    }
}

#[derive(Clone)]
pub struct Segment {
    pub road_type: RoadType,
    pub guide_points: Vec<Vec3>,
}

impl Segment {
    pub fn new(road_type: RoadType, guide_points: Vec<Vec3>) -> Self {
        Segment {
            road_type,
            guide_points,
        }
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
        selected_node: Option<SnapConfig>,
        snapped_node: Option<SnapConfig>,
    ) -> (RoadMesh, Option<SnapConfig>) {
        let (node_list, segment_list, road_type, reverse) = road.extract();
        let mut new_snap_index = 0;

        let mut nodes = vec![];
        if reverse {
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
            new_snap_index = nodes.len() - 1;
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
                self.segment_map.insert(id, segment.clone());
                self.road_meshes
                    .insert(RoadElementId::Segment(id), mesh.clone());
                self.forward_refs.insert(id, Vec::new());
                self.backward_refs.insert(id, Vec::new());
            });

        let mut node_ids = vec![];
        nodes.iter().enumerate().for_each(|(i, node)| {
            let node_id = match node {
                Some(snap_config) => {
                    let segment_id = match snap_config.reverse {
                        false => segment_ids[0],
                        true => segment_ids[segment_ids.len() - 1],
                    };
                    self.get_mut_node(snap_config.node_id)
                        .update_lane_map(snap_config.clone(), segment_id);
                    snap_config.node_id
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

        let new_snap_id = node_ids[new_snap_index];
        let new_snap = self
            .get_node(new_snap_id)
            .get_snap_configs(road_type.no_lanes, new_snap_id)
            .get(0)
            .cloned();

        // TODO recompute meshes for affected nodes
        (self.combine_road_meshes(), new_snap)
    }

    pub fn remove_road(&self, segment: SegmentId) {
        // remove segment and update affected nodes
    }

    fn get_mut_node(&mut self, node: NodeId) -> &mut Node {
        self.node_map
            .get_mut(&node)
            .expect("Node does not exist in node map")
    }

    pub fn get_node(&self, node: NodeId) -> &Node {
        self.node_map
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

    pub fn get_node_snap_configs(
        &self,
        pos: Vec3,
        no_lanes: u8,
    ) -> Option<(NodeId, Vec<SnapConfig>)> {
        // TODO match all nodes in range and combine the snap configs generated by all of them
        let mut closest_node = None;
        for (id, n) in self.node_map.iter() {
            if !n.has_snappable_lane() {
                continue;
            }
            let dist = (n.pos - pos).length();
            if let Some((_, old_dist)) = closest_node {
                if old_dist < dist {
                    continue;
                }
            }
            if dist < (n.no_lanes() + no_lanes) as f32 * LANE_WIDTH {
                closest_node = Some((id, dist));
            }
        }
        closest_node.map(|(id, _)| {
            let n = self.get_node(*id);
            let mut snap_configs = n.get_snap_configs(no_lanes, *id);
            snap_configs.sort_by(|a, b| {
                (a.pos - pos)
                    .length()
                    .partial_cmp(&(b.pos - pos).length())
                    .unwrap()
            });
            (*id, snap_configs)
        })
    }

    pub fn get_node_id_from_pos(&self, pos: Vec3) -> Option<NodeId> {
        let mut closest_node = None;
        for (id, n) in self.node_map.iter() {
            let dist = (n.pos - pos).length();
            if let Some((_, old_dist)) = closest_node {
                if old_dist < dist {
                    continue;
                }
            }
            if dist < n.no_lanes() as f32 * LANE_WIDTH {
                closest_node = Some((id, dist));
            }
        }
        closest_node.map(|(id, _)| *id)
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

    pub fn get_segment_inside(&self, ground_pos: Vec3) -> Option<SegmentId> {
        for (id, s) in self.segment_map.iter() {
            if curves::is_inside(
                &s.guide_points,
                ground_pos,
                s.road_type.no_lanes as f32 * LANE_WIDTH,
            ) {
                return Some(*id);
            }
        }
        None
    }
}

type LeadingPair = (NodeId, SegmentId);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(u32);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SegmentId(u32);

#[derive(PartialEq, Eq, Hash)]
pub enum RoadElementId {
    Node(NodeId),
    Segment(SegmentId),
}
