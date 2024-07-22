use super::NodeType;

use curves::{CurveShared, CurveSum, Spine};
use utils::{id::SegmentId, DirXZ, Loc};

use glam::*;

// #################################################################################################
// Definitions for others to construct an LNode
// #################################################################################################
#[derive(Debug, Clone, Copy)]
pub struct LNodeBuilder {
    loc: Loc,
    node_type: NodeType,
}

impl LNodeBuilder {
    pub fn new(loc: Loc, node_type: NodeType) -> Self {
        LNodeBuilder { loc, node_type }
    }

    pub fn consume(self) -> (Loc, NodeType) {
        (self.loc, self.node_type)
    }

    pub fn pos(&self) -> Vec3 {
        self.loc.pos
    }

    pub fn dir(&self) -> DirXZ {
        self.loc.dir
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn flip_dir(&mut self) {
        self.loc.dir.flip(true);
    }
}

/// Specifies the configuration of segments when a new node is created.
pub enum LaneMapConfig {
    Sym {
        incoming: SegmentId,
        outgoing: SegmentId,
    },
    In {
        incoming: SegmentId,
    },
    Out {
        outgoing: SegmentId,
    },
}

// #################################################################################################
// Definition for others to construct an LSegment
// #################################################################################################
#[derive(Debug, Clone)]
pub enum LSegmentBuilderType {
    /// Find a better naming convention for these types.
    Same(NodeType),
    // SameWidth
    // SameNoLanes
}

#[derive(Debug, Clone)]
pub struct LSegmentBuilder {
    node_config: LSegmentBuilderType,
    curve: CurveSum,
}

impl LSegmentBuilder {
    pub fn new(node_type: NodeType, curve: CurveSum) -> Self {
        let node_config = LSegmentBuilderType::Same(node_type);

        Self { node_config, curve }
    }

    pub fn consume(self) -> (LSegmentBuilderType, CurveSum) {
        (self.node_config, self.curve)
    }

    pub fn get_curve(&self) -> &CurveSum {
        &self.curve
    }

    pub fn get_spine(&self) -> &Spine {
        &self.curve.get_spine()
    }
}
