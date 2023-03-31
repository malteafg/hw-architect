use world::roads::{NodeType, SegmentType};

#[derive(Debug, Clone, Copy, Default)]
pub struct ToolState {
    pub road_state: RoadState,
    pub bulldoze_state: BulldozeState,
}

#[derive(Debug, Clone, Copy)]
pub struct BulldozeState {
    pub bulldoze_segments: bool,
    pub bulldoze_trees: bool,
}

impl Default for BulldozeState {
    fn default() -> Self {
        Self {
            bulldoze_segments: true,
            bulldoze_trees: true,
        }
    }
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
    pub fn _new(node_type: NodeType, segment_type: SegmentType) -> Self {
        Self {
            node_type,
            segment_type,
        }
    }
}
