use super::{LNodeBuilder, LSegmentBuilder, NodeType, SnapConfig};

use curves::{Curve, CurveShared, GuidePoints, Straight};
use utils::consts::ROAD_MIN_LENGTH;
use utils::{Loc, VecUtils};

use glam::Vec3;

/// TODO add better error types.
pub enum RoadGenErr {
    Placeholder,
    CCSFailed,
    DoubleSnapFailed,
    Collision,
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

    fn get_pos(&self) -> Vec3 {
        match self {
            New(node_builder) => node_builder.pos(),
            Old(snap_config) => snap_config.pos(),
        }
    }

    fn get_pos_and_dir(&self) -> (Vec3, Vec3) {
        match self {
            New(node_builder) => (node_builder.pos(), node_builder.dir()),
            Old(snap_config) => (snap_config.pos(), snap_config.dir()),
        }
    }

    fn node_type(&self) -> NodeType {
        match self {
            New(b) => b.node_type(),
            Old(s) => s.node_type(),
        }
    }
}

fn flip_dir_on_new(nodes: &mut [LNodeBuilderType]) {
    nodes.iter_mut().for_each(|n| flip_dir_single(n));
}

fn flip_dir_single(node: &mut LNodeBuilderType) {
    match node {
        New(node_builder) => node_builder.flip_dir(),
        _ => {}
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

    /// NOTE: temporary should be removed once transition segments
    pub fn get_first_node_type(&self) -> NodeType {
        self.nodes[0].node_type()
    }

    pub fn get_segments(&self) -> &Vec<LSegmentBuilder> {
        &self.segments
    }

    pub fn gen_stub(pos: Vec3, dir: Vec3, node_type: NodeType, reverse: bool) -> Self {
        Self::gen_sfd(pos, node_type, pos + dir, node_type, reverse)
    }

    /// Generates a straight free direction road. This is a straight segment from one position to
    /// another. If the segment generated is too short, it will be extended to the minimum length.
    pub fn gen_sfd(
        first_pos: Vec3,
        first_type: NodeType,
        last_pos: Vec3,
        last_type: NodeType,
        reverse: bool,
    ) -> Self {
        // match Curve::<Straight>::from_free(first_pos, last_pos) {
        //     Ok(curve) => {
        //         let mut nodes = vec![
        //             New(LNodeBuilder::new(
        //                 curve.first().pos,
        //                 curve.first().dir.into(),
        //                 first_type,
        //             )),
        //             New(LNodeBuilder::new(
        //                 curve.last().pos,
        //                 curve.last().dir.into(),
        //                 last_type,
        //             )),
        //         ];

        //         if reverse {
        //             nodes.reverse();
        //             flip_dir_on_new(&mut nodes);
        //         }

        //         let segments = vec![LSegmentBuilder::new(first_type, curve.into())];

        //         // TODO fix such that it does not set false, or maybe remove reverse from LRoadBuilder
        //         Self::new(nodes, segments, reverse)
        //     }
        //     Err(_) => unimplemented!(),
        // }

        unimplemented!()
    }

    /// Generates a straight locked direction road. Then end_pos is projected unto the line defined
    /// by the start_node.
    pub fn gen_sld(
        first_node: SnapConfig,
        last_pos: Vec3,
        last_type: NodeType,
        reverse: bool,
    ) -> Self {
        // let first = Loc::new(first_node.pos(), first_node.dir().flip(reverse).into());

        // match Curve::<Straight>::from_start_locked(first, last_pos) {
        //     Ok(curve) => {
        //         let mut nodes = vec![
        //             Old(first_node),
        //             New(LNodeBuilder::new(
        //                 curve.last().pos,
        //                 curve.last().dir.into(),
        //                 last_type,
        //             )),
        //         ];

        //         if reverse {
        //             nodes.reverse();
        //             flip_dir_on_new(&mut nodes);
        //         }

        //         let segments = vec![LSegmentBuilder::new(last_type, curve.into())];

        //         // TODO fix such that it does not set false, or maybe remove reverse from LRoadBuilder
        //         Self::new(nodes, segments, reverse)
        //     }
        //     Err(_) => unimplemented!(),
        // }
        unimplemented!()
    }

    /// Generates a circle curved road. The circle curve starts from start_pos and start_dir, and
    /// then end_pos is projected to smallest curvature and 270 degrees.
    pub fn gen_cc(
        first_node: LNodeBuilderType,
        last_pos: Vec3,
        last_type: NodeType,
        reverse: bool,
    ) -> Self {
        let (start_pos, start_dir) = match &first_node {
            New(node_builder) => (node_builder.pos(), node_builder.dir()),
            Old(snap_config) => (snap_config.pos(), snap_config.dir().flip(reverse)),
        };

        let last_pos = if (last_pos - start_pos).length() < ROAD_MIN_LENGTH {
            start_pos
                + (last_pos - start_pos).try_normalize().unwrap_or(start_dir) * ROAD_MIN_LENGTH
        } else {
            last_pos
        };
        let mut g_points_vec = curves::three_quarter_circle_curve(
            start_pos,
            start_dir,
            last_pos,
            std::f32::consts::PI / 12.0,
            false,
            true,
        )
        .expect("Should allow projection");

        if reverse {
            // TODO shouldn't we use GuidePoints::reverse_vec?
            g_points_vec.reverse()
        }

        let (g_points_vec, _) = curves::guide_points_and_direction(g_points_vec);

        let mut nodes = vec![];
        if reverse {
            g_points_vec.iter().for_each(|(g_points, node_dir)| {
                let node_pos = g_points[g_points.len() - 1];
                nodes.push(New(LNodeBuilder::new(node_pos, *node_dir, last_type)));
            });
            nodes.push(first_node);
            flip_dir_on_new(&mut nodes);
        } else {
            nodes.push(first_node);
            g_points_vec.iter().for_each(|(g_points, node_dir)| {
                let node_pos = g_points[g_points.len() - 1];
                nodes.push(New(LNodeBuilder::new(node_pos, *node_dir, last_type)));
            });
        }

        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, _)| {
            let curve = curves::Circular::from_guide_points(g_points);
            segments.push(LSegmentBuilder::new(last_type, curve.into()));
        });

        let nodes: Vec<LNodeBuilderType> = nodes.into_iter().map(|n| n).collect();
        Self::new(nodes, segments, reverse)
    }

    /// Generates a circle curved road. This differs from gen_cc in that positions are fixed and
    /// can not be projected. If the curve then can not be generated, an error is returned.
    pub fn gen_ccs(
        first_pos: Vec3,
        first_type: NodeType,
        last_node: SnapConfig,
        reverse: bool,
    ) -> Result<Self, RoadGenErr> {
        let end_dir = last_node.dir().flip(!reverse);
        let mut g_points_vec = curves::three_quarter_circle_curve(
            last_node.pos(),
            end_dir,
            first_pos,
            0.0,
            false,
            false,
        )
        .ok_or(RoadGenErr::CCSFailed)?;

        if !reverse {
            GuidePoints::reverse_vec(&mut g_points_vec);
        }

        let (g_points_vec, first_dir) = curves::guide_points_and_direction(g_points_vec);

        let mut nodes = vec![];
        if reverse {
            nodes.push(Old(last_node));
            for i in 0..g_points_vec.len() {
                let (g_points, node_dir) = &g_points_vec[i];
                let node_pos = g_points[g_points.len() - 1];
                nodes.push(New(LNodeBuilder::new(node_pos, *node_dir, first_type)));
            }
        } else {
            nodes.push(New(LNodeBuilder::new(first_pos, first_dir, first_type)));
            for i in 1..g_points_vec.len() {
                let (g_points, node_dir) = &g_points_vec[i];
                let node_pos = g_points[g_points.len() - 1];
                nodes.push(New(LNodeBuilder::new(node_pos, *node_dir, first_type)));
            }
            nodes.push(Old(last_node));
        }

        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, _)| {
            let curve = curves::Circular::from_guide_points(g_points);
            segments.push(LSegmentBuilder::new(first_type, curve.into()));
        });

        let nodes: Vec<LNodeBuilderType> = nodes.into_iter().map(|n| n).collect();
        Result::Ok(Self::new(nodes, segments, reverse))
    }

    /// Attempts a double snap between the given positions and directions. If double snap fails, an
    /// error is returned.
    pub fn gen_ds(
        mut first_node: LNodeBuilderType,
        last_node: SnapConfig,
        reverse: bool,
    ) -> Result<Self, RoadGenErr> {
        let node_type = last_node.node_type();
        let (start_node, (start_pos, start_dir), end_node, (end_pos, end_dir)) = if reverse {
            flip_dir_single(&mut first_node);
            let start_pos_and_dir = first_node.get_pos_and_dir();
            let end_pos_and_dir = last_node.pos_and_dir();
            (
                Old(last_node),
                end_pos_and_dir,
                first_node,
                start_pos_and_dir,
            )
        } else {
            let start_pos_and_dir = first_node.get_pos_and_dir();
            let end_pos_and_dir = last_node.pos_and_dir();
            (
                first_node,
                start_pos_and_dir,
                Old(last_node),
                end_pos_and_dir,
            )
        };

        let snap_case = curves::double_snap_curve_case(
            start_pos,
            start_dir,
            end_pos,
            end_dir,
            node_type.no_lanes(),
        )
        .map_err(|_| RoadGenErr::DoubleSnapFailed)?;

        let (g_points_vec, _) = curves::guide_points_and_direction(
            curves::match_double_snap_curve_case(start_pos, start_dir, end_pos, end_dir, snap_case),
        );

        let mut nodes = vec![start_node];
        g_points_vec.iter().for_each(|(g_points, dir)| {
            let pos = g_points[g_points.len() - 1];
            nodes.push(New(LNodeBuilder::new(pos, *dir, node_type)));
        });
        nodes.pop();
        nodes.push(end_node);

        let mut segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, _)| {
            let curve = curves::Circular::from_guide_points(g_points);
            segments.push(LSegmentBuilder::new(node_type, curve.into()));
        });

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
