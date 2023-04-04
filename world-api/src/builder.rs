use super::{NodeType, SegmentType};

use curves::{curve_gen, GuidePoints, SpinePoints};
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
pub enum LSegmentBuilderType {
    /// Find a better naming convention for these types.
    Same(NodeType),
    // SameWidth
    // SameNoLanes
}

#[derive(Debug, Clone)]
pub struct LSegmentBuilder {
    segment_type: SegmentType,
    node_config: LSegmentBuilderType,
    guide_points: GuidePoints,
    spine_points: SpinePoints,
    spine_dirs: SpinePoints,
}

impl LSegmentBuilder {
    pub fn new(segment_type: SegmentType, node_type: NodeType, guide_points: GuidePoints) -> Self {
        let node_config = LSegmentBuilderType::Same(node_type);

        // TODO check the results of this num_of_cuts
        let num_of_cuts = (utils::consts::VERTEX_DENSITY * (1000.0 + guide_points.dist())) as u32;
        let (spine_points, spine_dirs) = curve_gen::spine_points_and_dir(
            &guide_points,
            1.0 / (num_of_cuts as f32 - 1.0),
            utils::consts::CUT_LENGTH,
            num_of_cuts,
        );

        Self {
            segment_type,
            node_config,
            guide_points,
            spine_points,
            spine_dirs,
        }
    }

    pub fn consume(
        self,
    ) -> (
        SegmentType,
        LSegmentBuilderType,
        GuidePoints,
        SpinePoints,
        SpinePoints,
    ) {
        (
            self.segment_type,
            self.node_config,
            self.guide_points,
            self.spine_points,
            self.spine_dirs,
        )
    }

    pub fn guide_points(&self) -> &GuidePoints {
        &self.guide_points
    }

    pub fn spine_points(&self) -> &SpinePoints {
        &self.spine_points
    }

    pub fn spine_dirs(&self) -> &SpinePoints {
        &self.spine_dirs
    }
}
