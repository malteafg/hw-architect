use super::SegmentType;
use crate::curves::{GuidePoints, SpinePoints};
use serde::{Deserialize, Serialize};
use utils::id::NodeId;

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

#[derive(Debug, Clone)]
pub struct LSegmentBuilder {
    pub segment_type: SegmentType,
    pub guide_points: GuidePoints,
    pub spine_points: SpinePoints,
}

// #################################################################################################
// Implementation of LSegment
// #################################################################################################
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

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn get_type(&self) -> SegmentType {
        self.segment_type
    }

    pub fn get_guide_points(&self) -> &GuidePoints {
        &self.guide_points
    }

    pub fn get_spine_points(&self) -> &SpinePoints {
        &self.spine_points
    }

    pub fn get_from_node(&self) -> NodeId {
        self.from_node
    }

    pub fn get_to_node(&self) -> NodeId {
        self.to_node
    }
}

// #################################################################################################
// Implementation of LSegmentBuilder
// #################################################################################################
impl LSegmentBuilder {
    pub fn new(
        segment_type: SegmentType,
        guide_points: GuidePoints,
        spine_points: SpinePoints,
    ) -> Self {
        LSegmentBuilder {
            segment_type,
            guide_points,
            spine_points,
        }
    }

    pub fn build(self, width: f32, from_node: NodeId, to_node: NodeId) -> LSegment {
        LSegment::new(
            width,
            self.segment_type,
            self.guide_points,
            self.spine_points,
            from_node,
            to_node,
        )
    }
}
