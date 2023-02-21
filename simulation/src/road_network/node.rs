use glam::*;
use utils::consts::LANE_WIDTH;
use utils::id::{NodeId, SegmentId};
use utils::VecUtils;

use std::collections::VecDeque;

use super::snap::{SnapConfig, SnapRange};
use super::{NodeType, Side};

/// Defines the configuration of the segments that are attached opposite to the main side of a
/// node.
#[derive(Clone, Debug, PartialEq)]
struct AttachedSegment {
    segment_id: SegmentId,
    node_type: NodeType,
    /// These snap ranges should only have positive indexes.
    snap_range: SnapRange,
}

impl AttachedSegment {
    fn new(segment_id: SegmentId, node_type: NodeType, snap_range: SnapRange) -> Self {
        Self {
            segment_id,
            node_type,
            snap_range,
        }
    }

    fn no_lanes(&self) -> u8 {
        self.node_type.no_lanes
    }
}

/// Represents a configuration of segments that are connected to a node on one side.
///
/// # INVARIANTS
/// Attached segments are always stored in the order from left to right.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneMap {
    segment_list: VecDeque<AttachedSegment>,
    no_lanes: u8,
}

impl core::ops::Deref for LaneMap {
    type Target = VecDeque<AttachedSegment>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.segment_list
    }
}

impl core::ops::DerefMut for LaneMap {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.segment_list
    }
}

impl LaneMap {
    /// Creates a new empty lane map.
    fn empty(no_lanes: u8) -> Self {
        LaneMap {
            segment_list: VecDeque::new(),
            no_lanes,
        }
    }

    fn update_no_lanes(&mut self, no_lanes: u8) {
        self.no_lanes = no_lanes
    }

    /// Returns the index of the position of the given segment.
    fn get_index_of_segment(&self, id: SegmentId) -> u8 {
        for (i, s) in self.iter().enumerate() {
            if id == s.segment_id {
                return i as u8;
            }
        }
        // ADD LOGGING: This is an error and should never be able to happen.
        0
    }

    /// Returns the number of lanes that are closed by a segment being attached to that lane.
    fn no_lanes_closed(&self) -> u8 {
        self.iter().fold(0, |acc, s| acc + s.no_lanes())
    }

    /// Returns true if the number of lanes occupied by segments is less than the number of lanes
    /// in the node.
    fn has_opening(&self) -> bool {
        self.no_lanes_closed() < self.no_lanes
    }

    /// Returns the smallest index in the lane map. This also corresponds to the number of open
    /// slots from and including index 0. Should not be called if the lane map is empty.
    fn smallest(&self) -> u8 {
        self[0].snap_range.smallest() as u8
    }

    /// Returns the largest index in the lane map. Should not be called if the lane map is empty.
    fn largest(&self) -> u8 {
        self[self.len() - 1].snap_range.largest() as u8
    }

    /// Returns true if there are no open lanes between closed lanes.
    fn is_continuous(&self) -> bool {
        if self.len() <= 1 {
            return true;
        }
        for i in 0..self.len() - 1 {
            if self[i + 1].snap_range.smallest() - self[i].snap_range.largest() > 1 {
                return false;
            }
        }
        true
    }

    /// Returns true if there are segments on both sides of this segment in this node's
    /// configuration.
    fn is_middle_segment(&self, id: SegmentId) -> bool {
        let index = self.get_index_of_segment(id);
        index != 0 && index != self.len() as u8 - 1
    }

    /// Shifts the snap ranges of the segments, such that they are correct when a node is resized.
    fn shift(&mut self, amount: i8) {
        self.iter_mut().for_each(|s| s.snap_range.shift(amount))
    }

    /// Adds a segment to the correct position such that the segments are in order from left to
    /// right.
    fn add_segment(&mut self, new_segment: AttachedSegment) {
        let position = 0;
        for (i, s) in self.iter().enumerate() {
            if new_segment.snap_range.largest() < s.snap_range.smallest() {
                position = i;
                break;
            }
        }
        self.insert(position, new_segment);
    }

    /// Removes the given segment from the lane map.
    fn remove_segment(&mut self, segment_id: SegmentId) {
        self.retain(|s| s.segment_id != segment_id);
    }
}

