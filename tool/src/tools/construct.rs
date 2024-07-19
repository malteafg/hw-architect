use super::{Tool, ToolUnique};
use crate::cycle_selection;
use crate::gfx_gen::segment_gen;
use crate::tool_state::{CurveType, SelectedRoad};

use utils::id::{IdMap, SegmentId};
use utils::{input, VecUtils};
use world_api::{
    LNodeBuilder, LNodeBuilderType, LRoadBuilder, LaneWidth, NodeType, Side, SnapConfig,
    WorldManipulator,
};

use gfx_api::{GfxWorldData, RoadMesh};
use glam::*;

mod curve_tools {
    use std::marker::PhantomData;

    use curves::{CompositeCurve, Curve, CurveError, CurveInfo, CurveSpec, Straight};
    use glam::Vec3;
    use utils::Loc;
    use world_api::SnapConfig;

    #[derive(Debug, Clone)]
    pub enum EndPoint {
        /// Position from which to build from. This must not be projected.
        New(Vec3),
        /// Location of the snapconfig to build from
        Old(SnapConfig),
    }

    /// Describes the action that should be taken by the construct tool.
    pub enum CurveAction<C: CurveSpec> {
        /// Construct the given curve as part of the world.
        Construct(CompositeCurve<C>),
        /// Render the given curve while still constructing it.
        Render(CompositeCurve<C>, CurveInfo),
        /// A direction needs to be chosen.
        Direction(Loc, f32),
        /// A control point needs to be chosen.
        ControlPoint(Vec3, Vec3),
        /// A small road stub should be rendered to indicate that the user can snap to this node.
        Stub,
        /// The curve builder has nothing to render.
        Nothing,
    }

    impl<C: CurveSpec> From<(C, CurveInfo)> for CurveAction<C> {
        fn from((curve, curve_info): (C, CurveInfo)) -> Self {
            CurveAction::Render(CompositeCurve::Single(curve), curve_info)
        }
    }

    pub type CurveActionResult<C> = Result<CurveAction<C>, CurveError<C>>;

    pub trait CurveToolSpec<C: CurveSpec> {
        /// The tool shall process a left click.
        fn left_click(&mut self, first: EndPoint, last: EndPoint) -> CurveActionResult<C>;

        /// The tool shall process a right click.
        fn right_click(&mut self, first: EndPoint, last: EndPoint) -> CurveActionResult<C>;

        /// Called whenever there the ground_pos has been updated due to a change in camera or
        /// cursor position.
        fn compute_curve(&mut self, first: EndPoint, last: EndPoint) -> CurveActionResult<C>;
    }

    pub struct CurveTool<CT, C: CurveSpec>
    where
        CT: CurveToolSpec<C>,
    {
        instance: CT,
        first_point: Option<EndPoint>,
        snapped_node: Option<SnapConfig>,
        _marker: PhantomData<C>,
    }

    impl<CT, C: CurveSpec> CurveTool<CT, C>
    where
        CT: CurveToolSpec<C>,
    {
        /// Selects the first point if it has not already been selected, otherwise delegates the
        /// call to instance.
        pub fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
            if let Some(first_point) = &self.first_point {
                let last_point = if let Some(snap_config) = self.snapped_node.clone() {
                    EndPoint::Old(snap_config)
                } else {
                    EndPoint::New(ground_pos)
                };
                return self.instance.left_click(first_point.clone(), last_point);
            }

            if let Some(snap_config) = self.snapped_node.clone() {
                self.first_point = Some(EndPoint::Old(snap_config));
                return Ok(CurveAction::Stub);
            }

            let first_point = EndPoint::New(ground_pos);
            self.first_point = Some(first_point.clone());
            self.instance
                .compute_curve(first_point, EndPoint::New(ground_pos))
        }

        pub fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
            if let Some(first_point) = &self.first_point {
                let last_point = if let Some(snap_config) = self.snapped_node.clone() {
                    EndPoint::Old(snap_config)
                } else {
                    EndPoint::New(ground_pos)
                };

                match self.instance.right_click(first_point.clone(), last_point) {
                    Ok(CurveAction::Nothing) => {
                        self.first_point = None;
                        return Ok(CurveAction::Nothing);
                    }
                    curve_result => return curve_result,
                }
            }

