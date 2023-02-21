use glam::*;
use utils::consts::LANE_WIDTH;
use utils::id::{NodeId, SegmentId};
use utils::VecUtils;

use super::lane::LaneMap;
use super::snap::{SnapConfig, SnapRange};
use super::NodeType;

#[derive(Clone, Debug)]
struct AttachedSegment {
    segment_id: SegmentId,
    node_type: NodeType,
    snap_range: SnapRange,
}

/// Defines if the main segment is incoming or outgoing in an asymmetric node.
#[derive(Clone, Copy, Debug)]
enum Side {
    In,
    Out,
}

#[derive(Clone, Debug)]
enum Mode {
    Sym {
        incoming: SegmentId,
        outgoing: SegmentId,
    },
    Asym {
        /// This is the main segment of an asymmetric node, the largest segment in terms of no of
        /// lanes.
        segment_id: SegmentId,
        side: Side,
        segments: Vec<AttachedSegment>,
        /// This is the lane map of lanes opposite the main segment.
        lane_map: LaneMap,
    },
}

// #[derive(Clone, Debug)]
// pub struct LNode {
//     pos: Vec3,
//     dir: Vec3,
//     /// This type corresponds with the incoming and outgoing segment in a symmetric node, and the
//     /// main segment of an asymmetric node.
//     node_type: NodeType,
//     mode: Mode,
// }

// #[derive(Clone, Copy, Debug)]
// pub struct LNodeBuilder {
//     pos: Vec3,
//     dir: Vec3,
// }

// impl LNodeBuilder {
//     pub fn new(pos: Vec3, dir: Vec3) -> Self {
//         LNodeBuilder { pos, dir }
//     }

//     /// # Panics
//     ///
//     /// The function panics if `lane_map` is `(None, None)` because you cannot construct a node
//     /// that is not connected to any segment.
//     pub fn build(
//         self,
//         node_type: NodeType,
//         lane_map: (Option<SegmentId>, Option<SegmentId>),
//     ) -> LNode {
//         let mode = match lane_map {
//             (Some(in_id), Some(out_id)) => Mode::Sym {
//                 incoming: in_id,
//                 outgoing: out_id,
//             },
//             (Some(in_id), None) => Mode::Asym {
//                 segment_id: in_id,
//                 side: Side::In,
//                 segments: vec![],
//             },
//             (None, Some(out_id)) => Mode::Asym {
//                 segment_id: out_id,
//                 side: Side::Out,
//                 segments: vec![],
//             },
//             (None, None) => panic!(),
//         };
//         LNode {
//             pos: self.pos,
//             dir: self.dir,
//             node_type,
//             mode,
//         }
//     }
// }

/// Represents a logical road node. The data is the data necessary to do logical work with a road
/// node.
///
/// INVARIANTS:
/// The length of the fields incoming_lanes and outgoing_lanes is always the same.
#[derive(Clone, Debug)]
pub struct LNode {
    pos: Vec3,
    dir: Vec3,
    incoming_lanes: LaneMap,
    outgoing_lanes: LaneMap,
}

