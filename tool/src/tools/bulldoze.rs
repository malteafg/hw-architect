use crate::tool_state::ToolState;

use super::ToolStrategy;

use utils::input;
use world::WorldManipulator;

use gfx_api::GfxSuper;
use glam::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct BulldozeTool {
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,
    world: Box<dyn WorldManipulator>,
    state_handle: Rc<RefCell<ToolState>>,

    ground_pos: Vec3,
}

impl ToolStrategy for BulldozeTool {
    fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        use input::KeyState::*;
        match key {
            (ToggleBulldozeRoads, Press) => {
                let curr = self.bd_segments();
                self.state_handle
                    .borrow_mut()
                    .bulldoze_state
                    .bulldoze_segments = !curr;
                if curr {
                    self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
                }
                self.update_markings();
            }
            (ToggleBulldozeTrees, Press) => {
                let curr = self.bd_trees();
                self.state_handle.borrow_mut().bulldoze_state.bulldoze_trees = !curr;
                self.update_markings();
            }
            _ => {}
        }
    }

    fn left_click(&mut self) {
        if self.bd_trees() {
            if let Some(tree_id) = self.world.get_tree_from_pos(self.ground_pos) {
                self.world.remove_tree(tree_id);
                self.gfx_handle
                    .borrow_mut()
                    .remove_tree(tree_id, utils::consts::TREE_MODEL_ID);
                self.update_markings();
                return;
            }
        }

        if self.bd_segments() {
            if let Some(segment_id) = self.world.get_segment_from_pos(self.ground_pos) {
                if self.world.remove_segment(segment_id) {
                    self.gfx_handle
                        .borrow_mut()
                        .remove_road_meshes(vec![segment_id]);
                    self.update_markings();
                }
            }
        }
    }

    fn right_click(&mut self) {}

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
        self.update_markings();
    }

    /// Unmark any marked segment.
    fn destroy(self: Box<Self>) -> Box<dyn WorldManipulator> {
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
        self.world
    }
}

impl BulldozeTool {
    pub fn new(
        gfx_handle: Rc<RefCell<dyn GfxSuper>>,
        world: Box<dyn WorldManipulator>,
        state_handle: Rc<RefCell<ToolState>>,
        ground_pos: Vec3,
    ) -> Self {
        let mut tool = Self {
            gfx_handle,
            world,
            state_handle,
            ground_pos,
        };
        tool.check_segment();
        tool
    }

    fn bd_segments(&self) -> bool {
        self.state_handle.borrow().bulldoze_state.bulldoze_segments
    }

    fn bd_trees(&self) -> bool {
        self.state_handle.borrow().bulldoze_state.bulldoze_trees
    }

    fn update_markings(&mut self) {
        if self.state_handle.borrow().bulldoze_state.bulldoze_segments {
            self.check_segment()
        }
    }

    fn check_segment(&mut self) {
        let segment_id = self.world.get_segment_from_pos(self.ground_pos);
        if let Some(id) = segment_id {
            self.gfx_handle.borrow_mut().mark_road_segments(vec![id]);
            return;
        }
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
    }
}
