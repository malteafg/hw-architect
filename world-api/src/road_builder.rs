use super::{NodeType, SnapConfig};

use curves::{CurveShared, CurveSum, Spine};
use utils::{
    id::SegmentId,
    math::{DirXZ, Loc},
};

use glam::Vec3;

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
    pub curve: CurveSum,
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
    pub fn new(
        nodes: Vec<LNodeBuilderType>,
        segments: Vec<LSegmentBuilder>,
        reverse: bool,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            for j in 0..(nodes.len() - 1) {
                match &nodes[j] {
                    LNodeBuilderType::New(node) => {
                        assert_eq!(node.dir(), segments[j].get_curve().first().dir)
                    }
                    LNodeBuilderType::Old(snap) => {
                        assert_eq!(snap.dir(), segments[j].get_curve().first().dir)
                    }
                }
            }
            match &nodes[nodes.len() - 1] {
                LNodeBuilderType::New(node) => {
                    assert_eq!(node.dir(), segments[nodes.len() - 2].get_curve().last().dir)
                }
                LNodeBuilderType::Old(snap) => {
                    assert_eq!(snap.dir(), segments[nodes.len() - 2].get_curve().last().dir)
                }
            }
        }

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

    pub fn get_curves(&self) -> Vec<CurveSum> {
        self.segments.clone().into_iter().map(|s| s.curve).collect()
    }
}
