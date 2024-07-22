use curves::Spine;
use utils::consts::{LANE_MARKINGS_WIDTH, ROAD_HEIGHT};
use utils::math::DirXZ;

use gfx_api::RoadMesh;
use world_api::NodeType;

use glam::*;

/// Generates and returns the road mesh generated from the given uniform spine points and the type
/// of the node, which is used to get the lane width and total width of the mesh to generate.
pub fn gen_road_mesh_with_lanes(spine: &Spine, node_type: NodeType) -> RoadMesh {
    let no_lanes = node_type.no_lanes();

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut lane_vertices = vec![];
    let mut lane_indices = vec![];
    let m_verts = (no_lanes * 2) as u32;

    let first_pos = spine[0].pos;
    let first_dir = spine[0].dir;
    let first_cut = generate_clean_cut(first_pos, first_dir, node_type);
    vertices.append(&mut first_cut.clone());
    lane_vertices.append(&mut first_cut[1..5].to_vec());

    for i in 1..spine.len() {
        let pos = spine[i].pos;
        let dir = spine[i].dir;
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

/// Generates the cut where no lane markings are present.
fn generate_clean_cut(pos: Vec3, dir: DirXZ, node_type: NodeType) -> Vec<[f32; 3]> {
    let right_dir = dir.right_hand();
    let mut vertices = vec![];
    let height = Vec3::new(0.0, ROAD_HEIGHT, 0.0);
    let road_width = node_type.lane_width_f32() * node_type.no_lanes() as f32;

    let mut pos = pos - Vec3::from(right_dir) * (LANE_MARKINGS_WIDTH * 1.5 + road_width / 2.0);
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH + height;
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * (road_width - LANE_MARKINGS_WIDTH);
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH - height;
    vertices.push(pos.into());

    vertices
}

/// Generates the cut where lane markings are present.
fn generate_markings_cut(pos: Vec3, dir: DirXZ, node_type: NodeType) -> Vec<[f32; 3]> {
    let right_dir = dir.right_hand();
    let mut vertices = vec![];
    let height = Vec3::new(0.0, ROAD_HEIGHT, 0.0);
    let lane_width = node_type.lane_width_f32();
    let no_lanes = node_type.no_lanes();
    let road_width = lane_width * no_lanes as f32;

    let mut pos = pos - Vec3::from(right_dir) * (LANE_MARKINGS_WIDTH * 1.5 + road_width / 2.0);
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH + height;
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    // Lanes in between outer lanes
    for _ in 0..no_lanes - 1 {
        pos += Vec3::from(right_dir) * (lane_width - LANE_MARKINGS_WIDTH);
        vertices.push(pos.into());

        pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH;
        vertices.push(pos.into());
    }

    pos += Vec3::from(right_dir) * (lane_width - LANE_MARKINGS_WIDTH);
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH;
    vertices.push(pos.into());

    pos += Vec3::from(right_dir) * LANE_MARKINGS_WIDTH - height;
    vertices.push(pos.into());

    vertices
}

// iterate over road_meshes and return vec of RoadVertex
// in the future separate road_meshes into "chunks"
pub fn _combine_road_meshes(meshes: Vec<RoadMesh>) -> RoadMesh {
    let mut road_mesh: RoadMesh = RoadMesh::default();

    let mut indices_count = 0;
    let mut lane_indices_count = 0;

    for mut mesh in meshes.into_iter() {
        road_mesh.vertices.append(&mut mesh.vertices);
        road_mesh.indices.append(
            &mut mesh
                .indices
                .into_iter()
                .map(|i| i + indices_count)
                .collect(),
        );
        indices_count += mesh.vertices.len() as u32;

        road_mesh.lane_vertices.append(&mut mesh.lane_vertices);
        road_mesh.lane_indices.append(
            &mut mesh
                .lane_indices
                .into_iter()
                .map(|i| i + lane_indices_count)
                .collect(),
        );
        lane_indices_count += mesh.lane_vertices.len() as u32;
    }
    road_mesh
}

pub fn combine_road_meshes_bad(meshes: Vec<RoadMesh>) -> RoadMesh {
    let mut indices_count = 0;
    let mut road_mesh: RoadMesh = RoadMesh::default();

    for mesh in meshes.iter() {
        let mesh = mesh.clone();
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
    for mesh in meshes.iter() {
        let mesh = mesh.clone();
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
