use super::{Tool, ToolUnique};
use crate::gfx_gen::segment_gen;
use crate::tool_state::{CurveType, SelectedRoad};

use curve_tools::{CurveAction, CurveActionResult, CurveTool, CurveToolSum, StraightBuilder};
use curves::{CompositeCurve, Curve, CurveError, CurveShared, Straight};
use utils::{input, Loc};
use world_api::{
    LNodeBuilderType, LRoadBuilder, LaneWidth, NodeType, SnapConfig, WorldManipulator,
};

use gfx_api::{GfxWorldData, RoadMesh};
use glam::*;

use curve_tools::CurveToolSpec;

mod curve_tools {
    use std::marker::PhantomData;

    use curves::{CompositeCurve, Curve, CurveError, CurveInfo, CurveSpec, CurveSum, Straight};
    use enum_dispatch::enum_dispatch;
    use glam::Vec3;
    use utils::{Loc, PosOrLoc, VecUtils};
    use world_api::SnapConfig;

    #[derive(Debug, Clone)]
    pub enum EndPoint {
        /// Position from which to build from. This must not be projected.
        New(Vec3),
        /// Location of the snapconfig to build from
        Old(SnapConfig),
    }

    impl From<EndPoint> for PosOrLoc {
        fn from(value: EndPoint) -> Self {
            match value {
                EndPoint::New(pos) => PosOrLoc::Pos(pos),
                EndPoint::Old(snap_config) => {
                    if snap_config.is_reverse() {
                        PosOrLoc::Loc(Loc::new(
                            snap_config.pos(),
                            snap_config.dir().flip(true).into(),
                        ))
                    } else {
                        PosOrLoc::Loc(Loc::new(snap_config.pos(), snap_config.dir().into()))
                    }
                }
            }
        }
    }

    /// Describes the action that should be taken by the construct tool.
    pub enum CurveAction {
        /// Construct the given curve as part of the world.
        Construct(CompositeCurve),
        /// Render the given curve while still constructing it.
        Render(CompositeCurve, CurveInfo),
        /// A direction needs to be chosen.
        Direction(Loc, f32),
        /// A control point needs to be chosen.
        ControlPoint(Vec3, Vec3),
        /// A small road stub should be rendered to indicate that the user can snap to this node.
        Stub(Loc),
        /// The curve builder has nothing to render.
        Nothing,
    }

    pub type CurveActionResult = Result<CurveAction, CurveError>;

    impl<C: Into<CurveSum>> From<(C, CurveInfo)> for CurveAction {
        fn from((curve, curve_info): (C, CurveInfo)) -> Self {
            CurveAction::Render(CompositeCurve::Single(curve.into()), curve_info)
        }
    }

    pub trait CurveToolSpecInternal {
        /// The tool shall process a left click.
        fn left_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult;

        /// The tool shall process a right click.
        fn right_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult;

        /// Called whenever there the ground_pos has been updated due to a change in camera or
        /// cursor position.
        fn compute_curve(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult;
    }

    #[enum_dispatch]
    pub trait CurveToolSpec {
        /// Selects the first point if it has not already been selected, otherwise delegates the
        /// call to instance.
        fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult;

        fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult;

        /// A node has been snapped to.
        fn update_snap(&mut self, snap_config: SnapConfig) -> CurveActionResult;

        /// A node is no longer snapped.
        fn update_no_snap(&mut self, ground_pos: Vec3) -> CurveActionResult;

        fn get_selected_node(&self) -> Option<SnapConfig>;

        fn get_snapped_node(&self) -> Option<SnapConfig>;
    }

    #[enum_dispatch(CurveToolSpec)]
    pub enum CurveToolSum {
        Straight(CurveTool<StraightBuilder, Curve<Straight>>),
    }

    pub struct CurveTool<CT, C: CurveSpec>
    where
        CT: CurveToolSpecInternal,
    {
        instance: CT,
        first_point: Option<EndPoint>,
        snapped_node: Option<SnapConfig>,
        _marker: PhantomData<C>,
    }

    impl<CT: Default, C: CurveSpec> Default for CurveTool<CT, C>
    where
        CT: CurveToolSpecInternal,
    {
        fn default() -> Self {
            Self {
                instance: CT::default(),
                first_point: None,
                snapped_node: None,
                _marker: PhantomData,
            }
        }
    }

