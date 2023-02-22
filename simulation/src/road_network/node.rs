use glam::*;
use utils::consts::LANE_WIDTH;
use utils::id::{NodeId, SegmentId};
use utils::VecUtils;

use std::collections::VecDeque;

use super::snap::{SnapConfig, SnapRange};
use super::{NodeType, Side};

#[derive(Clone, Copy)]
pub struct LNodeBuilder {
    pos: Vec3,
    dir: Vec3,
}

impl LNodeBuilder {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        LNodeBuilder { pos, dir }
    }

    /// # Panics
    ///
    /// The function panics if `lane_map` is `(None, None)` because you cannot construct a node
    /// that is not connected to any segment.
    pub fn build(
        self,
        node_type: NodeType,
        lane_map: (Option<SegmentId>, Option<SegmentId>),
    ) -> LNode {
        // add enum type to make sure that lane map can never be None, None
        let mode = match lane_map {
            (Some(in_id), Some(out_id)) => Mode::Sym {
                incoming: in_id,
                outgoing: out_id,
            },
            (Some(in_id), None) => Mode::Asym {
                main_segment: in_id,
                main_side: Side::In,
            },
            (None, Some(out_id)) => Mode::Asym {
                main_segment: out_id,
                main_side: Side::Out,
            },
            (None, None) => panic!(),
        };
        LNode::new(
            self.pos,
            self.dir,
            node_type,
            mode,
            LaneMap::empty(node_type.no_lanes),
        )
    }
}

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
struct LaneMap {
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

