use super::curves;
use super::network;
use super::network::*;
use super::LANE_WIDTH;
use crate::math_utils::VecUtils;
use glam::*;

const VERTEX_DENSITY: f32 = 0.05;

#[derive(Clone)]
pub struct RoadGenerator {
    nodes: Vec<(Vec3, Vec3)>,
    segments: Vec<(Segment, RoadMesh)>,
    dir_locked: bool,
    start_road_type: RoadType,
    reverse: bool,
}

impl RoadGenerator {
    pub fn new(
        ground_pos: Vec3,
        selected_road: RoadType,
        selected_dir: Option<Vec3>,
        reverse: bool,
    ) -> Self {
        let (start_dir, dir_locked) = match selected_dir {
            Some(dir) => (dir.normalize(), true),
            None => (Vec3::new(1.0, 0.0, 0.0), false),
        };

        let (start_pos, end_pos) = if reverse {
            (ground_pos - start_dir * 10.0, ground_pos)
        } else {
            (ground_pos, ground_pos + start_dir * 10.0)
        };

        let mesh = generate_straight_mesh(start_pos, end_pos, selected_road);

        let nodes = vec![(start_pos, start_dir), (end_pos, start_dir)];
        let segments = vec![(Segment::new(selected_road, vec![start_pos, end_pos]), mesh)];

        RoadGenerator {
            nodes,
            segments,
            dir_locked,
            start_road_type: selected_road,
            reverse,
        }
    }

