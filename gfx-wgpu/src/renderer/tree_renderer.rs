use crate::primitives;
use crate::primitives::{DBuffer, Instance, Model};

use gfx_api::InstanceRaw;

use glam::*;

use std::rc::Rc;

pub struct TreeState {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    tree_render_pipeline: wgpu::RenderPipeline,
    trees: Vec<Instance>,
    instance_buffer: DBuffer,
    tree_model: Model,
}

impl TreeState {
    pub fn new(
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        color_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        // the following parameters should be removed after simpler rendering of road markers.
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        light_bind_group_layout: &wgpu::BindGroupLayout,
        basic_shader: wgpu::ShaderModule,
        tree_model: Model,
    ) -> Self {
        use primitives::Vertex;
        let tree_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("tree_pipeline_layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            super::create_render_pipeline(
                &device,
                &layout,
                color_format,
                Some(primitives::Texture::DEPTH_FORMAT),
                &[primitives::ModelVertex::desc(), InstanceRaw::desc()],
                basic_shader,
                "tree_render_pipeline",
            )
        };

        let instance_buffer = DBuffer::new("tree_buffer", wgpu::BufferUsages::VERTEX, &device);
        let trees = Vec::new();

        let mut result = Self {
            device,
            queue,
            tree_render_pipeline,
            trees,
            instance_buffer,
            tree_model,
        };

        result.add_instance(Vec3::new(1.0, 1.0, 0.0));
        result
    }

    fn add_instance(&mut self, position: Vec3) {
        let rotation = if position == Vec3::ZERO {
            Quat::from_axis_angle(Vec3::Z, 0.0)
        } else {
            Quat::from_axis_angle(position.normalize(), std::f32::consts::PI / 4.)
        };
        self.trees.push(Instance::new(position, rotation));
        let instance_data = self.trees.iter().map(Instance::to_raw).collect::<Vec<_>>();
        self.instance_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
    }

    fn _remove_instance(&mut self) {
        if self.trees.len() != 0 {
            self.trees.remove(0);
            let instance_data = self.trees.iter().map(Instance::to_raw).collect::<Vec<_>>();
            self.instance_buffer.write(
                &self.queue,
                &self.device,
                &bytemuck::cast_slice(&instance_data),
            );
        }
    }
}

pub trait RenderTrees<'a> {
    /// The function that implements rendering for roads.
    fn render_trees(
        &mut self,
        tree_state: &'a TreeState,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> RenderTrees<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_trees(
        &mut self,
        tree_state: &'a TreeState,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        use primitives::DrawModel;
        let Some(buffer_slice) = tree_state.instance_buffer.get_buffer_slice() else {
            return;
        };
        self.set_vertex_buffer(1, buffer_slice);
        self.set_pipeline(&tree_state.tree_render_pipeline);
        self.draw_model_instanced(
            &tree_state.tree_model,
            0..tree_state.trees.len() as u32,
            &camera_bind_group,
            &light_bind_group,
        );
    }
}
