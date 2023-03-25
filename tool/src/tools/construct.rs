use super::ToolStrategy;
use crate::cycle_selection;
use crate::road_gen::mesh_gen;
use crate::tool_state::{SelectedRoad, ToolState};

use utils::{input, VecUtils};
use world::roads::{CurveType, LNodeBuilderType, LRoadBuilder, NodeType, SegmentType, SnapConfig};
use world::{RoadManipulator, World};

use gfx_api::{GfxSuper, RoadMesh};
use glam::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DEFAULT_DIR: Vec3 = Vec3::new(1.0, 0.0, 0.0);

/// Defines the mode of the construct tool. At any time can the user snap to a node, which will
/// result in a change in the generated node. Data is small so clone is fine.
#[derive(Default)]
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
        snap: SnapConfig,
        road_builder: LRoadBuilder,
    },
}
use Mode::*;

pub struct ConstructTool {
    // gfx_handle: Rc<RefCell<dyn GfxRoadData>>,
    gfx_handle: Rc<RefCell<dyn GfxSuper>>,
    world: World,

    state_handle: Rc<RefCell<ToolState>>,
    snapped_node: Option<SnapConfig>,

    ground_pos: Vec3,
    mode: Mode,
}

impl ToolStrategy for ConstructTool {
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
        let prev_mode = std::mem::take(&mut self.mode);
        match prev_mode {
            SelectPos => {
                if let Some(snapped_node) = self.snapped_node.clone() {
                    self.select_node(snapped_node);
                } else {
                    let pos = self.ground_pos;
                    // RG SFD
                    let road_type = self.get_sel_road_type();
                    let node_type = road_type.node_type;
                    let segment_type = road_type.segment_type;
                    let road_builder =
                        LRoadBuilder::gen_sfd(pos, node_type, pos, node_type, segment_type);
                    self.update_road_tool_mesh(&road_builder);
                    self.mode = SelectDir {
                        pos,
                        init_node_type: road_type.node_type,
                        road_builder,
                    };
                }
            }
            SelectDir {
                pos, road_builder, ..
            } => match self.get_sel_road_type().segment_type.curve_type {
                CurveType::Straight => self.build_road(road_builder),
                CurveType::Curved => {
                    if self.snapped_node.is_some() {
                        self.build_road(road_builder)
                    } else {
                        let dir = (pos - self.ground_pos).normalize_else();
                        self.mode = CurveEnd {
                            pos,
                            dir,
                            init_node_type: self.get_sel_road_type().node_type,
                            road_builder,
                        };
                    }
                }
            },
            CurveEnd { road_builder, .. } => self.build_road(road_builder),
            SelNode { road_builder, .. } => self.build_road(road_builder),
        }
        self.show_snappable_nodes();
    }

    fn right_click(&mut self) {
        match &self.mode {
            Mode::SelectPos => {
                #[cfg(debug_assertions)]
                {
                    self.world
                        .get_road_graph()
                        .debug_segment_from_pos(self.ground_pos);
                    self.world
                        .get_road_graph()
                        .debug_node_from_pos(self.ground_pos);
                }
            }
            SelectDir { .. } => self.reset(SelectPos),
            CurveEnd {
                pos,
                init_node_type,
                ..
            } => match self.get_sel_road_type().segment_type.curve_type {
                CurveType::Curved => {
                    // RG SFD
                    let road_type = self.get_sel_road_type();
                    let segment_type = road_type.segment_type;
                    let road_builder = LRoadBuilder::gen_sfd(
                        *pos,
                        *init_node_type,
                        *pos,
                        *init_node_type,
                        segment_type,
                    );
                    self.update_road_tool_mesh(&road_builder);
                    self.reset(SelectDir {
                        pos: *pos,
                        init_node_type: *init_node_type,
                        road_builder,
                    })
                }
                CurveType::Straight => self.reset(SelectPos),
            },
            SelNode { .. } => self.reset(SelectPos),
        }
        self.show_snappable_nodes();
    }

    fn update_ground_pos(&mut self, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
        self.check_snapping();
    }

    /// Remove node markings from gpu, and remove the road tool mesh.
    fn destroy(self: Box<Self>) -> World {
        self.gfx_handle.borrow_mut().set_node_markers(vec![]);
        self.gfx_handle
            .borrow_mut()
            .set_road_tool_mesh(Some(RoadMesh::empty()));
        self.world
    }
}

