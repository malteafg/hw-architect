use super::curves;
use super::network;
use super::network::*;
use super::LANE_WIDTH;
use crate::math_utils::VecUtils;
use glam::*;

const VERTEX_DENSITY: f32 = 0.05;
const DEFAULT_DIR: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const MIN_LENGTH: f32 = 10.0;

#[derive(Clone)]
pub struct RoadGenerator {
    nodes: Vec<NodeBuilder>,
    segments: Vec<SegmentBuilder>,
    init_pos: Vec3,
    init_dir: Option<Vec3>,
    start_road_type: RoadType,
    reverse: bool,
    init_reverse: bool,
}

impl RoadGenerator {
    fn new(sel_pos: Vec3, sel_dir: Option<Vec3>, sel_road_type: RoadType, reverse: bool) -> Self {
        let start_dir = match sel_dir {
            Some(dir) => dir.try_normalize().unwrap_or(DEFAULT_DIR),
            None => DEFAULT_DIR,
        };

        let (start_pos, end_pos) = if reverse {
            (sel_pos - start_dir * 10.0, sel_pos)
        } else {
            (sel_pos, sel_pos + start_dir * 10.0)
        };

        let mesh = generate_straight_mesh(start_pos, end_pos, sel_road_type);

        let nodes = vec![
            NodeBuilder::new(start_pos, start_dir),
            NodeBuilder::new(end_pos, start_dir),
        ];
        let segments = vec![SegmentBuilder::new(
            sel_road_type,
            vec![start_pos, end_pos],
            mesh,
        )];

        RoadGenerator {
            nodes,
            segments,
            init_pos: sel_pos,
            init_dir: sel_dir,
            start_road_type: sel_road_type,
            reverse,
            init_reverse: reverse,
        }
    }

    // for use when building road
    pub fn extract(self) -> (Vec<NodeBuilder>, Vec<SegmentBuilder>, RoadType, bool) {
        (
            self.nodes,
            self.segments,
            self.start_road_type,
            self.reverse,
        )
    }

    fn update_dir_locked(&mut self, ground_pos: Vec3, dir: Vec3) {
        let pos = self.init_pos;
        let proj_dir = if self.reverse { -dir } else { dir };
        match self.start_road_type.curve_type {
            CurveType::Straight => {
                let proj_dir = if self.reverse { -dir } else { dir };
                let proj_pos = if (ground_pos - pos).dot(proj_dir) / proj_dir.length() > MIN_LENGTH
                {
                    (ground_pos - pos).proj(proj_dir) + pos
                } else {
                    pos + proj_dir * MIN_LENGTH
                };
                let (start_pos, end_pos) = get_start_end(pos, proj_pos, self.reverse);
                self.update_straight(start_pos, end_pos, dir);
            }
            CurveType::Curved => {
                let proj_pos = proj_too_small(pos, ground_pos, proj_dir);
                let mut g_points_vec = curves::three_quarter_circle_curve(
                    pos,
                    proj_dir,
                    proj_pos,
                    std::f32::consts::PI / 12.0,
                    false,
                    true,
                )
                .expect("Should allow projection");

                let mut start_pos = pos;
                if self.reverse {
                    g_points_vec = curves::reverse_g_points(g_points_vec);
                    start_pos = g_points_vec[0][0];
                }
                let (g_points_vec, start_dir) = curves::guide_points_and_direction(g_points_vec);

                self.nodes = vec![NodeBuilder::new(start_pos, start_dir)];
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
                    self.nodes.push(NodeBuilder::new(end_pos, end_dir));
                    self.segments
                        .push(SegmentBuilder::new(self.start_road_type, g_points, mesh));
                });
            }
        }
    }

    fn update_no_dir(&mut self, ground_pos: Vec3) {
        let proj_pos = proj_too_small(self.init_pos, ground_pos, DEFAULT_DIR);
        let (start_pos, end_pos) = get_start_end(self.init_pos, proj_pos, self.reverse);
        let dir = (end_pos - start_pos).normalize();
        self.update_straight(start_pos, end_pos, dir);
    }

    fn update_straight(&mut self, start_pos: Vec3, end_pos: Vec3, dir: Vec3) {
        let mesh = generate_straight_mesh(start_pos, end_pos, self.start_road_type);
        self.nodes = vec![NodeBuilder::new(start_pos, dir), NodeBuilder::new(end_pos, dir)];
        self.segments = vec![SegmentBuilder::new(
            self.start_road_type,
            vec![start_pos, end_pos],
            mesh,
        )];
    }

    fn try_double_snap(
        &mut self,
        init_pos: Vec3,
        init_dir: Vec3,
        snap_pos: Vec3,
        snap_dir: Vec3,
    ) -> Option<()> {
        let ((start_pos, start_dir), (end_pos, end_dir)) =
            get_start_end_with_dir((init_pos, init_dir), (snap_pos, snap_dir), self.reverse);
        let snap_case = curves::double_snap_curve_case(
            start_pos,
            start_dir,
            end_pos,
            end_dir,
            self.start_road_type.no_lanes,
        )
        .ok()?;

        let (g_points_vec, _) = curves::guide_points_and_direction(
            curves::match_double_snap_curve_case(start_pos, start_dir, end_pos, end_dir, snap_case),
        ); // use snap_three_quarter_circle_curve for snapping
           // and free_three_quarter_circle_curve otherwise
        self.nodes = vec![NodeBuilder::new(start_pos, start_dir)];
        self.segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
            let start_pos = g_points[0];
            let end_pos = g_points[g_points.len() - 1];
            let mesh =
                generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points.clone());
            self.nodes.push(NodeBuilder::new(end_pos, end_dir));
            // TODO update curvetype to be correct
            self.segments.push(SegmentBuilder::new(
                RoadType {
                    no_lanes: self.start_road_type.no_lanes,
                    curve_type: CurveType::Curved,
                },
                g_points,
                mesh,
            ));
        });
        Some(())
    }

    fn try_curve_snap(
        &mut self,
        mut start_pos: Vec3,
        mut start_dir: Vec3,
        end_pos: Vec3,
    ) -> Option<()> {
        if !self.reverse {
            start_dir *= -1.0;
        }
        let curve =
            curves::three_quarter_circle_curve(start_pos, start_dir, end_pos, 0.0, false, false)?;

        let mut g_points_vec = curve;
        if !self.reverse {
            g_points_vec = curves::reverse_g_points(g_points_vec);
            start_pos = g_points_vec[0][0];
        }

        let (g_points_vec, start_dir) = curves::guide_points_and_direction(g_points_vec);
        self.nodes = vec![NodeBuilder::new(start_pos, start_dir)];
        self.segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
            let start_pos = g_points[0];
            let end_pos = g_points[g_points.len() - 1];
            let mesh =
                generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points.clone());
            self.nodes.push(NodeBuilder::new(end_pos, end_dir));
            // TODO update curvetype to be correct
            self.segments.push(SegmentBuilder::new(
                RoadType {
                    no_lanes: self.start_road_type.no_lanes,
                    curve_type: CurveType::Curved,
                },
                g_points,
                mesh,
            ));
        });
        Some(())
    }
}

