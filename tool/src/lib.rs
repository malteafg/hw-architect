use gfx_api::GfxRoadData;
use utils::input;

use std::cell::RefCell;
use std::rc::Rc;

pub mod camera_controller;
mod road_tool;

pub trait Tool {
    fn process_keyboard(&mut self, key: input::KeyAction);
    fn mouse_input(&mut self, event: input::MouseEvent);

    /// This function should be called whenever there is an update to where the mouse points on the
    /// ground. This includes mouse movement and camera movement.
    fn update_ground_pos(&mut self, ground_pos: glam::Vec3);
}

pub struct WorldTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    road_tool: road_tool::ToolState,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>) -> Self {
        let gfx_handle_road = Rc::clone(&gfx_handle);
        WorldTool {
            gfx_handle,
            road_tool: road_tool::ToolState::new(gfx_handle_road),
        }
    }
}

impl Tool for WorldTool {
    fn process_keyboard(&mut self, key: input::KeyAction) {
        // switch tools using leader keybindings
        self.road_tool.process_keyboard(key);
    }

    fn mouse_input(&mut self, event: input::MouseEvent) {
        self.road_tool.mouse_input(event);
    }

    fn update_ground_pos(&mut self, ground_pos: glam::Vec3) {
        self.road_tool.update_ground_pos(ground_pos);
    }
}
