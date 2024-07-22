mod circular_tool;
mod curve_tool_spec;
mod straight_tool;

use circular_tool::CircularTool;
use curve_tool_spec::{CurveAction, CurveActionResult, CurveTool, CurveToolSpec, CurveToolSum};
use straight_tool::StraightTool;

use super::{Tool, ToolUnique};

use crate::cycle_selection;
use crate::gfx_gen::segment_gen;
use crate::tool_state::{CurveType, SelectedRoad};

use curves::{Circular, CompositeCurveSum, Curve, CurveError, CurveShared, Straight};
use utils::id::{IdMap, SegmentId};
use utils::{input, Loc};
use world_api::{
    LNodeBuilder, LNodeBuilderType, LRoadBuilder, LSegmentBuilder, LaneWidth, NodeType, SnapConfig,
    WorldManipulator,
};

use gfx_api::{GfxWorldData, RoadMesh};
use glam::*;

pub struct Construct {
    curve_tool: CurveToolSum,
}

impl Default for Construct {
    fn default() -> Self {
        Self {
            curve_tool: CurveTool::<CircularTool, Curve<Circular>>::default().into(),
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
                self.state_handle.road_state.snapping = !self.state_handle.road_state.snapping;
                dbg!(self.state_handle.road_state.snapping);

                self.update_view(gfx_handle);
                self.show_snappable_nodes(gfx_handle);
            }
            (ToggleReverse, Press) => {
                self.state_handle.road_state.reverse = !self.state_handle.road_state.reverse;
                dbg!(self.state_handle.road_state.reverse);
            }
            (CycleCurveType, Scroll(scroll_state)) => {
                let new_curve_type =
                    cycle_selection::scroll(self.get_sel_curve_type(), scroll_state);
                dbg!(self.get_sel_curve_type());
                self.state_handle.road_state.set_curve_type(new_curve_type);

                match new_curve_type {
                    CurveType::Straight => {
                        self.instance.curve_tool =
                            CurveTool::<StraightTool, Curve<Straight>>::default().into()
                    }
                    CurveType::Circular => {
                        self.instance.curve_tool =
                            CurveTool::<CircularTool, Curve<Circular>>::default().into()
                    }
                }

                self.instance.curve_tool.reset(None);
                self.update_view(gfx_handle);
                self.show_snappable_nodes(gfx_handle);
            }
            (CycleLaneWidth, Scroll(scroll_state)) => {
                let new_lane_width =
                    cycle_selection::scroll(self.get_sel_lane_width(), scroll_state);
                dbg!(new_lane_width);
                self.state_handle.road_state.set_lane_width(new_lane_width);

                self.instance.curve_tool.reset(None);
                self.update_view(gfx_handle);
                self.show_snappable_nodes(gfx_handle);
            }
            (CycleNoLanes, Scroll(scroll_state)) => {
                let new_no_lanes = cycle_selection::scroll(self.get_sel_no_lanes(), scroll_state);
                dbg!(new_no_lanes);
                self.state_handle.road_state.set_no_lanes(new_no_lanes);

                self.instance.curve_tool.reset(None);
                self.update_view(gfx_handle);
                self.show_snappable_nodes(gfx_handle);
            }
            _ => {}
        }
    }

    fn left_click(&mut self, gfx_handle: &mut G) {
        let action = self.instance.curve_tool.left_click(self.ground_pos);
        self.handle_curve_action_result(gfx_handle, action);
    }

    fn right_click(&mut self, gfx_handle: &mut G) {
        let action = self.instance.curve_tool.right_click(self.ground_pos);
        self.handle_curve_action_result(gfx_handle, action);
    }

    fn update_view(&mut self, gfx_handle: &mut G) {
        let snap = self.check_snapping();
        let action = match snap {
            Some(snap) => self.instance.curve_tool.update_snap(snap),
            None => self.instance.curve_tool.update_no_snap(self.ground_pos),
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
        self.show_snappable_nodes(gfx_handle);
    }

    fn handle_curve_action<G: GfxWorldData>(&mut self, gfx_handle: &mut G, action: CurveAction) {
        use CurveAction::*;
        match action {
            Construct(curve) => self.construct_road(gfx_handle, curve),
            Render(curve, curve_info) => {
                self.set_road_tool_mesh(gfx_handle, curve, self.get_sel_node_type());
                dbg!(curve_info);
            }
            Direction(loc, pos) => {
                dbg!(loc);
                dbg!(pos);
            }
            ControlPoint(_first, _last) => unimplemented!(),
            Stub(loc) => {
                let reverse = self
                    .instance
                    .curve_tool
                    .is_building_reverse(self.is_reverse());
                let (curve, _) =
                    Curve::<Straight>::from_free(loc.pos, loc.pos + loc.dir.flip(!reverse));
                self.set_road_tool_mesh(gfx_handle, curve.into(), self.get_sel_node_type());
            }
            Nothing => {}
        }
    }

    fn handle_curve_error<G: GfxWorldData>(&mut self, _gfx_handle: &mut G, error: CurveError) {
        dbg!(error);
    }

    fn construct_road<G: GfxWorldData>(&mut self, gfx_handle: &mut G, curve: CompositeCurveSum) {
        let road_builder = match curve {
            CompositeCurveSum::Single(mut curve) => {
                let (first, last, reverse) = self.construct_compute_end_nodes();
                if reverse {
                    curve.reverse();
                }

                let nodes = vec![
                    self.map_end_point(first, curve.first()),
                    self.map_end_point(last, curve.last()),
                ];
                let segments = vec![LSegmentBuilder::new(
                    self.get_sel_road_type().node_type,
                    curve,
                )];
                LRoadBuilder::new(nodes, segments, reverse)
            }
            CompositeCurveSum::Double(mut curve1, mut curve2) => {
                let (first, last, reverse) = self.construct_compute_end_nodes();
                if reverse {
                    curve1.reverse();
                    curve2.reverse();
                    let temp = curve1;
                    curve1 = curve2;
                    curve2 = temp;
                }

                let nodes = vec![
                    self.map_end_point(first, curve1.first()),
                    self.map_end_point(None, curve1.last()),
                    self.map_end_point(last, curve2.last()),
                ];
                let segments = vec![
                    LSegmentBuilder::new(self.get_sel_road_type().node_type, curve1),
                    LSegmentBuilder::new(self.get_sel_road_type().node_type, curve2),
                ];
                LRoadBuilder::new(nodes, segments, reverse)
            }
        };

        let road_meshes = self.gen_road_mesh_from_builder(&road_builder, self.get_sel_node_type());
        let (new_snap, segment_ids) = self.world.add_road(road_builder, self.get_sel_node_type());

        let mut mesh_map: IdMap<SegmentId, RoadMesh> = IdMap::new();
        for i in 0..segment_ids.len() {
            mesh_map.insert(segment_ids[i], road_meshes[i].clone());
        }
        gfx_handle.add_road_meshes(mesh_map);

        self.instance.curve_tool.reset(new_snap);
        self.update_view(gfx_handle);
    }

    fn construct_compute_end_nodes(&self) -> (Option<SnapConfig>, Option<SnapConfig>, bool) {
        let reverse = self
            .instance
            .curve_tool
            .is_building_reverse(self.is_reverse());

        let result = (
            self.instance.curve_tool.get_selected_node(),
            self.instance.curve_tool.get_snapped_node(),
        );
        if reverse {
            (result.1, result.0, reverse)
        } else {
            (result.0, result.1, reverse)
        }
    }

    fn map_end_point(&self, snap: Option<SnapConfig>, loc: Loc) -> LNodeBuilderType {
        match snap {
            Some(snap) => LNodeBuilderType::Old(snap),
            None => {
                LNodeBuilderType::New(LNodeBuilder::new(loc, self.get_sel_road_type().node_type))
            }
        }
    }

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

        if let Some(snap_config) = &self.instance.curve_tool.get_selected_node() {
            snap_configs.retain(|s| s.side() != snap_config.side());
        }

        if snap_configs.is_empty() {
            return None;
        }

        Some(snap_configs[0].clone())
    }

    // #############################################################################################
    // Gfx handling
    // #############################################################################################
    /// Marks the nodes that can be snapped to on the gpu.
    fn show_snappable_nodes<G: GfxWorldData>(&mut self, gfx_handle: &mut G) {
        if !self.state_handle.road_state.snapping {
            return;
        }
        let side = if let Some(snap_config) = &self.instance.curve_tool.get_selected_node() {
            Some(snap_config.side())
        } else {
            None
        };
        let possible_snaps = self
            .world
            .get_possible_snap_nodes(side, self.get_sel_road_type().node_type)
            .iter()
            .map(|(_id, loc)| {
                (
                    <[f32; 3]>::from(loc.pos),
                    <[f32; 3]>::from(Vec3::from(loc.dir)),
                )
            })
            .collect();

        gfx_handle.set_node_markers(possible_snaps);
    }

    fn set_road_tool_mesh<G: GfxWorldData>(
        &self,
        gfx_handle: &mut G,
        curve: CompositeCurveSum,
        node_type: NodeType,
    ) {
        let mesh = match curve {
            CompositeCurveSum::Single(curve) => {
                segment_gen::gen_road_mesh_with_lanes(curve.get_spine(), node_type)
            }
            CompositeCurveSum::Double(curve1, curve2) => {
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
}
