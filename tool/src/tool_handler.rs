use crate::tool_state::ToolState;
use crate::tools::{Bulldoze, Construct, DummyTool, NoTool, Tool, ToolSpec, TreePlopper};

use gfx_api::GfxWorldData;
use utils::input;
use world_api::WorldManipulator;

use glam::Vec3;

use std::time::Duration;

#[derive(Debug, Clone, Copy)]
enum ToolMarker {
    NoTool,
    Construct,
    Bulldoze,
    TreePlopper,
}

/// The main tool that controls how other tools are invoked.
pub struct ToolHandler<G: GfxWorldData> {
    ground_pos: glam::Vec3,

    curr_tool_handle: Box<dyn ToolSpec<G>>,
    curr_tool: ToolMarker,
    saved_tool: Option<ToolMarker>,
}

impl<G: GfxWorldData> ToolHandler<G> {
    pub fn new(gfx_handle: &mut G, world: Box<dyn WorldManipulator>) -> Self {
        let state = ToolState::default();
        let start_tool = Box::new(Tool::<NoTool>::new(state, world, Vec3::ZERO));
        let mut result = ToolHandler {
            ground_pos: Vec3::ZERO,
            curr_tool_handle: start_tool,
            curr_tool: ToolMarker::NoTool,
            saved_tool: None,
        };
        result.enter_construct_mode(gfx_handle);
        result
    }

    fn enter_bulldoze_mode(&mut self, gfx_handle: &mut G) {
        self.curr_tool = ToolMarker::Bulldoze;
        self.enter_tool::<Bulldoze>(gfx_handle);
    }

    fn enter_construct_mode(&mut self, gfx_handle: &mut G) {
        self.saved_tool = None;
        self.curr_tool = ToolMarker::Construct;
        self.enter_tool::<Construct>(gfx_handle);
    }

    fn enter_tree_plopper_mode(&mut self, gfx_handle: &mut G) {
        self.saved_tool = None;
        self.curr_tool = ToolMarker::TreePlopper;
        self.enter_tool::<TreePlopper>(gfx_handle);
    }

    fn enter_no_tool(&mut self, gfx_handle: &mut G) {
        self.saved_tool = None;
        self.curr_tool = ToolMarker::NoTool;
        self.enter_tool::<NoTool>(gfx_handle);
    }

    fn enter_tool<T: Default + 'static>(&mut self, gfx_handle: &mut G)
    where
        Tool<T>: ToolSpec<G>,
    {
        let mut old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        old_tool.clean_gfx(gfx_handle);
        let (tool_state, world) = old_tool.destroy();

        self.curr_tool_handle = Box::new(Tool::<T>::new(tool_state, world, self.ground_pos));
        self.curr_tool_handle.init(gfx_handle);
    }

    pub fn process_keyboard(&mut self, gfx_handle: &mut G, key: input::KeyAction) {
        // TODO add leader keybindings, but maybe they should be in InputHandler.
        use input::Action::*;
        use input::KeyState::*;
        use ToolMarker::*;
        match key {
            (EnterBulldoze, Press) => match &mut self.curr_tool {
                Construct => {
                    self.saved_tool = Some(Construct);
                    self.enter_bulldoze_mode(gfx_handle);
                }
                Bulldoze => return,
                _ => self.enter_bulldoze_mode(gfx_handle),
            },
            (EnterConstruct, Press) => match &mut self.curr_tool {
                Construct => return,
                _ => self.enter_construct_mode(gfx_handle),
            },
            (EnterTreePlopper, Press) => match &mut self.curr_tool {
                TreePlopper => return,
                _ => self.enter_tree_plopper_mode(gfx_handle),
            },
            (Esc, Press) => match &mut self.curr_tool {
                Bulldoze => match &self.saved_tool {
                    Some(_) => self.enter_construct_mode(gfx_handle),
                    None => self.enter_no_tool(gfx_handle),
                },
                NoTool => return,
                _ => self.enter_no_tool(gfx_handle),
            },
            _ => self.curr_tool_handle.process_keyboard(gfx_handle, key),
        }
    }

    pub fn mouse_input(&mut self, gfx_handle: &mut G, event: input::MouseEvent) {
        use input::{Mouse, MouseEvent};

        let MouseEvent::Press(button) = event else {
            return;
        };

        match button {
            Mouse::Left => self.curr_tool_handle.left_click(gfx_handle),
            Mouse::Right => self.curr_tool_handle.right_click(gfx_handle),
            _ => {}
        }
    }

    pub fn update_ground_pos(&mut self, gfx_handle: &mut G, ground_pos: glam::Vec3) {
        self.ground_pos = ground_pos;
        self.curr_tool_handle.update_ground_pos(ground_pos);
        self.curr_tool_handle.update_view(gfx_handle);
    }

    pub fn update(&mut self, dt: Duration) {
        self.curr_tool_handle.get_world_mut().update(dt);
    }
}
