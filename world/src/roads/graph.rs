use super::node::LNode;
use super::segment::LSegment;

use world_api::{LNodeBuilderType, LRoadBuilder, LaneMapConfig, NodeType, Side, SnapConfig};

use utils::id::{IdManager, IdMap, IdSet, NodeId, SegmentId, UnsafeMap};
use utils::math::Loc;

use glam::*;
use serde::{Deserialize, Serialize};

type LeadingPair = (NodeId, SegmentId);

#[derive(Serialize, Deserialize)]
pub struct RoadGraph {
    node_map: IdMap<NodeId, LNode, UnsafeMap>,
    segment_map: IdMap<SegmentId, LSegment, UnsafeMap>,
    /// Defines for each node, the set of nodes that are reachable from this node, through exactly
    /// one segment in the direction of the segment.
    forward_refs: IdMap<NodeId, Vec<LeadingPair>, UnsafeMap>,
    /// Defines for each node, the set of nodes that are reachable from this node, through exactly
    /// one segment in the opposite direction of the segment.
    backward_refs: IdMap<NodeId, Vec<LeadingPair>, UnsafeMap>,

    /// These are basic nodes where the main segment is outgoing and open nodes where the open
    /// side is incoming
    starting_nodes: IdSet<NodeId>,
    /// These are basic nodes where the main segment is incoming and open nodes where the open
    /// side is outgoing
    ending_nodes: IdSet<NodeId>,

    node_id_manager: IdManager<NodeId>,
    segment_id_manager: IdManager<SegmentId>,
}

impl Default for RoadGraph {
    fn default() -> Self {
        let node_map = IdMap::new();
        let segment_map = IdMap::new();
        let forward_refs = IdMap::new();
        let backward_refs = IdMap::new();
        let starting_nodes = IdSet::new();
        let ending_nodes = IdSet::new();

        Self {
            node_map,
            segment_map,
            forward_refs,
            backward_refs,
            starting_nodes,
            ending_nodes,
            node_id_manager: IdManager::new(),
            segment_id_manager: IdManager::new(),
        }
    }
}

impl RoadGraph {
    fn get_lnode(&self, node: NodeId) -> &LNode {
        self.node_map.get(node)
    }

    fn get_lnode_mut(&mut self, node: NodeId) -> &mut LNode {
        self.node_map.get_mut(node)
    }

    pub fn get_lsegment(&self, segment: SegmentId) -> &LSegment {
        self.segment_map.get(segment)
    }

    fn _get_lsegment_mut(&mut self, segment: SegmentId) -> &mut LSegment {
        self.segment_map.get_mut(segment)
    }

    pub fn _get_forwards_ref(&self, node: NodeId) -> &Vec<LeadingPair> {
        self.forward_refs.get(node)
    }

    pub fn _get_backwards_ref(&self, node: NodeId) -> &Vec<LeadingPair> {
        self.backward_refs.get(node)
    }

    fn remove_node(&mut self, node_id: NodeId) {
        self.node_map.remove(node_id);
        self.forward_refs.remove(node_id);
        self.backward_refs.remove(node_id);
    }

    pub fn get_node_from_pos(&self, pos: Vec3) -> Option<NodeId> {
        for (id, n) in self.node_map.iter() {
            if n.contains_pos(pos) {
                return Some(id);
            }
        }
        None
    }

    pub fn get_segment_from_pos(&self, pos: Vec3) -> Option<SegmentId> {
        for (id, s) in self.segment_map.iter() {
            if s.contains_pos(pos) {
                return Some(id);
            }
        }
        None
    }

    /// Returns ending segments, and the node they backward_refs to as a LeadingPair.
    pub fn _get_ending_segments(&self) -> Vec<LeadingPair> {
        let mut ending_segments = Vec::with_capacity(self.ending_nodes.len());
        for node_id in self.ending_nodes.iter() {
            match self._get_backwards_ref(node_id).as_slice() {
                [] => {}
                leading_pairs => leading_pairs.iter().for_each(|p| ending_segments.push(*p)),
            }
        }
        ending_segments
    }

    fn update_starting_ending(&mut self, nodes: &[NodeId]) {
        for id in nodes.iter() {
            let node = self.get_lnode(*id);
            match (node.is_starting(), node.is_ending()) {
                (true, false) => {
                    self.starting_nodes.insert(*id);
                    self.ending_nodes.remove(*id);
                }
                (false, true) => {
                    self.starting_nodes.remove(*id);
                    self.ending_nodes.insert(*id);
                }
                (true, true) => {
                    panic!("A node cannot possibly be both and ending and a starting node")
                }
                _ => {}
            }
        }
    }
}

