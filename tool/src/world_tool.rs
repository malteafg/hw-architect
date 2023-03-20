use gfx_api::GfxSuper;
use std::cell::RefCell;
use std::rc::Rc;
use utils::id::{IdManager, TreeId};
use utils::input;
use world::World;

use crate::tool_state::ToolState;
use crate::tools::{BulldozeTool, ConstructTool, ToolStrategy, TreePlopperTool};

#[derive(Debug, Clone, Copy)]
enum Tool {
    NoTool,
    Construct,
    Bulldoze,
    TreePlopper,
}

/// The main tool that controls how other tools are invoked.
pub struct WorldTool {
    // gfx_handle: Rc<RefCell<dyn GfxWorldData>>,
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,

    state: Rc<RefCell<ToolState>>,

    ground_pos: glam::Vec3,

    curr_tool_handle: Box<dyn ToolStrategy>,
    curr_tool: Tool,
    saved_tool: Option<Tool>,

    // only temporary
    tree_id_manager: IdManager<TreeId>,
    tree_id: TreeId,
}

impl WorldTool {
    pub fn new(gfx_handle: Rc<RefCell<dyn GfxSuper>>, world: World) -> Self {
        let start_tool = Box::new(NoTool::new(world));
        let state = Rc::new(RefCell::new(ToolState::default()));

        let mut tree_id_manager = IdManager::new();
        let tree_id = tree_id_manager.gen();

        let mut result = WorldTool {
            gfx_handle,
            state,
            ground_pos: glam::Vec3::ZERO,
            curr_tool_handle: start_tool,
            curr_tool: Tool::NoTool,
            saved_tool: None,
            tree_id_manager,
            tree_id,
        };
        result.enter_construct_mode();
        result
    }

    fn enter_bulldoze_mode(&mut self) {
        let old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        let world = old_tool.destroy();

        let tool_gfx_handle = Rc::clone(&self.gfx_handle);

        self.curr_tool = Tool::Bulldoze;
        self.curr_tool_handle = Box::new(BulldozeTool::new(tool_gfx_handle, world, self.ground_pos))
    }

    fn enter_construct_mode(&mut self) {
        let old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        let world = old_tool.destroy();

        let tool_gfx_handle = Rc::clone(&self.gfx_handle);

        self.saved_tool = None;
        self.curr_tool = Tool::Construct;
        self.curr_tool_handle = Box::new(ConstructTool::new(
            tool_gfx_handle,
            world,
            Rc::clone(&self.state),
            self.ground_pos,
        ))
    }

    fn enter_tree_plopper_mode(&mut self) {
        let old_tool = std::mem::replace(&mut self.curr_tool_handle, Box::new(DummyTool));
        let world = old_tool.destroy();

        let tool_gfx_handle = Rc::clone(&self.gfx_handle);

        self.saved_tool = None;
        self.curr_tool = Tool::TreePlopper;
        self.curr_tool_handle = Box::new(TreePlopperTool::new(
            tool_gfx_handle,
            world,
            self.ground_pos,
            self.tree_id,
        ));
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
                    None => self.enter_dummy_mode(),
                },
                NoTool => return,
                _ => self.enter_dummy_mode(),
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
