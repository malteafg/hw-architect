use super::{NodeType, SegmentType};

use utils::curves::GuidePoints;
use utils::id::SegmentId;

use glam::*;

// #################################################################################################
// Definitions for others to construct an LNode
// #################################################################################################
#[derive(Debug, Clone, Copy)]
pub struct LNodeBuilder {
    pos: Vec3,
    dir: Vec3,
    node_type: NodeType,
}

impl LNodeBuilder {
    pub fn new(pos: Vec3, dir: Vec3, node_type: NodeType) -> Self {
        LNodeBuilder {
            pos,
            dir,
            node_type,
        }
    }

    pub fn consume(self) -> (Vec3, Vec3, NodeType) {
        (self.pos, self.dir, self.node_type)
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn dir(&self) -> Vec3 {
        self.dir
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn flip_dir(&mut self) {
        self.dir *= -1.
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
pub struct LSegmentBuilder {
    width: f32,
    segment_type: SegmentType,
    guide_points: GuidePoints,
}

impl LSegmentBuilder {
    pub fn new(width: f32, segment_type: SegmentType, guide_points: GuidePoints) -> Self {
        LSegmentBuilder {
            width,
            segment_type,
            guide_points,
        }
    }

    pub fn consume(self) -> (f32, SegmentType, GuidePoints) {
        (self.width, self.segment_type, self.guide_points)
    }

    pub fn guide_points(&self) -> &GuidePoints {
        &self.guide_points
    }
}
