use crate::primitives::{
    DBuffer, DrawSimpleModel, InstanceRaw, SimpleModel, SimpleModelVertex, Texture, Vertex,
};
use crate::render_utils::*;

use glam::Vec4;

use std::collections::HashMap;
use std::rc::Rc;

pub type SimpleModelId = u128;
pub const TORUS_MODEL: SimpleModelId = 0;
pub const ARROW_MODEL: SimpleModelId = 1;
pub const SPHERE_MODEL: SimpleModelId = 2;

pub struct SimpleRenderer {
    // device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    render_pipeline: wgpu::RenderPipeline,

    models: HashMap<u128, SimpleModel>,

    camera_bind_group: Rc<wgpu::BindGroup>,
    color_bind_group: wgpu::BindGroup,

    color_buffer: wgpu::Buffer,
}

impl SimpleRenderer {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,

        models: HashMap<u128, SimpleModel>,

        shader: wgpu::ShaderModule,

        camera_bind_group: Rc<wgpu::BindGroup>,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        color_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let render_pipeline = create_render_pipeline(
            &device,
            &[&camera_bind_group_layout, &color_bind_group_layout],
            color_format,
            Some(Texture::DEPTH_FORMAT),
            &[SimpleModelVertex::desc(), InstanceRaw::desc()],
            shader,
            "simple",
        );

        let (color_buffer, color_bind_group) = create_color(
            &device,
            color_bind_group_layout,
            Vec4::new(1.0, 0.2, 0.5, 1.0),
            "simple_color",
        );

        Self {
            // device,
            queue,
            render_pipeline,

            models,

            camera_bind_group,
            color_bind_group,

            color_buffer,
        }
    }

    fn update_color(&self, color: Vec4) {
        self.queue
            .write_buffer(&self.color_buffer, 0, &bytemuck::cast_slice(&[color]));
    }
}

pub trait RenderSimpleModel<'a> {
    fn render_simple_model(
        &mut self,
        simple_renderer: &'a SimpleRenderer,
        model: SimpleModelId,
        color: Vec4,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    );
}

impl<'a, 'b> RenderSimpleModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_simple_model(
        &mut self,
        simple_renderer: &'a SimpleRenderer,
        model: SimpleModelId,
        color: Vec4,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            simple_renderer.update_color(color);
            let model = simple_renderer.models.get(&model).unwrap();

            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&simple_renderer.render_pipeline);
            self.set_bind_group(1, &simple_renderer.color_bind_group, &[]);
            self.draw_mesh_instanced(&model, 0..no_instances, &simple_renderer.camera_bind_group);
        }
    }
}
