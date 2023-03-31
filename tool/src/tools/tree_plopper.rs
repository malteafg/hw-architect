use super::ToolStrategy;
// use crate::tool_state::ToolState;

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
}

impl ToolStrategy for TreePlopperTool {
    fn process_keyboard(&mut self, _key: utils::input::KeyAction) {}

    fn left_click(&mut self) {
        self.world
            .add_tree(Tree::new(self.ground_pos), utils::consts::TREE_MODEL_ID);
        let raw_trees: Vec<_> = self
            .world
            .get_trees()
            .iter()
            .flat_map(|(_model_id, model_map)| {
                model_map
                    .iter()
                    .map(|(id, tree)| (*id, tree.pos().into(), tree.yrot()))
            })
            .collect();
        self.gfx_handle
            .borrow_mut()
            .add_trees(utils::consts::TREE_MODEL_ID, raw_trees);
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
    ) -> Self {
        Self {
            gfx_handle,
            world,
            ground_pos,
        }
    }
}
