use super::{LNodeBuilder, LSegmentBuilder, SnapConfig};

pub enum LNodeBuilderType {
    New(LNodeBuilder),
    Old(SnapConfig),
}

/// This struct defines exactly the data that a road graph needs in order to add new segments to
/// it.
/// Nodes and segments are generated in the direction that the car drives.
/// This should always only be able to generate a valid road.
/// There is always one more node than segment.
pub struct LRoadBuilder {
    nodes: Vec<LNodeBuilderType>,
    segments: Vec<LSegmentBuilder>,
    reverse: bool,
}

impl LRoadBuilder {
    pub fn new(
        nodes: Vec<LNodeBuilderType>,
        segments: Vec<LSegmentBuilder>,
        reverse: bool,
    ) -> Self {
        Self {
            nodes,
            segments,
            reverse,
        }
    }

    pub fn consume(self) -> (Vec<LNodeBuilderType>, Vec<LSegmentBuilder>, bool) {
        (self.nodes, self.segments, self.reverse)
    }
}