fn get_start_end(start: Vec3, end: Vec3, reverse: bool) -> (Vec3, Vec3) {
    if reverse {
        (end, start)
    } else {
        (start, end)
    }
}

fn get_start_end_with_dir(
    start: (Vec3, Vec3),
    end: (Vec3, Vec3),
    reverse: bool,
) -> ((Vec3, Vec3), (Vec3, Vec3)) {
    if reverse {
        (end, start)
    } else {
        (start, end)
    }
}

fn proj_too_small(start_pos: Vec3, pref_pos: Vec3, proj_dir: Vec3) -> Vec3 {
    if (pref_pos - start_pos).length() < MIN_LENGTH {
        start_pos + (pref_pos - start_pos).try_normalize().unwrap_or(proj_dir) * MIN_LENGTH
    } else {
        pref_pos
    }
}

#[derive(Default)]
pub struct RoadGeneratorTool {
    road: Option<RoadGenerator>,
}

impl RoadGeneratorTool {
    pub fn new(
        sel_pos: Vec3,
        sel_dir: Option<Vec3>,
        sel_road_type: RoadType,
        reverse: bool,
    ) -> Self {
        RoadGeneratorTool {
            road: Some(RoadGenerator::new(sel_pos, sel_dir, sel_road_type, reverse)),
        }
    }

    pub fn update_pos(&mut self, ground_pos: Vec3) {
        if let Some(road) = self.road.as_mut() {
            road.reverse = road.init_reverse;
            if let Some(dir) = road.init_dir {
                road.update_dir_locked(ground_pos, dir);
            } else {
                road.update_no_dir(ground_pos);
            }
        }
    }

    pub fn try_snap(
        &mut self,
        snap_config: network::SnapConfig,
        reverse_locked: bool,
    ) -> Option<()> {
        if let Some(road) = self.road.as_mut() {
            if let Some(dir) = road.init_dir {
                if reverse_locked {
                    if snap_config.reverse == road.reverse {
                        // snapping opposing roads
                        None
                    } else {
                        road.try_double_snap(road.init_pos, dir, snap_config.pos, snap_config.dir)
                    }
                } else {
                    road.reverse = !snap_config.reverse;
                    let dir = if road.reverse { -dir } else { dir };
                    road.try_double_snap(road.init_pos, dir, snap_config.pos, snap_config.dir)
                }
            } else {
                road.reverse = !snap_config.reverse;
                road.try_curve_snap(snap_config.pos, snap_config.dir, road.init_pos)
            }
        } else {
            None
        }
    }

    pub fn update_no_lanes(&mut self, no_lanes: u8) {
        if let Some(road) = self.road.as_mut() {
            road.start_road_type.no_lanes = no_lanes;
        }
    }

    pub fn update_curve_type(&mut self, curve: network::CurveType) {
        if let Some(road) = self.road.as_mut() {
            road.start_road_type.curve_type = curve;
        }
    }

    pub fn lock_dir(&mut self, ground_pos: Vec3) {
        if let Some(road) = self.road.as_mut() {
            road.init_dir = Some(
                (ground_pos - road.init_pos)
                    .try_normalize()
                    .unwrap_or(DEFAULT_DIR),
            )
        }
    }

    pub fn unlock_dir(&mut self) {
        if let Some(road) = self.road.as_mut() {
            road.init_dir = None
        }
    }

    pub fn extract(&mut self) -> RoadGenerator {
        self.road
            .take()
            .expect("road generator extracted without being set")
    }

    pub fn get_mesh(&self) -> Option<RoadMesh> {
        self.road
            .as_ref()
            .map(|road| combine_road_meshes(road.segments.clone()))
    }
}

// iterate over road_meshes and return vec of RoadVertex
// in the future separate road_meshes into "chunks"
pub fn combine_road_meshes(meshes: Vec<SegmentBuilder>) -> RoadMesh {
    let mut indices_count = 0;
    let mut road_mesh: RoadMesh = RoadMesh::default();

    for segment_builder in meshes.iter() {
        let mesh = segment_builder.mesh.clone();
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
        curves::calc_bezier_dir(g_points, 1.0),
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
