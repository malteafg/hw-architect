mod bulldoze_tool;
mod construct_tool;
mod generator;

pub(crate) use bulldoze_tool::BulldozeTool;
pub(crate) use construct_tool::ConstructTool;

use simulation::{CurveType, NodeType, SegmentType};

/// This defines a road type that is being constructed.
#[derive(Debug, Default, Clone, Copy)]
pub struct SelectedRoad {
    pub node_type: NodeType,
    pub segment_type: SegmentType,
}

impl SelectedRoad {
    pub fn new(lane_width: f32, no_lanes: u8, curve_type: CurveType) -> Self {
        let node_type = NodeType {
            lane_width,
            no_lanes,
        };
        let segment_type = SegmentType { curve_type };
        Self {
            node_type,
            segment_type,
        }
    }
}
