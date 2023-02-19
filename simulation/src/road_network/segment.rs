use super::RoadType;
use crate::curves::{GuidePoints, SpinePoints};
use utils::id::NodeId;

#[derive(Debug, Clone)]
pub struct LSegment {
    road_type: RoadType,
    guide_points: GuidePoints,
    spine_points: SpinePoints,
    from_node: NodeId,
    to_node: NodeId,
}

impl LSegment {
    pub fn get_road_type(&self) -> RoadType {
        self.road_type
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

#[derive(Debug, Clone)]
pub struct LSegmentBuilder {
    pub road_type: RoadType,
    pub guide_points: GuidePoints,
    pub spine_points: SpinePoints,
}

impl LSegmentBuilder {
    pub fn new(road_type: RoadType, guide_points: GuidePoints, spine_points: SpinePoints) -> Self {
        LSegmentBuilder {
            road_type,
            guide_points,
            spine_points,
        }
    }

    pub fn build(self, from_node: NodeId, to_node: NodeId) -> LSegment {
        LSegment {
            road_type: self.road_type,
            guide_points: self.guide_points,
            spine_points: self.spine_points,
            from_node,
            to_node,
        }
    }
}
