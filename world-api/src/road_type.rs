//! This module defines all the types associated with the configuration of roads. As such this file
//! exclusively defines the set of roads that can be constructed. All types are and should be
//! discrete.
use serde::{Deserialize, Serialize};

/// Defines the type of curves that are possible for roads.
///
/// TODO: add euler spirals.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CurveType {
    Straight,
    #[default]
    Curved,
}

/// This enum defines the discrete set of road widths that are possible. This is discrete, such
/// that it is easy to see if two road nodes are compatible (same lane width).
///
/// TODO: implement such that the user can add new discrete values. Could be implemented using an
/// Id system, similar to that of nodes and segments.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LaneWidth {
    Narrow,
    #[default]
    Standard,
    Wide,
}

impl LaneWidth {
    /// Returns the width of this lane as an f32.
    pub fn getf32(&self) -> f32 {
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeType {
    pub lane_width: LaneWidth,
    pub no_lanes: u8,
}

impl Default for NodeType {
    fn default() -> Self {
        Self {
            lane_width: LaneWidth::default(),
            no_lanes: 3,
        }
    }
}

impl NodeType {
    pub fn compute_width(&self) -> f32 {
        self.lane_width.getf32() * self.no_lanes as f32
    }
}

/// Defines the types of segments that can be constructed.
///
/// TODO: expand to include transition segments.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SegmentType {
    pub curve_type: CurveType,
}

impl SegmentType {
    pub fn new(curve_type: CurveType) -> Self {
        Self { curve_type }
    }
}

/// Defines the two sides of a node.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Side {
    /// The incoming side of the node.
    In,
    /// The outgoing side of the node.
    Out,
}

impl Side {
    pub fn switch(&self) -> Self {
        match self {
            Side::In => Side::Out,
            Side::Out => Side::In,
        }
    }
}
