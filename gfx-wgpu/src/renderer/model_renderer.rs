use crate::primitives::{
    ColoredInstanceRaw, DBuffer, InstanceRaw, ModelVertex, SimpleModelVertex, Texture, Vertex,
};
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
    simple_model_c_rp: wgpu::RenderPipeline,
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

        let simple_model_c_rp = gfx.create_render_pipeline(
            &[gfx.camera_bgl()],
            gfx.color_format(),
            Some(Texture::DEPTH_FORMAT),
            &[SimpleModelVertex::desc(), ColoredInstanceRaw::desc()],
            gfx.shader(resources::shaders::SIMPLE_C),
            "simple_c",
        );

        let (color_buffer, color_bg) = gfx.create_color(colors::DEFAULT, "simple_color");

        Self {
            model_rp,
            models,
            simple_model_rp,
            simple_model_c_rp,
            simple_models,
            color_bg,
            color_buffer,
        }
    }

    pub fn render_model<'a>(
        &'a self,
        gfx_handle: &'a GfxHandle,
        render_pass: &mut wgpu::RenderPass<'a>,
        model: ModelId,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            let model = self.models.get(&model).unwrap();

            render_pass.set_vertex_buffer(1, buffer_slice);
            render_pass.set_pipeline(&self.model_rp);
            render_pass.draw_model_instanced(
                &model,
                0..no_instances,
                &gfx_handle.camera_bg,
                &gfx_handle.light_bg,
            );
        }
    }

    pub fn render_simple_model<'a>(
        &'a self,
        gfx_handle: &'a GfxHandle,
        render_pass: &mut wgpu::RenderPass<'a>,
        model: SimpleModelId,
        color: colors::RGBAColor,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawSimpleModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            let model = self.simple_models.get(&model).unwrap();

            render_pass.set_vertex_buffer(1, buffer_slice);
            render_pass.set_pipeline(&self.simple_model_rp);
            render_pass.set_bind_group(1, &self.color_bg, &[]);

            gfx_handle
                .queue
                .write_buffer(&self.color_buffer, 0, &bytemuck::cast_slice(&color));

            render_pass.draw_mesh_instanced(&model, 0..no_instances, &gfx_handle.camera_bg);
        }
    }

    pub fn render_simple_model_c<'a>(
        &'a self,
        gfx_handle: &'a GfxHandle,
        render_pass: &mut wgpu::RenderPass<'a>,
        model: SimpleModelId,
        instances_buffer: &'a DBuffer,
        no_instances: u32,
    ) {
        use crate::primitives::DrawSimpleModel;
        if let Some(buffer_slice) = instances_buffer.get_buffer_slice() {
            let model = self.simple_models.get(&model).unwrap();

            render_pass.set_vertex_buffer(1, buffer_slice);
            render_pass.set_pipeline(&self.simple_model_rp);
            render_pass.draw_mesh_instanced(&model, 0..no_instances, &gfx_handle.camera_bg);
        }
    }
}
