use super::{Tool, ToolSpec, ToolUnique};

use gfx_api::{
    colors::{self, rgba_d},
    GfxWorldData,
};
use world_api::{Tree, WorldManipulator};

pub struct TreePlopper {
    tree_builder: Option<Tree>,
}

impl<G: GfxWorldData, W: WorldManipulator> ToolSpec<G, W> for Tool<TreePlopper, W> {}

impl Default for TreePlopper {
    fn default() -> Self {
        Self { tree_builder: None }
    }
}

impl<G: GfxWorldData, W: WorldManipulator> ToolUnique<G> for Tool<TreePlopper, W> {
    fn init(&mut self, gfx_handle: &mut G) {
        self.update_view(gfx_handle);
    }

    fn process_keyboard(&mut self, _gfx_handle: &mut G, _key: utils::input::KeyAction) {}

    fn left_click(&mut self, gfx_handle: &mut G) {
        if let Some(tree) = self.instance.tree_builder {
            let id = self.world.add_tree(tree, utils::consts::TREE_MODEL_ID);
            let raw_trees = vec![(id, tree.pos().into(), tree.yrot())];
            gfx_handle.add_trees(utils::consts::TREE_MODEL_ID, raw_trees);
        }
    }

    fn right_click(&mut self, _gfx_handle: &mut G) {}

    fn update_view(&mut self, gfx_handle: &mut G) {
        let ground_pos = self.ground_pos;
        if self.world.get_segment_from_pos(ground_pos).is_none() {
            let tree = Tree::new(self.ground_pos);
            gfx_handle.set_tree_tool(0, vec![(tree.pos().into(), tree.yrot())]);
            gfx_handle.set_tree_markers(vec![ground_pos.to_array()], Some(rgba_d(colors::GREEN)));
            self.instance.tree_builder = Some(tree);
        } else {
            gfx_handle.set_tree_tool(0, vec![]);
            gfx_handle.set_tree_markers(vec![ground_pos.to_array()], Some(rgba_d(colors::RED)));
            self.instance.tree_builder = None;
        }
    }

    fn clean_gfx(&mut self, gfx_handle: &mut G) {
        gfx_handle.set_tree_tool(0, vec![]);
        gfx_handle.set_tree_markers(vec![], None);
    }
}