impl ConstructTool {
    pub(crate) fn new(
        gfx_handle: Rc<RefCell<dyn GfxSuper>>,
        world: World,
        state_handle: Rc<RefCell<ToolState>>,
        ground_pos: Vec3,
    ) -> Self {
        let mut tool = Self {
            gfx_handle,
            world,
            state_handle,
            snapped_node: None,
            ground_pos,
            mode: SelectPos,
        };
        tool.show_snappable_nodes();
        tool.update_ground_pos(ground_pos);
        tool
    }

    fn get_sel_road_type(&self) -> SelectedRoad {
        self.state_handle.borrow().road_state.selected_road
    }

    fn get_sel_curve_type(&self) -> CurveType {
        self.state_handle
            .borrow()
            .road_state
            .selected_road
            .segment_type
            .curve_type
    }

    fn get_sel_segment_type(&self) -> SegmentType {
        self.state_handle
            .borrow()
            .road_state
            .selected_road
            .segment_type
    }

    fn get_sel_node_type(&self) -> NodeType {
        self.state_handle
            .borrow()
            .road_state
            .selected_road
            .node_type
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
        // RG update
        match &self.mode {
            SelectPos => {}
            SelectDir { road_builder, .. } | CurveEnd { road_builder, .. } => {
                self.update_road_tool_mesh(&road_builder);
            }
            SelNode { .. } => {
                self.reset(SelectPos);
                return;
            }
        }
        self.check_snapping();
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
                // RG update
                // TODO regenerate road_builder if not in select mode and fix road mesh!
                if let CurveEnd { .. } | SelNode { .. } = self.mode {
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
                // RG update
                if let CurveEnd {
                    pos,
                    init_node_type,
                    ..
                } = self.mode
                {
                    // SG SFD
                    let road_type = self.get_sel_road_type();
                    let segment_type = road_type.segment_type;
                    let road_builder = LRoadBuilder::gen_sfd(
                        pos,
                        init_node_type,
                        pos,
                        init_node_type,
                        segment_type,
                    );
                    self.update_road_tool_mesh(&road_builder);
                    self.mode = SelectDir {
                        pos,
                        init_node_type,
                        road_builder,
                    };
                    self.check_snapping();
                }
                if let CurveEnd { .. } | SelNode { .. } = self.mode {
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
        // let node_dir = self
        //     .world
        //     .get_road_graph()
        //     .get_node(snapped_node.get_id())
        //     .get_dir();

        // RG CC
        // self.road_generator = generator::RoadGeneratorTool::new(
        //     snapped_node.get_pos(),
        //     Some(node_dir),
        //     self.get_sel_road_type().clone(),
        //     snapped_node.is_reverse(),
        // );
        // self.road_generator.update_ground_pos(self.ground_pos);
        // self.gfx_handle
        //     .borrow_mut()
        //     .set_road_tool_mesh(self.road_generator.get_mesh());

        let pos = snapped_node.get_pos();
        let node_type = snapped_node.get_node_type();
        let segment_type = self.get_sel_segment_type();

        let road_builder = match self.get_sel_curve_type() {
            CurveType::Straight => {
                LRoadBuilder::gen_sfd(pos, node_type, pos, node_type, segment_type)
            }
            CurveType::Curved => LRoadBuilder::gen_cc(
                LNodeBuilderType::Old(snapped_node.clone()),
                self.ground_pos,
                node_type,
                segment_type,
            ),
        };
        self.update_road_tool_mesh(&road_builder);

        self.mode = SelNode {
            snap: snapped_node,
            road_builder,
        };
        self.snapped_node = None;
    }

    /// Returns the optionally selected node.
    fn get_selected_node(&self) -> Option<SnapConfig> {
        match &self.mode {
            SelectPos | SelectDir { .. } | CurveEnd { .. } => None,
            SelNode { snap, .. } => Some(snap.clone()),
        }
    }

    /// Constructs the road that is being generated.
    fn build_road(&mut self, road_builder: LRoadBuilder) {
        let road_meshes = self.gen_road_mesh_from_builder(&road_builder, self.get_sel_node_type());

        // let sel_node = self.get_selected_node();

        // TODO: code this smarter and remove node_type
        let node_type = self.get_sel_node_type();
        let (new_node, segment_ids) = self
            .world
            .mut_road_graph()
            .add_road(road_builder, node_type);

        let mut mesh_map = HashMap::new();
        for i in 0..segment_ids.len() {
            mesh_map.insert(segment_ids[i], road_meshes[i].clone());
        }
        self.gfx_handle.borrow_mut().add_road_meshes(mesh_map);

        // TODO have add_road return new_node in such a way that is not necessary to check snapped_node
        if self.snapped_node.is_some() {
            self.reset(SelectPos);
        } else if let Some(new_node) = new_node {
            self.select_node(new_node);
        } else {
            self.reset(SelectPos);
        }
    }

    /// Resets to the given mode, maintaining necessary information and unsnapping and deselecting
    /// nodes.
    fn reset(&mut self, new_mode: Mode) {
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
        let empty_mesh = Some(RoadMesh::empty());
        match self.mode {
            SelectPos => self.gfx_handle.borrow_mut().set_road_tool_mesh(empty_mesh),
            SelectDir { .. } | CurveEnd { .. } | SelNode { .. } => {
                // RG update
                // self.road_generator.update_ground_pos(self.ground_pos);
                // self.gfx_handle
                //     .borrow_mut()
                //     .set_road_tool_mesh(self.road_generator.get_mesh());
            }
        };
    }

    /// Updates the construct tool with the snap configs from the snapped node.
    ///
    /// # Panics
    /// Panics if there are no snap_configs passed to the function.
    fn update_snap(&mut self, snap_configs: Vec<SnapConfig>) {
        match self.mode {
            SelectPos => {
                let snap_config = snap_configs.into_iter().nth(0).unwrap();
                let pos = snap_config.get_pos();
                let node_type = snap_config.get_node_type();
                // RG just generate a small stub
                let road_builder = LRoadBuilder::gen_sfd(
                    pos,
                    node_type,
                    pos,
                    node_type,
                    self.get_sel_segment_type(),
                );
                self.update_road_tool_mesh(&road_builder);
                self.snapped_node = Some(snap_config);
            }
            // TODO make nice road generator to remove code duplication
            SelectDir { .. } | CurveEnd { .. } => {
                for _snap_config in snap_configs {
                    // if self
                    //     .road_generator
                    //     .try_snap(snap_config.clone(), false)
                    //     .is_some()
                    // {
                    //     // RG DS
                    // self.snapped_node = Some(snap_config);
                    // self.gfx_handle
                    //     .borrow_mut()
                    //     .set_road_tool_mesh(self.road_generator.get_mesh());
                    // TODO update RG
                    return;
                    // }
                }
                self.update_no_snap();
            }
            SelNode { .. } => {
                for _snap_config in snap_configs {
                    // if self
                    //     .road_generator
                    //     .try_snap(snap_config.clone(), true)
                    //     .is_some()
                    // {
                    //     // RG CCS
                    //     self.snapped_node = Some(snap_config);
                    //     self.gfx_handle
                    //         .borrow_mut()
                    //         .set_road_tool_mesh(self.road_generator.get_mesh());
                    // TODO update RG
                    return;
                    // }
                }
                self.update_no_snap();
            }
        }
    }

    /// Checks if there is a node that we should snap to, and in that case it snaps to that node.
    fn check_snapping(&mut self) {
        let node_snap_configs = if self.state_handle.borrow().road_state.snapping {
            // TODO add functionality to report why a node cannot be snapped to.
            self.world
                .get_road_graph()
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

    // #############################################################################################
    // Gfx handling
    // #############################################################################################
    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes(&mut self) {
        if !self.state_handle.borrow().road_state.snapping {
            return;
        }
        let side = if let SelNode { snap, .. } = &self.mode {
            Some(snap.get_side())
        } else {
            None
        };
        let possible_snaps = self
            .world
            .get_road_graph()
            .get_possible_snap_nodes(side, self.get_sel_road_type().node_type)
            .iter()
            .map(|id| self.world.get_road_graph().get_node(*id).get_pos().into())
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
            .map(|s| mesh_gen::generate_simple_mesh(s.get_guide_points(), node_type))
            .collect::<Vec<RoadMesh>>()
    }

    fn update_road_tool_mesh(&self, road_builder: &LRoadBuilder) {
        let meshes =
            self.gen_road_mesh_from_builder(road_builder, self.get_sel_road_type().node_type);
        let mesh = mesh_gen::combine_road_meshes(meshes);
        self.gfx_handle.borrow_mut().set_road_tool_mesh(Some(mesh));
    }
}
