use gfx_api::GfxRoadData;
use simulation::RoadGraph;
use std::cell::RefCell;
use std::rc::Rc;
use utils::input;

use crate::road_tool::{BulldozeTool, ConstructTool};

pub mod camera_controller;
mod road_tool;

trait Tool {
    // type InitParameters;

    // fn new(params: InitParameters, selection: Selection) -> Self;

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

    /// This function is used to reset whatever a tool has given to the gpu, such that the next
    /// tool can manipulate the graphics from scratch, as it desires.
    fn gfx_clean(&mut self);

    // fn destroy(self) -> Selection;
}

/// The main tool that controls how other tools are invoked.
pub struct WorldTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    /// Uses {`Rc`}+{`RefCell`} as only one tool modifies the road_graph at one point as specified
    /// by {`curr_tool`}.
    road_graph: Rc<RefCell<RoadGraph>>,
    curr_tool: Box<dyn Tool>,
    ground_pos: glam::Vec3,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>, road_graph: RoadGraph) -> Self {
        let road_graph = Rc::new(RefCell::new(road_graph));
        let start_tool = Box::new(DummyTool);

        WorldTool {
            gfx_handle,
            road_graph,
            curr_tool: start_tool,
            ground_pos: glam::Vec3::ZERO,
        }
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        // TODO add leader keybindings, but maybe they should be in InputHandler.
        use input::Action::*;
        let (action, _) = key;
        match action {
            EnterBulldoze => {
                self.curr_tool.gfx_clean();
                self.curr_tool = Box::new(BulldozeTool::new(
                    Rc::clone(&self.gfx_handle),
                    Rc::clone(&self.road_graph),
                    self.ground_pos,
                ))
            }
            EnterConstruct => {
                self.curr_tool.gfx_clean();
                self.curr_tool = Box::new(ConstructTool::new(
                    Rc::clone(&self.gfx_handle),
                    Rc::clone(&self.road_graph),
                    self.ground_pos,
                ))
            }
            Esc => {
                self.curr_tool.gfx_clean();
                self.curr_tool = Box::new(DummyTool)
            }
            _ => self.curr_tool.process_keyboard(key),
        }
    }

    pub fn mouse_input(&mut self, event: input::MouseEvent) {
        use input::{Mouse, MouseEvent};

        let MouseEvent::Click(button) = event else {
            return
        };

        match button {
            Mouse::Left => self.curr_tool.left_click(),
            Mouse::Right => self.curr_tool.right_click(),
            _ => {}
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: glam::Vec3) {
        self.ground_pos = ground_pos;
        self.curr_tool.update_ground_pos(ground_pos);
    }
}

/// Used as the default tool, when no tool is used.
struct DummyTool;

impl Tool for DummyTool {
    fn process_keyboard(&mut self, _key: input::KeyAction) {}
    fn left_click(&mut self) {}
    fn right_click(&mut self) {}
    fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
    fn gfx_clean(&mut self) {}
}
