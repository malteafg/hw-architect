use world_api::{LNodeBuilder, LaneMapConfig, NodeType, Side, SnapConfig, SnapRange};

use utils::id::{NodeId, SegmentId};
use utils::{DirXZ, Loc};

use glam::*;
use serde::{Deserialize, Serialize};

use std::mem;

// Located at the bottom of this file.
use lanes::LaneMap;

// #################################################################################################
// Definitions of LNode itself
// #################################################################################################
#[derive(Clone, Debug, Serialize, Deserialize)]
enum Mode {
    /// A basic node is a node where only one segment connects to.
    Basic {
        main_segment: SegmentId,
        main_side: Side,
    },
    /// A symmetric node is where both segments are main segments, i.e. they fit exactly with the
    /// node_type of this node.
    Sym {
        incoming: SegmentId,
        outgoing: SegmentId,
    },
    /// An asymmetric node is where a main segment exists on only one side.
    Asym {
        /// This is the main segment of an asymmetric node, the largest segment in terms of no of
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
use Mode::*;

/// Represents a logical road node. The data is the data necessary to do logical work with a road
/// node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LNode {
    loc: Loc,
    /// This type corresponds with the incoming and outgoing segment in a symmetric node, and the
    /// main segment of an asymmetric node.
    node_type: NodeType,
    mode: Mode,
}

// #################################################################################################
// Implementation of LNode
// #################################################################################################
impl LNode {
    fn new(loc: Loc, node_type: NodeType, mode: Mode) -> Self {
        Self {
            loc,
            node_type,
            mode,
        }
    }

    pub fn from_builder(builder: LNodeBuilder, lane_map: LaneMapConfig) -> Self {
        let mode = match lane_map {
            LaneMapConfig::Sym { incoming, outgoing } => Mode::Sym { incoming, outgoing },
            LaneMapConfig::In { incoming } => Mode::Basic {
                main_segment: incoming,
                main_side: Side::In,
            },
            LaneMapConfig::Out { outgoing } => Mode::Basic {
                main_segment: outgoing,
                main_side: Side::Out,
            },
        };
        let (loc, node_type) = builder.consume();
        Self::new(loc, node_type, mode)
    }

    pub fn pos(&self) -> Vec3 {
        self.loc.pos
    }

    pub fn dir(&self) -> DirXZ {
        self.loc.dir
    }

    pub fn loc(&self) -> Loc {
        self.loc
    }

    /// Returns the number of lanes of this node's type. This is the number of lanes in the main
    /// segment.
    pub fn no_lanes(&self) -> u8 {
        self.node_type.no_lanes()
    }

    pub fn lane_width(&self) -> f32 {
        self.node_type.lane_width_f32()
    }

    fn width(&self) -> f32 {
        self.lane_width() * self.no_lanes() as f32
    }

    pub fn contains_pos(&self, pos: Vec3) -> bool {
        (self.loc.pos - pos).length() < self.width()
    }

    pub fn is_starting(&self) -> bool {
        match self.mode {
            Basic { main_side, .. } => main_side == Side::Out,
            Open { open_side, .. } => open_side == Side::In,
            _ => false,
        }
    }

    pub fn is_ending(&self) -> bool {
        match self.mode {
            Basic { main_side, .. } => main_side == Side::In,
            Open { open_side, .. } => open_side == Side::Out,
            _ => false,
        }
    }

