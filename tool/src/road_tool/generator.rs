use gfx_api::{RoadMesh, RoadVertex};
use glam::*;
use simulation::curves;
use simulation::network;
use simulation::network::{CurveType, LNodeBuilder, RoadType, SegmentBuilder};
use utils::consts::{LANE_MARKINGS_WIDTH, LANE_WIDTH, ROAD_HEIGHT};
use utils::VecUtils;

const VERTEX_DENSITY: f32 = 0.05;
const DEFAULT_DIR: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const MIN_LENGTH: f32 = 10.0;
const CUT_LENGTH: f32 = 5.0;

#[derive(Clone)]
pub struct RoadGenerator {
    nodes: Vec<LNodeBuilder>,
    segments: Vec<SegmentBuilder>,
    init_pos: Vec3,
    init_dir: Option<Vec3>,
    start_road_type: RoadType,
    reverse: bool,
    init_reverse: bool,
}

impl network::RoadGen for RoadGenerator {
    fn extract(self) -> (Vec<LNodeBuilder>, Vec<SegmentBuilder>, RoadType, bool) {
        (
            self.nodes,
            self.segments,
            self.start_road_type,
            self.reverse,
        )
    }
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
            LNodeBuilder::new(start_pos, start_dir),
            LNodeBuilder::new(end_pos, start_dir),
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

