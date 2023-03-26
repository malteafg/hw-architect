use world::roads::{NodeType, SegmentType};

#[derive(Debug, Clone, Copy, Default)]
pub struct ToolState {
    pub road_state: RoadState,
}

#[derive(Debug, Clone, Copy)]
pub struct RoadState {
    pub selected_road: SelectedRoad,
    pub snapping: bool,
    pub reverse: bool,
}

impl Default for RoadState {
    fn default() -> Self {
        Self {
            selected_road: SelectedRoad::default(),
            snapping: true,
            reverse: false,
        }
    }
}

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
