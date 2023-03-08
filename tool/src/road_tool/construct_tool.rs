use crate::cycle_selection;
use crate::Tool;

use super::generator;
use super::generator::RoadGeneratorTool;
use super::SelectedRoad;

use gfx_api::GfxRoadData;
use glam::*;
use simulation::{CurveType, RoadGraph, SnapConfig};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use utils::input;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    SelectPos,
    SelectDir,
    Build,
}

pub struct ConstructTool {
    gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    road_graph: Rc<RefCell<RoadGraph>>,

    state_handle: Rc<RefCell<crate::ToolState>>,
    sel_node: Option<SnapConfig>,
    snapped_node: Option<SnapConfig>,
    road_generator: RoadGeneratorTool,

    ground_pos: Vec3,
    mode: Mode,
}

impl crate::Tool for ConstructTool {
    fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        use input::KeyState::*;
        match key {
            (CycleRoadType, Press) => self.cycle_curve_type(),
            (ToggleSnapping, Press) => self.toggle_snapping(),
            (CycleLaneWidth, Scroll(scroll_state)) => self.cycle_lane_width(scroll_state),
            (OneLane, Press) => self.set_lane_no(1),
            (TwoLane, Press) => self.set_lane_no(2),
            (ThreeLane, Press) => self.set_lane_no(3),
            (FourLane, Press) => self.set_lane_no(4),
            (FiveLane, Press) => self.set_lane_no(5),
            (SixLane, Press) => self.set_lane_no(6),
            _ => {}
        }
    }

    fn left_click(&mut self) {
        match self.mode {
            Mode::SelectPos => {
                if let Some(snapped_node) = self.snapped_node.clone() {
                    self.select_node(snapped_node);
                } else {
                    self.road_generator = RoadGeneratorTool::new(
                        self.ground_pos,
                        None,
                        self.get_sel_road_type().clone(),
                        false,
                    );
                    self.gfx_handle
                        .borrow_mut()
                        .set_road_tool_mesh(self.road_generator.get_mesh());
                    self.mode = Mode::SelectDir;
                }
            }
            Mode::SelectDir => match self.get_sel_road_type().segment_type.curve_type {
                CurveType::Straight => self.build_road(),
                CurveType::Curved => {
                    self.road_generator.lock_dir(self.ground_pos);
                    self.road_generator.update_pos(self.ground_pos);
                    self.gfx_handle
                        .borrow_mut()
                        .set_road_tool_mesh(self.road_generator.get_mesh());

                    self.mode = Mode::Build;
                }
            },
            Mode::Build => self.build_road(),
        }
        self.show_snappable_nodes();
    }

    fn right_click(&mut self) {
        match self.mode {
            Mode::SelectPos => {
                #[cfg(debug_assertions)]
                {
                    self.road_graph
                        .borrow()
                        .debug_segment_from_pos(self.ground_pos);
                    self.road_graph
                        .borrow()
                        .debug_node_from_pos(self.ground_pos);
                }
            }
            Mode::SelectDir => self.reset(Mode::SelectPos),
            Mode::Build => match (
                self.get_sel_road_type().segment_type.curve_type,
                self.sel_node.clone(),
            ) {
                (CurveType::Curved, None) => {
                    self.road_generator.unlock_dir();
                    self.reset(Mode::SelectDir)
                }
                _ => self.reset(Mode::SelectPos),
            },
        }
        self.show_snappable_nodes();
    }

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
        self.check_snapping();
    }

    /// Remove node markings from gpu, and remove the road tool mesh.
    fn gfx_clean(&mut self) {
        self.gfx_handle.borrow_mut().set_node_markers(vec![]);
        self.gfx_handle
            .borrow_mut()
            .set_road_tool_mesh(Some(generator::empty_mesh()));
    }
}

impl ConstructTool {
    pub(crate) fn new(
        gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
        road_graph: Rc<RefCell<RoadGraph>>,
        state_handle: Rc<RefCell<crate::ToolState>>,
        ground_pos: Vec3,
    ) -> Self {
        let mut tool = Self {
            gfx_handle,
            road_graph,
            state_handle,
            sel_node: None,
            snapped_node: None,
            road_generator: RoadGeneratorTool::default(),
            ground_pos,
            mode: Mode::SelectPos,
        };
        tool.show_snappable_nodes();
        tool.update_ground_pos(ground_pos);
        tool
    }

