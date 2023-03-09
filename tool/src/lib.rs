use utils::input;

pub mod camera_controller;
mod cycle_selection;
mod road_tools;
mod world_tool;

use cycle_selection::CycleSelection;
pub use world_tool::WorldTool;

trait ToolStrategy {
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

#[derive(Debug, Clone, Copy)]
struct RoadState {
    pub selected_road: road_tools::SelectedRoad,
    pub snapping: bool,
}

impl Default for RoadState {
    fn default() -> Self {
        Self {
            selected_road: road_tools::SelectedRoad::default(),
            snapping: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ToolState {
    pub road_state: RoadState,
}

// trait TestTool {
//     // type InitParameters;

//     // fn new(params: InitParameters, selection: Selection) -> Self;

//     /// The tool shall process the given {`KeyAction`}. This happens when a key click should be
//     /// used by the tool in question.
//     fn process_keyboard(&mut self, key: input::KeyAction);

//     /// The tool shall process a left click.
//     fn left_click(&mut self);

//     /// The tool shall process a right click.
//     fn right_click(&mut self);

//     /// This function should be called whenever there is an update to where the mouse points on the
//     /// ground. This includes mouse movement and camera movement.
//     fn update_ground_pos(&mut self, ground_pos: glam::Vec3);

//     /// This function is used to reset whatever a tool has given to the gpu, such that the next
//     /// tool can manipulate the graphics from scratch, as it desires.
//     fn gfx_clean(&mut self);

//     // fn destroy(self) -> Selection;
// }

// use gfx_api::GfxRoadData;
// use glam::*;
// use simulation::{CurveType, RoadGraph, SnapConfig};
// use std::cell::RefCell;
// use std::rc::Rc;

// struct SuperTool<InvokedTool = NoTool> {
//     curr_tool: InvokedTool,
// }

// trait ToolInput<InvokedTool> {
//     fn process_keyboard(&mut self, _key: input::KeyAction);
// }

// impl SuperTool {
//     fn new(ground_pos: glam::Vec3) -> Self {
//         Self {
//             curr_tool: NoTool { ground_pos },
//         }
//     }
// }

// struct BuilderTool {
//     gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
//     road_graph: Rc<RefCell<RoadGraph>>,

//     sel_node: Option<SnapConfig>,
//     snapped_node: Option<SnapConfig>,

//     ground_pos: Vec3,
// }

// impl Tool for BuilderTool {
//     fn process_keyboard(&mut self, _key: input::KeyAction) {}
//     fn left_click(&mut self) {}
//     fn right_click(&mut self) {}
//     fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
//     fn gfx_clean(&mut self) {}
// }

// struct BulldozerTool {
//     gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
//     road_graph: Rc<RefCell<RoadGraph>>,
//     ground_pos: Vec3,
// }

// impl Tool for BulldozerTool {
//     fn process_keyboard(&mut self, _key: input::KeyAction) {}
//     fn left_click(&mut self) {}
//     fn right_click(&mut self) {}
//     fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
//     fn gfx_clean(&mut self) {}
// }

// struct NoTool {
//     ground_pos: glam::Vec3,
// }

// impl Tool for NoTool {
//     fn process_keyboard(&mut self, _key: input::KeyAction) {}
//     fn left_click(&mut self) {}
//     fn right_click(&mut self) {}
//     fn update_ground_pos(&mut self, _ground_pos: glam::Vec3) {}
//     fn gfx_clean(&mut self) {}
// }

// impl ToolInput<NoTool> for SuperTool<NoTool> {
//     fn process_keyboard(&mut self, _key: input::KeyAction) {

//     }
// }

// impl SuperTool<NoTool> {
//     fn update_tool(&mut self, key: input::KeyAction) {
//         match key {
//             EnterBulldoze => {
//                 self.curr_tool.gfx_clean();
//                 self.curr_tool.ground_pos = glam::Vec3::new(0.0, 0.0, 0.0);
//             }
//             _ => self.curr_tool.process_keyboard(key),
//         }
//     }
// }

// trait Test {
//     fn test(&mut self);
// }

// struct ToolTest<A: Tool> {
//     tool: A,
//     something: u32,
// }

// impl Test for ToolTest {
//     fn test(&mut self) {
//         print!("asori");
//     }
// }
