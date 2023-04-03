use super::{Tool, ToolInstance, ToolStrategy};

use utils::input;

use gfx_api::colors::{self, rgba_d};
use glam::*;

#[derive(Default)]
pub struct BulldozeTool;

impl Tool for ToolInstance<BulldozeTool> {}

impl ToolStrategy for ToolInstance<BulldozeTool> {
    fn init(&mut self) {
        self.update_view();
    }

    fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        use input::KeyState::*;
        match key {
            (ToggleBulldozeRoads, Press) => {
                let curr = self.bd_segments();
                self.state_handle.bulldoze_state.bulldoze_segments = !curr;
                if curr {
                    self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
                }
                self.update_markings();
            }
            (ToggleBulldozeTrees, Press) => {
                let curr = self.bd_trees();
                self.state_handle.bulldoze_state.bulldoze_trees = !curr;
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

    fn update_view(&mut self) {
        self.update_markings();
    }

    /// Unmark any marked segment.
    fn clean_gfx(&mut self) {
        self.gfx_handle.borrow_mut().set_tree_markers(vec![], None);
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
    }
}

impl ToolInstance<BulldozeTool> {
    fn bd_trees(&self) -> bool {
        self.state_handle.bulldoze_state.bulldoze_trees
    }

    fn bd_segments(&self) -> bool {
        self.state_handle.bulldoze_state.bulldoze_segments
    }

    fn update_markings(&mut self) {
        self.gfx_handle.borrow_mut().set_tree_markers(vec![], None);
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);

        if self.bd_trees() {
            if let Some(id) = self.world.get_tree_from_pos(self.ground_pos) {
                self.gfx_handle.borrow_mut().set_tree_markers(
                    vec![self.world.get_tree_pos(id).into()],
                    Some(rgba_d(colors::RED)),
                );
                return;
            }
        }
        if self.bd_segments() {
            if let Some(id) = self.world.get_segment_from_pos(self.ground_pos) {
                self.gfx_handle.borrow_mut().mark_road_segments(vec![id]);
            }
        }
    }
}
