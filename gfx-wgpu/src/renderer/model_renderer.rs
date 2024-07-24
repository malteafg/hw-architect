use crate::primitives::{DBuffer, InstanceRaw, ModelVertex, SimpleModelVertex, Texture, Vertex};
use crate::resources::models::{ModelId, ModelMap};
use crate::resources::simple_models::{SimpleModelId, SimpleModelMap};
use crate::{render_utils, resources};

use gfx_api::colors;

use std::rc::Rc;

use super::{GfxHandle, GfxInit};

pub struct ModelRenderer {
    model_rp: wgpu::RenderPipeline,
    models: ModelMap,

    simple_model_rp: wgpu::RenderPipeline,
    simple_models: SimpleModelMap,

    /// The color used for the simple model that is drawn.
    color_bg: wgpu::BindGroup,
    color_buffer: wgpu::Buffer,
}

impl ModelRenderer {
    pub fn new(gfx: &GfxInit, models: ModelMap, simple_models: SimpleModelMap) -> Self {
        let model_rp = gfx.create_render_pipeline(
            &[gfx.texture_bgl(), gfx.camera_bgl(), gfx.light_bgl()],
            gfx.color_format(),
            Some(Texture::DEPTH_FORMAT),
            &[ModelVertex::desc(), InstanceRaw::desc()],
            gfx.shader(resources::shaders::BASIC),
            "model",
        );

        let simple_model_rp = gfx.create_render_pipeline(
            &[gfx.camera_bgl(), gfx.color_bgl()],
            gfx.color_format(),
            Some(Texture::DEPTH_FORMAT),
            &[SimpleModelVertex::desc(), InstanceRaw::desc()],
            gfx.shader(resources::shaders::SIMPLE),
            "simple",
        );

        let (color_buffer, color_bg) = gfx.create_color(colors::DEFAULT, "simple_color");

        Self {
            model_rp,
            models,
            simple_model_rp,
            simple_models,
            color_bg,
            color_buffer,
        }
    }
}

pub trait RenderModel<'a> {
    fn render_model(
        &mut self,
        gfx_handle: &'a GfxHandle,
        model_renderer: &'a ModelRenderer,
        model: ModelId,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    );

    fn render_simple_model(
        &mut self,
        gfx_handle: &'a GfxHandle,
        model_renderer: &'a ModelRenderer,
        model: SimpleModelId,
        color: colors::RGBAColor,
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
        gfx_handle: &'a GfxHandle,
        model_renderer: &'a ModelRenderer,
        model: ModelId,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            let model = model_renderer.models.get(&model).unwrap();

            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&model_renderer.model_rp);
            self.draw_model_instanced(
                &model,
                0..no_instances,
                &gfx_handle.camera_bg,
                &gfx_handle.light_bg,
            );
        }
    }
    fn render_simple_model(
        &mut self,
        gfx_handle: &'a GfxHandle,
        model_renderer: &'a ModelRenderer,
        model: SimpleModelId,
        color: colors::RGBAColor,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawSimpleModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            gfx_handle.queue.write_buffer(
                &model_renderer.color_buffer,
                0,
                &bytemuck::cast_slice(&color),
            );

            let model = model_renderer.simple_models.get(&model).unwrap();

            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&model_renderer.simple_model_rp);
            self.set_bind_group(1, &model_renderer.color_bg, &[]);
            self.draw_mesh_instanced(&model, 0..no_instances, &gfx_handle.camera_bg);
        }
    }
}
