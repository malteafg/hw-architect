use gfx_api::GfxRoadData;
use simulation::RoadGraph;
use std::cell::RefCell;
use std::rc::Rc;
use utils::input;

use crate::road_tool::{BulldozeTool, ConstructTool};
use crate::Tool;

#[derive(Debug, Clone, Copy)]
enum Tools {
    Dummy,
    Construct,
    Bulldoze,
}

/// The main tool that controls how other tools are invoked.
pub struct WorldTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    /// Uses {`Rc`}+{`RefCell`} as only one tool modifies the road_graph at one point as specified
    /// by {`curr_tool`}.
    road_graph: Rc<RefCell<RoadGraph>>,

    state: Rc<RefCell<super::ToolState>>,

    ground_pos: glam::Vec3,

    curr_tool_handle: Box<dyn Tool>,
    curr_tool: Tools,
    saved_tool: Option<Tools>,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>, road_graph: RoadGraph) -> Self {
        let road_graph = Rc::new(RefCell::new(road_graph));
        let start_tool = Box::new(DummyTool);
        let state = Rc::new(RefCell::new(super::ToolState::default()));

        let mut result = WorldTool {
            gfx_handle,
            road_graph,
            state,
            ground_pos: glam::Vec3::ZERO,
            curr_tool_handle: start_tool,
            curr_tool: Tools::Dummy,
            saved_tool: None,
        };
        result.enter_construct_mode();
        result
    }

    fn enter_bulldoze_mode(&mut self) {
        self.curr_tool = Tools::Bulldoze;
        self.curr_tool_handle.gfx_clean();
        self.curr_tool_handle = Box::new(BulldozeTool::new(
            Rc::clone(&self.gfx_handle),
            Rc::clone(&self.road_graph),
            self.ground_pos,
        ))
    }

    fn enter_construct_mode(&mut self) {
        self.saved_tool = None;
        self.curr_tool = Tools::Construct;
        self.curr_tool_handle.gfx_clean();
        self.curr_tool_handle = Box::new(ConstructTool::new(
            Rc::clone(&self.gfx_handle),
            Rc::clone(&self.road_graph),
            Rc::clone(&self.state),
            self.ground_pos,
        ))
    }

    fn enter_dummy_mode(&mut self) {
        self.saved_tool = None;
        self.curr_tool = Tools::Dummy;
        self.curr_tool_handle.gfx_clean();
        self.curr_tool_handle = Box::new(DummyTool)
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        // TODO add leader keybindings, but maybe they should be in InputHandler.
        use input::Action::*;
        use Tools::*;
        let (action, key_state) = key;
        if key_state == false {
            self.curr_tool_handle.process_keyboard(key);
            return;
        }
        match action {
            EnterBulldoze => match &mut self.curr_tool {
                Dummy => self.enter_bulldoze_mode(),
                Construct => {
                    self.saved_tool = Some(Construct);
                    self.enter_bulldoze_mode();
                }
                _ => {}
            },
            EnterConstruct => match &mut self.curr_tool {
                Dummy | Bulldoze => self.enter_construct_mode(),
                _ => {}
            },
            Esc => match &mut self.curr_tool {
                Bulldoze => match &self.saved_tool {
                    Some(_) => self.enter_construct_mode(),
                    None => self.enter_dummy_mode(),
                },
                Construct => self.enter_dummy_mode(),
                _ => {}
            },
            _ => self.curr_tool_handle.process_keyboard(key),
        }
    }

    pub fn mouse_input(&mut self, event: input::MouseEvent) {
        use input::{Mouse, MouseEvent};

        let MouseEvent::Click(button) = event else {
            return
        };

        match button {
            Mouse::Left => self.curr_tool_handle.left_click(),
            Mouse::Right => self.curr_tool_handle.right_click(),
            _ => {}
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: glam::Vec3) {
        self.ground_pos = ground_pos;
        self.curr_tool_handle.update_ground_pos(ground_pos);
    }
}

/// Used as the default tool, when no tool is used.
pub struct DummyTool;

impl Tool for DummyTool {
    fn process_keyboard(&mut self, _key: input::KeyAction) {}
    fn left_click(&mut self) {}
    fn right_click(&mut self) {}
    fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
    fn gfx_clean(&mut self) {}
}
