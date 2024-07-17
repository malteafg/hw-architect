use super::{Tool, ToolSpec, ToolUnique};

use utils::input;

use gfx_api::{
    colors::{self, rgba_d},
    GfxWorldData,
};
use glam::*;
use world_api::WorldManipulator;

#[derive(Default)]
pub struct Bulldoze;

impl<G: GfxWorldData, W: WorldManipulator> ToolSpec<G, W> for Tool<Bulldoze, W> {}

impl<G: GfxWorldData, W: WorldManipulator> ToolUnique<G> for Tool<Bulldoze, W> {
    fn init(&mut self, gfx_handle: &mut G) {
        self.update_view(gfx_handle);
    }

    fn process_keyboard(&mut self, gfx_handle: &mut G, key: input::KeyAction) {
        use input::Action::*;
        use input::KeyState::*;
        match key {
            (ToggleBulldozeRoads, Press) => {
                let curr = self.bd_segments();
                self.state_handle.bulldoze_state.bulldoze_segments = !curr;
                if curr {
                    gfx_handle.mark_road_segments(vec![]);
                }
                self.update_markings(gfx_handle);
            }
            (ToggleBulldozeTrees, Press) => {
                let curr = self.bd_trees();
                self.state_handle.bulldoze_state.bulldoze_trees = !curr;
                self.update_markings(gfx_handle);
            }
            _ => {}
        }
    }

    fn left_click(&mut self, gfx_handle: &mut G) {
        if self.bd_trees() {
            if let Some(tree_id) = self.world.get_tree_from_pos(self.ground_pos) {
                self.world.remove_tree(tree_id);
                gfx_handle.remove_tree(tree_id, utils::consts::TREE_MODEL_ID);
                self.update_markings(gfx_handle);
                return;
            }
        }

        if self.bd_segments() {
            if let Some(segment_id) = self.world.get_segment_from_pos(self.ground_pos) {
                if self.world.remove_segment(segment_id) {
                    gfx_handle.remove_road_meshes(vec![segment_id]);
                    self.update_markings(gfx_handle);
                }
            }
        }
    }

    fn right_click(&mut self, _gfx_handle: &mut G) {}

    fn update_view(&mut self, gfx_handle: &mut G) {
        self.update_markings(gfx_handle);
    }

    /// Unmark any marked segment.
    fn clean_gfx(&mut self, gfx_handle: &mut G) {
        gfx_handle.set_tree_markers(vec![], None);
        gfx_handle.mark_road_segments(vec![]);
    }
}

impl<W: WorldManipulator> Tool<Bulldoze, W> {
    fn bd_trees(&self) -> bool {
        self.state_handle.bulldoze_state.bulldoze_trees
    }

    fn bd_segments(&self) -> bool {
        self.state_handle.bulldoze_state.bulldoze_segments
    }

    fn update_markings<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        gfx_handle.set_tree_markers(vec![], None);
        gfx_handle.mark_road_segments(vec![]);

        if self.bd_trees() {
            if let Some(id) = self.world.get_tree_from_pos(self.ground_pos) {
                gfx_handle.set_tree_markers(
                    vec![self.world.get_tree_pos(id).into()],
                    Some(rgba_d(colors::RED)),
                );
                return;
            }
        }
        if self.bd_segments() {
            if let Some(id) = self.world.get_segment_from_pos(self.ground_pos) {
                gfx_handle.mark_road_segments(vec![id]);
            }
        }
    }
}
