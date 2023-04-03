use super::ToolStrategy;
// use crate::tool_state::ToolState;

use world_api::Tree;
use world_api::WorldManipulator;

use gfx_api::GfxSuper;

use glam::Vec3;

use std::cell::RefCell;
use std::rc::Rc;

pub struct TreePlopperTool {
    // gfx_handle: Rc<RefCell<dyn GfxTreeData>>,
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,
    world: Box<dyn WorldManipulator>,

    ground_pos: Vec3,

    tool: Option<Tree>,
}

impl ToolStrategy for TreePlopperTool {
    fn process_keyboard(&mut self, _key: utils::input::KeyAction) {}

    fn left_click(&mut self) {
        if let Some(tree) = self.tool {
            let id = self.world.add_tree(tree, utils::consts::TREE_MODEL_ID);
            let raw_trees = vec![(id, tree.pos().into(), tree.yrot())];
            self.gfx_handle
                .borrow_mut()
                .add_trees(utils::consts::TREE_MODEL_ID, raw_trees);
        }
    }

    fn right_click(&mut self) {}

    fn update_ground_pos(&mut self, ground_pos: glam::Vec3) {
        self.ground_pos = ground_pos;
        if self.world.get_segment_from_pos(ground_pos).is_none() {
            let tree = Tree::new(self.ground_pos);
            self.gfx_handle
                .borrow_mut()
                .set_tree_tool(0, vec![(tree.pos().into(), tree.yrot())]);
            self.gfx_handle
                .borrow_mut()
                .set_tree_markers(vec![ground_pos.to_array()]);
            self.tool = Some(tree);
        } else {
            self.gfx_handle.borrow_mut().set_tree_tool(0, vec![]);
            self.gfx_handle
                .borrow_mut()
                .set_tree_markers(vec![ground_pos.to_array()]);
            self.tool = None;
        }
    }

    fn destroy(self: Box<Self>) -> Box<dyn WorldManipulator> {
        self.gfx_handle.borrow_mut().set_tree_tool(0, vec![]);
        self.gfx_handle.borrow_mut().set_tree_markers(vec![]);
        self.world
    }
}

impl TreePlopperTool {
    pub fn new(
        gfx_handle: Rc<RefCell<dyn GfxSuper>>,
        world: Box<dyn WorldManipulator>,
        ground_pos: Vec3,
    ) -> Self {
        let mut tree_plopper_tool = Self {
            gfx_handle,
            world,
            ground_pos,
            tool: None,
        };
        tree_plopper_tool.update_ground_pos(ground_pos);
        tree_plopper_tool
    }
}
