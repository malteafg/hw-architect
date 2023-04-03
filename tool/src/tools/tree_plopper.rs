use super::{Tool, ToolInstance, ToolStrategy};

use gfx_api::colors::{self, rgba_d};
use world_api::Tree;

pub struct TreePlopperTool {
    tree_builder: Option<Tree>,
}

impl Tool for ToolInstance<TreePlopperTool> {}

impl Default for TreePlopperTool {
    fn default() -> Self {
        Self { tree_builder: None }
    }
}

impl ToolStrategy for ToolInstance<TreePlopperTool> {
    fn init(&mut self) {
        self.update_view();
    }

    fn process_keyboard(&mut self, _key: utils::input::KeyAction) {}

    fn left_click(&mut self) {
        if let Some(tree) = self.self_tool.tree_builder {
            let id = self.world.add_tree(tree, utils::consts::TREE_MODEL_ID);
            let raw_trees = vec![(id, tree.pos().into(), tree.yrot())];
            self.gfx_handle
                .borrow_mut()
                .add_trees(utils::consts::TREE_MODEL_ID, raw_trees);
        }
    }

    fn right_click(&mut self) {}

    fn update_view(&mut self) {
        let ground_pos = self.ground_pos;
        if self.world.get_segment_from_pos(ground_pos).is_none() {
            let tree = Tree::new(self.ground_pos);
            self.gfx_handle
                .borrow_mut()
                .set_tree_tool(0, vec![(tree.pos().into(), tree.yrot())]);
            self.gfx_handle
                .borrow_mut()
                .set_tree_markers(vec![ground_pos.to_array()], Some(rgba_d(colors::GREEN)));
            self.self_tool.tree_builder = Some(tree);
        } else {
            self.gfx_handle.borrow_mut().set_tree_tool(0, vec![]);
            self.gfx_handle
                .borrow_mut()
                .set_tree_markers(vec![ground_pos.to_array()], Some(rgba_d(colors::RED)));
            self.self_tool.tree_builder = None;
        }
    }

    fn clean_gfx(&mut self) {
        self.gfx_handle.borrow_mut().set_tree_tool(0, vec![]);
        self.gfx_handle.borrow_mut().set_tree_markers(vec![], None);
    }
}
