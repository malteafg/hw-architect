use super::{Tool, ToolInstance, ToolStrategy};
use crate::cycle_selection;
use crate::gfx_gen::segment_gen;
use crate::tool_state::SelectedRoad;

use utils::{input, VecUtils};
use world_api::{
    CurveType, LNodeBuilder, LNodeBuilderType, LRoadBuilder, LaneWidth, NodeType, SegmentType,
    Side, SnapConfig,
};

use gfx_api::RoadMesh;
use glam::*;

use std::collections::HashMap;

#[derive(Default)]
/// Defines the mode of the construct tool. At any time can the user snap to a node, which will
/// result in a change in the generated node. Data is small so clone is fine.
enum Mode {
    /// The user must select a position to build a road from.
    #[default]
    SelectPos,
    /// The user must select the direction that the road shall have. Left clicking will build the
    /// road in straight mode, or set the curves direction in curve mode.
    SelectDir {
        pos: Vec3,
        init_node_type: NodeType,
        road_builder: LRoadBuilder,
    },
    /// The user must select where the road should be built to.
    CurveEnd {
        pos: Vec3,
        dir: Vec3,
        init_node_type: NodeType,
        road_builder: LRoadBuilder,
    },
    /// The user has selected a node and must therefore select where the road should be built to.
    SelNode {
        snap_config: SnapConfig,
        road_builder: LRoadBuilder,
    },
}
use Mode::*;

pub struct ConstructTool {
    snapped_node: Option<SnapConfig>,
    mode: Mode,
}

impl Tool for ToolInstance<ConstructTool> {}

impl Default for ConstructTool {
    fn default() -> Self {
        Self {
            snapped_node: None,
            mode: Mode::default(),
        }
    }
}

impl ToolStrategy for ToolInstance<ConstructTool> {
    fn init(&mut self) {
        self.update_view();
        self.show_snappable_nodes();
    }

    fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        use input::KeyState::*;
        match key {
            (ToggleSnapping, Press) => self.toggle_snapping(),
            (ToggleReverse, Press) => self.toggle_reverse(),
            (CycleCurveType, Scroll(scroll_state)) => {
                let new_curve_type = cycle_selection::scroll_mut(
                    &mut self
                        .state_handle
                        .road_state
                        .selected_road
                        .segment_type
                        .curve_type,
                    scroll_state,
                );
                self.set_curve_type(new_curve_type);
            }
            (CycleLaneWidth, Scroll(scroll_state)) => {
                let new_lane_width = cycle_selection::scroll_mut(
                    &mut self
                        .state_handle
                        .road_state
                        .selected_road
                        .node_type
                        .lane_width(),
                    scroll_state,
                );
                self.set_lane_width(new_lane_width);
            }
            (CycleNoLanes, Scroll(scroll_state)) => {
                let new_no_lanes = cycle_selection::scroll_mut(
                    &mut self
                        .state_handle
                        .road_state
                        .selected_road
                        .node_type
                        .no_lanes(),
                    scroll_state,
                );
                self.set_no_lanes(new_no_lanes);
            }
            _ => {}
        }
    }

    fn left_click(&mut self) {
        let prev_move = std::mem::take(&mut self.self_tool.mode);
        // The proper mode should be set in all branches of match.
        match prev_move {
            SelectPos => {
                if self.try_select_node() {
                    return;
                }
                self.update_to_select_dir(self.ground_pos, self.get_sel_node_type())
            }
            SelectDir {
                pos,
                init_node_type,
                road_builder,
            } => match self.get_sel_curve_type() {
                CurveType::Straight => self.build_road(road_builder),
                CurveType::Curved => {
                    if self.self_tool.snapped_node.is_some() {
                        self.build_road(road_builder)
                    } else {
                        let dir = (self.ground_pos - pos).normalize_else();
                        self.update_to_cc_curve_end(pos, dir, init_node_type)
                    }
                }
            },
            CurveEnd { road_builder, .. } => self.build_road(road_builder),
            SelNode { road_builder, .. } => self.build_road(road_builder),
        }
    }

    fn right_click(&mut self) {
        match &self.self_tool.mode {
            Mode::SelectPos => {
                #[cfg(debug_assertions)]
                {
                    if let Some(id) = self.world.get_node_from_pos(self.ground_pos) {
                        self.world.debug_node(id);
                    } else if let Some(id) = self.world.get_segment_from_pos(self.ground_pos) {
                        self.world.debug_segment(id);
                    }
                }
            }
            SelectDir { .. } => self.reset(),
            CurveEnd {
                pos,
                init_node_type,
                ..
            } => self.update_to_select_dir(*pos, *init_node_type),
            SelNode { .. } => self.reset(),
        }
    }

    fn update_view(&mut self) {
        self.check_snapping();
    }

    /// Remove node markings from gpu, and remove the road tool mesh.
    fn clean_gfx(&mut self) {
        self.gfx_handle.borrow_mut().set_node_markers(vec![]);
        self.gfx_handle.borrow_mut().set_road_tool_mesh(None);
    }
}

