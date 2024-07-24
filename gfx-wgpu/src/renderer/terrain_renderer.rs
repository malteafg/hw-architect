use crate::{primitives, render_utils, resources};

use rand::prelude::*;
use wgpu::{util::DeviceExt, RenderPass};

use std::rc::Rc;

use super::{GfxHandle, GfxInit, StateRender};

pub struct TerrainState {
    terrain_mesh: TerrainMesh,
    terrain_render_pipeline: wgpu::RenderPipeline,
}

impl TerrainState {
    pub fn new(gfx: &GfxInit) -> Self {
        let terrain_mesh = TerrainMesh::new(&gfx.device);
        use primitives::Vertex;
        let terrain_render_pipeline = gfx.create_render_pipeline(
            &[gfx.camera_bgl()],
            gfx.color_format(),
            Some(primitives::Texture::DEPTH_FORMAT),
            &[primitives::TerrainVertex::desc()],
            gfx.shader(resources::shaders::TERRAIN),
            "terrain",
        );

        Self {
            terrain_mesh,
            terrain_render_pipeline,
        }
    }
}

impl<'a> StateRender<'a> for TerrainState {
    fn render(&self, gfx_handle: &'a GfxHandle, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.terrain_render_pipeline);

        // render terrain
        render_pass.set_pipeline(&self.terrain_render_pipeline);
        render_pass.set_vertex_buffer(0, self.terrain_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.terrain_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.set_bind_group(0, &gfx_handle.camera_bg, &[]);
        render_pass.draw_indexed(0..self.terrain_mesh.size as u32, 0, 0..1);
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
        const MAP_SIZE: u32 = 1000;
        const VERTEX_LENGTH: u32 = MAP_SIZE + 1;
        let mut rng = rand::thread_rng();
        let size = MAP_SIZE * MAP_SIZE * 6;

        let vertices = (0..VERTEX_LENGTH)
            .flat_map(|x| {
                (0..VERTEX_LENGTH)
                    .map(|y| {
                        let r: f32 = rng.gen::<f32>() * 0.06 + 0.97;
                        primitives::TerrainVertex {
                            position: [
                                ((0 - (MAP_SIZE as i32) / 2) + (x as i32)) as f32,
                                0.0,
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
                            x * VERTEX_LENGTH + y,
                            x * VERTEX_LENGTH + y + VERTEX_LENGTH + 1,
                            x * VERTEX_LENGTH + y + VERTEX_LENGTH,
                            x * VERTEX_LENGTH + y + VERTEX_LENGTH + 1,
                            x * VERTEX_LENGTH + y,
                            x * VERTEX_LENGTH + y + 1,
                        ]
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_index_buffer"),
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
