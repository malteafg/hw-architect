use std::collections::VecDeque;

use super::{LNodeBuilder, LSegmentBuilder, NodeType, SegmentType, Side, SnapConfig};

use utils::consts::ROAD_MIN_LENGTH;
use utils::curves::{curve_gen, GuidePoints};
use utils::VecUtils;

use glam::Vec3;

/// TODO add better error types.
pub enum RoadGenErr {
    Placeholder,
    CCSFailed,
    DoubleSnapFailed,
}

#[derive(Debug, Clone)]
pub enum LNodeBuilderType {
    New(LNodeBuilder),
    Old(SnapConfig),
}

impl LNodeBuilderType {
    pub fn new(pos: Vec3, dir: Vec3, node_type: NodeType) -> Self {
        New(LNodeBuilder::new(pos, dir, node_type))
    }

    fn get_pos_and_dir(&self) -> (Vec3, Vec3) {
        match self {
            New(node_builder) => (node_builder.get_pos(), node_builder.get_dir()),
            Old(snap_config) => (snap_config.get_pos(), snap_config.get_dir()),
        }
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

    pub fn get_segments(&self) -> &Vec<LSegmentBuilder> {
        &self.segments
    }

    pub fn gen_stub(pos: Vec3, dir: Vec3, node_type: NodeType) -> Self {
        Self::gen_sfd(pos, node_type, pos + dir, node_type)
    }

    /// Generates a straight free direction road. This is a straight segment from one position to
    /// another. If the segment generated is too short, it will be extended to the minimum length.
    pub fn gen_sfd(
        first_pos: Vec3,
        first_type: NodeType,
        last_pos: Vec3,
        last_type: NodeType,
    ) -> Self {
        let dir = (last_pos - first_pos).normalize_else();
        let end_pos = proj_straight_too_short(first_pos, last_pos, dir);

        let nodes = vec![
            New(LNodeBuilder::new(first_pos, dir, first_type)),
            New(LNodeBuilder::new(end_pos, dir, last_type)),
        ];
        let segments = vec![LSegmentBuilder::new(
            first_type.compute_width(),
            SegmentType {
                curve_type: super::CurveType::Straight,
            },
            GuidePoints::from_two_points(first_pos, end_pos),
        )];

        // TODO fix such that it does not set false, or maybe remove reverse from LRoadBuilder
        Self::new(nodes, segments, false)
    }

    /// Generates a straight locked direction road. Then end_pos is projected unto the line defined
    /// by the start_node.
    pub fn gen_sld(first_node: SnapConfig, last_pos: Vec3, last_type: NodeType) -> Self {
        let first_pos = first_node.get_pos();
        let first_dir = first_node.get_dir();
        let first_to_last = last_pos - first_pos;
        let proj_pos = if first_to_last.dot(first_dir) / first_dir.length() > ROAD_MIN_LENGTH {
            // The projection will yield a long enough segment
            first_to_last.proj(first_dir) + first_pos
        } else {
            // The projection will be to short and therefore we set proj_pos to min road length
            first_pos + first_dir * ROAD_MIN_LENGTH
        };

        let side = first_node.get_side();
        let mut nodes = vec![
            Old(first_node),
            New(LNodeBuilder::new(proj_pos, first_dir, last_type)),
        ];
        let mut reverse = false;
        if let Side::In = side {
            reverse = true;
            nodes.reverse();
        };
        let segments = vec![LSegmentBuilder::new(
            last_type.compute_width(),
            SegmentType {
                curve_type: super::CurveType::Straight,
            },
            GuidePoints::from_two_points(first_pos, proj_pos),
        )];

        Self::new(nodes, segments, reverse)
    }

    /// Generates a circle curved road. The circle curve starts from start_pos and start_dir, and
    /// then end_pos is projected to smallest curvature and 270 degrees.
    pub fn gen_cc(first_node: LNodeBuilderType, last_pos: Vec3, last_type: NodeType) -> Self {
        let (start_pos, start_dir, reverse) = match &first_node {
            New(node_builder) => (node_builder.get_pos(), node_builder.get_dir(), false),
            Old(snap_config) => (
                snap_config.get_pos(),
                snap_config.get_dir(),
                Side::In == snap_config.get_side(),
            ),
        };

        let last_pos = if (last_pos - start_pos).length() < ROAD_MIN_LENGTH {
            start_pos
                + (last_pos - start_pos).try_normalize().unwrap_or(start_dir) * ROAD_MIN_LENGTH
        } else {
            last_pos
        };
        let mut g_points_vec = curve_gen::three_quarter_circle_curve(
            start_pos,
            start_dir,
            last_pos,
            std::f32::consts::PI / 12.0,
            false,
            true,
        )
        .expect("Should allow projection");

        // if reverse {
        //     curve_gen::reverse_g_points_vec(&mut g_points_vec);
        // }
        let (g_points_vec, _start_dir) = curve_gen::guide_points_and_direction(g_points_vec);

        let mut nodes = VecDeque::new();
        nodes.push_back(first_node);
        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, node_dir)| {
            let node_pos = g_points[g_points.len() - 1];
            nodes.push_back(New(LNodeBuilder::new(node_pos, node_dir, last_type)));
            segments.push(LSegmentBuilder::new(
                last_type.compute_width(),
                SegmentType {
                    curve_type: super::CurveType::Curved,
                },
                g_points,
            ));
        });
        // if reverse {
        //     nodes.push_back(first_node)
        // } else {
        //     nodes.push_front(first_node)
        // }
        let nodes: Vec<LNodeBuilderType> = nodes.into_iter().map(|n| n).collect();
        Self::new(nodes, segments, reverse)
    }

    /// Generates a circle curved road. This differs from gen_cc in that positions are fixed and
    /// can not be projected. If the curve then can not be generated, an error is returned.
    pub fn gen_ccs(
        first_pos: Vec3,
        first_type: NodeType,
        last_node: SnapConfig,
        segment_type: SegmentType,
    ) -> Result<Self, RoadGenErr> {
        let reverse = last_node.get_side() == Side::Out;
        let mut end_dir = last_node.get_dir();
        if !reverse {
            end_dir *= -1.0;
        }
        let curve = curve_gen::three_quarter_circle_curve(
            last_node.get_pos(),
            end_dir,
            first_pos,
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
            nodes.push_back(New(LNodeBuilder::new(node_pos, node_dir, first_type)));
            segments.push(LSegmentBuilder::new(
                first_type.compute_width(),
                segment_type,
                g_points,
            ));
        });
        if reverse {
            nodes.push_back(Old(last_node))
        } else {
            nodes.push_front(Old(last_node))
        }
        let nodes: Vec<LNodeBuilderType> = nodes.into_iter().map(|n| n).collect();
        Result::Ok(Self::new(nodes, segments, reverse))
    }

    /// Attempts a double snap between the given positions and directions. If double snap fails, an
    /// error is returned.
    pub fn gen_ds(
        first_node: LNodeBuilderType,
        last_node: SnapConfig,
        segment_type: SegmentType,
    ) -> Result<Self, RoadGenErr> {
        let node_type = last_node.get_node_type();
        let (start_node, (start_pos, start_dir), end_node, (end_pos, end_dir), reverse) =
            if last_node.get_side() == Side::In {
                let start_pos_and_dir = first_node.get_pos_and_dir();
                let end_pos_and_dir = last_node.get_pos_and_dir();
                (
                    first_node,
                    start_pos_and_dir,
                    Old(last_node),
                    end_pos_and_dir,
                    false,
                )
            } else {
                let start_pos_and_dir = first_node.get_pos_and_dir();
                let end_pos_and_dir = last_node.get_pos_and_dir();
                (
                    Old(last_node),
                    end_pos_and_dir,
                    first_node,
                    start_pos_and_dir,
                    true,
                )
            };

        let snap_case = curve_gen::double_snap_curve_case(
            start_pos,
            start_dir,
            end_pos,
            end_dir,
            node_type.no_lanes,
        )
        .map_err(|_| RoadGenErr::DoubleSnapFailed)?;

        let (g_points_vec, _) =
            curve_gen::guide_points_and_direction(curve_gen::match_double_snap_curve_case(
                start_pos, start_dir, end_pos, end_dir, snap_case,
            ));
        let mut nodes = vec![start_node];
        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
            let end_pos = g_points[g_points.len() - 1];
            nodes.push(New(LNodeBuilder::new(end_pos, end_dir, node_type)));
            segments.push(LSegmentBuilder::new(
                node_type.compute_width(),
                segment_type,
                g_points,
            ));
        });
        nodes.pop();
        nodes.push(end_node);
        Result::Ok(Self::new(nodes, segments, reverse))
    }
}

fn proj_straight_too_short(start_pos: Vec3, pref_pos: Vec3, proj_dir: Vec3) -> Vec3 {
    if (pref_pos - start_pos).length() < ROAD_MIN_LENGTH {
        start_pos + (pref_pos - start_pos).try_normalize().unwrap_or(proj_dir) * ROAD_MIN_LENGTH
    } else {
        pref_pos
    }
}
