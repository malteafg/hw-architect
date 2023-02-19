mod graph;
mod lanes;
mod node;
mod segment;
mod snap;

pub use graph::RoadGraph;
pub use node::LNodeBuilder;
pub use segment::LSegmentBuilder;
pub use snap::SnapConfig;

/// Probably temporary
pub trait RoadGen {
    fn extract(self) -> (Vec<LNodeBuilder>, Vec<LSegmentBuilder>, RoadType, bool);
}

/// The road types should probably be moved to tool.
#[derive(Debug, Default, Clone, Copy)]
pub enum CurveType {
    #[default]
    Straight,
    Curved,
}

/// The road types should probably be moved to tool.
#[derive(Debug, Default, Clone, Copy)]
pub struct RoadType {
    pub no_lanes: u8,
    pub curve_type: CurveType,
}
