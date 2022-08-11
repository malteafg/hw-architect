use super::curves;
use super::network::*;
use super::LANE_WIDTH;
use cgmath::*;

#[derive(Clone)]
pub struct RoadGenerator {
    nodes: Vec<(Vector3<f32>, Vector3<f32>)>,
    segments: Vec<(Segment, RoadMesh)>,
    start_node_locked: bool,
    start_road_type: RoadType,
}

impl RoadGenerator {
    pub fn new(
        ground_pos: Vector3<f32>,
        selected_road: RoadType,
        selected_dir: Option<Vector3<f32>>,
    ) -> Self {
        let start_pos = ground_pos;
        let (start_dir, start_node_locked) = match selected_dir {
            Some(dir) => (dir.normalize(), true),
            None => (Vector3::new(1.0, 0.0, 0.0), false),
        };
        let end_pos = ground_pos + start_dir * 10.0;

        let mesh = generate_straight_mesh(start_pos, end_pos, selected_road);

        let nodes = vec![(start_pos, start_dir), (end_pos, start_dir)];
        let segments = vec![(Segment::new(selected_road.curve_type), mesh)];

        RoadGenerator {
            nodes,
            segments,
            start_node_locked,
            start_road_type: selected_road,
        }
    }

    pub fn update_pos(&mut self, ground_pos: Vector3<f32>) {
        let (start_pos, start_dir) = self.get_start_node();
        let end_pos = ground_pos;

        let curve_type = self.start_road_type.curve_type;
        if self.start_node_locked {
            match curve_type {
                CurveType::Straight => {
                    // TODO project ground pos onto dir of road
                }
                CurveType::Curved => {
                    let (g_points, end_dir) = curves::circle(start_pos, start_dir, end_pos);
                    let mesh =
                        generate_circular_mesh(start_pos, end_pos, self.start_road_type, g_points);

                    self.nodes = vec![(start_pos, start_dir), (end_pos, end_dir)];
                    self.segments = vec![(Segment::new(curve_type), mesh)];
                }
            }
        } else {
            let start_dir = (end_pos - start_pos).normalize();
            let mesh = generate_straight_mesh(start_pos, end_pos, self.start_road_type);

            self.nodes = vec![(start_pos, start_dir), (end_pos, start_dir)];
            self.segments = vec![(Segment::new(curve_type), mesh)];
        }
    }

    fn get_start_node(&self) -> (Vector3<f32>, Vector3<f32>) {
        self.nodes[0]
    }

    fn get_end_node(&self) -> (Vector3<f32>, Vector3<f32>) {
        self.nodes[self.nodes.len() - 1]
    }

    pub fn get_nodes(&self) -> &Vec<(Vector3<f32>, Vector3<f32>)> {
        &self.nodes
    }

    pub fn get_segments(&self) -> &Vec<(Segment, RoadMesh)> {
        &self.segments
    }

    pub fn lock(&mut self) {
        self.start_node_locked = true;
    }

    pub fn get_mesh(&self) -> RoadMesh {
        combine_road_meshes(self.segments.clone())
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

pub fn generate_straight_mesh(
    start_pos: Vector3<f32>,
    end_pos: Vector3<f32>,
    selected_road: RoadType,
) -> RoadMesh {
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
    start_pos: Vector3<f32>,
    end_pos: Vector3<f32>,
    selected_road: RoadType,
    g_points: Vec<Vector3<f32>>,
) -> RoadMesh {
    let no_lanes = selected_road.no_lanes;
    let width = LANE_WIDTH * no_lanes as f32;

    let mut t = 0.0;
    let dt = 0.1;
    let mut vertices = Vec::new();

    let mut vertices2 = generate_road_cut(
        curves::calc_bezier_pos(g_points.clone(), 0.0),
        curves::calc_bezier_dir(g_points.clone(), 0.0),
        width,
    );
    vertices.append(&mut vertices2);
    for _ in 0..((1.0 / dt - 1.0) as u32) {
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
    let indices = generate_indices(11);
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

fn generate_road_cut(pos: Vector3<f32>, dir: Vector3<f32>, width: f32) -> Vec<Vector3<f32>> {
    let dir = dir.normalize();
    let left = Vector3::new(-dir.z, dir.y, dir.x);
    let height = Vector3::new(0.0, 0.2, 0.0);
    let half = width / 2.0;
    [
        pos + (left * half) + height,
        pos + (left * -half) + height,
        pos + (left * (half + 0.1)),
        pos + (left * -(half + 0.1)),
    ]
    .to_vec()
}
