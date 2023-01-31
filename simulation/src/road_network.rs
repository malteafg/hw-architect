use crate::curves;
use gfx_api::RoadMesh;
use glam::*;
use std::collections::HashMap;
use utils::consts::LANE_WIDTH;
use utils::id::{NodeId, SegmentId};
use utils::VecUtils;

mod snap;
pub use snap::SnapConfig;

use snap::SnapRange;
mod lanes;
use lanes::LaneMap;

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

#[derive(Clone, Debug)]
pub struct LNode {
    pos: Vec3,
    dir: Vec3,
    incoming_lanes: LaneMap,
    outgoing_lanes: LaneMap,
}

#[derive(Clone, Copy)]
pub struct LNodeBuilder {
    pub pos: Vec3,
    pub dir: Vec3,
}

impl LNodeBuilder {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        LNodeBuilder { pos, dir }
    }

    fn build(self, no_lanes: u8, lane_map: (Option<SegmentId>, Option<SegmentId>)) -> LNode {
        LNode {
            pos: self.pos,
            dir: self.dir,
            incoming_lanes: LaneMap::create(no_lanes, lane_map.0),
            outgoing_lanes: LaneMap::create(no_lanes, lane_map.1),
        }
    }
}

impl LNode {
    pub fn get_dir(&self) -> Vec3 {
        self.dir
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

    fn can_remove_segment(&self, segment_id: SegmentId, reverse: bool) -> bool {
        if reverse {
            (self.outgoing_lanes.contains_some()
                || !self.incoming_lanes.is_middle_segment(segment_id))
                && (!self.incoming_lanes.is_same() || self.outgoing_lanes.is_continuous())
        } else {
            (self.incoming_lanes.contains_some()
                || !self.outgoing_lanes.is_middle_segment(segment_id))
                && (!self.outgoing_lanes.is_same() || self.incoming_lanes.is_continuous())
        }
    }

    fn remove_segment_from_lane_map(&mut self, segment_id: SegmentId) {
        self.incoming_lanes.remove_segment(segment_id);
        self.outgoing_lanes.remove_segment(segment_id);
        let mut delete_list = vec![];
        for i in 0..self.incoming_lanes.len() {
            if self.incoming_lanes[i] == None && self.outgoing_lanes[i] == None {
                delete_list.push(i);
            }
        }
        delete_list.reverse();
        for &i in delete_list.iter() {
            self.incoming_lanes.remove(i);
            self.outgoing_lanes.remove(i);
        }
    }

    fn get_snap_configs_from_map(
        &self,
        lane_map: &LaneMap,
        reverse: bool,
        no_lanes: u8,
        node_id: NodeId,
        opposite_same: bool,
    ) -> Vec<SnapConfig> {
        let lane_width_dir = self.dir.right_hand() * LANE_WIDTH;
        if lane_map.contains_some() {
            // lane map contains some so look for snap ranges in between segments
            let mut snap_configs = vec![];
            let mut possible_snaps: Vec<SnapRange> = vec![];
            let diff = self.no_lanes() as i8 - no_lanes as i8;
            let start_pos = self.pos - lane_width_dir * diff as f32 / 2.0;
            for (i, l) in lane_map.iter().enumerate() {
                if l.is_none() {
                    possible_snaps.push(SnapRange::empty());
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
        } else if no_lanes >= self.no_lanes() {
            // lane_map is all nones, therefore, if we are building larger segment with more than
            // or equal no_lanes then all snap possibilities exist
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
        } else if opposite_same && no_lanes < self.no_lanes() {
            // if we are building a segment with fewer no_lanes then we can only do it if the
            // opposite node is the same node, otherwise we create a many to many node
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
            // cannot snap as the opposite is not the same segment, and this sides no_lanes is
            // too small
            vec![]
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

#[derive(Debug, Clone)]
struct LSegment {
    road_type: RoadType,
    guide_points: curves::GuidePoints,
    from_node: NodeId,
    to_node: NodeId,
}

#[derive(Debug, Clone)]
pub struct SegmentBuilder {
    pub road_type: RoadType,
    pub guide_points: curves::GuidePoints,
    pub mesh: RoadMesh,
}

impl SegmentBuilder {
    pub fn new(road_type: RoadType, guide_points: curves::GuidePoints, mesh: RoadMesh) -> Self {
        SegmentBuilder {
            road_type,
            guide_points,
            mesh,
        }
    }

    fn build(self, from_node: NodeId, to_node: NodeId) -> LSegment {
        LSegment {
            road_type: self.road_type,
            guide_points: self.guide_points,
            from_node,
            to_node,
        }
    }
}

type LeadingPair = (NodeId, SegmentId);

pub struct RoadGraph {
    node_map: HashMap<NodeId, LNode>,
    segment_map: HashMap<SegmentId, LSegment>,
    forward_refs: HashMap<NodeId, Vec<LeadingPair>>,
    backward_refs: HashMap<NodeId, Vec<LeadingPair>>,
}

impl Default for RoadGraph {
    fn default() -> Self {
        let node_map = HashMap::new();
        let segment_map = HashMap::new();
        let forward_refs = HashMap::new();
        let backward_refs = HashMap::new();

        Self {
            node_map,
            segment_map,
            forward_refs,
            backward_refs,
        }
    }
}

impl RoadGraph {
    /// At this point the road generator tool has allowed the construction of this road. The return
    /// of a segment id is temporary.
    pub fn add_road(
        &mut self,
        road: impl RoadGen,
        selected_node: Option<SnapConfig>,
        snapped_node: Option<SnapConfig>,
        node_ids: Vec<NodeId>,
        segment_ids: Vec<SegmentId>,
    ) -> Option<SnapConfig> {
        let (node_list, segment_list, road_type, reverse) = road.extract();
        let mut new_snap_index = 0;

        // Create list of new and old nodes in correct order
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

        let mut node_id_counter = 0;
        let mut new_node_ids = vec![];
        nodes.iter().enumerate().for_each(|(i, node)| {
            let node_id = match node {
                Some(snap_config) => {
                    // update existing node lane_map
                    let segment_id = match snap_config.reverse {
                        false => segment_ids[0],
                        true => segment_ids[segment_ids.len() - 1],
                    };
                    self.get_node_mut(snap_config.node_id)
                        .update_lane_map(snap_config.clone(), segment_id);
                    snap_config.node_id
                }
                None => {
                    // generate new node
                    let node_id = node_ids[node_id_counter];
                    node_id_counter += 1;
                    self.forward_refs.insert(node_id, Vec::new());
                    self.backward_refs.insert(node_id, Vec::new());
                    self.node_map.insert(
                        node_id,
                        node_list[i].build(
                            road_type.no_lanes,
                            (
                                // TODO hacky solution generalize to VecUtils trait?
                                segment_ids.get(((i as i32 - 1) % 100) as usize).copied(),
                                segment_ids.get(i).copied(),
                            ),
                        ),
                    );
                    node_id
                }
            };
            new_node_ids.push(node_id);
        });

        segment_list
            .into_iter()
            .enumerate()
            .for_each(|(i, segment_builder)| {
                let segment = segment_builder.build(new_node_ids[i], new_node_ids[i + 1]);
                let id = segment_ids[i];
                self.segment_map.insert(id, segment);
            });

        // update forward_refs and backward_refs
        new_node_ids.iter().enumerate().for_each(|(i, node_id)| {
            if let Some(backward_id) = segment_ids.get(((i as i32 - 1) % 100) as usize) {
                self.backward_refs
                    .get_mut(node_id)
                    .expect("NodeId does not exist in backward_refs")
                    .push((new_node_ids[i - 1], *backward_id));
            }
            if let Some(forward_id) = segment_ids.get(i) {
                self.forward_refs
                    .get_mut(node_id)
                    .expect("NodeId does not exist in forward_refs")
                    .push((new_node_ids[i + 1], *forward_id));
            }
        });

        let new_snap_id = new_node_ids[new_snap_index];
        let new_snap = self
            .get_node(new_snap_id)
            .get_snap_configs(road_type.no_lanes, new_snap_id)
            .get(0)
            .cloned();

        new_snap
    }

    fn remove_node_if_not_exists(&mut self, node_id: NodeId) {
        if self
            .forward_refs
            .get(&node_id)
            .expect("node does not exist in forward map")
            .is_empty()
            && self
                .backward_refs
                .get(&node_id)
                .expect("node does not exist in backward map")
                .is_empty()
        {
            self.node_map.remove(&node_id);
            self.forward_refs.remove(&node_id);
            self.backward_refs.remove(&node_id);
        }
    }

    /// The return bool signals whether the segment was allowed to be removed or not.
    pub fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        // check if deletion is valid
        let segment = self.get_segment(segment_id).clone();
        let from_node = self.get_node(segment.from_node);
        let to_node = self.get_node(segment.to_node);
        if !from_node.can_remove_segment(segment_id, false)
            || !to_node.can_remove_segment(segment_id, true)
        {
            dbg!("Cannot bulldoze segment");
            return false;
        }

        // remove any reference to this segment
        self.segment_map.remove(&segment_id);
        self.get_node_mut(segment.from_node)
            .remove_segment_from_lane_map(segment_id);
        self.get_node_mut(segment.to_node)
            .remove_segment_from_lane_map(segment_id);
        self.forward_refs
            .get_mut(&segment.from_node)
            .expect("node does not exist in forward map")
            .retain(|(_, id)| *id != segment_id);
        self.backward_refs
            .get_mut(&segment.to_node)
            .expect("node does not exist in backward map")
            .retain(|(_, id)| *id != segment_id);

        // remove sorrounding nodes if they do not connect to segments
        self.remove_node_if_not_exists(segment.from_node);
        self.remove_node_if_not_exists(segment.to_node);

        true
    }

    fn get_node_mut(&mut self, node: NodeId) -> &mut LNode {
        self.node_map
            .get_mut(&node)
            .expect("Node does not exist in node map")
    }

    pub fn get_node(&self, node: NodeId) -> &LNode {
        self.node_map
            .get(&node)
            .expect("Node does not exist in node map")
    }

    fn _get_segment_mut(&mut self, segment: SegmentId) -> &mut LSegment {
        self.segment_map
            .get_mut(&segment)
            .expect("Segment does not exist in segment map")
    }

    fn get_segment(&self, segment: SegmentId) -> &LSegment {
        self.segment_map
            .get(&segment)
            .expect("Segment does not exist in segment map")
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

    #[cfg(debug_assertions)]
    pub fn debug_node_from_pos(&self, pos: Vec3) {
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
        if let Some(id) = closest_node.map(|(id, _)| *id) {
            println!("Node: {} -------------------------", id.0);
            dbg!(self.node_map.get(&id));
            dbg!(self.forward_refs.get(&id));
            dbg!(self.backward_refs.get(&id));
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug_segment_from_pos(&self, pos: Vec3) {
        if let Some(id) = self.get_segment_inside(pos) {
            println!("Segment: {} ----------------------", id.0);
            dbg!(self.segment_map.get(&id));
        }
    }
}

pub trait RoadGen {
    fn extract(self) -> (Vec<LNodeBuilder>, Vec<SegmentBuilder>, RoadType, bool);
}