impl ToolInstance<ConstructTool> {
    fn get_sel_road_type(&self) -> SelectedRoad {
        self.state_handle.road_state.selected_road
    }

    fn get_sel_curve_type(&self) -> CurveType {
        self.state_handle
            .road_state
            .selected_road
            .segment_type
            .curve_type
    }

    fn _get_sel_segment_type(&self) -> SegmentType {
        self.state_handle.road_state.selected_road.segment_type
    }

    fn get_sel_node_type(&self) -> NodeType {
        self.state_handle.road_state.selected_road.node_type
    }

    fn is_reverse(&self) -> bool {
        self.state_handle.road_state.reverse
    }

    fn compute_reverse(&self) -> bool {
        match &self.self_tool.mode {
            SelectPos | SelectDir { .. } | CurveEnd { .. } => {
                if let Some(snap) = &self.self_tool.snapped_node {
                    snap.side() == Side::Out
                } else {
                    self.is_reverse()
                }
            }
            SelNode { snap_config, .. } => snap_config.side() == Side::In,
        }
    }

    // #############################################################################################
    // Tool State Changes
    // #############################################################################################
    /// Toggles snapping.
    fn toggle_snapping(&mut self) {
        let curr = self.state_handle.road_state.snapping;
        self.state_handle.road_state.snapping = !curr;
        // Turn snapping on
        if !curr {
            self.check_snapping();
            self.show_snappable_nodes();
            dbg!(self.state_handle.road_state.snapping);
            return;
        }
        // Turn snapping off
        if self.self_tool.snapped_node.is_some() {
            self.self_tool.snapped_node = None;
            self.check_snapping();
        }
        self.gfx_handle.borrow_mut().set_node_markers(vec![]);
        dbg!(self.state_handle.road_state.snapping);
    }

    /// Toggles reverse.
    fn toggle_reverse(&mut self) {
        let curr = self.state_handle.road_state.reverse;
        self.state_handle.road_state.reverse = !curr;
        dbg!(self.state_handle.road_state.reverse);
    }

    /// Sets the curve type in use.
    fn set_curve_type(&mut self, new_curve_type: CurveType) {
        match new_curve_type {
            CurveType::Straight => match &self.self_tool.mode {
                SelectPos | SelectDir { .. } => {}
                CurveEnd {
                    pos,
                    init_node_type,
                    ..
                } => {
                    self.update_to_select_dir(*pos, *init_node_type);
                }
                SelNode { .. } => self.update_no_snap(),
            },
            CurveType::Curved => match &self.self_tool.mode {
                SelectPos | SelectDir { .. } => {}
                CurveEnd { .. } | SelNode { .. } => self.update_no_snap(),
            },
        };
        dbg!(new_curve_type);
    }

    /// Sets the lane width in use.
    fn set_lane_width(&mut self, _new_lane_width: LaneWidth) {
        self.reset();
        dbg!(self.get_sel_road_type().node_type.lane_width_f32());
    }

    /// Sets the selected number of lanes.
    fn set_no_lanes(&mut self, _no_lanes: u8) {
        dbg!(self.get_sel_road_type().node_type.no_lanes());
        self.show_snappable_nodes();
        if let SelNode { .. } = self.self_tool.mode {
            self.reset();
        } else {
            self.check_snapping();
        }
    }

    // #############################################################################################
    // General tool implementations
    // #############################################################################################
    fn try_select_node(&mut self) -> bool {
        if let Some(snap_config) = self.self_tool.snapped_node.take() {
            self.select_node(snap_config);
            return true;
        };
        false
    }

    /// Invoked when a snapped node becomes selected.
    fn select_node(&mut self, snap_config: SnapConfig) {
        self.update_to_sld(snap_config);
        self.show_snappable_nodes();
    }

    /// Constructs the road that is being generated.
    fn build_road(&mut self, road_builder: LRoadBuilder) {
        let next_node_type = self.get_sel_node_type();
        let road_meshes = self.gen_road_mesh_from_builder(&road_builder, self.get_sel_node_type());
        let (new_node, segment_ids) = self.world.add_road(road_builder, next_node_type);

        let mut mesh_map = HashMap::new();
        for i in 0..segment_ids.len() {
            mesh_map.insert(segment_ids[i], road_meshes[i].clone());
        }
        self.gfx_handle.borrow_mut().add_road_meshes(mesh_map);

        if self.self_tool.snapped_node.is_some() {
            self.self_tool.mode = SelectPos;
        } else if let Some(new_node) = new_node {
            self.select_node(new_node);
        } else {
            self.self_tool.mode = SelectPos;
        }
        self.show_snappable_nodes();
        self.check_snapping();
    }

