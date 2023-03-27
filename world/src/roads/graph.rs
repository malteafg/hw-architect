use glam::*;
use std::collections::HashMap;

use utils::id::{IdManager, NodeId, SegmentId};

use super::node::LNode;
use super::road_builder::{LNodeBuilderType, LRoadBuilder};
use super::segment::LSegment;
use super::snap::SnapConfig;
use super::{NodeType, Side};

use serde::{Deserialize, Serialize};

type LeadingPair = (NodeId, SegmentId);

#[derive(Serialize, Deserialize)]
pub struct RoadGraph {
    node_map: HashMap<NodeId, LNode>,
    segment_map: HashMap<SegmentId, LSegment>,
    /// Defines for each node, the set of nodes that are reachable from this node, through exactly
    /// one segment in the direction of the segment.
    forward_refs: HashMap<NodeId, Vec<LeadingPair>>,
    /// Defines for each node, the set of nodes that are reachable from this node, through exactly
    /// one segment in the opposite direction of the segment.
    backward_refs: HashMap<NodeId, Vec<LeadingPair>>,

    node_id_manager: IdManager<NodeId>,
    segment_id_manager: IdManager<SegmentId>,
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
            node_id_manager: IdManager::new(),
            segment_id_manager: IdManager::new(),
        }
    }
}

impl RoadGraph {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_node_mut(&mut self, node: NodeId) -> &mut LNode {
        self.node_map
            .get_mut(&node)
            .expect("Node does not exist in node map")
    }

    fn get_node(&self, node: NodeId) -> &LNode {
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

    fn remove_node(&mut self, node_id: NodeId) {
        self.node_map.remove(&node_id);
        self.forward_refs.remove(&node_id);
        self.backward_refs.remove(&node_id);
    }
}

impl crate::RoadManipulator for RoadGraph {
    fn get_node_pos(&self, node: NodeId) -> Vec3 {
        self.get_node(node).get_pos()
    }

    fn get_node_dir(&self, node: NodeId) -> Vec3 {
        self.get_node(node).get_dir()
    }

    fn get_node_positions(&self) -> Vec<Vec3> {
        self.node_map.iter().map(|(_, n)| n.get_pos()).collect()
    }

    fn get_segment_inside(&self, pos: Vec3) -> Option<SegmentId> {
        for (id, s) in self.segment_map.iter() {
            if s.get_guide_points().is_inside(pos, s.get_width()) {
                return Some(*id);
            }
        }
        None
    }

    fn add_road(
        &mut self,
        road: LRoadBuilder,
        sel_node_type: NodeType,
    ) -> (Option<SnapConfig>, Vec<SegmentId>) {
        let (node_builders, segment_builders, reverse) = road.consume();
        let num_nodes = node_builders.len();

        // Generate node ids
        let node_builders_with_id: Vec<(NodeId, LNodeBuilderType)> = node_builders
            .into_iter()
            .map(|n| match n {
                new @ LNodeBuilderType::New(_) => (self.node_id_manager.gen(), new),
                LNodeBuilderType::Old(snap) => (snap.get_id(), LNodeBuilderType::Old(snap)),
            })
            .collect();

        // Generate segment ids
        let num_segment_ids = segment_builders.len();
        let segment_ids: Vec<SegmentId> = (0..num_segment_ids)
            .map(|_| self.segment_id_manager.gen())
            .collect();

        // Create new nodes and update old ones
        let node_ids: Vec<NodeId> = node_builders_with_id
            .into_iter()
            .enumerate()
            .map(|(i, (node_id, node_builder))| {
                match node_builder {
                    LNodeBuilderType::New(node_builder) => {
                        // generate new node
                        self.forward_refs.insert(node_id, Vec::new());
                        self.backward_refs.insert(node_id, Vec::new());
                        use super::LaneMapConfig::*;
                        let lane_map_config = if i == 0 {
                            Out {
                                outgoing: segment_ids[0],
                            }
                        } else if i == num_nodes - 1 {
                            In {
                                incoming: segment_ids[i - 1],
                            }
                        } else {
                            Sym {
                                incoming: segment_ids[i - 1],
                                outgoing: segment_ids[i],
                            }
                        };

                        self.node_map
                            .insert(node_id, node_builder.build(lane_map_config));
                    }
                    LNodeBuilderType::Old(snap_config) => {
                        // update existing node
                        let segment_id = match snap_config.get_side() {
                            Side::Out => segment_ids[0],
                            Side::In => segment_ids[segment_ids.len() - 1],
                        };
                        self.get_node_mut(node_id)
                            .add_segment(segment_id, snap_config);
                    }
                };
                node_id
            })
            .collect();

        segment_builders
            .into_iter()
            .enumerate()
            .for_each(|(i, segment_builder)| {
                let segment = segment_builder.build(node_ids[i], node_ids[i + 1]);
                let id = segment_ids[i];
                self.segment_map.insert(id, segment);
            });

        // update forward_refs and backward_refs
        node_ids.iter().enumerate().for_each(|(i, node_id)| {
            if let Some(backward_id) = segment_ids.get(((i as i32 - 1) % 100) as usize) {
                self.backward_refs
                    .get_mut(node_id)
                    .expect("NodeId does not exist in backward_refs")
                    .push((node_ids[i - 1], *backward_id));
            }
            if let Some(forward_id) = segment_ids.get(i) {
                self.forward_refs
                    .get_mut(node_id)
                    .expect("NodeId does not exist in forward_refs")
                    .push((node_ids[i + 1], *forward_id));
            }
        });

        // compute the new node that the tool can snap to, if any.
        let new_snap_id = node_ids[if reverse { 0 } else { node_ids.len() - 1 }];
        let new_snap = self
            .get_node(new_snap_id)
            .construct_snap_configs(sel_node_type, new_snap_id)
            .get(0)
            .cloned();

        #[cfg(debug_assertions)]
        {
            assert_eq!(self.node_map.len(), self.forward_refs.len());
            assert_eq!(self.node_map.len(), self.backward_refs.len());
        }

        (new_snap, segment_ids)
    }

    fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
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

        // TODO put this code in remove node
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

    fn get_possible_snap_nodes(&self, side: Option<Side>, node_type: NodeType) -> Vec<NodeId> {
        self.node_map
            .iter()
            .filter(|(&id, n)| {
                if !n.can_add_some_segment() {
                    return false;
                };
                let Some(snap_config) = n.construct_snap_configs(node_type, id).pop() else {
                    return false
                };
                if let Some(side) = side {
                    return side != snap_config.get_side();
                };
                true
            })
            .map(|(&id, _)| id)
            .collect()
    }

    fn get_snap_configs_closest_node(
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
            if dist < (n.no_lanes() + node_type.no_lanes) as f32 * node_type.lane_width.getf32() {
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
    fn debug_node_from_pos(&self, pos: Vec3) {
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
            dbg!("Node: {} -------------------------", id);
            dbg!(self.node_map.get(&id));
            dbg!(self.forward_refs.get(&id));
            dbg!(self.backward_refs.get(&id));
        }
    }

    #[cfg(debug_assertions)]
    fn debug_segment_from_pos(&self, pos: Vec3) {
        if let Some(id) = self.get_segment_inside(pos) {
            dbg!("Segment: {} ----------------------", id);
            dbg!(id);
        }
    }
}
