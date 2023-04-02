use crate::primitives;
use crate::primitives::{DBuffer, Instance, InstanceRaw, Model, SimpleModel};

use utils::id::TreeId;

use glam::*;
use wgpu::util::DeviceExt;

use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub type TreeMap = BTreeMap<u128, HashMap<TreeId, InstanceRaw>>;

pub struct TreeState {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    tree_render_pipeline: wgpu::RenderPipeline,
    tree_map: TreeMap,
    /// TODO in the future we need to have a buffer for every model probably.
    tree_buffer: DBuffer,
    tree_model: Model,
    tool_buffer: DBuffer,

    simple_render_pipeline: Rc<wgpu::RenderPipeline>,
    torus_model: SimpleModel,
    simple_color: wgpu::BindGroup,

    markers_buffer: DBuffer,
    num_markers: u32,
    num_tool_trees: u32,
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
        simple_render_pipeline: Rc<wgpu::RenderPipeline>,
        tree_model: Model,
        torus_model: SimpleModel,
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

        let tree_buffer = DBuffer::new("tree_buffer", wgpu::BufferUsages::VERTEX, &device);
        let tool_buffer = DBuffer::new("tree_tool_buffer", wgpu::BufferUsages::VERTEX, &device);
        let markers_buffer =
            DBuffer::new("tree_markers_buffer", wgpu::BufferUsages::VERTEX, &device);

        let marked_color_bind_group_layout =
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
                label: Some("marked_color_bind_group_layout"),
            });

        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("simple_color_buffer"),
            contents: bytemuck::cast_slice(&[Vec4::new(1.0, 0.2, 0.8, 0.7)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let simple_color = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &marked_color_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: color_buffer.as_entire_binding(),
            }],
            label: Some("simple_color_bind_group"),
        });

        Self {
            device,
            queue,
            tree_render_pipeline,
            tree_map: BTreeMap::new(),
            tree_buffer,
            tool_buffer,
            tree_model,

            simple_render_pipeline,
            torus_model,
            simple_color,

            markers_buffer,
            num_markers: 0,
            num_tool_trees: 0,
        }
    }

    fn write_to_buffer(&mut self) {
        let instance_data: Vec<InstanceRaw> = self
            .tree_map
            .values()
            .flat_map(|model_map| model_map.values().map(|t| *t))
            .collect();
        self.tree_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
    }

    fn num_trees(&self) -> u32 {
        self.tree_map.values().map(|m| m.len()).sum::<usize>() as u32
    }
}

fn tree_to_raw(pos: [f32; 3], yrot: f32) -> InstanceRaw {
    Instance::to_raw(&Instance::new(
        Vec3::from_array(pos),
        Quat::from_rotation_y(yrot),
    ))
}

fn insert_trees(model_map: &mut HashMap<TreeId, InstanceRaw>, trees: Vec<(TreeId, [f32; 3], f32)>) {
    for (id, pos, yrot) in trees.into_iter() {
        model_map.insert(id, tree_to_raw(pos, yrot));
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

    // Not smart as it cannot modify the size of the marker
    fn mark_trees(&mut self, _ids: Vec<TreeId>) {
        // let mut instance_data: Vec<InstanceRaw> = vec![];
        // println!("marking trees");
        // for (_, model_map) in self.tree_map.iter() {
        //     ids.retain(|id| {
        //         if let Some(instance) = model_map.get(id) {
        //             println!("pushing instance");
        //             instance_data.push(*instance);
        //             false
        //         } else {
        //             true
        //         }
        //     })
        // }

        // self.num_markers = instance_data.len() as u32;
        // self.markers_buffer.write(
        //     &self.queue,
        //     &self.device,
        //     &bytemuck::cast_slice(&instance_data),
        // );
    }

    fn set_tree_markers(&mut self, positions: Vec<[f32; 3]>) {
        self.num_markers = positions.len() as u32;
        let instance_data = positions
            .into_iter()
            .map(|pos| {
                Instance::to_raw_with_scale(
                    &Instance::new(Vec3::from_array(pos), glam::Quat::IDENTITY),
                    5.,
                )
            })
            .collect::<Vec<_>>();

        self.markers_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
    }

    /// model_id should be used when there are several trees models.
    fn set_tree_tool(&mut self, _model_id: u128, trees: Vec<[f32; 3]>) {
        self.num_tool_trees = trees.len() as u32;
        let instance_data = trees
            .into_iter()
            .map(|pos| {
                Instance::to_raw(&Instance::new(Vec3::from_array(pos), glam::Quat::IDENTITY))
            })
            .collect::<Vec<_>>();

        self.tool_buffer.write(
            &self.queue,
            &self.device,
            &bytemuck::cast_slice(&instance_data),
        );
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
        if let Some(buffer_slice) = tree_state.tree_buffer.get_buffer_slice() {
            use primitives::DrawModel;
            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&tree_state.tree_render_pipeline);
            self.draw_model_instanced(
                &tree_state.tree_model,
                0..tree_state.num_trees(),
                &camera_bind_group,
                &light_bind_group,
            );
        };

        if let Some(buffer_slice) = tree_state.tool_buffer.get_buffer_slice() {
            use primitives::DrawModel;
            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&tree_state.tree_render_pipeline);
            self.draw_model_instanced(
                &tree_state.tree_model,
                0..tree_state.num_tool_trees,
                &camera_bind_group,
                &light_bind_group,
            );
        };

        if let Some(buffer_slice) = tree_state.markers_buffer.get_buffer_slice() {
            use primitives::DrawSimpleModel;
            self.set_vertex_buffer(1, buffer_slice);
            self.set_pipeline(&tree_state.simple_render_pipeline);
            self.set_bind_group(1, &tree_state.simple_color, &[]);
            self.draw_mesh_instanced(
                &tree_state.torus_model,
                0..tree_state.num_markers,
                &camera_bind_group,
            );
        };
    }
}
