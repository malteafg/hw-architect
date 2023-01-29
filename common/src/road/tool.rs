use super::generator;
use super::network;
use generator::RoadGeneratorTool;
use gfx_api::RoadMesh;
use glam::*;
use utils::input;

#[derive(Clone, Copy)]
enum Mode {
    SelectPos,
    SelectDir,
    Build,
    Bulldoze,
}

pub struct ToolState {
    road_graph: network::RoadGraph,
    sel_road_type: network::RoadType,
    sel_node: Option<network::SnapConfig>,
    snapped_node: Option<network::SnapConfig>,
    road_generator: generator::RoadGeneratorTool,
    ground_pos: Vec3,
    mode: Mode,
}

#[derive(Default)]
pub struct GfxData {
    pub road_mesh: Option<RoadMesh>,
    pub road_tool_mesh: Option<RoadMesh>,
}

impl Default for ToolState {
    fn default() -> Self {
        ToolState {
            road_graph: network::RoadGraph::default(),
            sel_road_type: network::RoadType {
                no_lanes: 3,
                curve_type: network::CurveType::Straight,
            },
            sel_node: None,
            snapped_node: None,
            road_generator: RoadGeneratorTool::default(),
            ground_pos: Vec3::new(0.0, 0.0, 0.0),
            mode: Mode::SelectPos,
        }
    }
}

impl ToolState {
    pub fn process_keyboard(&mut self, gfx_data: &mut GfxData, key: input::KeyAction) {
        use input::Action::*;
        let (action, pressed) = key;
        if pressed {
            return;
        }
        match self.mode {
            Mode::Bulldoze => {
                if action == ToggleBulldoze {
                    self.reset_snap(gfx_data, Mode::SelectPos);
                }
            }
            _ => match action {
                CycleRoadType => self.switch_curve_type(gfx_data),
                ToggleBulldoze => {
                    self.mode = Mode::Bulldoze;
                    gfx_data.road_tool_mesh = Some(generator::empty_mesh());
                }
                OneLane => self.switch_lane_no(gfx_data, 1),
                TwoLane => self.switch_lane_no(gfx_data, 2),
                ThreeLane => self.switch_lane_no(gfx_data, 3),
                FourLane => self.switch_lane_no(gfx_data, 4),
                FiveLane => self.switch_lane_no(gfx_data, 5),
                SixLane => self.switch_lane_no(gfx_data, 6),
                _ => {}
            },
        }
    }

    fn switch_lane_no(&mut self, gfx_data: &mut GfxData, no_lanes: u8) {
        if self.sel_road_type.no_lanes == no_lanes {
            return;
        };
        self.sel_road_type.no_lanes = no_lanes;
        self.road_generator.update_no_lanes(no_lanes);
        if self.sel_node.is_none() {
            self.check_snapping(gfx_data);
        } else {
            self.reset_snap(gfx_data, Mode::SelectPos);
        }
    }

    fn switch_curve_type(&mut self, gfx_data: &mut GfxData) {
        use network::CurveType::*;
        match self.sel_road_type.curve_type {
            Straight => {
                self.sel_road_type.curve_type = Curved;
                self.road_generator.update_curve_type(Curved);
                if let Mode::Build = self.mode {
                    self.check_snapping(gfx_data);
                }
            }
            Curved => {
                self.sel_road_type.curve_type = Straight;
                self.road_generator.update_curve_type(Straight);
                if let Mode::Build = self.mode {
                    if self.sel_node.is_none() {
                        self.road_generator.unlock_dir();
                        self.mode = Mode::SelectDir;
                    }
                    self.check_snapping(gfx_data);
                }
            }
        }
    }

    pub fn mouse_input(&mut self, gfx_data: &mut GfxData, event: input::MouseEvent) {
        use input::MouseEvent;

        match event {
            MouseEvent::Click(button) if button == input::Mouse::Left => self.left_click(gfx_data),
            MouseEvent::Click(button) if button == input::Mouse::Right => {
                self.right_click(gfx_data);
            }
            _ => {}
        }
    }

