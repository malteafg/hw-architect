use gfx_api::GfxRoadData;
use utils::input;

use std::cell::RefCell;
use std::rc::Rc;

pub mod camera_controller;
mod road_tool;


trait Tool {
    /// The tool shall process the given {`KeyAction`}. This happens when a key click should be
    /// used by the tool in question.
    fn process_keyboard(&mut self, key: input::KeyAction);

    /// The tool shall process a left click.
    fn left_click(&mut self);

    /// The tool shall process a right click.
    fn right_click(&mut self);

    /// This function should be called whenever there is an update to where the mouse points on the
    /// ground. This includes mouse movement and camera movement.
    fn update_ground_pos(&mut self, ground_pos: glam::Vec3);
}

pub struct WorldTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    road_tool: road_tool::RoadToolState,
    // curr_tool: Option<dyn Tool>,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>) -> Self {
        let gfx_handle_road = Rc::clone(&gfx_handle);
        WorldTool {
            gfx_handle,
            road_tool: road_tool::RoadToolState::new(gfx_handle_road),
        }
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        // switch tools using leader keybindings
        self.road_tool.process_keyboard(key);
    }

    pub fn mouse_input(&mut self, event: input::MouseEvent) {
        use input::{Mouse, MouseEvent};

        let MouseEvent::Click(button) = event else {
            return
        };

        match button {
            Mouse::Left => self.road_tool.left_click(),
            Mouse::Right => self.road_tool.right_click(),
            _ => {}
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: glam::Vec3) {
        self.road_tool.update_ground_pos(ground_pos);
    }

    // add gfx_clean method?
}
