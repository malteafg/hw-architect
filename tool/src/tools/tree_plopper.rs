use super::ToolStrategy;
// use crate::tool_state::ToolState;

use world::nature::Tree;
use world::WorldManipulator;

use gfx_api::GfxSuper;

use glam::Vec3;

use std::cell::RefCell;
use std::rc::Rc;

/// For now we only have one model, but change this in the future and not use const. Maybe compute
/// hash of models.
const TREE_MODEL_ID: u128 = 0;

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
            .add_tree(Tree::new(self.ground_pos), TREE_MODEL_ID);
        let raw_trees: Vec<_> = self
            .world
            .get_trees()
            .iter()
            .flat_map(|(_model_id, model_map)| {
                model_map
                    .iter()
                    .map(|(_id, tree)| (tree.get_pos().into(), tree.get_yrot()))
            })
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
    ) -> Self {
        Self {
            gfx_handle,
            world,
            ground_pos,
        }
    }
}