    impl<CT, C: CurveSpec> CurveToolSpec for CurveTool<CT, C>
    where
        CT: CurveToolSpecInternal,
    {
        fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult {
            if let Some(first_point) = &self.first_point {
                let last_point = if let Some(snap_config) = self.snapped_node.clone() {
                    EndPoint::Old(snap_config)
                } else {
                    EndPoint::New(ground_pos)
                };
                return self
                    .instance
                    .left_click(first_point.clone().into(), last_point.into());
            }

            if let Some(snap_config) = self.snapped_node.clone() {
                self.first_point = Some(EndPoint::Old(snap_config.clone()));
                return Ok(CurveAction::Stub(Loc::new(
                    snap_config.pos(),
                    snap_config.dir().into(),
                )));
            }

            let first_point = EndPoint::New(ground_pos);
            self.first_point = Some(first_point.clone());
            self.instance
                .compute_curve(first_point.into(), ground_pos.into())
        }

        fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult {
            if let Some(first_point) = &self.first_point {
                let last_point = if let Some(snap_config) = self.snapped_node.clone() {
                    EndPoint::Old(snap_config)
                } else {
                    EndPoint::New(ground_pos)
                };

                match self
                    .instance
                    .right_click(first_point.clone().into(), last_point.into())
                {
                    Ok(CurveAction::Nothing) => {
                        self.first_point = None;
                        return Ok(CurveAction::Nothing);
                    }
                    curve_result => return curve_result,
                }
            }

            Ok(CurveAction::Nothing)
        }

        fn update_snap(&mut self, snap_config: SnapConfig) -> CurveActionResult {
            self.snapped_node = Some(snap_config.clone());

            if let Some(first_point) = &self.first_point {
                return self.instance.compute_curve(
                    first_point.clone().into(),
                    EndPoint::Old(snap_config).into(),
                );
            }

            Ok(CurveAction::Stub(Loc::new(
                snap_config.pos(),
                snap_config.dir().into(),
            )))
        }

        fn update_no_snap(&mut self, ground_pos: Vec3) -> CurveActionResult {
            self.snapped_node = None;

            if let Some(first_point) = &self.first_point {
                return self
                    .instance
                    .compute_curve(first_point.clone().into(), ground_pos.into());
            };

            Ok(CurveAction::Nothing)
        }

        fn get_selected_node(&self) -> Option<SnapConfig> {
            self.first_point.clone().and_then(|x| match x {
                EndPoint::New(_) => None,
                EndPoint::Old(snap_config) => Some(snap_config),
            })
        }

        fn get_snapped_node(&self) -> Option<SnapConfig> {
            self.snapped_node.clone()
        }
    }

    #[derive(Default)]
    pub struct StraightBuilder;

    impl CurveToolSpecInternal for StraightBuilder {
        fn left_click(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
            match self.compute_curve(first, last) {
                Ok(CurveAction::Render(curve, _curve_info)) => Ok(CurveAction::Construct(curve)),
                curve_result => curve_result,
            }
        }

        fn right_click(&mut self, _first: PosOrLoc, _last: PosOrLoc) -> CurveActionResult {
            Ok(CurveAction::Nothing)
        }

        fn compute_curve(&mut self, first: PosOrLoc, last: PosOrLoc) -> CurveActionResult {
            use PosOrLoc::*;
            match (first, last) {
                (Pos(first_pos), Pos(last_pos)) => {
                    Ok(Curve::<Straight>::from_free(first_pos, last_pos).into())
                }
                (Loc(first), Pos(last_pos)) => {
                    Ok(Curve::<Straight>::from_first_locked(first.into(), last_pos).into())
                }
                (Pos(_first_pos), Loc(_last)) => Err(CurveError::Impossible),
                (Loc(_first), Loc(_last)) => Err(CurveError::Impossible),
            }
        }
    }
}

pub struct Construct {
    curve_builder: CurveToolSum,
}

impl Default for Construct {
    fn default() -> Self {
        Self {
            curve_builder: CurveToolSum::Straight(
                CurveTool::<StraightBuilder, Curve<Straight>>::default(),
            ),
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
            (ToggleSnapping, Press) => {
                // self.toggle_snapping(gfx_handle)
            }
            (ToggleReverse, Press) => {
                // self.toggle_reverse()
            }
            (CycleCurveType, Scroll(_scroll_state)) => {
                // let new_curve_type =
                //     cycle_selection::scroll(self.get_sel_curve_type(), scroll_state);
                // self.state_handle.road_state.set_curve_type(new_curve_type);
                // self.set_curve_type(gfx_handle, new_curve_type);
            }
            (CycleLaneWidth, Scroll(scroll_state)) => {
                // let new_lane_width =
                //     cycle_selection::scroll(self.get_sel_lane_width(), scroll_state);
                // self.state_handle.road_state.set_lane_width(new_lane_width);
                // self.set_lane_width(gfx_handle, new_lane_width);
            }
            (CycleNoLanes, Scroll(scroll_state)) => {
                // let new_no_lanes = cycle_selection::scroll(self.get_sel_no_lanes(), scroll_state);
                // self.state_handle.road_state.set_no_lanes(new_no_lanes);
                // self.set_no_lanes(gfx_handle, new_no_lanes);
            }
            _ => {}
        }
    }

    fn left_click(&mut self, gfx_handle: &mut G) {
        let action = self.instance.curve_builder.left_click(self.ground_pos);
        self.handle_curve_action_result(gfx_handle, action);
    }

    fn right_click(&mut self, gfx_handle: &mut G) {
        let action = self.instance.curve_builder.right_click(self.ground_pos);
        self.handle_curve_action_result(gfx_handle, action);
    }

