use gfx_api::GfxRoadData;
use std::cell::RefCell;
use std::rc::Rc;
use utils::input;
use world::World;

use crate::tool_state::ToolState;
use crate::tools::{BulldozeTool, ConstructTool, ToolStrategy};

#[derive(Debug, Clone, Copy)]
enum Tool {
    NoTool,
    Construct,
    Bulldoze,
}

/// The main tool that controls how other tools are invoked.
pub struct WorldTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,

    state: Rc<RefCell<ToolState>>,

    ground_pos: glam::Vec3,

    curr_tool_handle: Box<dyn ToolStrategy>,
    curr_tool: Tool,
    saved_tool: Option<Tool>,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxRoadData>>, world: World) -> Self {
        let start_tool = Box::new(NoTool::new(world));
        let state = Rc::new(RefCell::new(ToolState::default()));

        let mut result = WorldTool {
            gfx_handle,
            state,
            ground_pos: glam::Vec3::ZERO,
            curr_tool_handle: start_tool,
            curr_tool: Tool::NoTool,
            saved_tool: None,
        };
        result.enter_construct_mode();
        result
    }

    fn enter_bulldoze_mode(&mut self) {
        let old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        let world = old_tool.destroy();

        self.curr_tool = Tool::Bulldoze;
        self.curr_tool_handle = Box::new(BulldozeTool::new(
            Rc::clone(&self.gfx_handle),
            world,
            self.ground_pos,
        ))
    }

    fn enter_construct_mode(&mut self) {
        let old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        let world = old_tool.destroy();

        self.saved_tool = None;
        self.curr_tool = Tool::Construct;
        self.curr_tool_handle = Box::new(ConstructTool::new(
            Rc::clone(&self.gfx_handle),
            world,
            Rc::clone(&self.state),
            self.ground_pos,
        ))
    }

    fn enter_dummy_mode(&mut self) {
        let old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        let world = old_tool.destroy();

        self.saved_tool = None;
        self.curr_tool = Tool::NoTool;
        self.curr_tool_handle = Box::new(NoTool::new(world))
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        // TODO add leader keybindings, but maybe they should be in InputHandler.
        use input::Action::*;
        use input::KeyState::*;
        use Tool::*;
        match key {
            (EnterBulldoze, Press) => match &mut self.curr_tool {
                NoTool => self.enter_bulldoze_mode(),
                Construct => {
                    self.saved_tool = Some(Construct);
                    self.enter_bulldoze_mode();
                }
                _ => {}
            },
            (EnterConstruct, Press) => match &mut self.curr_tool {
                NoTool | Bulldoze => self.enter_construct_mode(),
                _ => {}
            },
            (Esc, Press) => match &mut self.curr_tool {
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

        let MouseEvent::Press(button) = event else {
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
struct NoTool {
    world: World,
}

impl NoTool {
    fn new(world: World) -> Self {
        NoTool { world }
    }
}

impl ToolStrategy for NoTool {
    fn process_keyboard(&mut self, _key: input::KeyAction) {}
    fn left_click(&mut self) {}
    fn right_click(&mut self) {}
    fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
    fn destroy(self: Box<Self>) -> World {
        self.world
    }
}

/// This is a bit silly maybe find a cleaner implementation?
struct DummyTool;
impl ToolStrategy for DummyTool {
    fn process_keyboard(&mut self, _key: input::KeyAction) {}
    fn left_click(&mut self) {}
    fn right_click(&mut self) {}
    fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
    fn destroy(self: Box<Self>) -> World {
        todo!()
    }
}