                self.nodes = vec![LNodeBuilder::new(start_pos, start_dir)];
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
                    self.nodes.push(LNodeBuilder::new(end_pos, end_dir));
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
        self.nodes = vec![
            LNodeBuilder::new(start_pos, dir),
            LNodeBuilder::new(end_pos, dir),
        ];
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
        self.nodes = vec![LNodeBuilder::new(start_pos, start_dir)];
        self.segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
            let start_pos = g_points[0];
            let end_pos = g_points[g_points.len() - 1];
            let mesh =
                generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points.clone());
            self.nodes.push(LNodeBuilder::new(end_pos, end_dir));
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
        self.nodes = vec![LNodeBuilder::new(start_pos, start_dir)];
        self.segments = vec![];
        g_points_vec.into_iter().for_each(|(g_points, end_dir)| {
            let start_pos = g_points[0];
            let end_pos = g_points[g_points.len() - 1];
            let mesh =
                generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points.clone());
            self.nodes.push(LNodeBuilder::new(end_pos, end_dir));
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

    indices_count = 0;
    for segment_builder in meshes.iter() {
        let mesh = segment_builder.mesh.clone();
        road_mesh
            .lane_vertices
            .append(&mut mesh.lane_vertices.clone());
        road_mesh.lane_indices.append(
            &mut mesh
                .lane_indices
                .clone()
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += mesh.lane_vertices.len() as u32;
    }

    road_mesh
}

pub fn empty_mesh() -> RoadMesh {
    let vertices = vec![];
    let indices = vec![];
    RoadMesh {
        vertices,
        indices,
        lane_vertices: vec![],
        lane_indices: vec![],
    }
}

pub fn generate_straight_mesh(start_pos: Vec3, end_pos: Vec3, selected_road: RoadType) -> RoadMesh {
    let no_lanes = selected_road.no_lanes;
    let dir = end_pos - start_pos;

    let (spine_points, spine_dirs) =
        curves::calc_uniform_spine_points(vec![start_pos, end_pos], vec![dir, dir], CUT_LENGTH);

    generate_road_mesh_with_lanes(spine_points, spine_dirs, no_lanes)

    // let width = LANE_WIDTH * no_lanes as f32;
    // let mut vertices = vec![];
    // for i in 0..spine_points.len() {
    //     vertices.append(&mut generate_road_cut(
    //         spine_points[i],
    //         spine_dirs[i],
    //         width,
    //     ));
    // }
    //
    // let vertices = vertices
    //     .iter()
    //     .map(|p| RoadVertex {
    //         position: [p.x, p.y, p.z],
    //     })
    //     .collect::<Vec<_>>();
    //
    // let indices = [0, 5, 1, 5, 0, 4, 2, 4, 0, 4, 2, 6, 1, 7, 3, 7, 1, 5].to_vec();
    //
    // RoadMesh {
    //     vertices,
    //     indices,
    //     lane_vertices: vec![],
    //     lane_indices: vec![],
    // }
}

pub fn generate_circular_mesh(
    start_pos: Vec3,
    end_pos: Vec3,
    selected_road: RoadType,
    g_points: Vec<Vec3>,
) -> RoadMesh {
    let no_lanes = selected_road.no_lanes;
    let num_of_cuts = (VERTEX_DENSITY * (1000.0 + (end_pos - start_pos).length())) as u32;
    let (spine_points, spine_dirs) = curves::spine_points_and_dir(
        &g_points,
        1.0 / (num_of_cuts as f32 - 1.0),
        CUT_LENGTH,
        num_of_cuts,
    );

    generate_road_mesh_with_lanes(spine_points, spine_dirs, no_lanes)

    // let width = LANE_WIDTH * no_lanes as f32;
    // let mut vertices = vec![];
    // for i in 0..spine_points.len() {
    //     vertices.append(&mut generate_road_cut(spine_points[i], spine_dirs[i], width));
    // }
    //
    // let vertices = vertices
    //     .iter()
    //     .map(|p| RoadVertex {
    //         position: [p.x, p.y, p.z],
    //     })
    //     .collect::<Vec<_>>();
    // let indices = generate_indices(spine_points.len() as u32);
    //
    // RoadMesh {
    //     vertices,
    //     indices,
    //     lane_vertices: vec![],
    //     lane_indices: vec![],
    // }
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

fn generate_clean_cut(pos: Vec3, dir: Vec3, no_lanes: u8) -> Vec<RoadVertex> {
    let right_dir = dir.right_hand().normalize();
    let mut vertices = vec![];
    let height = Vec3::new(0.0, ROAD_HEIGHT, 0.0);
    let road_width = LANE_WIDTH * no_lanes as f32;

    let mut pos = pos - right_dir * (LANE_MARKINGS_WIDTH * 1.5 + road_width / 2.0);
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH + height;
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * (road_width - LANE_MARKINGS_WIDTH);
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH - height;
    vertices.push(RoadVertex::from_vec3(pos));

    vertices
}

fn generate_markings_cut(pos: Vec3, dir: Vec3, no_lanes: u8) -> Vec<RoadVertex> {
    let right_dir = dir.right_hand().normalize();
    let mut vertices = vec![];
    let height = Vec3::new(0.0, ROAD_HEIGHT, 0.0);
    let road_width = LANE_WIDTH * no_lanes as f32;

    let mut pos = pos - right_dir * (LANE_MARKINGS_WIDTH * 1.5 + road_width / 2.0);
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH + height;
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(RoadVertex::from_vec3(pos));

    // Lanes in between outer lanes
    for _ in 0..no_lanes - 1 {
        pos += right_dir * (LANE_WIDTH - LANE_MARKINGS_WIDTH);
        vertices.push(RoadVertex::from_vec3(pos));

        pos += right_dir * LANE_MARKINGS_WIDTH;
        vertices.push(RoadVertex::from_vec3(pos));
    }

    pos += right_dir * (LANE_WIDTH - LANE_MARKINGS_WIDTH);
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(RoadVertex::from_vec3(pos));

    pos += right_dir * LANE_MARKINGS_WIDTH - height;
    vertices.push(RoadVertex::from_vec3(pos));

    vertices
}

fn generate_road_mesh_with_lanes(
    spine_pos: curves::SpinePoints,
    spine_dir: curves::SpinePoints,
    no_lanes: u8,
) -> RoadMesh {
    let mut vertices = vec![];
    let mut indices = vec![];
    let mut lane_vertices = vec![];
    let mut lane_indices = vec![];
    let m_verts = (no_lanes * 2) as u32;

    let first_pos = spine_pos[0];
    let first_dir = spine_dir[0];
    let first_cut = generate_clean_cut(first_pos, first_dir, no_lanes);
    vertices.append(&mut first_cut.clone());
    lane_vertices.append(&mut first_cut[1..5].to_vec());

    for i in 1..spine_pos.len() {
        let pos = spine_pos[i];
        let dir = spine_dir[i];
        if i % 3 == 0 {
            let cut = generate_clean_cut(pos, dir, no_lanes);

            let previ = (vertices.len() - 4 - m_verts as usize) as u32;
            let curri = vertices.len() as u32;
            vertices.append(&mut cut.clone());
            indices.append(&mut vec![
                previ,
                previ + 1,
                curri,
                curri,
                previ + 1,
                curri + 1,
            ]);
            // connect all middle vertices to corner
            for i in 0..m_verts - 1 {
                let i = i as u32;
                indices.append(&mut vec![curri + 3, previ + 2 + i, previ + 3 + i])
            }
            // last triangle for other half of middle
            indices.append(&mut vec![curri + 3, curri + 2, previ + 2]);

            indices.append(&mut vec![
                previ + m_verts + 2,
                previ + m_verts + 3,
                curri + 4,
                curri + 4,
                previ + m_verts + 3,
                curri + 5,
            ]);

            let previ = (lane_vertices.len() - 2 - m_verts as usize) as u32;
            let curri = lane_vertices.len() as u32;
            lane_vertices.append(&mut cut[1..cut.len() - 1].to_vec());
            lane_indices.append(&mut vec![
                previ,
                previ + 1,
                curri,
                curri,
                previ + 1,
                curri + 1,
            ]);
            lane_indices.append(&mut vec![
                previ + m_verts,
                previ + m_verts + 1,
                curri + 2,
                curri + 2,
                previ + m_verts + 1,
                curri + 3,
            ]);
        } else if i % 3 == 1 {
            let cut = generate_markings_cut(pos, dir, no_lanes);

            let previ = (vertices.len() - 6) as u32;
            let curri = vertices.len() as u32;
            vertices.append(&mut cut.clone());
            indices.append(&mut vec![
                previ,
                previ + 1,
                curri,
                curri,
                previ + 1,
                curri + 1,
            ]);
            // connect all middle vertices to corner
            for i in 0..m_verts - 1 {
                let i = i as u32;
                indices.append(&mut vec![previ + 3, curri + 3 + i, curri + 2 + i])
            }
            // last triangle for other half of middle
            indices.append(&mut vec![previ + 2, previ + 3, curri + 2]);

            indices.append(&mut vec![
                previ + 4,
                previ + 5,
                curri + m_verts + 2,
                curri + m_verts + 2,
                previ + 5,
                curri + m_verts + 3,
            ]);

            let previ = (lane_vertices.len() - 4) as u32;
            let curri = lane_vertices.len() as u32;
            lane_vertices.append(&mut cut[1..cut.len() - 1].to_vec());
            lane_indices.append(&mut vec![
                previ,
                previ + 1,
                curri,
                curri,
                previ + 1,
                curri + 1,
            ]);
            lane_indices.append(&mut vec![
                curri + m_verts,
                previ + 2,
                curri + m_verts + 1,
                previ + 2,
                previ + 3,
                curri + m_verts + 1,
            ]);
        } else {
            // generates lanes
            let cut = generate_markings_cut(pos, dir, no_lanes);

            let previ = (vertices.len() - 4 - m_verts as usize) as u32;
            let curri = vertices.len() as u32;
            vertices.append(&mut cut.clone());
            indices.append(&mut vec![
                previ,
                previ + 1,
                curri,
                curri,
                previ + 1,
                curri + 1,
            ]);

            for i in 0..no_lanes {
                let i = i as u32 * 2;
                indices.append(&mut vec![
                    previ + i + 2,
                    previ + i + 3,
                    curri + i + 2,
                    curri + i + 2,
                    previ + i + 3,
                    curri + i + 3,
                ]);
            }

            indices.append(&mut vec![
                previ + m_verts + 2,
                previ + m_verts + 3,
                curri + m_verts + 2,
                curri + m_verts + 2,
                previ + m_verts + 3,
                curri + m_verts + 3,
            ]);

            let previ = (lane_vertices.len() - 2 - m_verts as usize) as u32;
            let curri = lane_vertices.len() as u32;
            lane_vertices.append(&mut cut[1..cut.len() - 1].to_vec());
            for i in 0..no_lanes + 1 {
                let i = i as u32 * 2;
                lane_indices.append(&mut vec![
                    previ + i,
                    previ + i + 1,
                    curri + i,
                    curri + i,
                    previ + i + 1,
                    curri + i + 1,
                ]);
            }
        }
    }

    RoadMesh {
        vertices,
        indices,
        lane_vertices,
        lane_indices,
    }
}

trait VertexGenerator {
    fn from_vec3(pos: Vec3) -> Self;
}

impl VertexGenerator for RoadVertex {
    fn from_vec3(pos: Vec3) -> Self {
        RoadVertex {
            position: [pos.x, pos.y, pos.z],
        }
    }
}
