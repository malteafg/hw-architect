use super::ToolStrategy;

use utils::input;
use world::{RoadManipulator, World};

use gfx_api::GfxRoadData;
use glam::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct BulldozeTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    world: World,
    ground_pos: Vec3,
}

impl ToolStrategy for BulldozeTool {
    fn process_keyboard(&mut self, _key: input::KeyAction) {}

    fn left_click(&mut self) {
        let segment_id = self
            .world
            .get_road_graph()
            .get_segment_inside(self.ground_pos);
        if let Some(id) = segment_id {
            if self.world.mut_road_graph().remove_segment(id) {
                self.gfx_handle.borrow_mut().remove_road_meshes(vec![id])
            }
        }
    }

    fn right_click(&mut self) {}

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
        self.check_segment();
    }

    /// Unmark any marked segment.
    fn destroy(self: Box<Self>) -> World {
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
        self.world
    }
}

impl BulldozeTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>, world: World, ground_pos: Vec3) -> Self {
        let mut tool = Self {
            gfx_handle,
            world,
            ground_pos,
        };
        tool.check_segment();
        tool
    }

    fn check_segment(&mut self) {
        let segment_id = self
            .world
            .get_road_graph()
            .get_segment_inside(self.ground_pos);
        if let Some(id) = segment_id {
            self.gfx_handle.borrow_mut().mark_road_segments(vec![id]);
            return;
        }
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
    }
}
