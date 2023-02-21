use super::lane::LaneMap;
use super::node::LNode;
use super::segment::LSegment;
use super::{NodeType, SegmentType};
use crate::curves::{GuidePoints, SpinePoints};
use glam::Vec3;
use utils::id::{NodeId, SegmentId};

#[derive(Clone, Copy)]
pub struct LNodeBuilder {
    pos: Vec3,
    dir: Vec3,
}

// #[derive(Clone, Copy, Debug)]
// pub struct LNodeBuilder {
//     pos: Vec3,
//     dir: Vec3,
// }

#[derive(Debug, Clone)]
pub struct LSegmentBuilder {
    pub segment_type: SegmentType,
    pub guide_points: GuidePoints,
    pub spine_points: SpinePoints,
}

/// This struct defines exactly the data that a road graph needs in order to add new segments to
/// it.
pub struct LRoadGenerator {
    node_builders: Vec<LNodeBuilder>,
    segment_builders: Vec<LSegmentBuilder>,
    node_type: NodeType,
    segment_type: SegmentType,
    reverse: bool,
}

impl LNodeBuilder {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        LNodeBuilder { pos, dir }
    }

    pub fn build(self, no_lanes: u8, lane_map: (Option<SegmentId>, Option<SegmentId>)) -> LNode {
        LNode::new(
            self.pos,
            self.dir,
            LaneMap::create(no_lanes, lane_map.0),
            LaneMap::create(no_lanes, lane_map.1),
        )
    }
}

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
//     // add enum type to make sure that lane map can never be None, None
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
//         LNode::new(
//             self.pos,
//             self.dir,
//             node_type,
//             mode,
//         )
//     }
// }

impl LSegmentBuilder {
    pub fn new(
        segment_type: SegmentType,
        guide_points: GuidePoints,
        spine_points: SpinePoints,
    ) -> Self {
        LSegmentBuilder {
            segment_type,
            guide_points,
            spine_points,
        }
    }

    pub fn build(self, width: f32, from_node: NodeId, to_node: NodeId) -> LSegment {
        LSegment::new(
            width,
            self.segment_type,
            self.guide_points,
            self.spine_points,
            from_node,
            to_node,
        )
    }
}

impl LRoadGenerator {
    pub fn new(
        node_builders: Vec<LNodeBuilder>,
        segment_builders: Vec<LSegmentBuilder>,
        node_type: NodeType,
        segment_type: SegmentType,
        reverse: bool,
    ) -> Self {
        Self {
            node_builders,
            segment_builders,
            node_type,
            segment_type,
            reverse,
        }
    }

    pub fn extract(
        self,
    ) -> (
        Vec<LNodeBuilder>,
        Vec<LSegmentBuilder>,
        NodeType,
        SegmentType,
        bool,
    ) {
        (
            self.node_builders,
            self.segment_builders,
            self.node_type,
            self.segment_type,
            self.reverse,
        )
    }
}
