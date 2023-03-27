use super::SegmentType;
use utils::curves::{GuidePoints, SpinePoints};

use utils::id::NodeId;

use glam::Vec3;

use serde::{Deserialize, Serialize};

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

    pub fn guide_points(&self) -> &GuidePoints {
        &self.guide_points
    }

    pub fn build(self, from_node: NodeId, to_node: NodeId) -> LSegment {
        // TODO fix 0.05, and figure out what to do with it.
        let spine_points = self.guide_points.get_spine_points(0.05);
        LSegment::new(
            self.width,
            self.segment_type,
            self.guide_points,
            spine_points,
            from_node,
            to_node,
        )
    }
}

// #################################################################################################
// Implementation of LSegment
// #################################################################################################
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSegment {
    /// This field is used for checking if a position is inside this road segment.
    ///
    /// TODO: use smarter format than f32, such that width changes according to transition segments
    /// once those are implemented.
    width: f32,
    segment_type: SegmentType,
    guide_points: GuidePoints,
    spine_points: SpinePoints,
    from_node: NodeId,
    to_node: NodeId,
}

impl LSegment {
    pub fn new(
        width: f32,
        segment_type: SegmentType,
        guide_points: GuidePoints,
        spine_points: SpinePoints,
        from_node: NodeId,
        to_node: NodeId,
    ) -> Self {
        LSegment {
            width,
            segment_type,
            guide_points,
            spine_points,
            from_node,
            to_node,
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn segment_type(&self) -> SegmentType {
        self.segment_type
    }

    pub fn guide_points(&self) -> &GuidePoints {
        &self.guide_points
    }

    pub fn spine_points(&self) -> &SpinePoints {
        &self.spine_points
    }

    pub fn get_from_node(&self) -> NodeId {
        self.from_node
    }

    pub fn get_to_node(&self) -> NodeId {
        self.to_node
    }

    pub fn contains_pos(&self, pos: Vec3) -> bool {
        self.guide_points().is_inside(pos, self.width())
    }
}