    pub fn update_pos(&mut self, ground_pos: Vec3) {
        let (node_pos, node_dir) = if self.reverse {
            self.get_end_node()
        } else {
            self.get_start_node()
        };
        let curve_type = self.start_road_type.curve_type;
        if self.dir_locked {
            match curve_type {
                CurveType::Straight => {
                    let proj_dir = if self.reverse { -node_dir } else { node_dir };
                    let proj_pos = (ground_pos - node_pos).proj(proj_dir) + node_pos;
                    let proj_pos =
                        if (ground_pos - node_pos).dot(proj_dir) / proj_dir.length() > 10.0 {
                            proj_pos
                        } else {
                            node_pos + proj_dir * 10.0
                        };
                    let (start_pos, end_pos) = if self.reverse {
                        (proj_pos, node_pos)
                    } else {
                        (node_pos, proj_pos)
                    };
                    let mesh = generate_straight_mesh(start_pos, end_pos, self.start_road_type);

                    self.nodes = vec![(start_pos, node_dir), (end_pos, node_dir)];
                    self.segments = vec![(
                        Segment::new(self.start_road_type, vec![start_pos, end_pos]),
                        mesh,
                    )];
                }
                CurveType::Curved => {
                    let node_dir = if self.reverse { -node_dir } else { node_dir };
                    let proj_pos = if (ground_pos - node_pos).length() == 0.0 {
                        // TODO can we just use straight mesh?
                        node_pos + node_dir * 10.0
                    } else if (ground_pos - node_pos).length() < 10.0 {
                        node_pos + (ground_pos - node_pos).normalize() * 10.0
                    } else {
                        ground_pos
                    };
                    let mut g_points_vec = curves::three_quarter_circle_curve(
                        node_pos,
                        node_dir,
                        proj_pos,
                        std::f32::consts::PI / 12.0,
                        false,
                        true,
                    )
                    .expect("Should allow projection");
                    let mut start_pos = node_pos;
                    if self.reverse {
                        g_points_vec = curves::reverse_g_points(g_points_vec);
                        start_pos = g_points_vec[0][0];
                    }
                    let (g_points_vec, start_dir) =
                        curves::guide_points_and_direction(g_points_vec);
                    // let g_points_vec = curves::guide_points_and_direction(
                    //     curves::three_quarter_circle_curve(node_pos, node_dir, proj_pos, false),

                    self.nodes = vec![(start_pos, start_dir)];
                    self.segments = vec![];
                    g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
                        let start_pos = g_points[0];
                        let end_pos = g_points[g_points.len() - 1];
                        let mesh = generate_circular_mesh(
                            start_pos,
                            end_pos,
                            self.start_road_type,
                            g_points.clone(),
                        );
                        self.nodes.push((end_pos, end_dir));
                        self.segments
                            .push((Segment::new(self.start_road_type, g_points), mesh));
                    });
                }
            }
        } else {
            let proj_pos = if (ground_pos - node_pos).length() == 0.0 {
                node_pos + Vec3::new(1.0, 0.0, 0.0) * 10.0
            } else if (ground_pos - node_pos).length() < 10.0 {
                node_pos + (ground_pos - node_pos).normalize() * 10.0
            } else {
                ground_pos
            };
            let (start_pos, end_pos) = if self.reverse {
                (proj_pos, node_pos)
            } else {
                (node_pos, proj_pos)
            };
            let road_dir = (end_pos - start_pos).normalize();
            let mesh = generate_straight_mesh(start_pos, end_pos, self.start_road_type);

            self.nodes = vec![(start_pos, road_dir), (end_pos, road_dir)];
            self.segments = vec![(
                Segment::new(self.start_road_type, vec![start_pos, end_pos]),
                mesh,
            )];
        }
    }

    // pub fn double_snap(
    //     &mut self,
    //     snap_case: curves::DoubleSnapCurveCase,
    //     snap_pos: Vec3,
    //     snap_dir: Vec3,
    // ) {
    //     let ((start_pos, start_dir), (end_pos, end_dir)) = if self.reverse {
    //         ((snap_pos, snap_dir), self.get_end_node())
    //     } else {
    //         (self.get_start_node(), (snap_pos, snap_dir))
    //     };
    //     let (g_points_vec, _) = curves::guide_points_and_direction(
    //         curves::match_double_snap_curve_case(start_pos, start_dir, end_pos, end_dir, snap_case),
    //     ); // use snap_three_quarter_circle_curve for snapping
    //        // and free_three_quarter_circle_curve otherwise
    //     self.nodes = vec![(start_pos, start_dir)];
    //     self.segments = vec![];
    //     g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
    //         let start_pos = g_points[0];
    //         let end_pos = g_points[g_points.len() - 1];
    //         let mesh = generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points);
    //         self.nodes.push((end_pos, end_dir));
    //         // TODO update curvetype to be correct
    //         self.segments.push((Segment::new(CurveType::Curved), mesh));
    //     });
    // }

    pub fn try_curve_snap(
        &mut self,
        snap_config: network::SnapConfig,
        sel_road_type: network::RoadType,
    ) -> Option<()> {
        if snap_config.reverse == self.reverse {
            return None;
        }
        let ((mut start_pos, start_dir), (end_pos, _)) = if self.reverse {
            (self.get_end_node(), (snap_config.pos, snap_config.dir))
        } else {
            ((snap_config.pos, -snap_config.dir), self.get_start_node())
        };

        let curve = curves::three_quarter_circle_curve(
            start_pos, start_dir, end_pos, 0.0, false, false)?;

        let mut g_points_vec = curve;
        if !self.reverse {
            g_points_vec = curves::reverse_g_points(g_points_vec);
            start_pos = g_points_vec[0][0];
        }

        let (g_points_vec, _) = curves::guide_points_and_direction(g_points_vec);
        self.nodes = vec![(start_pos, start_dir)];
        self.segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
            let start_pos = g_points[0];
            let end_pos = g_points[g_points.len() - 1];
            let mesh =
                generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points.clone());
            self.nodes.push((end_pos, end_dir));
            // TODO update curvetype to be correct
            self.segments.push((
                Segment::new(
                    RoadType {
                        no_lanes: self.start_road_type.no_lanes,
                        curve_type: CurveType::Curved,
                    },
                    g_points,
                ),
                mesh,
            ));
        });

        Some(())
    }

    pub fn try_double_snap(
        &mut self,
        snap_config: network::SnapConfig,
        sel_road_type: network::RoadType,
    ) -> Option<()> {
        if snap_config.reverse == self.reverse {
            return None;
        }
        let ((start_pos, start_dir), (end_pos, end_dir)) = if self.reverse {
            ((snap_config.pos, snap_config.dir), self.get_end_node())
        } else {
            (self.get_start_node(), (snap_config.pos, snap_config.dir))
        };
        use curves::DoubleSnapCurveCase::*;
        match curves::double_snap_curve_case(
            start_pos,
            start_dir,
            end_pos,
            end_dir,
            sel_road_type.no_lanes,
        ) {
            ErrorTooSmall | ErrorSegmentAngle | ErrorCurveAngle | ErrorTooBig => return None,
            snap_case => {
                let (g_points_vec, _) =
                    curves::guide_points_and_direction(curves::match_double_snap_curve_case(
                        start_pos, start_dir, end_pos, end_dir, snap_case,
                    )); // use snap_three_quarter_circle_curve for snapping
                        // and free_three_quarter_circle_curve otherwise
                self.nodes = vec![(start_pos, start_dir)];
                self.segments = vec![];
                g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
                    let start_pos = g_points[0];
                    let end_pos = g_points[g_points.len() - 1];
                    let mesh = generate_circular_mesh(
                        start_pos,
                        end_pos,
                        self.start_road_type,
                        g_points.clone(),
                    );
                    self.nodes.push((end_pos, end_dir));
                    // TODO update curvetype to be correct
                    self.segments.push((
                        Segment::new(
                            RoadType {
                                no_lanes: self.start_road_type.no_lanes,
                                curve_type: CurveType::Curved,
                            },
                            g_points,
                        ),
                        mesh,
                    ));
                });
            }
        }
        Some(())
    }

    pub fn get_start_node(&self) -> (Vec3, Vec3) {
        self.nodes[0]
    }

    pub fn get_end_node(&self) -> (Vec3, Vec3) {
        self.nodes[self.nodes.len() - 1]
    }

    pub fn get_nodes(&self) -> &Vec<(Vec3, Vec3)> {
        &self.nodes
    }

    pub fn get_segments(&self) -> &Vec<(Segment, RoadMesh)> {
        &self.segments
    }

    pub fn lock(&mut self) {
        self.dir_locked = true;
    }

    pub fn unlock(&mut self) {
        self.dir_locked = false;
    }

    pub fn get_mesh(&self) -> Option<RoadMesh> {
        Some(combine_road_meshes(self.segments.clone()))
    }

    pub fn get_road_type(&self) -> RoadType {
        self.start_road_type
    }

    pub fn is_reverse(&self) -> bool {
        self.reverse
    }

    // pub fn can_snap
}