    /// Requires that `incoming_segment` is in fact an incoming_segment of this node, and that this
    /// node has an outgoing segment.
    pub fn get_next_segment_lane(
        &self,
        incoming_segment: SegmentId,
        lane: u8,
    ) -> Option<(SegmentId, u8)> {
        match &self.mode {
            Basic { main_side, .. } => {
                #[cfg(debug_assertions)]
                assert_eq!(*main_side, Side::In);
                None
            }
            Sym { incoming, outgoing } => {
                #[cfg(debug_assertions)]
                assert_eq!(*incoming, incoming_segment);
                Some((*outgoing, lane))
            }
            Asym {
                main_segment,
                main_side,
                attached_segments,
            } => match main_side {
                Side::In => {
                    #[cfg(debug_assertions)]
                    assert_eq!(*main_segment, incoming_segment);
                    attached_segments.get_segment_at_index(lane)
                }
                Side::Out => {
                    #[cfg(debug_assertions)]
                    assert!(attached_segments.contains_segment(incoming_segment));
                    Some((
                        *main_segment,
                        attached_segments.get_lane_from_segment_and_index(incoming_segment, lane),
                    ))
                }
            },
            Open { open_side, .. } => {
                #[cfg(debug_assertions)]
                assert_eq!(*open_side, Side::Out);
                None
            }
        }
    }

    /// Returns true if the given segment_id is part of this node.
    #[cfg(debug_assertions)]
    fn contains_segment(&self, segment_id: SegmentId) -> bool {
        match &self.mode {
            Basic { main_segment, .. } => *main_segment == segment_id,
            Sym { incoming, outgoing } => {
                if segment_id == *incoming || segment_id == *outgoing {
                    return true;
                }
                false
            }
            Asym {
                main_segment,
                attached_segments,
                ..
            } => {
                if segment_id == *main_segment {
                    return true;
                }
                attached_segments.contains_segment(segment_id)
            }
            Open {
                attached_segments, ..
            } => attached_segments.contains_segment(segment_id),
        }
    }

    /// Returns if there is any possibility of snapping a road to this node.
    pub fn can_add_some_segment(&self) -> bool {
        match &self.mode {
            // You can always snap to a basic node.
            Basic { .. } => true,
            // You can't snap, no positions are open.
            Sym { .. } => false,
            // You can snap if there are open positions on opposite side of main segment.
            Asym {
                attached_segments, ..
            } => attached_segments.has_opening(),
            // You can snap a segment that has more lanes than that of this node's type.
            Open { .. } => true,
        }
    }

