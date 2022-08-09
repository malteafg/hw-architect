use crate::road::network::*;
use cgmath::*;

const LANE_WIDTH: f32 = 3.5;

pub fn generate_mesh(
    start_point: Vector3<f32>,
    end_point: Vector3<f32>,
    no_lanes: u32,
) -> RoadMesh {
    // let num_cuts = path.len() as u32;
    // let vertices = path.iter().map(|point| {
    //     generate_road_cut(point, )
    // }).flatten().collect();
    let width = LANE_WIDTH * no_lanes as f32;
    let dir = end_point - start_point;
    let mut vertices = generate_road_cut(start_point, dir, width);
    let mut vertices2 = generate_road_cut(end_point, dir, width);
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
