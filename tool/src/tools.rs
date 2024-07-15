mod bulldoze;
mod construct;
mod tree_plopper;

pub use bulldoze::BulldozeTool;
pub use construct::ConstructTool;
pub use tree_plopper::TreePlopperTool;

use crate::tool_state::ToolState;

use gfx_api::GfxWorldData;
use utils::input;
use world_api::WorldManipulator;

use glam::Vec3;

pub trait Tool<G: GfxWorldData>: ToolStrategy<G> + ToolShared<G> {}

pub trait ToolShared<G: GfxWorldData> {
    fn destroy(self: Box<Self>) -> (ToolState, Box<dyn WorldManipulator>);

    // fn get_state(&self) -> &ToolState;
    // fn get_world(&self) -> &Box<dyn WorldManipulator>;
    fn get_world_mut(&mut self) -> &mut Box<dyn WorldManipulator>;

    fn update_ground_pos(&mut self, ground_pos: Vec3);
}

pub trait ToolStrategy<G: GfxWorldData> {
    /// Called when the tool is first created.
    fn init(&mut self, gfx_handle: &mut G);

    /// The tool shall process the given {`KeyAction`}. This happens when a key click should be
    /// used by the tool in question.
    fn process_keyboard(&mut self, gfx_handle: &mut G, key: input::KeyAction);

    /// The tool shall process a left click.
    fn left_click(&mut self, gfx_handle: &mut G);

    /// The tool shall process a right click.
    fn right_click(&mut self, gfx_handle: &mut G);

    /// This function should be called whenever there the ground_pos has been updated due to a
    /// change in camera or cursor position.
    fn update_view(&mut self, gfx_handle: &mut G);

    /// This function is used to reset whatever a tool has given to the gpu, such that the next
    /// tool can manipulate the graphics from scratch, as it desires.
    fn clean_gfx(&mut self, gfx_handle: &mut G);
}

pub struct ToolInstance<A: Default> {
    state_handle: ToolState,
    world: Box<dyn WorldManipulator>,
    ground_pos: Vec3,
    self_tool: A,
}

impl<A: Default> ToolInstance<A> {
    pub fn new(
        state_handle: ToolState,
        world: Box<dyn WorldManipulator>,
        ground_pos: Vec3,
    ) -> Self {
        Self {
            state_handle,
            world,
            ground_pos,
            self_tool: A::default(),
        }
    }
}

impl<A: Default, G: GfxWorldData> ToolShared<G> for ToolInstance<A> {
    // fn get_state(&self) -> &ToolState {
    //     &self.state_handle
    // }

    // fn get_world(&self) -> &Box<dyn WorldManipulator> {
    //     &self.world
    // }

    fn get_world_mut(&mut self) -> &mut Box<dyn WorldManipulator> {
        &mut self.world
    }

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
    }

    fn destroy(self: Box<Self>) -> (ToolState, Box<dyn WorldManipulator>) {
        (self.state_handle, self.world)
    }
}

/// Used as the default tool, when no tool is used.
#[derive(Default)]
pub struct NoTool;
impl<G: GfxWorldData> Tool<G> for ToolInstance<NoTool> {}
impl<G: GfxWorldData> ToolStrategy<G> for ToolInstance<NoTool> {
    fn init(&mut self, _gfx_handle: &mut G) {}
    fn process_keyboard(&mut self, _gfx_handle: &mut G, _key: input::KeyAction) {}
    fn left_click(&mut self, _gfx_handle: &mut G) {}
    fn right_click(&mut self, _gfx_handle: &mut G) {}
    fn update_view(&mut self, _gfx_handle: &mut G) {}
    fn clean_gfx(&mut self, _gfx_handle: &mut G) {}
}

/// This is a bit silly maybe find a cleaner implementation?
#[derive(Default)]
pub struct DummyTool;
impl<G: GfxWorldData> Tool<G> for DummyTool {}
impl<G: GfxWorldData> ToolStrategy<G> for DummyTool {
    fn init(&mut self, _gfx_handle: &mut G) {}
    fn process_keyboard(&mut self, _gfx_handle: &mut G, _key: input::KeyAction) {}
    fn left_click(&mut self, _gfx_handle: &mut G) {}
    fn right_click(&mut self, _gfx_handle: &mut G) {}
    fn update_view(&mut self, _gfx_handle: &mut G) {}
    fn clean_gfx(&mut self, _gfx_handle: &mut G) {}
}

impl<G: GfxWorldData> ToolShared<G> for DummyTool {
    fn destroy(self: Box<Self>) -> (ToolState, Box<dyn WorldManipulator>) {
        unreachable!()
    }

    // fn get_state(&self) -> &ToolState {
    //     unreachable!()
    // }

    // fn get_world(&self) -> &Box<dyn WorldManipulator> {
    //     unreachable!()
    // }

    fn get_world_mut(&mut self) -> &mut Box<dyn WorldManipulator> {
        unreachable!()
    }

    fn update_ground_pos(&mut self, _ground_pos: Vec3) {
        unreachable!()
    }
}