impl LNode {
    pub fn new(pos: Vec3, dir: Vec3, incoming_lanes: LaneMap, outgoing_lanes: LaneMap) -> Self {
        Self {
            pos,
            dir,
            incoming_lanes,
            outgoing_lanes,
        }
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn get_dir(&self) -> Vec3 {
        self.dir
    }

    pub fn no_lanes(&self) -> u8 {
        #[cfg(debug_assertions)]
        assert_eq!(self.incoming_lanes.len(), self.outgoing_lanes.len());

        self.incoming_lanes.len() as u8
    }

    pub fn has_snappable_lane(&self) -> bool {
        self.outgoing_lanes.contains_none() || self.incoming_lanes.contains_none()
    }

    pub fn can_remove_segment(&self, segment_id: SegmentId, reverse: bool) -> bool {
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

    fn expand_node(&mut self, snap_config: SnapConfig, segment_id: SegmentId) {
        self.pos = snap_config.get_pos();
        if snap_config.is_reverse() {
            self.incoming_lanes
                .expand(&snap_config.get_snap_range(), Some(segment_id));
            self.outgoing_lanes
                .expand(&snap_config.get_snap_range(), None);
        } else {
            self.incoming_lanes
                .expand(&snap_config.get_snap_range(), None);
            self.outgoing_lanes
                .expand(&snap_config.get_snap_range(), Some(segment_id));
        }
    }

    pub fn update_lane_map(&mut self, snap_config: SnapConfig, segment_id: SegmentId) {
        let sized_snap_range = snap_config.get_snap_range().reduce_size(self.no_lanes());
        if snap_config.is_reverse() {
            self.incoming_lanes.update(&sized_snap_range, segment_id)
        } else {
            self.outgoing_lanes.update(&sized_snap_range, segment_id)
        }
        if snap_config.get_snap_range().len() as u8 > self.no_lanes() {
            self.expand_node(snap_config, segment_id);
        }
    }

    pub fn remove_segment_from_lane_map(&mut self, segment_id: SegmentId) {
        self.incoming_lanes.remove_segment(segment_id);
        self.outgoing_lanes.remove_segment(segment_id);
        let mut left_delete_list = vec![];
        let mut right_delete_list = vec![];

        // This code assumes that we have checked that it is not a valid segment to delete.
        for i in 0..self.incoming_lanes.len() {
            if self.incoming_lanes[i] == None && self.outgoing_lanes[i] == None {
                left_delete_list.push(i);
            } else {
                break;
            }
        }

        for i in (0..self.incoming_lanes.len()).rev() {
            if self.incoming_lanes[i] == None && self.outgoing_lanes[i] == None {
                right_delete_list.push(i);
            } else {
                break;
            }
        }

        self.pos += ((left_delete_list.len() - right_delete_list.len()) as f32 / 2.0)
            * self.dir.right_hand()
            * LANE_WIDTH;

        for &i in right_delete_list.iter() {
            self.incoming_lanes.remove(i);
            self.outgoing_lanes.remove(i);
        }
        left_delete_list.reverse();
        for &i in left_delete_list.iter() {
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

        // lane map contains some so look for snap ranges in between segments
        if lane_map.contains_some() {
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
                            snap_configs.push(SnapConfig::new(
                                node_id,
                                start_pos
                                    + (i as i8 - (no_lanes as i8 - 1)) as f32 * lane_width_dir,
                                self.dir,
                                s.clone(),
                                reverse,
                            ));
                            false
                        } else {
                            true
                        }
                    });
                } else {
                    possible_snaps = vec![];
                }
            }
            return snap_configs;
        };

        // lane_map is all nones, therefore, if we are building larger segment with more than or
        // equal no_lanes then all snap possibilities exist
        if no_lanes >= self.no_lanes() {
            let mut snap_configs = vec![];
            let diff = no_lanes - self.no_lanes();
            for i in 0..(diff + 1) {
                snap_configs.push(SnapConfig::new(
                    node_id,
                    self.pos + (i as f32 - diff as f32 / 2.0) * lane_width_dir,
                    self.dir,
                    SnapRange::create(i as i8 - diff as i8, (i + no_lanes) as i8 - diff as i8),
                    reverse,
                ));
            }
            return snap_configs;
        };

        // if we are building a segment with fewer no_lanes then we can only do it if the opposite
        // node is the same node, otherwise we create a many to many node
        if opposite_same && no_lanes < self.no_lanes() {
            let mut snap_configs = vec![];
            let diff = self.no_lanes() - no_lanes;
            for i in 0..(diff + 1) {
                snap_configs.push(SnapConfig::new(
                    node_id,
                    self.pos + (i as f32 - diff as f32 / 2.0) * lane_width_dir,
                    self.dir,
                    SnapRange::create(i as i8, (i + no_lanes) as i8),
                    reverse,
                ));
            }
            return snap_configs;
        };

        // cannot snap as the opposite is not the same segment, and this sides no_lanes is too small
        vec![]
    }

    /// Returns the {`SnapConfig`}'s of this node, given the amount of lanes that can be snapped
    /// to.
    pub fn get_snap_configs(&self, no_lanes: u8, node_id: NodeId) -> Vec<SnapConfig> {
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
