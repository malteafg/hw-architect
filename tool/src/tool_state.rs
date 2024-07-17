use world_api::{LaneWidth, NodeType};

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

impl RoadState {
    pub fn set_curve_type(&mut self, curve_type: CurveType) {
        self.selected_road.curve_type = curve_type;
    }

    pub fn set_lane_width(&mut self, lane_width: LaneWidth) {
        self.selected_road.node_type.set_lane_width(lane_width);
    }

    pub fn set_no_lanes(&mut self, no_lanes: u8) {
        self.selected_road.node_type.set_no_lanes(no_lanes);
    }
}

/// The type of curve to be constructed
#[derive(Debug, Default, Clone, Copy)]
pub enum CurveType {
    Straight,
    #[default]
    Circular,
}

/// This defines a road type that is being constructed.
#[derive(Debug, Default, Clone, Copy)]
pub struct SelectedRoad {
    pub node_type: NodeType,
    pub curve_type: CurveType,
}

impl SelectedRoad {
    pub fn _new(node_type: NodeType, curve_type: CurveType) -> Self {
        Self {
            node_type,
            curve_type,
        }
    }
}
