use crate::model;
use rand::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl model::Vertex for TerrainVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct TerrainMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub size: usize,
}

impl TerrainMesh {
    pub fn new(device: &wgpu::Device) -> Self {
        const COLOR: [f32; 3] = [0.29, 0.61, 0.2];
        const MAP_SIZE: usize = 2;
        const VERTEX_LENGTH: usize = MAP_SIZE + 1;
        let mut rng = rand::thread_rng();
        let size = MAP_SIZE * MAP_SIZE * 6;

        let vertices = (0..VERTEX_LENGTH)
            .flat_map(|x| {
                (0..VERTEX_LENGTH)
                    .map(|y| {
                        let r: f32 = rng.gen::<f32>() * 0.06 + 0.97;
                        TerrainVertex {
                            position: [
                                ((0 - (MAP_SIZE as i32) / 2) + (x as i32)) as f32,
                                2.0,
                                ((0 - (MAP_SIZE as i32) / 2) + (y as i32)) as f32,
                            ],
                            color: [COLOR[0] * r, COLOR[1] * r, COLOR[2] * r],
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let indices = (0..MAP_SIZE)
            .flat_map(|x| {
                (0..MAP_SIZE)
                    .map(|y| {
                        [
                            [
                                x * VERTEX_LENGTH + y,
                                x * VERTEX_LENGTH + y + VERTEX_LENGTH,
                                x * VERTEX_LENGTH + y + VERTEX_LENGTH + 1,
                            ],
                            [
                                x * VERTEX_LENGTH + y + VERTEX_LENGTH + 1,
                                x * VERTEX_LENGTH + y + 1,
                                x * VERTEX_LENGTH + y,
                            ],
                        ]
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .flatten()
            .collect::<Vec<_>>();
        // let mut vertices = [0.0; vertex_length * vertex_length * 6];
        // let mut indices = [0; mapsize * mapsize * 6];
        // for x in 0..vertex_length {
        //     for y in 0..vertex_length {
        //         let r: f32 = rng.gen::<f32>() * 0.06 + 0.97;
        //         vertices[(x * vertex_length + y) * 6 + 0] =
        //             ((0 - (mapsize as i32) / 2) + (x as i32)) as f32;
        //         vertices[(x * vertex_length + y) * 6 + 1] = 0.0;
        //         vertices[(x * vertex_length + y) * 6 + 2] =
        //             ((0 - (mapsize as i32) / 2) + (y as i32)) as f32;
        //         vertices[(x * vertex_length + y) * 6 + 3] = color[0] * r;
        //         vertices[(x * vertex_length + y) * 6 + 4] = color[1] * r;
        //         vertices[(x * vertex_length + y) * 6 + 5] = color[2] * r;
        //         if y < mapsize && x < mapsize {
        //             indices[(x * mapsize + y) * 6 + 0] = x * vertex_length + y;
        //             indices[(x * mapsize + y) * 6 + 1] = x * vertex_length + y + vertex_length;
        //             indices[(x * mapsize + y) * 6 + 2] = x * vertex_length + y + vertex_length + 1;
        //             indices[(x * mapsize + y) * 6 + 3] = x * vertex_length + y + vertex_length + 1;
        //             indices[(x * mapsize + y) * 6 + 4] = x * vertex_length + y + 1;
        //             indices[(x * mapsize + y) * 6 + 5] = x * vertex_length + y;
        //         }
        //     }
        // }

        dbg!(vertices.clone());
        dbg!(indices.clone());
        println!("num verts: {}", vertices.len());
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        TerrainMesh {
            vertex_buffer,
            index_buffer,
            size: size as usize,
        }
    }
}
