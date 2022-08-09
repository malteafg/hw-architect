use super::LANE_WIDTH;
use crate::road::curves;
use crate::road::network::*;
use crate::road::tool::CurveType;
use cgmath::*;

pub fn generate_mesh(
    start_point: Vector3<f32>,
    end_point: Vector3<f32>,
    no_lanes: u32,
    curve_type: CurveType,
    g_points: Option<Vec<Vector3<f32>>>,
) -> RoadMesh {
    // let num_cuts = path.len() as u32;
    // let vertices = path.iter().map(|point| {
    //     generate_road_cut(point, )
    // }).flatten().collect();
    let width = LANE_WIDTH * no_lanes as f32;
    match g_points {
        None => {
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
        Some(g_points) => {
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
    }
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