            Ok(CurveAction::Nothing)
        }

        pub fn update_view(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
            if let Some(first_point) = &self.first_point {
                let last_point = if let Some(snap_config) = self.snapped_node.clone() {
                    EndPoint::Old(snap_config)
                } else {
                    EndPoint::New(ground_pos)
                };
                return self.instance.compute_curve(first_point.clone(), last_point);
            }

            Ok(CurveAction::Nothing)
        }

        /// A node has been snapped to.
        pub fn snap(&mut self, snap_config: SnapConfig) -> CurveActionResult<C> {
            self.snapped_node = Some(snap_config.clone());

            if let Some(first_point) = &self.first_point {
                return self
                    .instance
                    .compute_curve(first_point.clone(), EndPoint::Old(snap_config));
            }

            Ok(CurveAction::Stub)
        }

        /// A node is no longer snapped.
        pub fn unsnap(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
            self.snapped_node = None;

            if let Some(first_point) = &self.first_point {
                return self
                    .instance
                    .compute_curve(first_point.clone(), EndPoint::New(ground_pos));
            };

            Ok(CurveAction::Nothing)
        }
    }

    pub struct StraightBuilder;

    impl CurveToolSpec<Curve<Straight>> for StraightBuilder {
        fn left_click(
            &mut self,
            first: EndPoint,
            last: EndPoint,
        ) -> CurveActionResult<Curve<Straight>> {
            match self.compute_curve(first, last) {
                Ok(CurveAction::Render(curve, _curve_info)) => Ok(CurveAction::Construct(curve)),
                curve_result => curve_result,
            }
        }

        fn right_click(
            &mut self,
            _first: EndPoint,
            _last: EndPoint,
        ) -> CurveActionResult<Curve<Straight>> {
            Ok(CurveAction::Nothing)
        }

        fn compute_curve(
            &mut self,
            first: EndPoint,
            last: EndPoint,
        ) -> CurveActionResult<Curve<Straight>> {
            use EndPoint::*;
            match (first, last) {
                (New(first_pos), New(last_pos)) => {
                    Ok(Curve::<Straight>::from_free(first_pos, last_pos).into())
                }
                (Old(first), New(last_pos)) => {
                    Ok(Curve::<Straight>::from_first_locked(first.into(), last_pos).into())
                }
                (New(_first_pos), Old(_last)) => Err(CurveError::Impossible),
                (Old(_first), Old(_last)) => Err(CurveError::Impossible),
            }
        }
    }
}

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

pub struct Construct {
    snapped_node: Option<SnapConfig>,
    mode: Mode,
}

impl Default for Construct {
    fn default() -> Self {
        Self {
            snapped_node: None,
            mode: Mode::default(),
        }
    }
}

impl<G: GfxWorldData, W: WorldManipulator> ToolUnique<G> for Tool<Construct, W> {
    fn init(&mut self, gfx_handle: &mut G) {
        self.update_view(gfx_handle);
        self.show_snappable_nodes(gfx_handle);
    }

    fn process_keyboard(&mut self, gfx_handle: &mut G, key: input::KeyAction) {
        use input::Action::*;
        use input::KeyState::*;
        match key {
            (ToggleSnapping, Press) => self.toggle_snapping(gfx_handle),
            (ToggleReverse, Press) => self.toggle_reverse(),
            (CycleCurveType, Scroll(scroll_state)) => {
                let new_curve_type =
                    cycle_selection::scroll(self.get_sel_curve_type(), scroll_state);
                self.state_handle.road_state.set_curve_type(new_curve_type);
                self.set_curve_type(gfx_handle, new_curve_type);
            }
            (CycleLaneWidth, Scroll(scroll_state)) => {
                let new_lane_width =
                    cycle_selection::scroll(self.get_sel_lane_width(), scroll_state);
                self.state_handle.road_state.set_lane_width(new_lane_width);
                self.set_lane_width(gfx_handle, new_lane_width);
            }
            (CycleNoLanes, Scroll(scroll_state)) => {
                let new_no_lanes = cycle_selection::scroll(self.get_sel_no_lanes(), scroll_state);
                self.state_handle.road_state.set_no_lanes(new_no_lanes);
                self.set_no_lanes(gfx_handle, new_no_lanes);
            }
            _ => {}
        }
    }

    fn left_click(&mut self, gfx_handle: &mut G) {
        let prev_move = std::mem::take(&mut self.instance.mode);
        // The proper mode should be set in all branches of match.
        match prev_move {
            SelectPos => {
                if self.try_select_node(gfx_handle) {
                    return;
                }
                self.update_to_select_dir(gfx_handle, self.ground_pos, self.get_sel_node_type())
            }
            SelectDir {
                pos,
                init_node_type,
                road_builder,
            } => match self.get_sel_curve_type() {
                CurveType::Straight => self.build_road(gfx_handle, road_builder),
                CurveType::Circular => {
                    if self.instance.snapped_node.is_some() {
                        self.build_road(gfx_handle, road_builder)
                    } else {
                        let dir = (self.ground_pos - pos).normalize_else();
                        self.update_to_cc_curve_end(gfx_handle, pos, dir, init_node_type)
                    }
                }
            },
            CurveEnd { road_builder, .. } => self.build_road(gfx_handle, road_builder),
            SelNode { road_builder, .. } => self.build_road(gfx_handle, road_builder),
        }
    }

