use super::ToolStrategy;
// use crate::tool_state::ToolState;

use utils::id::TreeId;
use world::nature::Tree;
use world::WorldManipulator;

use gfx_api::GfxSuper;

use glam::Vec3;

use std::cell::RefCell;
use std::rc::Rc;

pub struct TreePlopperTool {
    // gfx_handle: Rc<RefCell<dyn GfxTreeData>>,
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,
    world: Box<dyn WorldManipulator>,

    ground_pos: Vec3,
    tree_id: TreeId,
}

impl ToolStrategy for TreePlopperTool {
    fn process_keyboard(&mut self, _key: utils::input::KeyAction) {}

    fn left_click(&mut self) {
        self.world
            .add_tree(Tree::new(self.ground_pos), self.tree_id);
        let raw_trees: Vec<_> = self
            .world
            .get_trees(self.tree_id)
            .iter()
            .map(|t| (t.get_pos().into(), t.get_yrot()))
            .collect();
        self.gfx_handle.borrow_mut().set_trees(raw_trees);
    }

    fn right_click(&mut self) {}

    fn update_ground_pos(&mut self, ground_pos: glam::Vec3) {
        self.ground_pos = ground_pos;
    }

    fn destroy(self: Box<Self>) -> Box<dyn WorldManipulator> {
        self.world
    }
}

impl TreePlopperTool {
    pub fn new(
        gfx_handle: Rc<RefCell<dyn GfxSuper>>,
        world: Box<dyn WorldManipulator>,
        ground_pos: Vec3,
        tree_id: TreeId,
    ) -> Self {
        Self {
            gfx_handle,
            world,
            ground_pos,
            tree_id,
        }
    }
}
