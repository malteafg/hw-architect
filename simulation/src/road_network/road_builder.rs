use super::{LNodeBuilder, LSegmentBuilder, NodeType, SegmentType};

/// This struct defines exactly the data that a road graph needs in order to add new segments to
/// it.
pub struct LRoadGenerator {
    node_builders: Vec<LNodeBuilder>,
    segment_builders: Vec<LSegmentBuilder>,
    node_type: NodeType,
    segment_type: SegmentType,
    reverse: bool,
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
