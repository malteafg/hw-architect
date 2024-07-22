mod bulldoze;
mod construct;
mod tree_plopper;

pub use bulldoze::Bulldoze;
pub use construct::Construct;
pub use tree_plopper::TreePlopper;

use crate::tool_state::ToolState;

use gfx_api::GfxWorldData;
use utils::input;
use world_api::WorldManipulator;

use glam::Vec3;

/// The total specification of a tool with both its shared and unique behaviour.
pub trait ToolSpec<G: GfxWorldData, W: WorldManipulator>: ToolUnique<G> + ToolShared<G, W> {}

/// Specification of the behaviour that is shared between tools.
pub trait ToolShared<G: GfxWorldData, W: WorldManipulator> {
    fn destroy(self: Box<Self>) -> (ToolState, W);

    // fn get_state(&self) -> &ToolState;
    // fn get_world(&self) -> &Box<dyn WorldManipulator>;
    fn get_world_mut(&mut self) -> &mut W;

    fn update_ground_pos(&mut self, ground_pos: Vec3);
}

/// Specification of the behaviour that is unique to a single tool.
pub trait ToolUnique<G: GfxWorldData> {
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

pub struct Tool<T: Default, W: WorldManipulator> {
    instance: T,
    state_handle: ToolState,
    world: W,
    ground_pos: Vec3,
}

impl<G: GfxWorldData, W: WorldManipulator, T: Default> ToolSpec<G, W> for Tool<T, W> where
    Tool<T, W>: ToolUnique<G>
{
}

impl<T: Default, W: WorldManipulator> Tool<T, W> {
    pub fn new(state_handle: ToolState, world: W, ground_pos: Vec3) -> Self {
        Self {
            instance: T::default(),
            state_handle,
            world,
            ground_pos,
        }
    }
}

impl<T: Default, G: GfxWorldData, W: WorldManipulator> ToolShared<G, W> for Tool<T, W> {
    // fn get_state(&self) -> &ToolState {
    //     &self.state_handle
    // }

    // fn get_world(&self) -> &Box<dyn WorldManipulator> {
    //     &self.world
    // }

    fn get_world_mut(&mut self) -> &mut W {
        &mut self.world
    }

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
    }

    fn destroy(self: Box<Self>) -> (ToolState, W) {
        (self.state_handle, self.world)
    }
}

/// Used as the default tool, when no tool is used.
#[derive(Default)]
pub struct NoTool;
impl<G: GfxWorldData, W: WorldManipulator> ToolUnique<G> for Tool<NoTool, W> {
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
impl<G: GfxWorldData, W: WorldManipulator> ToolSpec<G, W> for DummyTool {}
impl<G: GfxWorldData> ToolUnique<G> for DummyTool {
    fn init(&mut self, _gfx_handle: &mut G) {}
    fn process_keyboard(&mut self, _gfx_handle: &mut G, _key: input::KeyAction) {}
    fn left_click(&mut self, _gfx_handle: &mut G) {}
    fn right_click(&mut self, _gfx_handle: &mut G) {}
    fn update_view(&mut self, _gfx_handle: &mut G) {}
    fn clean_gfx(&mut self, _gfx_handle: &mut G) {}
}

impl<G: GfxWorldData, W: WorldManipulator> ToolShared<G, W> for DummyTool {
    fn destroy(self: Box<Self>) -> (ToolState, W) {
        unreachable!()
    }

    // fn get_state(&self) -> &ToolState {
    //     unreachable!()
    // }

    // fn get_world(&self) -> &Box<dyn WorldManipulator> {
    //     unreachable!()
    // }

    fn get_world_mut(&mut self) -> &mut W {
        unreachable!()
    }

    fn update_ground_pos(&mut self, _ground_pos: Vec3) {
        unreachable!()
    }
}
