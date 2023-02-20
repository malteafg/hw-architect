mod graph;
mod lane;
mod node;
mod road_type;
mod segment;
mod snap;

pub use graph::RoadGraph;
pub use node::LNodeBuilder;
pub use road_type::{CurveType, LaneWidth, NodeType, SegmentType, SelectedRoad};
pub use segment::LSegmentBuilder;
pub use snap::SnapConfig;

/// Probably temporary
pub trait RoadGen {
    fn extract(self) -> (Vec<LNodeBuilder>, Vec<LSegmentBuilder>, SelectedRoad, bool);
}
