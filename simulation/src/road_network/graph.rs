use glam::*;
use std::collections::HashMap;

use crate::curves;

use utils::id::{NodeId, SegmentId};

use super::node::LNode;
use super::road_builder::LRoadGenerator;
use super::segment::LSegment;
use super::snap::SnapConfig;
use super::{NodeType, Side};

type LeadingPair = (NodeId, SegmentId);

pub struct RoadGraph {
    node_map: HashMap<NodeId, LNode>,
    segment_map: HashMap<SegmentId, LSegment>,
    /// Defines for each node, the set of nodes that are reachable from this node, through exactly
    /// one segment in the direction of the segment.
    forward_refs: HashMap<NodeId, Vec<LeadingPair>>,
    /// Defines for each node, the set of nodes that are reachable from this node, through exactly
    /// one segment in the opposite direction of the segment.
    backward_refs: HashMap<NodeId, Vec<LeadingPair>>,

    node_id_count: u32,
    segment_id_count: u32,
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
            node_id_count: 0,
            segment_id_count: 0,
        }
    }
}

impl RoadGraph {
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

    pub fn get_node_positions(&self) -> Vec<Vec3> {
        self.node_map.iter().map(|(_, n)| n.get_pos()).collect()
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

    pub fn get_segment_inside(&self, ground_pos: Vec3) -> Option<SegmentId> {
        for (id, s) in self.segment_map.iter() {
            if curves::is_inside(&s.get_guide_points(), ground_pos, s.get_width()) {
                return Some(*id);
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

    /// At this point the road generator tool has allowed the construction of this road. The order
    /// of {`NodeId`}'s order always follows the direction of the road. The order of
    /// {`SegmentId`}'s follow whatever order was decided by the road generator.
    pub fn add_road(
        &mut self,
        road: LRoadGenerator,
        selected_node: Option<SnapConfig>,
        snapped_node: Option<SnapConfig>,
    ) -> (Option<SnapConfig>, Vec<SegmentId>) {
        let (node_builders, segment_builders, node_type, _segment_type, reverse) = road.extract();
        let node_type = node_type;
        let mut new_snap_index = 0;

        // Generate node ids
        let mut num_node_ids = segment_builders.len() - 1;
        if snapped_node.is_none() {
            num_node_ids += 1;
        };
        if selected_node.is_none() {
            num_node_ids += 1;
        };
        let node_ids: Vec<NodeId> = (0..num_node_ids).map(|_| self.generate_node_id()).collect();

        // Generate segment ids
        let num_segment_ids = segment_builders.len();
        let segment_ids: Vec<SegmentId> = (0..num_segment_ids)
            .map(|_| self.generate_segment_id())
            .collect();

        // Create list of new and old nodes in correct order
        let mut nodes = vec![];
        if reverse {
            nodes.push(snapped_node);
            for _ in 0..node_builders.len() - 2 {
                nodes.push(None);
            }
            nodes.push(selected_node);
        } else {
            nodes.push(selected_node);
            for _ in 0..node_builders.len() - 2 {
                nodes.push(None);
            }
            nodes.push(snapped_node);
            new_snap_index = nodes.len() - 1;
        }

        let mut node_id_counter = 0;
        let mut new_node_ids = vec![];
        nodes.into_iter().enumerate().for_each(|(i, node)| {
            let node_id = match node {
                Some(snap_config) => {
                    // update existing node lane_map
                    let segment_id = match snap_config.get_side() {
                        Side::Out => segment_ids[0],
                        Side::In => segment_ids[segment_ids.len() - 1],
                    };
                    let new_id = snap_config.get_id();
                    self.get_node_mut(snap_config.get_id())
                        .add_segment(segment_id, snap_config);
                    new_id
                }
                None => {
                    // generate new node
                    let node_id = node_ids[node_id_counter];
                    node_id_counter += 1;
                    self.forward_refs.insert(node_id, Vec::new());
                    self.backward_refs.insert(node_id, Vec::new());

                    // TODO hacky solution generalize to VecUtils trait?
                    let incoming = segment_ids.get(((i as i32 - 1) % 100) as usize).copied();
                    let outgoing = segment_ids.get(i).copied();

                    use super::LaneMapConfig::*;
                    let lane_map_config = match (incoming, outgoing) {
                        (Some(incoming), Some(outgoing)) => Sym { incoming, outgoing },
                        (Some(incoming), None) => In { incoming },
                        (None, Some(outgoing)) => Out { outgoing },
                        (None, None) => panic!("Cannot construct a new node with no segments"),
                    };
                    self.node_map
                        .insert(node_id, node_builders[i].build(node_type, lane_map_config));
                    node_id
                }
            };
            new_node_ids.push(node_id);
        });

        let segment_width = node_type.lane_width.get() * node_type.no_lanes as f32;
        segment_builders
            .into_iter()
            .enumerate()
            .for_each(|(i, segment_builder)| {
                let segment =
                    segment_builder.build(segment_width, new_node_ids[i], new_node_ids[i + 1]);
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
            .construct_snap_configs(node_type, new_snap_id)
            .get(0)
            .cloned();

        #[cfg(debug_assertions)]
        {
            assert_eq!(self.node_map.len(), self.forward_refs.len());
            assert_eq!(self.node_map.len(), self.backward_refs.len());
        }

        (new_snap, segment_ids)
    }

    fn remove_node(&mut self, node_id: NodeId) {
        self.node_map.remove(&node_id);
        self.forward_refs.remove(&node_id);
        self.backward_refs.remove(&node_id);
    }

    /// The return bool signals whether the segment was allowed to be removed or not.
    pub fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        // check if deletion is valid
        let segment = self.get_segment(segment_id).clone();
        let from_node = self.get_node(segment.get_from_node());
        let to_node = self.get_node(segment.get_to_node());
        if !from_node.can_remove_segment(segment_id) || !to_node.can_remove_segment(segment_id) {
            dbg!("Cannot bulldoze segment");
            return false;
        }

        // remove any reference to this segment
        self.segment_map.remove(&segment_id);
        self.forward_refs
            .get_mut(&segment.get_from_node())
            .expect("node does not exist in forward map")
            .retain(|(_, id)| *id != segment_id);
        self.backward_refs
            .get_mut(&segment.get_to_node())
            .expect("node does not exist in backward map")
            .retain(|(_, id)| *id != segment_id);

        if self
            .get_node_mut(segment.get_from_node())
            .remove_segment(segment_id)
        {
            self.remove_node(segment.get_from_node())
        }
        if self
            .get_node_mut(segment.get_to_node())
            .remove_segment(segment_id)
        {
            self.remove_node(segment.get_to_node())
        }

        #[cfg(debug_assertions)]
        {
            assert_eq!(self.node_map.len(), self.forward_refs.len());
            assert_eq!(self.node_map.len(), self.backward_refs.len());
        }

        true
    }

    /// Returns a list of node id's that have an open slot for the selected road type to snap to.
    /// If reverse parameter is set to {`None`}, then no direction is checked when matching nodes.
    pub fn get_possible_snap_nodes(
        &self,
        reverse: Option<bool>,
        node_type: NodeType,
    ) -> Vec<NodeId> {
        self.node_map
            .iter()
            .filter(|(id, n)| {
                if !n.can_add_some_segment() {
                    return false;
                };
                let Some(snap_config) = n.construct_snap_configs(node_type, **id).pop() else {
                    return false
                };
                if let Some(reverse) = reverse {
                    return reverse != (snap_config.get_side() == Side::In);
                };
                true
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// If no node is within range of pos, then this function returns {`None`}. Otherwise it
    /// returns the closest node to pos, and all its possible {`SnapConfig`}'s.
    pub fn get_snap_configs_closest_node(
        &self,
        ground_pos: Vec3,
        node_type: NodeType,
    ) -> Option<(NodeId, Vec<SnapConfig>)> {
        // TODO match all nodes in range and combine the snap configs generated by all of them
        let mut closest_node = None;
        for (id, n) in self.node_map.iter() {
            if !n.can_add_some_segment() {
                continue;
            }
            let dist = (n.get_pos() - ground_pos).length();
            if let Some((_, old_dist)) = closest_node {
                if old_dist < dist {
                    continue;
                }
            }
            if dist < (n.no_lanes() + node_type.no_lanes) as f32 * node_type.lane_width.get() {
                closest_node = Some((id, dist));
            }
        }
        closest_node.map(|(id, _)| {
            let n = self.get_node(*id);
            let mut snap_configs = n.construct_snap_configs(node_type, *id);
            snap_configs.sort_by(|a, b| {
                (a.get_pos() - ground_pos)
                    .length()
                    .partial_cmp(&(b.get_pos() - ground_pos).length())
                    .unwrap()
            });
            (*id, snap_configs)
        })
    }

    #[cfg(debug_assertions)]
    pub fn debug_node_from_pos(&self, pos: Vec3) {
        let mut closest_node = None;
        for (id, n) in self.node_map.iter() {
            let dist = (n.get_pos() - pos).length();
            if let Some((_, old_dist)) = closest_node {
                if old_dist < dist {
                    continue;
                }
            }
            if dist < n.no_lanes() as f32 * n.get_lane_width() {
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
