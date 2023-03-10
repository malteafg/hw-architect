mod bulldoze;
mod construct;

pub use bulldoze::BulldozeTool;
pub use construct::ConstructTool;

use utils::input;

pub trait ToolStrategy {
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
    fn destroy(self: Box<Self>) -> simulation::World;
}
