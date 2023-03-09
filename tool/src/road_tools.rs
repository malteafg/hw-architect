mod bulldoze_tool;
mod construct_tool;
mod generator;

pub use bulldoze_tool::BulldozeTool;
pub use construct_tool::ConstructTool;

use simulation::{NodeType, SegmentType};

/// This defines a road type that is being constructed.
#[derive(Debug, Default, Clone, Copy)]
pub struct SelectedRoad {
    pub node_type: NodeType,
    pub segment_type: SegmentType,
}

impl SelectedRoad {
    pub fn new(node_type: NodeType, segment_type: SegmentType) -> Self {
        Self {
            node_type,
            segment_type,
        }
    }
}
