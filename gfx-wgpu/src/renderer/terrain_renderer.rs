use crate::primitives;

use rand::prelude::*;
use wgpu::util::DeviceExt;

use std::rc::Rc;

pub struct TerrainState {
    // device: Rc<wgpu::Device>,
    // queue: Rc<wgpu::Queue>,
    terrain_mesh: TerrainMesh,
    terrain_render_pipeline: wgpu::RenderPipeline,
}

impl TerrainState {
    pub fn new(
        device: Rc<wgpu::Device>,
        // queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,
        terrain_shader: wgpu::ShaderModule,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let terrain_mesh = TerrainMesh::new(&device);
        let terrain_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("terrain_pipeline_layout"),
                bind_group_layouts: &[camera_bind_group_layout],
                push_constant_ranges: &[],
            });
            use primitives::Vertex;
            super::create_render_pipeline(
                &device,
                &layout,
                color_format,
                Some(primitives::Texture::DEPTH_FORMAT),
                &[TerrainVertex::desc()],
                terrain_shader,
                "terrain_pipeline",
            )
        };

        Self {
            // device,
            // queue,
            terrain_mesh,
            terrain_render_pipeline,
        }
    }
}

pub trait RenderTerrain<'a> {
    fn render_terrain(
        &mut self,
        terrain_state: &'a TerrainState,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> RenderTerrain<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_terrain(
        &mut self,
        terrain_state: &'b TerrainState,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_pipeline(&terrain_state.terrain_render_pipeline);

        // render terrain
        self.set_pipeline(&terrain_state.terrain_render_pipeline);
        self.set_vertex_buffer(0, terrain_state.terrain_mesh.vertex_buffer.slice(..));
        self.set_index_buffer(
            terrain_state.terrain_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        self.set_bind_group(0, camera_bind_group, &[]);
        self.draw_indexed(0..terrain_state.terrain_mesh.size as u32, 0, 0..1);
    }
}

// TODO moves this code to somewhere on the cpu side / bridge side
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    position: [f32; 3],
    color: [f32; 3],
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
                        TerrainVertex {
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
