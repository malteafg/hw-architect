use world_api::{LSegmentBuilder, SegmentType};

use utils::curves::{GuidePoints, SpinePoints};
use utils::id::NodeId;

use glam::Vec3;

use serde::{Deserialize, Serialize};

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
    /// The is a SpinePoints for each lane path, sorted from left to right.
    spine_points: Vec<SpinePoints>,
    from_node: NodeId,
    to_node: NodeId,
}

impl LSegment {
    fn new(
        width: f32,
        segment_type: SegmentType,
        guide_points: GuidePoints,
        spine_points: Vec<SpinePoints>,
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

    pub fn from_builder(builder: LSegmentBuilder, from_node: NodeId, to_node: NodeId) -> Self {
        // TODO fix 0.05, and figure out what to do with it.
        let (width, segment_type, guide_points) = builder.consume();
        let spine_points = vec![guide_points.get_spine_points(0.05)];
        Self::new(
            width,
            segment_type,
            guide_points,
            spine_points,
            from_node,
            to_node,
        )
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    // pub fn segment_type(&self) -> SegmentType {
    //     self.segment_type
    // }

    pub fn guide_points(&self) -> &GuidePoints {
        &self.guide_points
    }

    // pub fn spine_points(&self) -> &SpinePoints {
    //     &self.spine_points
    // }

    pub fn get_from_node(&self) -> NodeId {
        self.from_node
    }

    pub fn get_to_node(&self) -> NodeId {
        self.to_node
    }

    pub fn no_lane_paths(&self) -> u8 {
        self.spine_points.len() as u8
    }

    pub fn contains_pos(&self, pos: Vec3) -> bool {
        self.guide_points().is_inside(pos, self.width())
    }
}
