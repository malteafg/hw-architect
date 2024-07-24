use crate::primitives::{DBuffer, InstanceRaw, ModelVertex, SimpleModelVertex, Texture, Vertex};
use crate::resources::models::{ModelId, ModelMap};
use crate::resources::simple_models::{SimpleModelId, SimpleModelMap};
use crate::{render_utils, resources};

use gfx_api::colors;

use std::rc::Rc;

use super::GfxInit;

pub struct ModelRenderer {
    render_pipeline: wgpu::RenderPipeline,
    camera_bind_group: Rc<wgpu::BindGroup>,

    models: ModelMap,

    light_bind_group: Rc<wgpu::BindGroup>,
}

impl ModelRenderer {
    pub fn new(gfx: &GfxInit, models: ModelMap, light_bind_group: Rc<wgpu::BindGroup>) -> Self {
        let render_pipeline = gfx.create_render_pipeline(
            &[gfx.texture_bgl(), gfx.camera_bgl(), gfx.light_bgl()],
            gfx.color_format(),
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            gfx.shader(resources::shaders::BASIC),
            "model",
        );
        Self {
            render_pipeline,
            models,
            camera_bind_group: gfx.camera_bg(),
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
    camera_bind_group: Rc<wgpu::BindGroup>,

    models: SimpleModelMap,

    color_bind_group: wgpu::BindGroup,
    color_buffer: wgpu::Buffer,
}

impl SimpleModelRenderer {
    pub fn new(gfx: &GfxInit, models: SimpleModelMap) -> Self {
        let render_pipeline = gfx.create_render_pipeline(
            &[gfx.camera_bgl(), gfx.color_bgl()],
            gfx.color_format(),
            Some(Texture::DEPTH_FORMAT),
            &[SimpleModelVertex::desc(), InstanceRaw::desc()],
            gfx.shader(resources::shaders::SIMPLE),
            "simple",
        );

        let (color_buffer, color_bind_group) = gfx.create_color(colors::DEFAULT, "simple_color");

        Self {
            render_pipeline,
            models,

            camera_bind_group: gfx.camera_bg(),
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