    fn right_click(&mut self, gfx_handle: &mut G) {
        match &self.instance.mode {
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
            SelectDir { .. } => self.reset(gfx_handle),
            CurveEnd {
                pos,
                init_node_type,
                ..
            } => self.update_to_select_dir(gfx_handle, *pos, *init_node_type),
            SelNode { .. } => self.reset(gfx_handle),
        }
    }

    fn update_view(&mut self, gfx_handle: &mut G) {
        self.check_snapping(gfx_handle);
    }

    /// Remove node markings from gpu, and remove the road tool mesh.
    fn clean_gfx(&mut self, gfx_handle: &mut G) {
        gfx_handle.set_node_markers(vec![]);
        gfx_handle.set_road_tool_mesh(None);
    }
}

impl<W: WorldManipulator> Tool<Construct, W> {
    fn get_sel_road_type(&self) -> SelectedRoad {
        self.state_handle.road_state.selected_road
    }

    fn get_sel_curve_type(&self) -> CurveType {
        self.state_handle.road_state.selected_road.curve_type
    }

    fn get_sel_node_type(&self) -> NodeType {
        self.state_handle.road_state.selected_road.node_type
    }

    fn get_sel_lane_width(&self) -> LaneWidth {
        self.get_sel_node_type().lane_width()
    }

    fn get_sel_no_lanes(&self) -> u8 {
        self.get_sel_node_type().no_lanes()
    }

    fn is_reverse(&self) -> bool {
        self.state_handle.road_state.reverse
    }

