use crate::primitives::{DBuffer, Instance, InstanceRaw};
use crate::resources;

use super::model_renderer::{ModelRenderer, RenderModel, RenderSimpleModel, SimpleModelRenderer};
use super::GfxHandle;

use gfx_api::colors;

use utils::id::{IdMap, TreeId};

use glam::*;

use std::collections::BTreeMap;
use std::rc::Rc;

pub type TreeMap = BTreeMap<u128, IdMap<TreeId, InstanceRaw>>;

pub struct TreeState {
    tree_map: TreeMap,
    /// TODO in the future we need to have a buffer for every model probably.
    tree_buffer: DBuffer,
    tool_buffer: DBuffer,
    markers_buffer: DBuffer,

    markers_color: colors::RGBAColor,
    num_markers: u32,
    num_tool_trees: u32,
}

impl TreeState {
    pub fn new(device: Rc<wgpu::Device>, queue: Rc<wgpu::Queue>) -> Self {
        let tree_buffer = DBuffer::new(
            device.clone(),
            queue.clone(),
            "tree_buffer",
            wgpu::BufferUsages::VERTEX,
        );
        let tool_buffer = DBuffer::new(
            device.clone(),
            queue.clone(),
            "tree_tool_buffer",
            wgpu::BufferUsages::VERTEX,
        );
        let markers_buffer = DBuffer::new(
            device,
            queue.clone(),
            "tree_markers_buffer",
            wgpu::BufferUsages::VERTEX,
        );
        let markers_color = colors::DEFAULT;

        Self {
            tree_map: BTreeMap::new(),
            tree_buffer,
            tool_buffer,
            markers_buffer,

            markers_color,
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

        self.tree_buffer
            .write(&bytemuck::cast_slice(&instance_data));
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

fn insert_trees(model_map: &mut IdMap<TreeId, InstanceRaw>, trees: Vec<(TreeId, [f32; 3], f32)>) {
    for (id, pos, yrot) in trees.into_iter() {
        model_map.insert(id, tree_to_raw(pos, yrot));
    }
}

impl gfx_api::GfxTreeData for TreeState {
    fn add_trees(&mut self, model_id: u128, trees: Vec<(TreeId, [f32; 3], f32)>) {
        let Some(mut model_map) = self.tree_map.get_mut(&model_id) else {
            let mut new_model_map = IdMap::new();
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
            if model_map.remove(tree_id).is_some() {
                self.write_to_buffer();
                return;
            }
        }
    }

    fn set_tree_markers(&mut self, positions: Vec<[f32; 3]>, color: Option<colors::RGBAColor>) {
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

        if let Some(color) = color {
            self.markers_color = color;
        }
        self.markers_buffer
            .write(&bytemuck::cast_slice(&instance_data));
    }

    /// model_id should be used when there are several trees models.
    fn set_tree_tool(&mut self, _model_id: u128, trees: Vec<([f32; 3], f32)>) {
        self.num_tool_trees = trees.len() as u32;
        let instance_data = trees
            .into_iter()
            .map(|(pos, yrot)| tree_to_raw(pos, yrot))
            .collect::<Vec<_>>();

        self.tool_buffer
            .write(&bytemuck::cast_slice(&instance_data));
    }
}

pub trait RenderTrees<'a> {
    /// The function that implements rendering for roads.
    fn render_trees(
        &mut self,
        gfx_handle: &'a GfxHandle,
        tree_state: &'a TreeState,
        simple_renderer: &'a SimpleModelRenderer,
        model_renderer: &'a ModelRenderer,
    );
}

impl<'a, 'b> RenderTrees<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_trees(
        &mut self,
        gfx_handle: &'a GfxHandle,
        tree_state: &'a TreeState,
        simple_renderer: &'a SimpleModelRenderer,
        model_renderer: &'a ModelRenderer,
    ) {
        self.render_model(
            gfx_handle,
            model_renderer,
            resources::models::TREE_MODEL,
            &tree_state.tree_buffer,
            tree_state.num_trees(),
        );

        self.render_model(
            gfx_handle,
            model_renderer,
            resources::models::TREE_MODEL,
            &tree_state.tool_buffer,
            tree_state.num_markers,
        );

        self.render_simple_model(
            gfx_handle,
            simple_renderer,
            resources::simple_models::TORUS_MODEL,
            tree_state.markers_color,
            &tree_state.markers_buffer,
            tree_state.num_markers,
        );
    }
}
