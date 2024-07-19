use super::{Tool, ToolUnique};
use crate::gfx_gen::segment_gen;
use crate::tool_state::{CurveType, SelectedRoad};

use curve_tools::{CurveTool, CurveToolSum, StraightBuilder};
use curves::{Curve, Straight};
use utils::input;
use world_api::{LRoadBuilder, LaneWidth, NodeType, Side, SnapConfig, WorldManipulator};

use gfx_api::{GfxWorldData, RoadMesh};
use glam::*;

mod curve_tools {
    use std::marker::PhantomData;

    use curves::{CompositeCurve, Curve, CurveError, CurveInfo, CurveSpec, Straight};
    use enum_dispatch::enum_dispatch;
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

    trait CurveToolSpecInternal<C: CurveSpec> {
        /// The tool shall process a left click.
        fn left_click(&mut self, first: EndPoint, last: EndPoint) -> CurveActionResult<C>;

        /// The tool shall process a right click.
        fn right_click(&mut self, first: EndPoint, last: EndPoint) -> CurveActionResult<C>;

        /// Called whenever there the ground_pos has been updated due to a change in camera or
        /// cursor position.
        fn compute_curve(&mut self, first: EndPoint, last: EndPoint) -> CurveActionResult<C>;
    }

    #[enum_dispatch]
    pub trait CurveToolSpec<C: CurveSpec> {
        /// Selects the first point if it has not already been selected, otherwise delegates the
        /// call to instance.
        fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult<C>;

        fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult<C>;

        fn update_view(&mut self, ground_pos: Vec3) -> CurveActionResult<C>;

        /// A node has been snapped to.
        fn snap(&mut self, snap_config: SnapConfig) -> CurveActionResult<C>;

        /// A node is no longer snapped.
        fn unsnap(&mut self, ground_pos: Vec3) -> CurveActionResult<C>;
    }

    #[enum_dispatch(CurveToolSpec)]
    pub enum CurveToolSum {
        Straight(CurveTool<StraightBuilder, Curve<Straight>>),
    }

    pub struct CurveTool<CT, C: CurveSpec>
    where
        CT: CurveToolSpecInternal<C>,
    {
        instance: CT,
        first_point: Option<EndPoint>,
        snapped_node: Option<SnapConfig>,
        _marker: PhantomData<C>,
    }

    impl<CT: Default, C: CurveSpec> Default for CurveTool<CT, C>
    where
        CT: CurveToolSpecInternal<C>,
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

    impl<CT, C: CurveSpec> CurveToolSpec<C> for CurveTool<CT, C>
    where
        CT: CurveToolSpecInternal<C>,
    {
        fn left_click(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
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

        fn right_click(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
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

        fn update_view(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
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

        fn snap(&mut self, snap_config: SnapConfig) -> CurveActionResult<C> {
            self.snapped_node = Some(snap_config.clone());

            if let Some(first_point) = &self.first_point {
                return self
                    .instance
                    .compute_curve(first_point.clone(), EndPoint::Old(snap_config));
            }

            Ok(CurveAction::Stub)
        }

        fn unsnap(&mut self, ground_pos: Vec3) -> CurveActionResult<C> {
            self.snapped_node = None;

            if let Some(first_point) = &self.first_point {
                return self
                    .instance
                    .compute_curve(first_point.clone(), EndPoint::New(ground_pos));
            };

            Ok(CurveAction::Nothing)
        }
    }

    #[derive(Default)]
    pub struct StraightBuilder;

    impl CurveToolSpecInternal<Curve<Straight>> for StraightBuilder {
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

    fn left_click(&mut self, gfx_handle: &mut G) {}

    fn right_click(&mut self, gfx_handle: &mut G) {}

    fn update_view(&mut self, gfx_handle: &mut G) {
        // self.check_snapping(gfx_handle);
    }

    /// Remove node markings from gpu, and remove the road tool mesh.
    fn clean_gfx(&mut self, gfx_handle: &mut G) {
        // gfx_handle.set_node_markers(vec![]);
        // gfx_handle.set_road_tool_mesh(None);
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
    // fn handle_curve_action(&mut self, )

    // #############################################################################################
    // Snapping
    // #############################################################################################
    /// Checks if there is a node that we should snap to, and in that case it snaps to that node.
    fn check_snapping<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        // TODO add functionality to report why a node cannot be snapped to.
        if !self.state_handle.road_state.snapping {
            // self.update_no_snap(gfx_handle);
            return;
        }

        // Get available snaps
        let node_snap_configs = self
            .world
            .get_snap_configs_closest_node(self.ground_pos, self.get_sel_road_type().node_type);

        let Some((_snap_id, mut snap_configs)) = node_snap_configs else {
            // self.update_no_snap(gfx_handle);
            return;
        };

        // if let SelNode { snap_config, .. } = &self.instance.mode {
        //     snap_configs.retain(|s| s.side() != snap_config.side());
        // }

        if snap_configs.is_empty() {
            // self.update_no_snap(gfx_handle);
            return;
        }

        // self.update_snap(gfx_handle, snap_configs);
        unimplemented!()
    }

    // #############################################################################################
    // Gfx handling
    // #############################################################################################
    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        if !self.state_handle.road_state.snapping {
            return;
        }
        let side = Side::In;
        // let side = if let SelNode { snap_config, .. } = &self.instance.mode {
        //     Some(snap_config.side())
        // } else {
        //     None
        // };
        let possible_snaps = self
            .world
            .get_possible_snap_nodes(Some(side), self.get_sel_road_type().node_type)
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