    fn compute_reverse(&self) -> bool {
        match &self.instance.mode {
            SelectPos | SelectDir { .. } | CurveEnd { .. } => {
                if let Some(snap) = &self.instance.snapped_node {
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
    fn toggle_snapping<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        let curr = self.state_handle.road_state.snapping;
        self.state_handle.road_state.snapping = !curr;
        // Turn snapping on
        if !curr {
            self.check_snapping(gfx_handle);
            self.show_snappable_nodes(gfx_handle);
            dbg!(self.state_handle.road_state.snapping);
            return;
        }
        // Turn snapping off
        if self.instance.snapped_node.is_some() {
            self.instance.snapped_node = None;
            self.check_snapping(gfx_handle);
        }
        gfx_handle.set_node_markers(vec![]);
        dbg!(self.state_handle.road_state.snapping);
    }

    /// Toggles reverse.
    fn toggle_reverse(&mut self) {
        let curr = self.state_handle.road_state.reverse;
        self.state_handle.road_state.reverse = !curr;
        dbg!(self.state_handle.road_state.reverse);
    }

    /// Sets the curve type in use.
    fn set_curve_type<G: GfxWorldData>(&mut self, gfx_handle: &mut G, new_curve_type: CurveType) {
        match new_curve_type {
            CurveType::Straight => match &self.instance.mode {
                SelectPos | SelectDir { .. } => {}
                CurveEnd {
                    pos,
                    init_node_type,
                    ..
                } => {
                    self.update_to_select_dir(gfx_handle, *pos, *init_node_type);
                }
                SelNode { .. } => self.update_no_snap(gfx_handle),
            },
            CurveType::Circular => match &self.instance.mode {
                SelectPos | SelectDir { .. } => {}
                CurveEnd { .. } | SelNode { .. } => self.update_no_snap(gfx_handle),
            },
        };
        dbg!(new_curve_type);
    }

    /// Sets the lane width in use.
    fn set_lane_width<G: GfxWorldData>(&mut self, gfx_handle: &mut G, new_lane_width: LaneWidth) {
        self.reset(gfx_handle);
        dbg!(new_lane_width.getf32());
    }

    /// Sets the selected number of lanes.
    fn set_no_lanes<G: GfxWorldData>(&mut self, gfx_handle: &mut G, no_lanes: u8) {
        dbg!(no_lanes);
        self.show_snappable_nodes(gfx_handle);
        if let SelNode { .. } = self.instance.mode {
            self.reset(gfx_handle);
        } else {
            self.check_snapping(gfx_handle);
        }
    }

    // #############################################################################################
    // General tool implementations
    // #############################################################################################
    fn try_select_node<G: GfxWorldData>(&mut self, gfx_handle: &mut G) -> bool {
        if let Some(snap_config) = self.instance.snapped_node.take() {
            self.select_node(gfx_handle, snap_config);
            return true;
        };
        false
    }

    /// Invoked when a snapped node becomes selected.
    fn select_node<G: GfxWorldData>(&mut self, gfx_handle: &mut G, snap_config: SnapConfig) {
        self.update_to_sld(gfx_handle, snap_config);
        self.show_snappable_nodes(gfx_handle);
    }

    /// Constructs the road that is being generated.
    fn build_road<G: GfxWorldData>(&mut self, gfx_handle: &mut G, road_builder: LRoadBuilder) {
        let next_node_type = self.get_sel_node_type();
        let road_meshes = self.gen_road_mesh_from_builder(&road_builder, self.get_sel_node_type());
        let (new_node, segment_ids) = self.world.add_road(road_builder, next_node_type);

        let mut mesh_map: IdMap<SegmentId, RoadMesh> = IdMap::new();
        for i in 0..segment_ids.len() {
            mesh_map.insert(segment_ids[i], road_meshes[i].clone());
        }
        gfx_handle.add_road_meshes(mesh_map);

        if self.instance.snapped_node.is_some() {
            self.instance.mode = SelectPos;
        } else if let Some(new_node) = new_node {
            self.select_node(gfx_handle, new_node);
        } else {
            self.instance.mode = SelectPos;
        }
        self.show_snappable_nodes(gfx_handle);
        self.check_snapping(gfx_handle);
    }

    // #############################################################################################
    // Updating
    // #############################################################################################
    /// Sets the mode to select pos and checks for snapping and snappable nodes.
    fn reset<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        self.instance.mode = SelectPos;
        self.show_snappable_nodes(gfx_handle);
        self.check_snapping(gfx_handle);
    }

    /// This function will generate an sfd and set the mode to select dir. This can always be
    /// called when entering or updating select dir mode.
    fn update_to_select_dir<G: GfxWorldData>(
        &mut self,
        gfx_handle: &mut G,
        first_pos: Vec3,
        init_node_type: NodeType,
    ) {
        let road_builder = LRoadBuilder::gen_sfd(
            first_pos,
            init_node_type,
            self.ground_pos,
            init_node_type,
            self.compute_reverse(),
        );
        self.update_road_tool_mesh(gfx_handle, &road_builder);
        self.instance.mode = SelectDir {
            pos: first_pos,
            init_node_type,
            road_builder,
        }
    }

    /// Generates and sld and sets the mode to SelNode.
    fn update_to_sld<G: GfxWorldData>(&mut self, gfx_handle: &mut G, snap_config: SnapConfig) {
        let reverse = snap_config.side() == Side::In;
        let road_builder = LRoadBuilder::gen_sld(
            snap_config.clone(),
            self.ground_pos,
            snap_config.node_type(),
            reverse,
        );
        self.update_road_tool_mesh(gfx_handle, &road_builder);
        self.instance.mode = SelNode {
            snap_config,
            road_builder,
        }
    }

    /// Generates a cc curve and sets the mode to CurveEnd.
    fn update_to_cc_curve_end<G: GfxWorldData>(
        &mut self,
        gfx_handle: &mut G,
        pos: Vec3,
        dir: Vec3,
        init_node_type: NodeType,
    ) {
        let last_pos = self.ground_pos;
        let road_builder = LRoadBuilder::gen_cc(
            LNodeBuilderType::new(pos, dir, init_node_type),
            last_pos,
            self.get_sel_node_type(),
            self.compute_reverse(),
        );
        self.update_road_tool_mesh(gfx_handle, &road_builder);
        self.instance.mode = CurveEnd {
            pos,
            dir,
            init_node_type,
            road_builder,
        }
    }

    /// Generates a cc curve and sets the mode to SelNode.
    fn update_to_cc_sel_node<G: GfxWorldData>(
        &mut self,
        gfx_handle: &mut G,
        snap_config: SnapConfig,
    ) {
        let last_pos = self.ground_pos;
        let road_builder = LRoadBuilder::gen_cc(
            LNodeBuilderType::Old(snap_config.clone()),
            last_pos,
            self.get_sel_node_type(),
            self.compute_reverse(),
        );
        self.update_road_tool_mesh(gfx_handle, &road_builder);
        self.instance.mode = SelNode {
            snap_config,
            road_builder,
        }
    }

    /// Updates the construct tool when there is no node that we should snap to.
    fn update_no_snap<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        self.instance.snapped_node = None;
        match &self.instance.mode {
            SelectPos => gfx_handle.set_road_tool_mesh(None),
            SelectDir {
                pos,
                init_node_type,
                ..
            } => self.update_to_select_dir(gfx_handle, *pos, *init_node_type),
            CurveEnd {
                pos,
                dir,
                init_node_type,
                ..
            } => self.update_to_cc_curve_end(gfx_handle, *pos, *dir, *init_node_type),
            SelNode { snap_config, .. } => match self.get_sel_curve_type() {
                CurveType::Straight => self.update_to_sld(gfx_handle, snap_config.clone()),
                CurveType::Circular => self.update_to_cc_sel_node(gfx_handle, snap_config.clone()),
            },
        };
    }

    // #############################################################################################
    // Snapping
    // #############################################################################################
    /// Updates the construct tool with the snap configs from the snapped node. If no snaps fit,
    /// then update_no_snap is called. This function is only called when there is at least one
    /// snap.
    fn update_snap<G: GfxWorldData>(&mut self, gfx_handle: &mut G, snap_configs: Vec<SnapConfig>) {
        match &self.instance.mode {
            SelectPos => {
                // Snap does not have to satisfy any curvature constraints.
                let snap_config = snap_configs.into_iter().nth(0).unwrap();
                let pos = snap_config.pos();
                let dir = snap_config.dir();
                let node_type = snap_config.node_type();
                let reverse = snap_config.side() == Side::In;

                let road_builder =
                    LRoadBuilder::gen_stub(pos, dir.flip(reverse), node_type, reverse);
                self.update_road_tool_mesh(gfx_handle, &road_builder);
                self.instance.snapped_node = Some(snap_config);
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
                    self.update_road_tool_mesh(gfx_handle, &road_builder);
                    self.instance.snapped_node = Some(snap_config);
                    self.instance.mode = SelectDir {
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
                    self.update_road_tool_mesh(gfx_handle, &road_builder);
                    self.instance.snapped_node = Some(snap_config);
                    self.instance.mode = CurveEnd {
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
                    self.update_road_tool_mesh(gfx_handle, &road_builder);
                    self.instance.snapped_node = Some(new_snap_config);
                    self.instance.mode = SelNode {
                        snap_config: snap_config.clone(),
                        road_builder,
                    };
                    return;
                }
            }
        }
        self.instance.snapped_node = None;
        self.update_no_snap(gfx_handle);
    }

    /// Checks if there is a node that we should snap to, and in that case it snaps to that node.
    fn check_snapping<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        // TODO add functionality to report why a node cannot be snapped to.
        if !self.state_handle.road_state.snapping {
            self.update_no_snap(gfx_handle);
            return;
        }

        // Get available snaps
        let node_snap_configs = self
            .world
            .get_snap_configs_closest_node(self.ground_pos, self.get_sel_road_type().node_type);

        let Some((_snap_id, mut snap_configs)) = node_snap_configs else {
            self.update_no_snap(gfx_handle);
            return;
        };

        if let SelNode { snap_config, .. } = &self.instance.mode {
            snap_configs.retain(|s| s.side() != snap_config.side());
        }

        if snap_configs.is_empty() {
            self.update_no_snap(gfx_handle);
            return;
        }

        self.update_snap(gfx_handle, snap_configs);
    }

    // #############################################################################################
    // Gfx handling
    // #############################################################################################
    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        if !self.state_handle.road_state.snapping {
            return;
        }
        let side = if let SelNode { snap_config, .. } = &self.instance.mode {
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

        gfx_handle.set_node_markers(possible_snaps);
    }

    fn gen_road_mesh_from_builder(
        &self,
        road_builder: &LRoadBuilder,
        node_type: NodeType,
    ) -> Vec<RoadMesh> {
        road_builder
            .get_segments()
            .iter()
            .map(|s| segment_gen::gen_road_mesh_with_lanes(s.get_spine(), node_type))
            .collect::<Vec<RoadMesh>>()
    }

    fn update_road_tool_mesh<G: GfxWorldData>(
        &self,
        gfx_handle: &mut G,
        road_builder: &LRoadBuilder,
    ) {
        let meshes =
            self.gen_road_mesh_from_builder(road_builder, self.get_sel_road_type().node_type);
        let mesh = segment_gen::combine_road_meshes_bad(meshes);
        gfx_handle.set_road_tool_mesh(Some(mesh));
    }
}
