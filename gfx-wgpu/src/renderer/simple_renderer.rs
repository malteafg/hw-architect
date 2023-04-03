use crate::primitives::{
    DBuffer, DrawSimpleModel, InstanceRaw, SimpleModelVertex, Texture, Vertex,
};
use crate::render_utils::*;
use crate::resources::simple_models::{SimpleModelId, SimpleModelMap};

use gfx_api::colors;

use std::rc::Rc;

pub struct SimpleRenderer {
    // device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    render_pipeline: wgpu::RenderPipeline,

    models: SimpleModelMap,

    camera_bind_group: Rc<wgpu::BindGroup>,
    color_bind_group: wgpu::BindGroup,

    color_buffer: wgpu::Buffer,
}

impl SimpleRenderer {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,

        models: SimpleModelMap,
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
            colors::DEFAULT,
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

    fn update_color(&self, color: colors::RGBAColor) {
        self.queue
            .write_buffer(&self.color_buffer, 0, &bytemuck::cast_slice(&color));
    }
}

pub trait RenderSimpleModel<'a> {
    fn render_simple_model(
        &mut self,
        simple_renderer: &'a SimpleRenderer,
        model: SimpleModelId,
        color: colors::RGBAColor,
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
        color: colors::RGBAColor,
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