    /// Returns true if this lane map contains the given snap, i.e. that this snap point is
    /// occupied by a segment.
    fn contains_snap(&self, snap: i8) -> bool {
        for s in self.iter() {
            if s.snap_range.contains(snap) {
                return true;
            }
        }
        false
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
        let mut position = 0;
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

    /// Checks if this lane map has space for the given snap_range.
    fn fits_snap_range(&self, snap_range: &SnapRange) -> bool {
        for s in snap_range.iter() {
            if self.contains_snap(*s) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub enum Mode {
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
    },
    /// An open node is where the main segment does not exist. It is then required that there are
    /// no gaps between the segments on the side opposite of `open_side` where the main segment is
    /// supposed to go. The only scenario where {`SnapConfig`}'s can be generated for such a node,
    /// is if the snap matches this nodes type and that the number of lanes of the snap is greater
    /// or equal to that of what is currently on this open node.
    Open { open_side: Side },
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
    attached_segments: LaneMap,
}

use Mode::*;

impl LNode {
    fn new(
        pos: Vec3,
        dir: Vec3,
        node_type: NodeType,
        mode: Mode,
        attached_segments: LaneMap,
    ) -> Self {
        Self {
            pos,
            dir,
            node_type,
            mode,
            attached_segments,
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
        match &self.mode {
            // You can't snap, no positions are open.
            Sym { .. } => false,
            // You can snap if there are open positions on opposite side of main segment.
            Asym { .. } => self.attached_segments.has_opening(),
            // You can snap a segment that has more lanes than that of this node's type.
            Open { .. } => true,
        }
    }

    /// A segment can only be removed, if the resulting node is not split. This would be an open
    /// node where there are emply segments in the lane_map.
    pub fn can_remove_segment(&self, segment_id: SegmentId) -> bool {
        // ADD LOGGING: if segment_id is not part of this node, report error.
        match &self.mode {
            Sym { .. } => true,
            Asym { main_segment, .. } => {
                if segment_id == *main_segment {
                    self.attached_segments.is_continuous()
                } else {
                    true
                }
            }
            Open { .. } => !self.attached_segments.is_middle_segment(segment_id),
        }
    }

    /// A segment can only be added if `snap_config` is valid.
    pub fn add_segment(&mut self, segment_id: SegmentId, snap_config: SnapConfig) {
        // ADD LOGGING: if snap_config is invalid, that is, it does not match the snap configs
        // generated by this node's get_snap_configs function.
        // self.pos = snap_config.get_pos();

        let snap_no_lanes = snap_config.get_snap_range().len() as u8;
        match &self.mode {
            Sym { .. } => {
                // ADD LOGGING: this should be impossible.
            }
            Asym {
                main_segment,
                main_side,
                ..
            } => {
                if snap_no_lanes > self.no_lanes() {
                    // Flip this asymmetric node to be asymmetric in the opposite direction.
                    // ADD LOGGING: this should only be possible if there are no attached segments.
                    #[cfg(debug_assertions)]
                    assert!(self.attached_segments.len() == 0);

                    // Computes the correct snap range for the old main segment.
                    let no_negatives = snap_config.get_snap_range().get_no_negatives();
                    let mut snap_range = snap_config.get_snap_range().clone();
                    snap_range.trim(no_negatives);
                    snap_range.shift(no_negatives as i8);

                    self.attached_segments.add_segment(AttachedSegment::new(
                        *main_segment,
                        self.node_type,
                        snap_range,
                    ));
                    self.attached_segments.update_no_lanes(snap_no_lanes);

                    self.node_type = snap_config.get_node_type();
                    self.mode = Asym {
                        main_segment: segment_id,
                        main_side: main_side.switch(),
                    }
                } else if snap_no_lanes == self.no_lanes() {
                    // Switch to be a symmetric node.
                    // ADD LOGGING: this should only be possible if there are no attached segments.
                    #[cfg(debug_assertions)]
                    assert!(self.attached_segments.len() == 0);

                    let (incoming, outgoing) = if *main_side == Side::In {
                        (*main_segment, segment_id)
                    } else {
                        (segment_id, *main_segment)
                    };
                    self.attached_segments = LaneMap::empty(self.no_lanes());
                    self.mode = Sym { incoming, outgoing }
                } else {
                    let new_segment = AttachedSegment::new(
                        segment_id,
                        snap_config.get_node_type(),
                        snap_config.consume_snap_range(),
                    );
                    self.attached_segments.add_segment(new_segment)
                }
            }
            Open { open_side } => {
                self.pos = snap_config.get_pos();
                self.node_type = snap_config.get_node_type();
                self.attached_segments
                    .shift(snap_config.get_snap_range().get_no_negatives() as i8);
                self.attached_segments.update_no_lanes(snap_no_lanes);
                self.mode = Asym {
                    main_segment: segment_id,
                    main_side: *open_side,
                }
            }
        }
    }

    /// This function assumes that it has been checked that can_remove_segment returns true. The
    /// return flag signals if all segments have been removed from the node. In that case the node
    /// should be removed from the graph that it is part of.
    pub fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        // ADD LOGGING: if can_remove_segment return false the report error.
        match &self.mode {
            Sym { incoming, outgoing } => {
                if *incoming == segment_id {
                    self.mode = Asym {
                        main_segment: *outgoing,
                        main_side: Side::Out,
                    }
                } else {
                    self.mode = Asym {
                        main_segment: *incoming,
                        main_side: Side::In,
                    }
                }
                false
            }
            Asym {
                main_segment,
                main_side,
            } => {
                if *main_segment != segment_id {
                    self.attached_segments.remove_segment(segment_id);
                    return false;
                }
                if self.attached_segments.is_empty() {
                    return true;
                }
                if self.attached_segments.len() == 1 {
                    self.node_type = self.attached_segments[0].node_type;
                    let segment_id = self.attached_segments[0].segment_id;
                    let no_lanes = self.attached_segments[0].no_lanes();
                    self.attached_segments = LaneMap::empty(no_lanes);
                    self.mode = Asym {
                        main_segment: segment_id,
                        main_side: main_side.switch(),
                    };
                    return false;
                }
                let empty_space = self.attached_segments.smallest();
                self.attached_segments.shift(-(empty_space as i8));
                let new_no_lanes = self.attached_segments.largest() + 1;
                self.attached_segments.update_no_lanes(new_no_lanes);
                self.node_type = NodeType {
                    no_lanes: new_no_lanes,
                    ..self.node_type
                };
                self.mode = Open {
                    open_side: *main_side,
                };
                false
            }
            Open { .. } => {
                self.attached_segments.remove_segment(segment_id);

                let empty_space = self.attached_segments.smallest();
                self.attached_segments.shift(-(empty_space as i8));
                let new_no_lanes = self.attached_segments.largest() + 1;
                self.attached_segments.update_no_lanes(new_no_lanes);
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

    /// Generates snap ranges and associated positions.
    fn gen_snap_range_and_pos(
        self_no_lanes: u8,
        snap_no_lanes: u8,
        node_pos: Vec3,
        lane_width_dir: Vec3,
    ) -> Vec<(SnapRange, Vec3)> {
        let lane_diff = snap_no_lanes as i8 - self_no_lanes as i8;
        let (lane_diff, base_shift) = if lane_diff < 0 {
            (-lane_diff, 0)
        } else {
            (lane_diff, lane_diff)
        };
        let start_pos = node_pos - (lane_diff as f32 / 2.0) * lane_width_dir;

        let mut snap_ranges_with_pos = vec![];
        for i in 0..=lane_diff {
            let mut snap_range = SnapRange::new(snap_no_lanes);
            snap_range.shift(i - base_shift);
            let pos = start_pos + i as f32 * lane_width_dir;
            snap_ranges_with_pos.push((snap_range, pos));
        }
        snap_ranges_with_pos
    }

    /// Constructs and returns the {`SnapConfig`}'s of this node, given the type of road that is
    /// trying to snap and the id of this node.
    pub fn construct_snap_configs(&self, node_type: NodeType, node_id: NodeId) -> Vec<SnapConfig> {
        let lane_width_dir = self.dir.right_hand() * LANE_WIDTH;
        let snap_no_lanes = node_type.no_lanes;

        let (snap_ranges_with_pos, side) = match &self.mode {
            Sym { .. } => return vec![],
            Asym { main_side, .. } => {
                let mut snap_ranges_with_pos = Self::gen_snap_range_and_pos(
                    self.no_lanes(),
                    snap_no_lanes,
                    self.pos,
                    lane_width_dir,
                );
                snap_ranges_with_pos
                    .retain(|(snap_range, _)| self.attached_segments.fits_snap_range(snap_range));

                (snap_ranges_with_pos, main_side.switch())
            }
            Open { open_side, .. } => {
                if snap_no_lanes < self.no_lanes() {
                    return vec![];
                }
                let snap_ranges_with_pos = Self::gen_snap_range_and_pos(
                    self.no_lanes(),
                    snap_no_lanes,
                    self.pos,
                    lane_width_dir,
                );
                (snap_ranges_with_pos, *open_side)
            }
        };
        let mut configs = vec![];
        snap_ranges_with_pos
            .into_iter()
            .for_each(|(snap_range, pos)| {
                configs.push(SnapConfig::new(
                    node_id, node_type, pos, self.dir, snap_range, side,
                ))
            });

        configs
    }
}
