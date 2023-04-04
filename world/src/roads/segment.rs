use world_api::{LSegmentBuilder, LSegmentBuilderType, SegmentType};

use curves::{GuidePoints, Spine};
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
    spine: Spine,
    from_node: NodeId,
    to_node: NodeId,
}

impl LSegment {
    fn new(
        width: f32,
        segment_type: SegmentType,
        guide_points: GuidePoints,
        spine: Spine,
        from_node: NodeId,
        to_node: NodeId,
    ) -> Self {
        LSegment {
            width,
            segment_type,
            guide_points,
            spine,
            from_node,
            to_node,
        }
    }

    pub fn from_builder(builder: LSegmentBuilder, from_node: NodeId, to_node: NodeId) -> Self {
        let (segment_type, node_config, guide_points, spine) = builder.consume();

        let width = match node_config {
            LSegmentBuilderType::Same(node_type) => node_type.compute_width(),
        };

        Self::new(width, segment_type, guide_points, spine, from_node, to_node)
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn _segment_type(&self) -> SegmentType {
        self.segment_type
    }

    pub fn guide_points(&self) -> &GuidePoints {
        &self.guide_points
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
