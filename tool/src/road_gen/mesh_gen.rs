use crate::road_gen::curve_gen;
use crate::tool_state::SelectedRoad;

use utils::consts::{LANE_MARKINGS_WIDTH, ROAD_HEIGHT};
use utils::VecUtils;
use world::curves::{GuidePoints, SpinePoints};
use world::roads::NodeType;

use gfx_api::RoadMesh;
use glam::*;

const VERTEX_DENSITY: f32 = 0.05;
const CUT_LENGTH: f32 = 5.0;

pub fn generate_straight_mesh(
    start_pos: Vec3,
    end_pos: Vec3,
    selected_road: SelectedRoad,
) -> (RoadMesh, SpinePoints) {
    let dir = end_pos - start_pos;

    let (spine_points, spine_dirs) = curve_gen::calc_uniform_spine_points(
        SpinePoints::from_vec(vec![start_pos, end_pos]),
        SpinePoints::from_vec(vec![dir, dir]),
        CUT_LENGTH,
    );

    (
        generate_road_mesh_with_lanes(spine_points.clone(), spine_dirs, selected_road.node_type),
        spine_points,
    )

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
    selected_road: SelectedRoad,
    g_points: GuidePoints,
) -> (RoadMesh, SpinePoints) {
    let num_of_cuts = (VERTEX_DENSITY * (1000.0 + (end_pos - start_pos).length())) as u32;
    let (spine_points, spine_dirs) = curve_gen::spine_points_and_dir(
        &g_points,
        1.0 / (num_of_cuts as f32 - 1.0),
        CUT_LENGTH,
        num_of_cuts,
    );

    (
        generate_road_mesh_with_lanes(spine_points.clone(), spine_dirs, selected_road.node_type),
        spine_points,
    )

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

// fn generate_indices(num_cuts: u32) -> Vec<u32> {
//     let base_indices = [0, 5, 1, 5, 0, 4, 2, 4, 0, 4, 2, 6, 1, 7, 3, 7, 1, 5].to_vec();
//     let mut indices = vec![];
//     for c in 0..num_cuts - 1 {
//         let mut new_indices: Vec<u32> = base_indices.iter().map(|i| i + (4 * c)).collect();
//         indices.append(&mut new_indices);
//     }
//     indices
// }

// fn generate_road_cut(pos: Vec3, dir: Vec3, width: f32) -> Vec<Vec3> {
//     let dir = dir.normalize();
//     let left = Vec3::new(-dir.z, dir.y, dir.x);
//     let height = Vec3::new(0.0, 0.2, 0.0);
//     let half = width / 2.0;
//     [
//         pos + (left * half) + height,
//         pos + (left * -half) + height,
//         pos + (left * (half + 0.1)),
//         pos + (left * -(half + 0.1)),
//     ]
//     .to_vec()
// }

fn generate_clean_cut(pos: Vec3, dir: Vec3, node_type: NodeType) -> Vec<[f32; 3]> {
    let right_dir = dir.right_hand().normalize();
    let mut vertices = vec![];
    let height = Vec3::new(0.0, ROAD_HEIGHT, 0.0);
    let road_width = node_type.lane_width.getf32() * node_type.no_lanes as f32;

    let mut pos = pos - right_dir * (LANE_MARKINGS_WIDTH * 1.5 + road_width / 2.0);
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH + height;
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    pos += right_dir * (road_width - LANE_MARKINGS_WIDTH);
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH - height;
    vertices.push(pos.into());

    vertices
}

fn generate_markings_cut(pos: Vec3, dir: Vec3, node_type: NodeType) -> Vec<[f32; 3]> {
    let right_dir = dir.right_hand().normalize();
    let mut vertices = vec![];
    let height = Vec3::new(0.0, ROAD_HEIGHT, 0.0);
    let lane_width = node_type.lane_width.getf32();
    let no_lanes = node_type.no_lanes;
    let road_width = lane_width * no_lanes as f32;

    let mut pos = pos - right_dir * (LANE_MARKINGS_WIDTH * 1.5 + road_width / 2.0);
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH + height;
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    // Lanes in between outer lanes
    for _ in 0..no_lanes - 1 {
        pos += right_dir * (lane_width - LANE_MARKINGS_WIDTH);
        vertices.push(pos.into());

        pos += right_dir * LANE_MARKINGS_WIDTH;
        vertices.push(pos.into());
    }

    pos += right_dir * (lane_width - LANE_MARKINGS_WIDTH);
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    pos += right_dir * LANE_MARKINGS_WIDTH - height;
    vertices.push(pos.into());

    vertices
}

fn generate_road_mesh_with_lanes(
    spine_pos: SpinePoints,
    spine_dir: SpinePoints,
    node_type: NodeType,
) -> RoadMesh {
    let no_lanes = node_type.no_lanes;

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut lane_vertices = vec![];
    let mut lane_indices = vec![];
    let m_verts = (no_lanes * 2) as u32;

    let first_pos = spine_pos[0];
    let first_dir = spine_dir[0];
    let first_cut = generate_clean_cut(first_pos, first_dir, node_type);
    vertices.append(&mut first_cut.clone());
    lane_vertices.append(&mut first_cut[1..5].to_vec());

    for i in 1..spine_pos.len() {
        let pos = spine_pos[i];
        let dir = spine_dir[i];
        if i % 3 == 0 {
            let cut = generate_clean_cut(pos, dir, node_type);

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
            let cut = generate_markings_cut(pos, dir, node_type);

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
            let cut = generate_markings_cut(pos, dir, node_type);

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