    fn get_sel_road_type(&self) -> SelectedRoad {
        self.state_handle.borrow().road_state.selected_road
    }

    // #############################################################################################
    // Tool State Changes
    // #############################################################################################
    /// Switches the selected number of lanes.
    fn set_lane_no(&mut self, no_lanes: u8) {
        if self.get_sel_road_type().node_type.no_lanes == no_lanes {
            return;
        };
        self.state_handle
            .borrow_mut()
            .road_state
            .selected_road
            .node_type
            .no_lanes = no_lanes;
        self.road_generator.update_no_lanes(no_lanes);
        if self.sel_node.is_none() {
            self.check_snapping();
        } else {
            self.reset(Mode::SelectPos);
        }
        self.show_snappable_nodes();
    }

    /// Switches the curve type in use.
    fn cycle_curve_type(&mut self) {
        use CurveType::*;
        match self.get_sel_road_type().segment_type.curve_type {
            Straight => {
                self.state_handle
                    .borrow_mut()
                    .road_state
                    .selected_road
                    .segment_type
                    .curve_type = Curved;
                self.road_generator.update_curve_type(Curved);
                if let Mode::Build = self.mode {
                    self.check_snapping();
                }
            }
            Curved => {
                self.state_handle
                    .borrow_mut()
                    .road_state
                    .selected_road
                    .segment_type
                    .curve_type = Straight;
                self.road_generator.update_curve_type(Straight);
                if let Mode::Build = self.mode {
                    if self.sel_node.is_none() {
                        self.road_generator.unlock_dir();
                        self.mode = Mode::SelectDir;
                    }
                    self.check_snapping();
                }
            }
        }
        self.show_snappable_nodes();
        dbg!(self.get_sel_road_type().segment_type.curve_type);
    }

    /// Toggles snapping.
    fn toggle_snapping(&mut self) {
        let curr = self.state_handle.borrow().road_state.snapping;
        self.state_handle.borrow_mut().road_state.snapping = !curr;
        // Turn snapping on again
        if !curr {
            self.check_snapping();
            self.show_snappable_nodes();
            return;
        }
        // Turn snapping off
        if self.snapped_node.is_some() {
            self.snapped_node = None;
            self.check_snapping();
        }
        self.gfx_handle.borrow_mut().set_node_markers(vec![]);
        dbg!(self.state_handle.borrow().road_state.snapping);
    }

    fn cycle_lane_width(&mut self, scroll_state: utils::input::ScrollState) {
        cycle_selection::scroll_mut(
            &mut self
                .state_handle
                .borrow_mut()
                .road_state
                .selected_road
                .node_type
                .lane_width,
            scroll_state,
        );
        self.reset(Mode::SelectPos);
        dbg!(self.get_sel_road_type().node_type.lane_width);
    }

    // #############################################################################################
    // General tool implementations
    // #############################################################################################
    /// Invoked when a snapped node becomes selected.
    fn select_node(&mut self, snapped_node: SnapConfig) {
        let node_dir = self
            .road_graph
            .borrow()
            .get_node(snapped_node.get_id())
            .get_dir();

        self.road_generator = generator::RoadGeneratorTool::new(
            snapped_node.get_pos(),
            Some(node_dir),
            self.get_sel_road_type().clone(),
            snapped_node.is_reverse(),
        );
        self.road_generator.update_pos(self.ground_pos);
        self.gfx_handle
            .borrow_mut()
            .set_road_tool_mesh(self.road_generator.get_mesh());

        self.sel_node = Some(snapped_node);
        self.snapped_node = None;
        self.mode = Mode::Build;
    }

