use super::{LNodeBuilder, LSegmentBuilder, NodeType, SnapConfig};

use utils::{DirXZ, Loc};

use glam::Vec3;

/// TODO add better error types.
pub enum RoadGenErr {
    Placeholder,
    CCSFailed,
    DoubleSnapFailed,
    Collision,
}

#[derive(Debug, Clone)]
pub enum LNodeBuilderType {
    New(LNodeBuilder),
    Old(SnapConfig),
}

impl LNodeBuilderType {
    pub fn new(loc: Loc, node_type: NodeType) -> Self {
        New(LNodeBuilder::new(loc, node_type))
    }

    fn _get_pos(&self) -> Vec3 {
        match self {
            New(node_builder) => node_builder.pos(),
            Old(snap_config) => snap_config.pos(),
        }
    }

    fn _get_pos_and_dir(&self) -> (Vec3, DirXZ) {
        match self {
            New(node_builder) => (node_builder.pos(), node_builder.dir()),
            Old(snap_config) => (snap_config.pos(), snap_config.dir()),
        }
    }

    fn node_type(&self) -> NodeType {
        match self {
            New(b) => b.node_type(),
            Old(s) => s.node_type(),
        }
    }
}

/// This struct defines exactly the data that a road graph needs in order to add new segments to
/// it.
/// Nodes and segments are generated in the direction that the car drives.
/// This should always only be able to generate a valid road.
/// There is always one more node than segment.
#[derive(Debug, Clone)]
pub struct LRoadBuilder {
    nodes: Vec<LNodeBuilderType>,
    segments: Vec<LSegmentBuilder>,
    reverse: bool,
}

use LNodeBuilderType::*;

impl LRoadBuilder {
    fn new(nodes: Vec<LNodeBuilderType>, segments: Vec<LSegmentBuilder>, reverse: bool) -> Self {
        Self {
            nodes,
            segments,
            reverse,
        }
    }

    pub fn consume(self) -> (Vec<LNodeBuilderType>, Vec<LSegmentBuilder>, bool) {
        (self.nodes, self.segments, self.reverse)
    }

    /// NOTE: temporary should be removed once transition segments
    pub fn get_first_node_type(&self) -> NodeType {
        self.nodes[0].node_type()
    }

    pub fn get_segments(&self) -> &Vec<LSegmentBuilder> {
        &self.segments
    }
}
