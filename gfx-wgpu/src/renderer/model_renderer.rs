use crate::primitives::{DBuffer, InstanceRaw, ModelVertex, SimpleModelVertex, Texture, Vertex};
use crate::render_utils;
use crate::resources::models::{ModelId, ModelMap};
use crate::resources::simple_models::{SimpleModelId, SimpleModelMap};

use gfx_api::colors;

use std::rc::Rc;

pub struct ModelRenderer {
    render_pipeline: wgpu::RenderPipeline,

    models: ModelMap,

    camera_bind_group: Rc<wgpu::BindGroup>,
    light_bind_group: Rc<wgpu::BindGroup>,
}

impl ModelRenderer {
    pub fn new(
        device: Rc<wgpu::Device>,
        // queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,

        models: ModelMap,
        shader: wgpu::ShaderModule,

        texture_bind_group_layout: &wgpu::BindGroupLayout,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        light_bind_group_layout: &wgpu::BindGroupLayout,

        camera_bind_group: Rc<wgpu::BindGroup>,
        light_bind_group: Rc<wgpu::BindGroup>,
    ) -> Self {
        let render_pipeline = super::create_render_pipeline(
            &device,
            &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            color_format,
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            shader,
            "model",
        );
        Self {
            // queue,
            render_pipeline,
            models,
            camera_bind_group,
            light_bind_group,
        }
    }
}

pub trait RenderModel<'a> {
    fn render_model(
        &mut self,
        model_renderer: &'a ModelRenderer,
        model: ModelId,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    );
}

impl<'a, 'b> RenderModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_model(
        &mut self,
        model_renderer: &'a ModelRenderer,
        model: ModelId,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            let model = model_renderer.models.get(&model).unwrap();

            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&model_renderer.render_pipeline);
            self.draw_model_instanced(
                &model,
                0..no_instances,
                &model_renderer.camera_bind_group,
                &model_renderer.light_bind_group,
            );
        }
    }
}

pub struct SimpleModelRenderer {
    render_pipeline: wgpu::RenderPipeline,

    models: SimpleModelMap,
    camera_bind_group: Rc<wgpu::BindGroup>,

    color_bind_group: wgpu::BindGroup,
    color_buffer: wgpu::Buffer,
}

impl SimpleModelRenderer {
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
        let render_pipeline = super::create_render_pipeline(
            &device,
            &[&camera_bind_group_layout, &color_bind_group_layout],
            color_format,
            Some(Texture::DEPTH_FORMAT),
            &[SimpleModelVertex::desc(), InstanceRaw::desc()],
            shader,
            "simple",
        );

        let (color_buffer, color_bind_group) = render_utils::create_color(
            &device,
            color_bind_group_layout,
            colors::DEFAULT,
            "simple_color",
        );

        Self {
            render_pipeline,

            models,

            camera_bind_group,
            color_bind_group,

            color_buffer,
        }
    }
}

pub trait RenderSimpleModel<'a> {
    fn render_simple_model(
        &mut self,
        simple_renderer: &'a SimpleModelRenderer,
        queue: &'a wgpu::Queue,
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
        simple_renderer: &'a SimpleModelRenderer,
        queue: &'a wgpu::Queue,
        model: SimpleModelId,
        color: colors::RGBAColor,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawSimpleModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            queue.write_buffer(
                &simple_renderer.color_buffer,
                0,
                &bytemuck::cast_slice(&color),
            );

            let model = simple_renderer.models.get(&model).unwrap();

            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&simple_renderer.render_pipeline);
            self.set_bind_group(1, &simple_renderer.color_bind_group, &[]);
            self.draw_mesh_instanced(&model, 0..no_instances, &simple_renderer.camera_bind_group);
        }
    }
}