// iterate over road_meshes and return vec of RoadVertex
// in the future separate road_meshes into "chunks"
pub fn combine_road_meshes(meshes: Vec<(Segment, RoadMesh)>) -> RoadMesh {
    let mut indices_count = 0;
    let mut road_mesh: RoadMesh = RoadMesh::new();

    for (_, mesh) in meshes.iter() {
        road_mesh.vertices.append(&mut mesh.vertices.clone());
        road_mesh.indices.append(
            &mut mesh
                .indices
                .clone()
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += mesh.vertices.len() as u32;
    }

    road_mesh
}

pub fn empty_mesh() -> RoadMesh {
    let vertices = vec![];
    let indices = vec![];
    RoadMesh { vertices, indices }
}

pub fn generate_straight_mesh(start_pos: Vec3, end_pos: Vec3, selected_road: RoadType) -> RoadMesh {
    let no_lanes = selected_road.no_lanes;
    let width = LANE_WIDTH * no_lanes as f32;

    let dir = end_pos - start_pos;
    let mut vertices = generate_road_cut(start_pos, dir, width);
    let mut vertices2 = generate_road_cut(end_pos, dir, width);
    vertices.append(&mut vertices2);
    let vertices = vertices
        .iter()
        .map(|p| RoadVertex {
            position: [p.x, p.y, p.z],
        })
        .collect::<Vec<_>>();

    let indices = [0, 5, 1, 5, 0, 4, 2, 4, 0, 4, 2, 6, 1, 7, 3, 7, 1, 5].to_vec();

    RoadMesh { vertices, indices }
}

pub fn generate_circular_mesh(
    start_pos: Vec3,
    end_pos: Vec3,
    selected_road: RoadType,
    g_points: Vec<Vec3>,
) -> RoadMesh {
    let no_lanes = selected_road.no_lanes;
    let width = LANE_WIDTH * no_lanes as f32;
    let num_of_cuts = (VERTEX_DENSITY * (1000.0 + (end_pos - start_pos).length())) as u32;
    let mut t = 0.0;
    let dt = 1.0 / (num_of_cuts as f32 - 1.0);
    let mut vertices = Vec::new();

    let mut vertices2 = generate_road_cut(
        curves::calc_bezier_pos(g_points.clone(), 0.0),
        curves::calc_bezier_dir(g_points.clone(), 0.0),
        width,
    );
    vertices.append(&mut vertices2);
    for _ in 0..(num_of_cuts - 2) {
        t += dt;
        let mut vertices2 = generate_road_cut(
            curves::calc_bezier_pos(g_points.clone(), t),
            curves::calc_bezier_dir(g_points.clone(), t),
            width,
        );
        vertices.append(&mut vertices2);
    }
    let mut vertices2 = generate_road_cut(
        curves::calc_bezier_pos(g_points.clone(), 1.0),
        curves::calc_bezier_dir(g_points.clone(), 1.0),
        width,
    );
    vertices.append(&mut vertices2);
    let vertices = vertices
        .iter()
        .map(|p| RoadVertex {
            position: [p.x, p.y, p.z],
        })
        .collect::<Vec<_>>();
    let indices = generate_indices(num_of_cuts);
    RoadMesh { vertices, indices }
}

fn generate_indices(num_cuts: u32) -> Vec<u32> {
    let base_indices = [0, 5, 1, 5, 0, 4, 2, 4, 0, 4, 2, 6, 1, 7, 3, 7, 1, 5].to_vec();
    let mut indices = vec![];
    for c in 0..num_cuts - 1 {
        let mut new_indices: Vec<u32> = base_indices.iter().map(|i| i + (4 * c)).collect();
        indices.append(&mut new_indices);
    }
    indices
}

fn generate_road_cut(pos: Vec3, dir: Vec3, width: f32) -> Vec<Vec3> {
    let dir = dir.normalize();
    let left = Vec3::new(-dir.z, dir.y, dir.x);
    let height = Vec3::new(0.0, 0.2, 0.0);
    let half = width / 2.0;
    [
        pos + (left * half) + height,
        pos + (left * -half) + height,
        pos + (left * (half + 0.1)),
        pos + (left * -(half + 0.1)),
    ]
    .to_vec()
}
