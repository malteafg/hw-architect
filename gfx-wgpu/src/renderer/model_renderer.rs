use crate::primitives::{DBuffer, DrawModel, InstanceRaw, ModelVertex, Texture, Vertex};
use crate::resources::models::{ModelId, ModelMap};

use std::rc::Rc;

pub struct ModelRenderer {
    // device: Rc<wgpu::Device>,
    // queue: Rc<wgpu::Queue>,
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
