use super::{
    BulldozeTool, ConstructTool, Tool, ToolInstance, ToolShared, ToolStrategy, TreePlopperTool,
};
use crate::tool_state::ToolState;

use gfx_api::GfxSuper;
use utils::input;
use world_api::WorldManipulator;

use glam::Vec3;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
enum ToolMarker {
    NoTool,
    Construct,
    Bulldoze,
    TreePlopper,
}

/// The main tool that controls how other tools are invoked.
pub struct WorldTool {
    // gfx_handle: Rc<RefCell<dyn GfxWorldData>>,
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,
    ground_pos: glam::Vec3,

    curr_tool_handle: Box<dyn Tool>,
    curr_tool: ToolMarker,
    saved_tool: Option<ToolMarker>,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxSuper>>, world: Box<dyn WorldManipulator>) -> Self {
        let state = ToolState::default();
        let start_tool = Box::new(ToolInstance::<NoTool>::new(
            Rc::clone(&gfx_handle),
            state,
            world,
            Vec3::ZERO,
        ));
        let mut result = WorldTool {
            gfx_handle,
            ground_pos: Vec3::ZERO,
            curr_tool_handle: start_tool,
            curr_tool: ToolMarker::NoTool,
            saved_tool: None,
        };
        result.enter_construct_mode();
        result
    }

    fn enter_bulldoze_mode(&mut self) {
        self.curr_tool = ToolMarker::Bulldoze;
        self.enter_tool::<BulldozeTool>();
    }

    fn enter_construct_mode(&mut self) {
        self.saved_tool = None;
        self.curr_tool = ToolMarker::Construct;
        self.enter_tool::<ConstructTool>();
    }

    fn enter_tree_plopper_mode(&mut self) {
        self.saved_tool = None;
        self.curr_tool = ToolMarker::TreePlopper;
        self.enter_tool::<TreePlopperTool>();
    }

    fn enter_no_tool(&mut self) {
        self.saved_tool = None;
        self.curr_tool = ToolMarker::NoTool;
        self.enter_tool::<NoTool>();
    }

    fn enter_tool<A: Default + 'static>(&mut self)
    where
        ToolInstance<A>: Tool,
    {
        let mut old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        old_tool.clean_gfx();
        let (tool_state, world) = old_tool.destroy();

        self.curr_tool_handle = Box::new(ToolInstance::<A>::new(
            Rc::clone(&self.gfx_handle),
            tool_state,
            world,
            self.ground_pos,
        ));
        self.curr_tool_handle.init();
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        // TODO add leader keybindings, but maybe they should be in InputHandler.
        use input::Action::*;
        use input::KeyState::*;
        use ToolMarker::*;
        match key {
            (EnterBulldoze, Press) => match &mut self.curr_tool {
                Construct => {
                    self.saved_tool = Some(Construct);
                    self.enter_bulldoze_mode();
                }
                Bulldoze => return,
                _ => self.enter_bulldoze_mode(),
            },
            (EnterConstruct, Press) => match &mut self.curr_tool {
                Construct => return,
                _ => self.enter_construct_mode(),
            },
            (EnterTreePlopper, Press) => match &mut self.curr_tool {
                TreePlopper => return,
                _ => self.enter_tree_plopper_mode(),
            },
            (Esc, Press) => match &mut self.curr_tool {
                Bulldoze => match &self.saved_tool {
                    Some(_) => self.enter_construct_mode(),
                    None => self.enter_no_tool(),
                },
                NoTool => return,
                _ => self.enter_no_tool(),
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
        self.curr_tool_handle.update_view();
    }

    pub fn prepare_gfx(&mut self) {
        self.gfx_handle.borrow_mut().set_cars(vec![]);
    }
}

/// Used as the default tool, when no tool is used.
#[derive(Default)]
struct NoTool;
impl Tool for ToolInstance<NoTool> {}
impl ToolStrategy for ToolInstance<NoTool> {
    fn init(&mut self) {}
    fn process_keyboard(&mut self, _key: input::KeyAction) {}
    fn left_click(&mut self) {}
    fn right_click(&mut self) {}
    fn update_view(&mut self) {}
    fn clean_gfx(&mut self) {}
}

/// This is a bit silly maybe find a cleaner implementation?
#[derive(Default)]
struct DummyTool;
impl Tool for DummyTool {}
impl ToolStrategy for DummyTool {
    fn init(&mut self) {}
    fn process_keyboard(&mut self, _key: input::KeyAction) {}
    fn left_click(&mut self) {}
    fn right_click(&mut self) {}
    fn update_view(&mut self) {}
    fn clean_gfx(&mut self) {}
}

impl ToolShared for DummyTool {
    fn destroy(self: Box<Self>) -> (ToolState, Box<dyn WorldManipulator>) {
        panic!()
    }

    fn get_state(&self) -> &ToolState {
        panic!()
    }

    fn get_world(&self) -> &Box<dyn WorldManipulator> {
        panic!()
    }

    fn update_ground_pos(&mut self, _ground_pos: Vec3) {
        panic!()
    }
}
