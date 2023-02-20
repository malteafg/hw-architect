//! This module defines all the types associated with the configuration of roads. As such this file
//! exclusively defines the set of roads that can be constructed. All types are and should be
//! discrete.

/// probably move to tool / generator
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

/// Defines the type of curves that are possible for roads.
///
/// TODO: add euler spirals.
#[derive(Debug, Default, Clone, Copy)]
pub enum CurveType {
    #[default]
    Straight,
    Curved,
}

/// This enum defines the discrete set of road widths that are possible. This is discrete, such
/// that it is easy to see if two road nodes are compatible (same lane width).
///
/// TODO: implement such that the user can add new discrete values. Could be implemented using an
/// Id system, similar to that of nodes and segments.
#[derive(Debug, Default, Clone, Copy)]
pub enum LaneWidth {
    Narrow,
    #[default]
    Standard,
    Wide,
}

impl LaneWidth {
    /// Returns the width of this lane as an f32.
    pub fn get(&self) -> f32 {
        use LaneWidth::*;
        match self {
            Narrow => 2.8,
            Standard => 3.5,
            Wide => 4.0,
        }
    }
}

/// Defines the types of nodes that are possible. Two nodes are compatible if they have the same
/// lane width.
#[derive(Debug, Default, Clone, Copy)]
pub struct NodeType {
    pub lane_width: f32,
    pub no_lanes: u8,
}

/// Defines the types of segments that can be constructed.
///
/// TODO: expand to include transition segments.
#[derive(Debug, Default, Clone, Copy)]
pub struct SegmentType {
    pub curve_type: CurveType,
}