    // #############################################################################################
    // Updating
    // #############################################################################################
    /// Sets the mode to select pos and checks for snapping and snappable nodes.
    fn reset(&mut self) {
        self.self_tool.mode = SelectPos;
        self.show_snappable_nodes();
        self.check_snapping();
    }

    /// This function will generate an sfd and set the mode to select dir. This can always be
    /// called when entering or updating select dir mode.
    fn update_to_select_dir(&mut self, first_pos: Vec3, init_node_type: NodeType) {
        let road_builder = LRoadBuilder::gen_sfd(
            first_pos,
            init_node_type,
            self.ground_pos,
            init_node_type,
            self.compute_reverse(),
        );
        self.update_road_tool_mesh(&road_builder);
        self.self_tool.mode = SelectDir {
            pos: first_pos,
            init_node_type,
            road_builder,
        }
    }

    /// Generates and sld and sets the mode to SelNode.
    fn update_to_sld(&mut self, snap_config: SnapConfig) {
        let reverse = snap_config.side() == Side::In;
        let road_builder = LRoadBuilder::gen_sld(
            snap_config.clone(),
            self.ground_pos,
            snap_config.node_type(),
            reverse,
        );
        self.update_road_tool_mesh(&road_builder);
        self.self_tool.mode = SelNode {
            snap_config,
            road_builder,
        }
    }

    /// Generates a cc curve and sets the mode to CurveEnd.
    fn update_to_cc_curve_end(&mut self, pos: Vec3, dir: Vec3, init_node_type: NodeType) {
        let last_pos = self.ground_pos;
        let road_builder = LRoadBuilder::gen_cc(
            LNodeBuilderType::new(pos, dir, init_node_type),
            last_pos,
            self.get_sel_node_type(),
            self.compute_reverse(),
        );
        self.update_road_tool_mesh(&road_builder);
        self.self_tool.mode = CurveEnd {
            pos,
            dir,
            init_node_type,
            road_builder,
        }
    }

    /// Generates a cc curve and sets the mode to SelNode.
    fn update_to_cc_sel_node(&mut self, snap_config: SnapConfig) {
        let last_pos = self.ground_pos;
        let road_builder = LRoadBuilder::gen_cc(
            LNodeBuilderType::Old(snap_config.clone()),
            last_pos,
            self.get_sel_node_type(),
            self.compute_reverse(),
        );
        self.update_road_tool_mesh(&road_builder);
        self.self_tool.mode = SelNode {
            snap_config,
            road_builder,
        }
    }

    /// Updates the construct tool when there is no node that we should snap to.
    fn update_no_snap(&mut self) {
        self.self_tool.snapped_node = None;
        match &self.self_tool.mode {
            SelectPos => self.gfx_handle.borrow_mut().set_road_tool_mesh(None),
            SelectDir {
                pos,
                init_node_type,
                ..
            } => self.update_to_select_dir(*pos, *init_node_type),
            CurveEnd {
                pos,
                dir,
                init_node_type,
                ..
            } => self.update_to_cc_curve_end(*pos, *dir, *init_node_type),
            SelNode { snap_config, .. } => match self.get_sel_curve_type() {
                CurveType::Straight => self.update_to_sld(snap_config.clone()),
                CurveType::Curved => self.update_to_cc_sel_node(snap_config.clone()),
            },
        };
    }

