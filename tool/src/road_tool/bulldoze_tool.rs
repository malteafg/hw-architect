use gfx_api::GfxRoadData;
use glam::*;
use simulation::RoadGraph;
use std::cell::RefCell;
use std::rc::Rc;
use utils::id::SegmentId;
use utils::input;

pub struct BulldozeTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    road_graph: Rc<RefCell<RoadGraph>>,

    ground_pos: Vec3,
}

impl crate::Tool for BulldozeTool {
    fn process_keyboard(&mut self, key: input::KeyAction) {
        let (_action, pressed) = key;
        if pressed {
            return;
        }
    }

    fn left_click(&mut self) {
        let segment_id = self.road_graph.borrow().get_segment_inside(self.ground_pos);
        if let Some(id) = segment_id {
            if self.road_graph.borrow_mut().remove_segment(id) {
                self.gfx_handle.borrow_mut().remove_road_meshes(vec![id])
            }
        }
    }

    fn right_click(&mut self) {}

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;

        let segment = self.road_graph.borrow().get_segment_inside(self.ground_pos);
        self.mark_segment(segment);
    }

    /// Unmark any marked segment.
    fn gfx_clean(&mut self) {
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
    }
}

impl BulldozeTool {
    pub fn new(
        gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
        road_graph: Rc<RefCell<RoadGraph>>,
    ) -> Self {
        Self {
            gfx_handle,
            road_graph,
            ground_pos: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    fn mark_segment(&mut self, segment_id: Option<SegmentId>) {
        if let Some(id) = segment_id {
            self.gfx_handle.borrow_mut().mark_road_segments(vec![id]);
            return;
        }
        self.gfx_handle.borrow_mut().mark_road_segments(vec![]);
    }
}
