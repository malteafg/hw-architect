use std::collections::VecDeque;

use super::{LNodeBuilder, LSegmentBuilder, NodeType, SegmentType, Side, SnapConfig};

use utils::consts::{DEFAULT_DIR, ROAD_MIN_LENGTH};
use utils::curves::{curve_gen, GuidePoints, SpinePoints};
use utils::VecUtils;

use glam::Vec3;

/// TODO add better error types.
pub enum RoadGenErr {
    Placeholder,
    CCSFailed,
    DoubleSnapFailed,
}

pub enum LNodeBuilderType {
    New(LNodeBuilder),
    Old(SnapConfig),
}

/// This struct defines exactly the data that a road graph needs in order to add new segments to
/// it.
/// Nodes and segments are generated in the direction that the car drives.
/// This should always only be able to generate a valid road.
/// There is always one more node than segment.
pub struct LRoadBuilder {
    nodes: Vec<LNodeBuilderType>,
    segments: Vec<LSegmentBuilder>,
    reverse: bool,
}

use LNodeBuilderType::*;

impl LRoadBuilder {
    fn new(nodes: Vec<LNodeBuilderType>, segments: Vec<LSegmentBuilder>, reverse: bool) -> Self {
        Self {
            nodes,
            segments,
            reverse,
        }
    }

    pub fn consume(self) -> (Vec<LNodeBuilderType>, Vec<LSegmentBuilder>, bool) {
        (self.nodes, self.segments, self.reverse)
    }

    /// Generates a straight free direction road. This is a straight segment from one position to
    /// another. If the segment generated is too short, it will be extended to the minimum length.
    pub fn gen_sfd(
        start_pos: Vec3,
        start_type: NodeType,
        end_pos: Vec3,
        end_type: NodeType,
        segment_type: SegmentType,
    ) -> Self {
        let dir = (end_pos - start_pos).normalize();
        let end_pos = proj_straight_too_short(start_pos, end_pos, dir);

        let nodes = vec![
            New(LNodeBuilder::new(start_pos, dir, start_type)),
            New(LNodeBuilder::new(end_pos, dir, end_type)),
        ];
        let segments = vec![LSegmentBuilder::new(
            segment_type,
            GuidePoints::from_vec(vec![start_pos, end_pos]),
            SpinePoints::from_vec(vec![start_pos, end_pos]),
        )];

        // TODO fix such that it does not set false, or maybe remove reverse from LRoadBuilder
        Self::new(nodes, segments, false)
    }

    /// Generates a straight locked direction road. Then end_pos is projected unto the line defined
    /// by the start_node.
    pub fn gen_sld(
        start_node: SnapConfig,
        end_pos: Vec3,
        end_type: NodeType,
        segment_type: SegmentType,
    ) -> Self {
        let start_pos = start_node.get_pos();
        let start_dir = start_node.get_dir();
        let actual_dir = (end_pos - start_pos).normalize();
        let proj_pos = if actual_dir.dot(start_dir) / start_dir.length() > ROAD_MIN_LENGTH {
            // The projection will yield a long enough segment
            actual_dir.proj(start_dir) + start_pos
        } else {
            // The projection will be to short and therefore we set proj_pos to min road length
            start_pos + start_dir * ROAD_MIN_LENGTH
        };

        let side = start_node.get_side();
        let mut nodes = vec![
            Old(start_node),
            New(LNodeBuilder::new(proj_pos, start_dir, end_type)),
        ];
        let mut reverse = false;
        if let Side::In = side {
            reverse = true;
            nodes.reverse();
        };
        let segments = vec![LSegmentBuilder::new(
            segment_type,
            GuidePoints::from_vec(vec![start_pos, end_pos]),
            SpinePoints::from_vec(vec![start_pos, end_pos]),
        )];

        Self::new(nodes, segments, reverse)
    }

