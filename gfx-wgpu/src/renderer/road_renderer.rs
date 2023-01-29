use crate::{
    buffer::{self, VIBuffer},
    texture,
};
use glam::*;
use wgpu::util::DeviceExt;
use gfx_api::RoadMesh;

pub struct RoadState {
    road_buffer: buffer::VIBuffer,
    road_markings_buffer: buffer::VIBuffer,
    road_tool_buffer: buffer::VIBuffer,
    road_render_pipeline: wgpu::RenderPipeline,
    road_color_bind_group: wgpu::BindGroup,
    road_markings_color_bind_group: wgpu::BindGroup,
    road_tool_color_bind_group: wgpu::BindGroup,
}

impl RoadState {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        road_shader: wgpu::ShaderModule,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let road_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("road_color_buffer"),
            contents: bytemuck::cast_slice(&[Vec4::new(0.12, 0.12, 0.12, 1.0)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let road_markings_color_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("road_markings_color_buffer"),
                contents: bytemuck::cast_slice(&[Vec4::new(0.95, 0.95, 0.95, 1.0)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let road_tool_color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("road_tool_color_buffer"),
            contents: bytemuck::cast_slice(&[Vec4::new(0.1, 0.1, 0.6, 0.5)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let road_color_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("road_color_bind_group_layout"),
            });

        let road_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: road_color_buffer.as_entire_binding(),
            }],
            label: Some("road_color_bind_group"),
        });

        let road_markings_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: road_markings_color_buffer.as_entire_binding(),
            }],
            label: Some("road_markings_color_bind_group"),
        });

        let road_tool_color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &road_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: road_tool_color_buffer.as_entire_binding(),
            }],
            label: Some("road_color_bind_group"),
        });

        let road_buffer = VIBuffer::new("road_buffer", device);
        let road_markings_buffer = VIBuffer::new("road_markings_buffer", device);
        let road_tool_buffer = VIBuffer::new("road_tool_buffer", device);
        let road_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("road_pipeline_layout"),
                bind_group_layouts: &[
                    camera_bind_group_layout,
                    &road_color_bind_group_layout,
                    //&texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            use crate::vertex::Vertex;
            super::create_render_pipeline(
                device,
                &layout,
                color_format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[gfx_api::RoadVertex::desc()],
                road_shader,
                "road_render_pipeline",
            )
        };

        // let diffuse_bytes = loader::load_binary("road-diffuse.png").await.unwrap();
        // let diffuse_texture =
        //     texture::Texture::from_bytes(&device, &queue, &diffuse_bytes, "road_diffuse", false)
        //         .unwrap();
        // let normal_bytes = loader::load_binary("road-normal.png").await.unwrap();
        // let normal_texture =
        //     texture::Texture::from_bytes(&device, &queue, &normal_bytes, "road_normal", true)
        //         .unwrap();

        // let road_material = model::Material::new(
        //     &device,
        //     "road_material",
        //     diffuse_texture,
        //     normal_texture,
        //     &texture_bind_group_layout,
        // );

        Self {
            road_buffer,
            road_markings_buffer,
            road_tool_buffer,
            road_render_pipeline,
            road_color_bind_group,
            road_markings_color_bind_group,
            road_tool_color_bind_group,
            // road_material,
        }
    }

    pub fn update_road_mesh(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        mesh: Option<RoadMesh>,
    ) {
        if let Some(mesh) = mesh {
            self.road_buffer.write(
                queue,
                device,
                bytemuck::cast_slice(&mesh.vertices),
                bytemuck::cast_slice(&mesh.indices),
                mesh.indices.len() as u32,
            );
            self.road_markings_buffer.write(
                queue,
                device,
                bytemuck::cast_slice(&mesh.lane_vertices),
                bytemuck::cast_slice(&mesh.lane_indices),
                mesh.lane_indices.len() as u32,
            );
        }
    }

    pub fn update_road_tool_mesh(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        mesh: Option<RoadMesh>,
    ) {
        if let Some(mesh) = mesh {
            self.road_tool_buffer.write(
                queue,
                device,
                bytemuck::cast_slice(&mesh.vertices),
                bytemuck::cast_slice(&mesh.indices),
                mesh.indices.len() as u32,
            );
        }
    }
}

pub trait RenderRoad<'a> {
    fn render_roads(&mut self, road_state: &'a RoadState, camera_bind_group: &'a wgpu::BindGroup);
}

impl<'a, 'b> RenderRoad<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_roads(&mut self, road_state: &'b RoadState, camera_bind_group: &'b wgpu::BindGroup) {
        self.set_pipeline(&road_state.road_render_pipeline);
        if let Ok((vertices, indices)) = road_state.road_buffer.get_buffer_slice() {
            self.set_vertex_buffer(0, vertices);
            self.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
            // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
            self.set_bind_group(0, camera_bind_group, &[]);
            self.set_bind_group(1, &road_state.road_color_bind_group, &[]);
            self.draw_indexed(0..road_state.road_buffer.get_num_indices(), 0, 0..1);
        }
        if let Ok((vertices, indices)) = road_state.road_markings_buffer.get_buffer_slice() {
            self.set_vertex_buffer(0, vertices);
            self.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
            // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
            self.set_bind_group(0, camera_bind_group, &[]);
            self.set_bind_group(1, &road_state.road_markings_color_bind_group, &[]);
            self.draw_indexed(
                0..road_state.road_markings_buffer.get_num_indices(),
                0,
                0..1,
            );
        }
        if let Ok((vertices, indices)) = road_state.road_tool_buffer.get_buffer_slice() {
            self.set_vertex_buffer(0, vertices);
            self.set_index_buffer(indices, wgpu::IndexFormat::Uint32);
            // render_pass.set_bind_group(0, &self.road_material.bind_group, &[]);
            self.set_bind_group(0, camera_bind_group, &[]);
            self.set_bind_group(1, &road_state.road_tool_color_bind_group, &[]);
            self.draw_indexed(0..road_state.road_tool_buffer.get_num_indices(), 0, 0..1);
        }
    }
}
