use super::segment::LSegment;
use super::{LNodeBuilder, NodeType, SegmentType};
use crate::curves::{GuidePoints, SpinePoints};
use utils::id::NodeId;

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