    /// Generates a circle curved road. The circle curve starts from start_pos and start_dir, and
    /// then end_pos is projected to smallest curvature and 270 degrees.
    pub fn gen_cc(
        start_node: LNodeBuilderType,
        end_pos: Vec3,
        end_type: NodeType,
        segment_type: SegmentType,
    ) -> Self {
        let (start_pos, start_dir, reverse) = match &start_node {
            New(node_builder) => (node_builder.get_pos(), node_builder.get_dir(), false),
            Old(snap_config) => (
                snap_config.get_pos(),
                snap_config.get_dir(),
                Side::In == snap_config.get_side(),
            ),
        };

        // Not sure why this line was here
        // let end_pos = proj_too_small(start_pos, end_pos, start_dir);
        let mut g_points_vec = curve_gen::three_quarter_circle_curve(
            start_pos,
            start_dir,
            end_pos,
            std::f32::consts::PI / 12.0,
            false,
            true,
        )
        .expect("Should allow projection");

        if reverse {
            curve_gen::reverse_g_points_vec(&mut g_points_vec);
        }
        let (g_points_vec, _start_dir) = curve_gen::guide_points_and_direction(g_points_vec);

        // let mut nodes = vec![LNodeBuilder::new(start_pos, start_dir, end_type)];
        let mut nodes = VecDeque::new();
        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, node_dir)| {
            let node_pos = g_points[g_points.len() - 1];
            // TODO fix 0.05, and figure out what to do with it.
            let spine_points = g_points.get_spine_points(0.05);
            nodes.push_back(New(LNodeBuilder::new(node_pos, node_dir, end_type)));
            segments.push(LSegmentBuilder::new(segment_type, g_points, spine_points));
        });
        if reverse {
            nodes.push_back(start_node)
        } else {
            nodes.push_front(start_node)
        }
        let nodes: Vec<LNodeBuilderType> = nodes.into_iter().map(|n| n).collect();
        Self::new(nodes, segments, reverse)
    }

    /// Generates a circle curved road. This differs from gen_cc in that positions are fixed and
    /// can not be projected. If the curve then can not be generated, an error is returned.
    pub fn gen_ccs(
        start_pos: Vec3,
        start_type: NodeType,
        end_node: SnapConfig,
        segment_type: SegmentType,
    ) -> Result<Self, RoadGenErr> {
        let reverse = end_node.get_side() == Side::In;
        let mut end_dir = end_node.get_dir();
        if !reverse {
            end_dir *= -1.0;
        }
        let curve = curve_gen::three_quarter_circle_curve(
            end_node.get_pos(),
            end_dir,
            start_pos,
            0.0,
            false,
            false,
        )
        .ok_or(RoadGenErr::CCSFailed)?;

        let mut g_points_vec = curve;
        if !reverse {
            curve_gen::reverse_g_points_vec(&mut g_points_vec);
        }

        let (g_points_vec, _start_dir) = curve_gen::guide_points_and_direction(g_points_vec);
        let mut nodes = VecDeque::new();
        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, node_dir)| {
            let node_pos = g_points[g_points.len() - 1];
            // TODO fix 0.05, and figure out what to do with it.
            let spine_points = g_points.get_spine_points(0.05);
            nodes.push_back(New(LNodeBuilder::new(node_pos, node_dir, start_type)));
            segments.push(LSegmentBuilder::new(segment_type, g_points, spine_points));
        });
        if reverse {
            nodes.push_back(Old(end_node))
        } else {
            nodes.push_front(Old(end_node))
        }
        let nodes: Vec<LNodeBuilderType> = nodes.into_iter().map(|n| n).collect();
        Result::Ok(Self::new(nodes, segments, reverse))
    }

    /// Attempts a double snap between the given positions and directions. If double snap fails, an
    /// error is returned.
    pub fn gen_ds(start_node: LNodeBuilderType, end_node: SnapConfig) -> Result<Self, RoadGenErr> {
        todo!()
    }
}

// fn get_start_end(start: Vec3, end: Vec3, side: Side) -> (Vec3, Vec3, bool) {
//     match side {
//         Side::In => (end, start, true),
//         Side::Out => (start, end, false),
//     }
// }

fn proj_straight_too_short(start_pos: Vec3, pref_pos: Vec3, proj_dir: Vec3) -> Vec3 {
    if (pref_pos - start_pos).length() < ROAD_MIN_LENGTH {
        start_pos + (pref_pos - start_pos).try_normalize().unwrap_or(proj_dir) * ROAD_MIN_LENGTH
    } else {
        pref_pos
    }
}
