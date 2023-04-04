use world_api::{LSegmentBuilder, LSegmentBuilderType, SegmentType};

use curves::{GuidePoints, SpinePoints};
use utils::{id::NodeId, VecUtils};

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
    lane_paths: Vec<SpinePoints>,
    from_node: NodeId,
    to_node: NodeId,
}

impl LSegment {
    fn new(
        width: f32,
        segment_type: SegmentType,
        guide_points: GuidePoints,
        lane_paths: Vec<SpinePoints>,
        from_node: NodeId,
        to_node: NodeId,
    ) -> Self {
        LSegment {
            width,
            segment_type,
            guide_points,
            lane_paths,
            from_node,
            to_node,
        }
    }

    pub fn from_builder(builder: LSegmentBuilder, from_node: NodeId, to_node: NodeId) -> Self {
        let (segment_type, node_config, guide_points, spine) = builder.consume();

        let (width, lane_paths) = match node_config {
            LSegmentBuilderType::Same(node_type) => {
                let width = node_type.compute_width();
                let lane_width = node_type.lane_width();
                let no_lane_paths = node_type.no_lanes;

                let mut lane_paths = Vec::with_capacity(no_lane_paths.into());
                for _ in 0..no_lane_paths {
                    lane_paths.push(SpinePoints::with_capacity(spine.len()));
                }

                for (pos, dir) in spine.iter() {
                    let space = dir.right_hand() * lane_width;
                    let left_most = *pos - (no_lane_paths as f32 / 2.) * space;
                    for (i, lane_path) in lane_paths.iter_mut().enumerate() {
                        let p = left_most + space * i as f32;
                        lane_path.push(p)
                    }
                }

                (width, lane_paths)
            }
        };
        Self::new(
            width,
            segment_type,
            guide_points,
            lane_paths,
            from_node,
            to_node,
        )
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

    pub fn no_lane_paths(&self) -> u8 {
        self.lane_paths.len() as u8
    }

    pub fn contains_pos(&self, pos: Vec3) -> bool {
        self.guide_points().is_inside(pos, self.width())
    }
}
