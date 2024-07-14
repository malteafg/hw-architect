mod bulldoze;
mod construct;
mod tree_plopper;
mod world_tool;

use bulldoze::BulldozeTool;
use construct::ConstructTool;
use tree_plopper::TreePlopperTool;
pub use world_tool::WorldTool;

use crate::tool_state::ToolState;

use gfx_api::GfxWorldData;
use utils::input;
use world_api::WorldManipulator;

use glam::Vec3;

pub trait Tool<G: GfxWorldData>: ToolStrategy<G> + ToolShared<G> {}

pub trait ToolShared<G: GfxWorldData> {
    fn destroy(self: Box<Self>) -> (ToolState, Box<dyn WorldManipulator>);

    fn get_state(&self) -> &ToolState;
    fn get_world(&self) -> &Box<dyn WorldManipulator>;
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
    fn get_state(&self) -> &ToolState {
        &self.state_handle
    }

    fn get_world(&self) -> &Box<dyn WorldManipulator> {
        &self.world
    }

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
