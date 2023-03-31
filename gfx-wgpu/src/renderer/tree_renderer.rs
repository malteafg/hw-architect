use crate::primitives;
use crate::primitives::{DBuffer, Instance, InstanceRaw, Model};

use utils::id::TreeId;

use glam::*;

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub type TreeMap = BTreeMap<u128, HashMap<TreeId, InstanceRaw>>;

pub struct TreeState {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    tree_render_pipeline: wgpu::RenderPipeline,
    tree_map: TreeMap,
    /// TODO in the future we need to have a buffer for every model probably.
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

        Self {
            device,
            queue,
            tree_render_pipeline,
            tree_map: BTreeMap::new(),
            instance_buffer,
            tree_model,
        }
    }

    fn write_to_buffer(&mut self) {
        let instance_data: Vec<InstanceRaw> = self
            .tree_map
            .values()
            .flat_map(|model_map| model_map.values().map(|t| *t))
            .collect();
        self.instance_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
    }

    fn num_trees(&self) -> u32 {
        self.tree_map.values().map(|m| m.len()).sum::<usize>() as u32
    }
}

fn insert_trees(model_map: &mut HashMap<TreeId, InstanceRaw>, trees: Vec<(TreeId, [f32; 3], f32)>) {
    for (id, pos, yrot) in trees.into_iter() {
        model_map.insert(
            id,
            Instance::to_raw(&Instance::new(
                Vec3::from_array(pos),
                Quat::from_rotation_y(yrot),
            )),
        );
    }
}

impl gfx_api::GfxTreeData for TreeState {
    fn add_trees(&mut self, model_id: u128, trees: Vec<(TreeId, [f32; 3], f32)>) {
        let Some(mut model_map) = self.tree_map.get_mut(&model_id) else {
            let mut new_model_map = HashMap::new();
            insert_trees(&mut new_model_map, trees);
            self.tree_map.insert(model_id, new_model_map);
            self.write_to_buffer();
            return;
        };
        insert_trees(&mut model_map, trees);
        self.write_to_buffer();
    }

    fn remove_tree(&mut self, tree_id: TreeId, _model_id: u128) {
        for (_, model_map) in self.tree_map.iter_mut() {
            if model_map.remove(&tree_id).is_some() {
                self.write_to_buffer();
                return;
            }
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
            0..tree_state.num_trees(),
            &camera_bind_group,
            &light_bind_group,
        );
    }
}