    /// A segment can only be removed, if the resulting node is not split. This would be an open
    /// node where there are emply segments in the lane_map.
    pub fn can_remove_segment(&self, segment_id: SegmentId) -> bool {
        #[cfg(debug_assertions)]
        assert!(self.contains_segment(segment_id));

        match &self.mode {
            Basic { .. } | Sym { .. } => true,
            Asym {
                main_segment,
                attached_segments,
                ..
            } => {
                if segment_id == *main_segment {
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
        let snap_no_lanes = snap_config.get_snap_range().len() as u8;
        let self_no_lanes = self.no_lanes();

        match &mut self.mode {
            Basic {
                main_segment,
                main_side,
            } => {
                if snap_no_lanes == self_no_lanes {
                    let (incoming, outgoing) = if *main_side == Side::In {
                        (*main_segment, segment_id)
                    } else {
                        (segment_id, *main_segment)
                    };
                    self.mode = Sym { incoming, outgoing }
                } else if snap_no_lanes > self_no_lanes {
                    // We need to be asymmetric in the opposite direction.

                    // Computes the correct snap range for the old main segment.
                    let no_negatives = snap_config.get_snap_range().no_negatives();
                    let mut new_snap_range = SnapRange::new(self_no_lanes);
                    new_snap_range.shift(no_negatives as i8);

                    let mut attached_segments = LaneMap::empty(snap_no_lanes);
                    attached_segments.add_segment(*main_segment, self.node_type, new_snap_range);

                    self.loc.pos = snap_config.pos();
                    self.node_type = snap_config.node_type();
                    self.mode = Asym {
                        main_segment: segment_id,
                        main_side: main_side.switch(),
                        attached_segments,
                    }
                } else {
                    let mut attached_segments = LaneMap::empty(self_no_lanes);
                    attached_segments.add_segment(
                        segment_id,
                        snap_config.node_type(),
                        snap_config.consume_snap_range(),
                    );

                    self.mode = Asym {
                        main_segment: *main_segment,
                        main_side: *main_side,
                        attached_segments,
                    }
                }
            }
            Sym { .. } => {
                #[cfg(debug_assertions)]
                panic!("You cannot add segments to a symmetric node");
            }
            Asym {
                attached_segments, ..
            } => {
                #[cfg(debug_assertions)]
                assert!(attached_segments.fits_snap_range(snap_config.get_snap_range()));

                attached_segments.add_segment(
                    segment_id,
                    snap_config.node_type(),
                    snap_config.consume_snap_range(),
                )
            }
            Open {
                open_side,
                attached_segments,
            } => {
                #[cfg(debug_assertions)]
                assert!(self_no_lanes <= snap_no_lanes);

                self.loc.pos = snap_config.pos();
                self.node_type = snap_config.node_type();
                attached_segments.shift(snap_config.get_snap_range().no_negatives() as i8);
                attached_segments.update_no_lanes(snap_no_lanes);
                self.mode = Asym {
                    main_segment: segment_id,
                    main_side: *open_side,
                    attached_segments: mem::take(attached_segments),
                }
            }
        }
    }

    /// This function assumes that it has been checked that can_remove_segment returns true. The
    /// return flag signals if all segments have been removed from the node. In that case the node
    /// should be removed from the graph that it is part of.
    pub fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        #[cfg(debug_assertions)]
        assert!(self.can_remove_segment(segment_id));

        let self_no_lanes = self.no_lanes();

        let lane_width_dir = Vec3::from(self.loc.dir.right_hand()) * self.lane_width();
        match &mut self.mode {
            Basic { main_segment, .. } => {
                #[cfg(debug_assertions)]
                assert!(*main_segment == segment_id);

                true
            }
            Sym { incoming, outgoing } => {
                if *incoming == segment_id {
                    self.mode = Basic {
                        main_segment: *outgoing,
                        main_side: Side::Out,
                    }
                } else {
                    self.mode = Basic {
                        main_segment: *incoming,
                        main_side: Side::In,
                    }
                }
                false
            }
            Asym {
                main_segment,
                main_side,
                attached_segments,
            } => {
                // We are not deleting the main segment, so remove from attached segments.
                if *main_segment != segment_id {
                    #[cfg(debug_assertions)]
                    assert!(attached_segments.contains_segment(segment_id));

                    attached_segments.remove_segment(segment_id);

                    if attached_segments.is_empty() {
                        self.mode = Basic {
                            main_segment: *main_segment,
                            main_side: *main_side,
                        }
                    }
                    return false;
                }
                #[cfg(debug_assertions)]
                assert!(*main_segment == segment_id);

                // We are deleting the main segment and there is only one attached segment, so we
                // must switch the side of this node.
                if attached_segments.len() == 1 {
                    let new_main = &attached_segments[0];
                    let segment_id = new_main.segment_id();

                    // Compute new node pos.
                    let left_space = new_main.snap_range().smallest();
                    let right_space = self_no_lanes as i8 - (new_main.snap_range().largest() + 1);
                    self.loc.pos += ((left_space - right_space) as f32 / 2.0) * lane_width_dir;

                    self.node_type = new_main.node_type();
                    self.mode = Basic {
                        main_segment: segment_id,
                        main_side: main_side.switch(),
                    };
                    return false;
                }

                // We remove the main segment, so we must switch to be an open node.
                let left_space = attached_segments.smallest();
                let right_space = self_no_lanes - (attached_segments.largest() + 1);
                self.loc.pos +=
                    ((left_space as i8 - right_space as i8) as f32 / 2.0) * lane_width_dir;

                let new_no_lanes = attached_segments.largest() + 1;

                attached_segments.shift(-(left_space as i8));
                attached_segments.update_no_lanes(new_no_lanes);
                self.node_type = NodeType::new(self.node_type.lane_width(), new_no_lanes);
                self.mode = Open {
                    open_side: *main_side,
                    attached_segments: mem::take(attached_segments),
                };
                false
            }
            Open {
                open_side,
                attached_segments,
            } => {
                attached_segments.remove_segment(segment_id);

                // We must move to be an Asym node, as there is only one segment in the node now.
                if attached_segments.len() == 1 {
                    let new_main = &attached_segments[0];
                    let segment_id = new_main.segment_id();

                    // Compute new node pos.
                    let left_space = new_main.snap_range().smallest();
                    let right_space = self_no_lanes as i8 - (new_main.snap_range().largest() + 1);
                    self.loc.pos += ((left_space - right_space) as f32 / 2.0) * lane_width_dir;

                    self.node_type = new_main.node_type();
                    self.mode = Basic {
                        main_segment: segment_id,
                        main_side: open_side.switch(),
                    };

                    return false;
                }

                let empty_space = attached_segments.smallest();
                attached_segments.shift(-(empty_space as i8));

                let new_no_lanes = attached_segments.largest() + 1;
                attached_segments.update_no_lanes(new_no_lanes);

                let pos_shift_change = if empty_space == 0 {
                    -((self.node_type.no_lanes() - new_no_lanes) as i8)
                } else {
                    empty_space as i8
                };
                self.loc.pos += (pos_shift_change as f32 / 2.0) * lane_width_dir;

                self.node_type = NodeType::new(self.node_type.lane_width(), new_no_lanes);
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
        // TODO in the future we should generate a transition segment probably
        if self.node_type.lane_width_f32() != node_type.lane_width_f32() {
            return vec![];
        }

        let lane_width_dir = Vec3::from(self.loc.dir.right_hand()) * self.lane_width();
        let snap_no_lanes = node_type.no_lanes();

        let (snap_ranges_with_pos, side): (Vec<(SnapRange, Vec3)>, Side) = match &self.mode {
            Basic { main_side, .. } => {
                let snap_ranges_with_pos = Self::gen_snap_range_and_pos(
                    self.no_lanes(),
                    snap_no_lanes,
                    self.loc.pos,
                    lane_width_dir,
                );
                (snap_ranges_with_pos, main_side.switch())
            }
            Sym { .. } => return vec![],
            Asym {
                main_side,
                attached_segments,
                ..
            } => {
                let mut snap_ranges_with_pos = Self::gen_snap_range_and_pos(
                    self.no_lanes(),
                    snap_no_lanes,
                    self.loc.pos,
                    lane_width_dir,
                );
                snap_ranges_with_pos
                    .retain(|(snap_range, _)| attached_segments.fits_snap_range(snap_range));

                (snap_ranges_with_pos, main_side.switch())
            }
            Open { open_side, .. } => {
                if snap_no_lanes < self.no_lanes() {
                    return vec![];
                }
                let snap_ranges_with_pos = Self::gen_snap_range_and_pos(
                    self.no_lanes(),
                    snap_no_lanes,
                    self.loc.pos,
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
                    node_id,
                    node_type,
                    Loc::new(pos, self.loc.dir),
                    snap_range,
                    side,
                ))
            });

        configs
    }
}

// #################################################################################################
// Implementation of lanes
// #################################################################################################
mod lanes {
    use super::*;

    /// Defines the configuration of the segments that are attached opposite to the main side of a
    /// node.
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub(super) struct AttachedSegment {
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

        pub fn no_lanes(&self) -> u8 {
            self.node_type.no_lanes()
        }

        pub fn segment_id(&self) -> SegmentId {
            self.segment_id
        }

        pub fn snap_range(&self) -> &SnapRange {
            &self.snap_range
        }

        pub fn smallest(&self) -> u8 {
            self.snap_range().smallest() as u8
        }

        pub fn largest(&self) -> u8 {
            self.snap_range().largest() as u8
        }

        pub fn contains(&self, index: u8) -> bool {
            #[cfg(debug_assertions)]
            assert!(self.snap_range().smallest() >= 0);
            self.snap_range().contains(index as i8)
        }

        pub fn node_type(&self) -> NodeType {
            self.node_type
        }
    }

    /// Represents a configuration of segments that are connected to a node on one side.
    ///
    /// # INVARIANTS
    /// Attached segments are always stored in the order from left to right.
    #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
    pub(super) struct LaneMap {
        segment_list: Vec<AttachedSegment>,
        no_lanes: u8,
    }

    impl core::ops::Deref for LaneMap {
        type Target = Vec<AttachedSegment>;

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
        pub fn empty(no_lanes: u8) -> Self {
            LaneMap {
                segment_list: Vec::new(),
                no_lanes,
            }
        }

        pub fn update_no_lanes(&mut self, no_lanes: u8) {
            self.no_lanes = no_lanes
        }

        /// Returns the index of the position of the given segment.
        fn get_index_of_segment(&self, id: SegmentId) -> u8 {
            for (i, s) in self.iter().enumerate() {
                if id == s.segment_id {
                    return i as u8;
                }
            }
            panic!("Requested segment_id does not exist in node");
        }

        /// Returns the lane number of this node for which the lane given by `id` and
        /// `index` is located at.
        pub fn get_lane_from_segment_and_index(&self, id: SegmentId, index: u8) -> u8 {
            for s in self.iter() {
                if id == s.segment_id {
                    return s.smallest() + index;
                }
            }
            panic!("Requested segment_id does not exist in node");
        }

        pub fn get_segment_at_index(&self, index: u8) -> Option<(SegmentId, u8)> {
            for s in self.iter() {
                if s.contains(index) {
                    let new_index = index - s.smallest();
                    return Some((s.segment_id(), new_index));
                }
            }
            None
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

        /// Returns true if the given segment_id is part of this lane map.
        #[cfg(debug_assertions)]
        pub fn contains_segment(&self, segment_id: SegmentId) -> bool {
            for s in self.iter() {
                if s.segment_id == segment_id {
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
        pub fn has_opening(&self) -> bool {
            self.no_lanes_closed() < self.no_lanes
        }

        /// Returns the smallest index in the lane map. This also corresponds to the number of open
        /// slots from and including index 0. Should not be called if the lane map is empty.
        pub fn smallest(&self) -> u8 {
            self[0].snap_range.smallest() as u8
        }

        /// Returns the largest index in the lane map. Should not be called if the lane map is empty.
        pub fn largest(&self) -> u8 {
            self[self.len() - 1].snap_range.largest() as u8
        }

        /// Returns true if there are no open lanes between closed lanes.
        pub fn is_continuous(&self) -> bool {
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
        pub fn is_middle_segment(&self, id: SegmentId) -> bool {
            let index = self.get_index_of_segment(id);
            index != 0 && index != self.len() as u8 - 1
        }

        /// Shifts the snap ranges of the segments, such that they are correct when a node is resized.
        pub fn shift(&mut self, amount: i8) {
            self.iter_mut().for_each(|s| s.snap_range.shift(amount))
        }

        /// Adds a segment to the correct position such that the segments are in order from left to
        /// right.
        pub fn add_segment(&mut self, id: SegmentId, node_type: NodeType, snap_range: SnapRange) {
            let new_segment = AttachedSegment::new(id, node_type, snap_range);
            let new_largest = new_segment.snap_range.largest();
            let new_smallest = new_segment.snap_range.smallest();

            if self.is_empty() {
                self.push(new_segment);
            } else if new_largest < self.smallest() as i8 {
                self.insert(0, new_segment);
            } else if new_smallest > self.largest() as i8 {
                self.push(new_segment);
            } else {
                for i in 0..self.len() - 1 {
                    if new_largest < self[i + 1].snap_range.smallest()
                        && new_smallest > self[i].snap_range.largest()
                    {
                        self.insert(i + 1, new_segment);
                        break;
                    }
                }
            }
        }

        /// Removes the given segment from the lane map.
        pub fn remove_segment(&mut self, segment_id: SegmentId) {
            self.retain(|s| s.segment_id != segment_id);
        }

        /// Checks if this lane map has space for the given snap_range.
        pub fn fits_snap_range(&self, snap_range: &SnapRange) -> bool {
            for s in snap_range.iter() {
                if self.contains_snap(*s) {
                    return false;
                }
            }
            true
        }
    }
}