    fn update_view(&mut self, gfx_handle: &mut G) {
        let snap = self.check_snapping();
        let action = match snap {
            Some(snap) => self.instance.curve_builder.update_snap(snap),
            None => self.instance.curve_builder.update_no_snap(self.ground_pos),
        };
        self.handle_curve_action_result(gfx_handle, action);
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

    // #############################################################################################
    // Handle curve actions
    // #############################################################################################
    fn handle_curve_action_result<G: GfxWorldData>(
        &mut self,
        gfx_handle: &mut G,
        action_result: CurveActionResult,
    ) {
        self.clean_gfx(gfx_handle);
        match action_result {
            Ok(action) => self.handle_curve_action(gfx_handle, action),
            Err(err) => self.handle_curve_error(gfx_handle, err),
        }
    }

    fn handle_curve_action<G: GfxWorldData>(&mut self, gfx_handle: &mut G, action: CurveAction) {
        use CurveAction::*;
        match action {
            Construct(curve) => self.construct_road(gfx_handle, curve),
            Render(curve, curve_info) => {
                self.set_road_tool_mesh(gfx_handle, curve, self.get_sel_node_type());
                dbg!(curve_info);
            }
            Direction(loc, len) => unimplemented!(),
            ControlPoint(first, last) => unimplemented!(),
            Stub(loc) => {
                let (curve, _) =
                    Curve::<Straight>::from_free(loc.pos, loc.pos + Vec3::from(loc.dir));
                self.set_road_tool_mesh(gfx_handle, curve.into(), self.get_sel_node_type());
            }
            Nothing => {}
        }
    }

    fn handle_curve_error<G: GfxWorldData>(&mut self, _gfx_handle: &mut G, _error: CurveError) {}

    fn construct_road<G: GfxWorldData>(&mut self, gfx_handle: &mut G, curve: CompositeCurve) {
        match curve {
            CompositeCurve::Single(mut curve) => {
                let (start, end, reverse) = self.construct_compute_end_nodes();
                if reverse {
                    curve.reverse();
                }
            }
            CompositeCurve::Double(curve1, curve2) => unimplemented!(),
        }

        // update selected node based on result from road_graph
    }

    fn construct_compute_end_nodes(&self) -> (Option<SnapConfig>, Option<SnapConfig>, bool) {
        let reverse = if let Some(selected_node) = self.instance.curve_builder.get_selected_node() {
            selected_node.is_reverse()
        } else {
            self.is_reverse()
        };

        let result = (
            self.instance.curve_builder.get_selected_node(),
            self.instance.curve_builder.get_snapped_node(),
        );
        if reverse {
            (result.1, result.0, reverse)
        } else {
            (result.0, result.1, reverse)
        }
    }

    fn map_end_point(&self, snap: Option<SnapConfig>, loc: Loc) -> LNodeBuilderType {
        unimplemented!()
    }

    // #############################################################################################
    // Snapping
    // #############################################################################################
    /// Checks if there is a node that we should snap to, and in that case it snaps to that node.
    fn check_snapping(&mut self) -> Option<SnapConfig> {
        // TODO add functionality to report why a node cannot be snapped to.
        if !self.state_handle.road_state.snapping {
            return None;
        }

        // Get available snaps
        let node_snap_configs = self
            .world
            .get_snap_configs_closest_node(self.ground_pos, self.get_sel_road_type().node_type);

        let Some((_snap_id, mut snap_configs)) = node_snap_configs else {
            return None;
        };

        if let Some(snap_config) = &self.instance.curve_builder.get_selected_node() {
            snap_configs.retain(|s| s.side() != snap_config.side());
        }

        if snap_configs.is_empty() {
            return Some(snap_configs[0].clone());
        }

        return None;
    }

    // #############################################################################################
    // Gfx handling
    // #############################################################################################
    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        if !self.state_handle.road_state.snapping {
            return;
        }
        let side = if let Some(snap_config) = &self.instance.curve_builder.get_selected_node() {
            Some(snap_config.side())
        } else {
            None
        };
        let possible_snaps = self
            .world
            .get_possible_snap_nodes(side, self.get_sel_road_type().node_type)
            .iter()
            .map(|(_id, loc)| (<[f32; 3]>::from(loc.pos), <[f32; 3]>::from(Vec3::from(loc.dir))))
            .collect();

        gfx_handle.set_node_markers(possible_snaps);
    }

    fn set_road_tool_mesh<G: GfxWorldData>(
        &self,
        gfx_handle: &mut G,
        curve: CompositeCurve,
        node_type: NodeType,
    ) {
        let mesh = match curve {
            CompositeCurve::Single(curve) => {
                segment_gen::gen_road_mesh_with_lanes(curve.get_spine(), node_type)
            }
            CompositeCurve::Double(curve1, curve2) => {
                let mesh1 = segment_gen::gen_road_mesh_with_lanes(curve1.get_spine(), node_type);
                let mesh2 = segment_gen::gen_road_mesh_with_lanes(curve2.get_spine(), node_type);
                segment_gen::combine_road_meshes_bad(vec![mesh1, mesh2])
            }
        };
        gfx_handle.set_road_tool_mesh(Some(mesh));
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