impl crate::RoadManipulator for RoadGraph {
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
                LNodeBuilderType::Old(snap) => (snap.id(), LNodeBuilderType::Old(snap)),
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
                        use LaneMapConfig::*;
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
                            .insert(node_id, LNode::from_builder(node_builder, lane_map_config));
                    }
                    LNodeBuilderType::Old(snap_config) => {
                        // update existing node
                        let segment_id = match snap_config.side() {
                            Side::Out => segment_ids[0],
                            Side::In => segment_ids[segment_ids.len() - 1],
                        };
                        self.get_lnode_mut(node_id)
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
                let segment = LSegment::from_builder(segment_builder, node_ids[i], node_ids[i + 1]);
                let id = segment_ids[i];
                self.segment_map.insert(id, segment);
            });

        // update forward_refs and backward_refs
        node_ids.iter().enumerate().for_each(|(i, node_id)| {
            if let Some(backward_id) = segment_ids.get(((i as i32 - 1) % 100) as usize) {
                self.backward_refs
                    .get_mut(*node_id)
                    .push((node_ids[i - 1], *backward_id));
            }
            if let Some(forward_id) = segment_ids.get(i) {
                self.forward_refs
                    .get_mut(*node_id)
                    .push((node_ids[i + 1], *forward_id));
            }
        });

        // update starting and endings nodes
        self.update_starting_ending(&node_ids);

        // compute the new node that the tool can snap to, if any.
        let new_snap_id = node_ids[if reverse { 0 } else { node_ids.len() - 1 }];
        let new_snap = self
            .get_lnode(new_snap_id)
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
        let segment = self.get_lsegment(segment_id);
        let from_node = self.get_lnode(segment.get_from_node());
        let to_node = self.get_lnode(segment.get_to_node());
        if !from_node.can_remove_segment(segment_id) || !to_node.can_remove_segment(segment_id) {
            dbg!("Cannot bulldoze segment");
            return false;
        }

        // remove any reference to this segment
        let segment = self.segment_map.remove(segment_id).unwrap();
        self.forward_refs
            .get_mut(segment.get_from_node())
            .retain(|(_, id)| *id != segment_id);
        self.backward_refs
            .get_mut(segment.get_to_node())
            .retain(|(_, id)| *id != segment_id);

        // TODO put this code in remove node
        let mut affected_nodes = vec![segment.get_to_node(), segment.get_from_node()];
        if self
            .get_lnode_mut(segment.get_from_node())
            .remove_segment(segment_id)
        {
            self.remove_node(segment.get_from_node());
            affected_nodes.remove(1);
        }
        if self
            .get_lnode_mut(segment.get_to_node())
            .remove_segment(segment_id)
        {
            self.remove_node(segment.get_to_node());
            affected_nodes.remove(0);
        }

        // update starting and endings nodes
        self.update_starting_ending(&affected_nodes);

        #[cfg(debug_assertions)]
        {
            assert_eq!(self.node_map.len(), self.forward_refs.len());
            assert_eq!(self.node_map.len(), self.backward_refs.len());
        }

        true
    }

    fn get_possible_snap_nodes(
        &self,
        side: Option<Side>,
        node_type: NodeType,
    ) -> Vec<(NodeId, Loc)> {
        self.node_map
            .iter()
            .filter(|(id, n)| {
                if !n.can_add_some_segment() {
                    return false;
                };
                let Some(snap_config) = n.construct_snap_configs(node_type, *id).pop() else {
                    return false;
                };
                if let Some(side) = side {
                    return side != snap_config.side();
                };
                true
            })
            .map(|(id, n)| (id, n.loc()))
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
            let dist = (n.pos() - ground_pos).length();
            if let Some((_, old_dist)) = closest_node {
                if old_dist < dist {
                    continue;
                }
            }
            if dist < (n.no_lanes() + node_type.no_lanes()) as f32 * node_type.lane_width_f32() {
                closest_node = Some((id, dist));
            }
        }
        closest_node.map(|(id, _)| {
            let n = self.get_lnode(id);
            let mut snap_configs = n.construct_snap_configs(node_type, id);
            snap_configs.sort_by(|a, b| {
                (a.pos() - ground_pos)
                    .length()
                    .partial_cmp(&(b.pos() - ground_pos).length())
                    .unwrap()
            });
            (id, snap_configs)
        })
    }

    fn debug_node(&self, id: NodeId) {
        // let mut closest_node = None;
        // for (id, n) in self.node_map.iter() {
        //     let dist = (n.pos() - pos).length();
        //     if let Some((_, old_dist)) = closest_node {
        //         if old_dist < dist {
        //             continue;
        //         }
        //     }
        //     if dist < n.no_lanes() as f32 * n.lane_width() {
        //         closest_node = Some((id, dist));
        //     }
        // }

        dbg!("Node: {} -------------------------", id);
        dbg!(self.node_map.get(id));
        dbg!(self.forward_refs.get(id));
        dbg!(self.backward_refs.get(id));
    }

    fn debug_segment(&self, id: SegmentId) {
        dbg!("Segment: {} ----------------------", id);
        dbg!(id);
    }
}