    /// Constructs the road that is being generated.
    fn build_road(&mut self) {
        let road_meshes = self.road_generator.get_road_meshes();
        let road_generator = self.road_generator.extract();

        let (new_node, segment_ids) = self.road_graph.borrow_mut().add_road(
            road_generator.into_lroad_generator(),
            self.sel_node.clone(),
            self.snapped_node.clone(),
        );

        let mut mesh_map = HashMap::new();
        for i in 0..segment_ids.len() {
            mesh_map.insert(segment_ids[i], road_meshes[i].clone());
        }
        self.gfx_handle.borrow_mut().add_road_meshes(mesh_map);

        // TODO have add_road return new_node in such a way that is not necessary to check snapped_node
        if self.snapped_node.is_some() {
            self.reset(Mode::SelectPos);
        } else if let Some(new_node) = new_node {
            self.select_node(new_node);
        } else {
            self.reset(Mode::SelectPos);
        }
    }

    /// Resets to the given mode, maintaining necessary information and unsnapping and deselecting
    /// nodes.
    fn reset(&mut self, new_mode: Mode) {
        if let Mode::SelectPos = new_mode {
            self.road_generator = RoadGeneratorTool::default();
        };
        self.sel_node = None;
        self.snapped_node = None;
        self.mode = new_mode;
        self.check_snapping();
        self.show_snappable_nodes();
    }

    // #############################################################################################
    // Snapping
    // #############################################################################################
    /// Updates the construct tool when there is no node that we should snap to.
    fn update_no_snap(&mut self) {
        self.snapped_node = None;
        let empty_mesh = Some(generator::empty_mesh());

        match self.mode {
            Mode::SelectPos => self.gfx_handle.borrow_mut().set_road_tool_mesh(empty_mesh),
            Mode::SelectDir => {
                self.road_generator.update_pos(self.ground_pos);
                self.gfx_handle
                    .borrow_mut()
                    .set_road_tool_mesh(self.road_generator.get_mesh());
            }
            Mode::Build => {
                self.road_generator.update_pos(self.ground_pos);
                self.gfx_handle
                    .borrow_mut()
                    .set_road_tool_mesh(self.road_generator.get_mesh());
            }
        };
    }

    /// Updates the construct tool with the snap configs from the snapped node.
    fn update_snap(&mut self, snap_configs: Vec<SnapConfig>) {
        match self.mode {
            Mode::SelectPos => {
                let snap_config = &snap_configs[0];
                let road_generator = RoadGeneratorTool::new(
                    snap_config.get_pos(),
                    Some(snap_config.get_dir()),
                    self.get_sel_road_type().clone(),
                    snap_config.is_reverse(),
                );
                self.snapped_node = Some(snap_config.clone());
                self.gfx_handle
                    .borrow_mut()
                    .set_road_tool_mesh(road_generator.get_mesh());
            }
            Mode::SelectDir | Mode::Build => {
                for snap_config in snap_configs {
                    if self
                        .road_generator
                        .try_snap(snap_config.clone(), self.sel_node.is_some())
                        .is_some()
                    {
                        self.snapped_node = Some(snap_config);
                        self.gfx_handle
                            .borrow_mut()
                            .set_road_tool_mesh(self.road_generator.get_mesh());
                        return;
                    }
                }
                self.update_no_snap();
            }
        };
    }

    /// Checks if there is a node that in should snap to, and in that case it snaps to that node.
    fn check_snapping(&mut self) {
        let node_snap_configs = if self.state_handle.borrow().road_state.snapping {
            self.road_graph
                .borrow()
                .get_snap_configs_closest_node(self.ground_pos, self.get_sel_road_type().node_type)
        } else {
            None
        };

        let Some((_snap_id, snap_configs)) = node_snap_configs else {
            self.update_no_snap();
            return
        };

        if snap_configs.is_empty() {
            self.update_no_snap();
        } else {
            self.update_snap(snap_configs);
        }
    }

    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes(&mut self) {
        if !self.state_handle.borrow().road_state.snapping {
            return;
        }
        let reverse = if self.sel_node.is_some() {
            self.road_generator.is_reverse()
        } else {
            None
        };
        let possible_snaps = self
            .road_graph
            .borrow()
            .get_possible_snap_nodes(reverse, self.get_sel_road_type().node_type)
            .iter()
            .map(|id| self.road_graph.borrow().get_node(*id).get_pos())
            .collect();

        self.gfx_handle
            .borrow_mut()
            .set_node_markers(possible_snaps);
    }
}