    // #############################################################################################
    // Snapping
    // #############################################################################################
    /// Updates the construct tool with the snap configs from the snapped node. If no snaps fit,
    /// then update_no_snap is called. This function is only called when there is at least one
    /// snap.
    fn update_snap(&mut self, snap_configs: Vec<SnapConfig>) {
        match &self.self_tool.mode {
            SelectPos => {
                // Snap does not have to satisfy any curvature constraints.
                let snap_config = snap_configs.into_iter().nth(0).unwrap();
                let pos = snap_config.pos();
                let dir = snap_config.dir();
                let node_type = snap_config.node_type();
                let reverse = snap_config.side() == Side::In;

                let road_builder =
                    LRoadBuilder::gen_stub(pos, dir.flip(reverse), node_type, reverse);
                self.update_road_tool_mesh(&road_builder);
                self.self_tool.snapped_node = Some(snap_config);
                return;
            }
            SelectDir {
                pos,
                init_node_type,
                ..
            } => {
                // attempt a ccs snap
                for snap_config in snap_configs.into_iter() {
                    let reverse = snap_config.side() == Side::Out;
                    let attempt =
                        LRoadBuilder::gen_ccs(*pos, *init_node_type, snap_config.clone(), reverse);
                    let Ok(road_builder) = attempt else {
                        // report to user?
                        continue;
                    };
                    self.update_road_tool_mesh(&road_builder);
                    self.self_tool.snapped_node = Some(snap_config);
                    self.self_tool.mode = SelectDir {
                        pos: *pos,
                        init_node_type: *init_node_type,
                        road_builder,
                    };
                    return;
                }
            }
            CurveEnd {
                pos,
                dir,
                init_node_type,
                ..
            } => {
                // attempt a ds snap
                for snap_config in snap_configs.into_iter() {
                    let reverse = snap_config.side() == Side::Out;
                    let attempt = LRoadBuilder::gen_ds(
                        LNodeBuilderType::New(LNodeBuilder::new(*pos, *dir, *init_node_type)),
                        snap_config.clone(),
                        reverse,
                    );
                    let Ok(road_builder) = attempt else {
                        // report to user?
                        continue;
                    };
                    self.update_road_tool_mesh(&road_builder);
                    self.self_tool.snapped_node = Some(snap_config);
                    self.self_tool.mode = CurveEnd {
                        pos: *pos,
                        dir: *dir,
                        init_node_type: *init_node_type,
                        road_builder,
                    };
                    return;
                }
            }
            SelNode { snap_config, .. } => {
                // attempt a ds snap
                for new_snap_config in snap_configs.into_iter() {
                    let reverse = self.compute_reverse();
                    let attempt = LRoadBuilder::gen_ds(
                        LNodeBuilderType::Old(snap_config.clone()),
                        new_snap_config.clone(),
                        reverse,
                    );
                    let Ok(road_builder) = attempt else {
                        // report to user?
                        continue;
                    };
                    self.update_road_tool_mesh(&road_builder);
                    self.self_tool.snapped_node = Some(new_snap_config);
                    self.self_tool.mode = SelNode {
                        snap_config: snap_config.clone(),
                        road_builder,
                    };
                    return;
                }
            }
        }
        self.self_tool.snapped_node = None;
        self.update_no_snap();
    }

    /// Checks if there is a node that we should snap to, and in that case it snaps to that node.
    fn check_snapping(&mut self) {
        // TODO add functionality to report why a node cannot be snapped to.
        if !self.state_handle.road_state.snapping {
            self.update_no_snap();
            return;
        }

        // Get available snaps
        let node_snap_configs = self
            .world
            .get_snap_configs_closest_node(self.ground_pos, self.get_sel_road_type().node_type);

        let Some((_snap_id, mut snap_configs)) = node_snap_configs else {
            self.update_no_snap();
            return
        };

        if let SelNode { snap_config, .. } = &self.self_tool.mode {
            snap_configs.retain(|s| s.side() != snap_config.side());
        }

        if snap_configs.is_empty() {
            self.update_no_snap();
            return;
        }

        self.update_snap(snap_configs);
    }

    // #############################################################################################
    // Gfx handling
    // #############################################################################################
    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes(&mut self) {
        if !self.state_handle.road_state.snapping {
            return;
        }
        let side = if let SelNode { snap_config, .. } = &self.self_tool.mode {
            Some(snap_config.side())
        } else {
            None
        };
        let possible_snaps = self
            .world
            .get_possible_snap_nodes(side, self.get_sel_road_type().node_type)
            .iter()
            .map(|(_id, pos, dir)| (<[f32; 3]>::from(*pos), <[f32; 3]>::from(*dir)))
            .collect();

        self.gfx_handle
            .borrow_mut()
            .set_node_markers(possible_snaps);
    }

    fn gen_road_mesh_from_builder(
        &self,
        road_builder: &LRoadBuilder,
        node_type: NodeType,
    ) -> Vec<RoadMesh> {
        road_builder
            .get_segments()
            .iter()
            .map(|s| segment_gen::gen_road_mesh_with_lanes(s.spine(), node_type))
            .collect::<Vec<RoadMesh>>()
    }

    fn update_road_tool_mesh(&self, road_builder: &LRoadBuilder) {
        let meshes =
            self.gen_road_mesh_from_builder(road_builder, self.get_sel_road_type().node_type);
        let mesh = segment_gen::combine_road_meshes_bad(meshes);
        self.gfx_handle.borrow_mut().set_road_tool_mesh(Some(mesh));
    }
}