#[derive(Clone, Debug)]
enum Mode {
    /// A symmetric node is where both segments are main segments, i.e. they fit exactly with the
    /// node_type of this node.
    Sym {
        incoming: SegmentId,
        outgoing: SegmentId,
    },
    /// An asymmetric node is where a main segment exists on only one side.
    Asym {
        /// This is the main segment of an asymmetric node, the largest segment in terms of no of
        /// lanes.
        main_segment: SegmentId,
        main_side: Side,
        attached_segments: LaneMap,
    },
    /// An open node is where the main segment does not exist. It is then required that there are
    /// no gaps between the segments on the side opposite of `open_side` where the main segment is
    /// supposed to go. The only scenario where {`SnapConfig`}'s can be generated for such a node,
    /// is if the snap matches this nodes type and that the number of lanes of the snap is greater
    /// or equal to that of what is currently on this open node.
    Open {
        open_side: Side,
        attached_segments: LaneMap,
    },
}

/// Represents a logical road node. The data is the data necessary to do logical work with a road
/// node.
#[derive(Clone, Debug)]
pub struct LNode {
    pos: Vec3,
    dir: Vec3,
    /// This type corresponds with the incoming and outgoing segment in a symmetric node, and the
    /// main segment of an asymmetric node.
    node_type: NodeType,
    mode: Mode,
}

use Mode::*;