    fn left_click(&mut self, gfx_data: &mut GfxData) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos => match self.snapped_node.clone() {
                Some(snapped_node) => {
                    self.select_node(gfx_data, snapped_node);
                }
                None => {
                    self.road_generator =
                        RoadGeneratorTool::new(self.ground_pos, None, self.sel_road_type, false);
                    gfx_data.road_tool_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::SelectDir;
                }
            },
            Mode::SelectDir => match self.sel_road_type.curve_type {
                CurveType::Straight => self.build_road(gfx_data),
                CurveType::Curved => {
                    self.road_generator.lock_dir(self.ground_pos);
                    self.road_generator.update_pos(self.ground_pos);
                    gfx_data.road_tool_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::Build;
                }
            },
            Mode::Build => self.build_road(gfx_data),
            Mode::Bulldoze => {
                let segment_id = self.road_graph.get_segment_inside(self.ground_pos);
                let road_mesh = segment_id.map(|id| self.road_graph.remove_segment(id));
                gfx_data.road_mesh = road_mesh.flatten();
            }
        }
    }

    fn right_click(&mut self, gfx_data: &mut GfxData) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos | Mode::Bulldoze => {
                #[cfg(debug_assertions)]
                {
                    self.road_graph.debug_segment_from_pos(self.ground_pos);
                    self.road_graph.debug_node_from_pos(self.ground_pos);
                }
            }
            Mode::SelectDir => self.reset_snap(gfx_data, Mode::SelectPos),
            Mode::Build => match (self.sel_road_type.curve_type, self.sel_node.clone()) {
                (CurveType::Curved, None) => {
                    self.road_generator.unlock_dir();
                    self.reset_snap(gfx_data, Mode::SelectDir)
                }
                _ => self.reset_snap(gfx_data, Mode::SelectPos),
            },
        }
    }

    fn select_node(&mut self, gfx_data: &mut GfxData, snapped_node: network::SnapConfig) {
        let node = self.road_graph.get_node(snapped_node.node_id);

        self.road_generator = generator::RoadGeneratorTool::new(
            snapped_node.pos,
            Some(node.get_dir()),
            self.sel_road_type,
            snapped_node.reverse,
        );
        self.road_generator.update_pos(self.ground_pos);
        gfx_data.road_tool_mesh = self.road_generator.get_mesh();

        self.sel_node = Some(snapped_node);
        self.snapped_node = None;
        self.mode = Mode::Build;
    }

    fn build_road(&mut self, gfx_data: &mut GfxData) {
        let (road_mesh, new_node) = self.road_graph.add_road(
            self.road_generator.extract(),
            self.sel_node.clone(),
            self.snapped_node.clone(),
        );
        gfx_data.road_mesh = Some(road_mesh);
        // TODO have add_road return new_node in such a way that is not necessary to check snapped_node
        if self.snapped_node.is_some() {
            self.reset_snap(gfx_data, Mode::SelectPos);
        } else if let Some(new_node) = new_node {
            self.select_node(gfx_data, new_node);
        } else {
            self.reset_snap(gfx_data, Mode::SelectPos);
        }
    }

    fn reset_snap(&mut self, gfx_data: &mut GfxData, new_mode: Mode) {
        if let Mode::SelectPos = new_mode {
            self.road_generator = RoadGeneratorTool::default();
        };
        self.sel_node = None;
        self.snapped_node = None;
        self.mode = new_mode;
        self.check_snapping(gfx_data);
    }

    fn update_no_snap(&mut self, gfx_data: &mut GfxData) {
        self.snapped_node = None;
        let empty_mesh = Some(generator::empty_mesh());

        match self.mode {
            Mode::SelectPos | Mode::Bulldoze => gfx_data.road_tool_mesh = empty_mesh,
            Mode::SelectDir => {
                self.road_generator.update_pos(self.ground_pos);
                gfx_data.road_tool_mesh = self.road_generator.get_mesh();
            }
            Mode::Build => {
                self.road_generator.update_pos(self.ground_pos);
                gfx_data.road_tool_mesh = self.road_generator.get_mesh();
            }
        };
    }

    fn update_snap(&mut self, gfx_data: &mut GfxData, snap_configs: Vec<network::SnapConfig>) {
        match self.mode {
            Mode::SelectPos => {
                let snap_config = &snap_configs[0];
                let road_generator = RoadGeneratorTool::new(
                    snap_config.pos,
                    Some(snap_config.dir),
                    self.sel_road_type,
                    snap_config.reverse,
                );
                self.snapped_node = Some(snap_config.clone());
                gfx_data.road_tool_mesh = road_generator.get_mesh();
            }
            Mode::SelectDir | Mode::Build => {
                for snap_config in snap_configs {
                    if self
                        .road_generator
                        .try_snap(snap_config.clone(), self.sel_node.is_some())
                        .is_some()
                    {
                        self.snapped_node = Some(snap_config);
                        gfx_data.road_tool_mesh = self.road_generator.get_mesh();
                        return;
                    }
                }
                self.update_no_snap(gfx_data);
            }
            Mode::Bulldoze => gfx_data.road_tool_mesh = Some(generator::empty_mesh()),
        };
    }

    fn check_snapping(&mut self, gfx_data: &mut GfxData) {
        if let Some((_snap_id, snap_configs)) = self
            .road_graph
            .get_node_snap_configs(self.ground_pos, self.sel_road_type.no_lanes)
        {
            if snap_configs.is_empty() {
                self.update_no_snap(gfx_data);
            } else {
                self.update_snap(gfx_data, snap_configs);
            }
        } else {
            self.update_no_snap(gfx_data);
        }
    }

    pub fn update_ground_pos(&mut self, gfx_data: &mut GfxData, ground_pos: Vec3) {
        self.ground_pos = ground_pos;
        self.check_snapping(gfx_data);
    }
}