impl LNode {
    pub fn new(pos: Vec3, dir: Vec3, node_type: NodeType, mode: Mode) -> Self {
        Self {
            pos,
            dir,
            node_type,
            mode,
        }
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn get_dir(&self) -> Vec3 {
        self.dir
    }

    /// Returns the number of lanes of this node's type. This is the number of lanes in the main
    /// segment.
    pub fn no_lanes(&self) -> u8 {
        self.node_type.no_lanes
    }

    /// Returns if there is any possibility of snapping a road to this node.
    pub fn can_add_some_segment(&self) -> bool {
        match self.mode {
            // You can't snap, no positions are open.
            Sym => false,
            // You can snap if there are open positions on opposite side of main segment.
            Asym {
                attached_segments, ..
            } => attached_segments.has_opening(),
            // You can snap a segment that has more lanes than that of this node's type.
            Open => true,
        }
    }

    /// A segment can only be removed, if the resulting node is not split. This would be an open
    /// node where there are emply segments in the lane_map.
    pub fn can_remove_segment(&self, segment_id: SegmentId, reverse: bool) -> bool {
        // ADD LOGGING: if segment_id is not part of this node, report error.
        match self.mode {
            Sym => true,
            Asym {
                main_segment,
                attached_segments,
                ..
            } => {
                if segment_id == main_segment {
                    attached_segments.is_continuous()
                } else {
                    true
                }
            }
            Open {
                attached_segments, ..
            } => !attached_segments.is_middle_segment(segment_id),
        }
    }

    /// A segment can only be added if `snap_config` is valid.
    pub fn add_segment(&mut self, segment_id: SegmentId, snap_config: SnapConfig) {
        // ADD LOGGING: if snap_config is invalid, that is, it does not match the snap configs
        // generated by this node's get_snap_configs function.
        // self.pos = snap_config.get_pos();

        let snap_no_lanes = snap_config.get_snap_range().len() as u8;
        match self.mode {
            Sym => {
                // ADD LOGGING: this should be impossible.
            }
            Asym {
                main_segment,
                main_side,
                attached_segments,
            } => {
                if snap_no_lanes > self.no_lanes() {
                    // Flip this asymmetric node to be asymmetric in the opposite direction.
                    // ADD LOGGING: this should only be possible if there are no attached segments.
                    #[cfg(debug_assertions)]
                    assert!(attached_segments.len() == 0);

                    // Computes the correct snap range for the old main segment.
                    let no_negatives = snap_config.get_snap_range().get_no_negatives();
                    let snap_range = snap_config.get_snap_range().clone();
                    snap_range.trim(no_negatives);
                    snap_range.shift(no_negatives as i8);

                    attached_segments.add_segment(AttachedSegment::new(
                        main_segment,
                        self.node_type,
                        snap_range,
                    ));
                    attached_segments.update_no_lanes(snap_no_lanes);

                    self.node_type = snap_config.get_node_type();
                    self.mode = Asym {
                        main_segment: segment_id,
                        main_side: main_side.switch(),
                        attached_segments,
                    }
                } else if snap_no_lanes == self.no_lanes() {
                    // Switch to be a symmetric node.
                    // ADD LOGGING: this should only be possible if there are no attached segments.
                    #[cfg(debug_assertions)]
                    assert!(attached_segments.len() == 0);

                    let (incoming, outgoing) = if main_side == Side::In {
                        (main_segment, segment_id)
                    } else {
                        (segment_id, main_segment)
                    };
                    self.mode = Sym { incoming, outgoing }
                } else {
                    let new_segment = AttachedSegment::new(
                        segment_id,
                        snap_config.get_node_type(),
                        *snap_config.get_snap_range(),
                    );
                    attached_segments.add_segment(new_segment)
                }
            }
            Open {
                open_side,
                attached_segments,
            } => {
                self.pos = snap_config.get_pos();
                self.node_type = snap_config.get_node_type();
                attached_segments.shift(snap_config.get_snap_range().get_no_negatives() as i8);
                attached_segments.update_no_lanes(snap_no_lanes);
                self.mode = Asym {
                    main_segment: segment_id,
                    main_side: open_side,
                    attached_segments,
                }
            }
        }
    }

    /// This function assumes that it has been checked that can_remove_segment returns true. The
    /// return flag signals if all segments have been removed from the node. In that case the node
    /// should be removed from the graph that it is part of.
    pub fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        // ADD LOGGING: if can_remove_segment return false the report error.
        match self.mode {
            Sym { incoming, outgoing } => {
                if incoming == segment_id {
                    self.mode = Asym {
                        main_segment: outgoing,
                        main_side: Side::Out,
                        attached_segments: LaneMap::empty(self.no_lanes()),
                    }
                } else {
                    self.mode = Asym {
                        main_segment: incoming,
                        main_side: Side::In,
                        attached_segments: LaneMap::empty(self.no_lanes()),
                    }
                }
                false
            }
            Asym {
                main_segment,
                main_side,
                attached_segments,
            } => {
                if main_segment == segment_id {
                    if attached_segments.is_empty() {
                        return true;
                    }
                    if attached_segments.len() == 1 {
                        self.node_type = attached_segments[0].node_type;
                        self.mode = Asym {
                            main_segment: attached_segments[0].segment_id,
                            main_side: main_side.switch(),
                            attached_segments: LaneMap::empty(attached_segments[0].no_lanes()),
                        };
                        return false;
                    }
                    let empty_space = attached_segments.smallest();
                    attached_segments.shift(-(empty_space as i8));
                    let new_no_lanes = attached_segments.largest() + 1;
                    attached_segments.update_no_lanes(new_no_lanes);
                    self.node_type = NodeType {
                        no_lanes: new_no_lanes,
                        ..self.node_type
                    };
                    self.mode = Open {
                        open_side: main_side,
                        attached_segments,
                    };
                    false
                } else {
                    attached_segments.remove_segment(segment_id);
                    false
                }
            }
            Open {
                open_side,
                attached_segments,
            } => {
                attached_segments.remove_segment(segment_id);

                let empty_space = attached_segments.smallest();
                attached_segments.shift(-(empty_space as i8));
                let new_no_lanes = attached_segments.largest() + 1;
                attached_segments.update_no_lanes(new_no_lanes);
                self.node_type = NodeType {
                    no_lanes: new_no_lanes,
                    ..self.node_type
                };
                // It is safe to return false, because if attached_segments is now empty, then it
                // would have been an Asym node in the first place, so this code would never have
                // been run.
                false
            }
        }
    }

    /// Constructs and returns the {`SnapConfig`}'s of this node, given the type of road that is
    /// trying to snap and the id of that node.
    fn construct_snap_configs(&self, node_type: NodeType, node_id: NodeId) -> Vec<SnapConfig> {
        match self.mode {
            Sym => vec![],
            Asym {
                main_side,
                attached_segments,
                ..
            } => {}
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

    /// Constructs and returns the {`SnapConfig`}'s of this node, given the type of road that is
    /// trying to snap and the id of that node.
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
