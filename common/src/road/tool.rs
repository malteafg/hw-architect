use super::generator;
use super::network;
use generator::RoadGeneratorTool;
use gfx_bridge::roads::RoadMesh;
use glam::*;
use utils::input;

#[derive(Clone, Copy)]
pub enum Mode {
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
    pub fn process_keyboard(&mut self, key: input::KeyAction) -> Option<RoadMesh> {
        use input::Action::*;
        let (action, pressed) = key;
        if pressed {
            return None;
        }
        match self.mode {
            Mode::Bulldoze => match action {
                ToggleBulldoze => self.reset_snap(Mode::SelectPos),
                _ => None,
            },
            _ => match action {
                CycleRoadType => self.switch_curve_type(),
                ToggleBulldoze => {
                    self.mode = Mode::Bulldoze;
                    Some(generator::empty_mesh())
                }
                OneLane => self.switch_lane_no(1),
                TwoLane => self.switch_lane_no(2),
                ThreeLane => self.switch_lane_no(3),
                FourLane => self.switch_lane_no(4),
                FiveLane => self.switch_lane_no(5),
                SixLane => self.switch_lane_no(6),
                _ => None,
            },
        }
    }

    fn switch_lane_no(&mut self, no_lanes: u8) -> Option<RoadMesh> {
        if self.sel_road_type.no_lanes == no_lanes {
            return None;
        };
        self.sel_road_type.no_lanes = no_lanes;
        self.road_generator.update_no_lanes(no_lanes);
        if self.sel_node.is_none() {
            self.check_snapping()
        } else {
            self.reset_snap(Mode::SelectPos)
        }
    }

    fn switch_curve_type(&mut self) -> Option<RoadMesh> {
        use network::CurveType::*;
        match self.sel_road_type.curve_type {
            Straight => {
                self.sel_road_type.curve_type = Curved;
                self.road_generator.update_curve_type(Curved);
                if let Mode::Build = self.mode {
                    self.check_snapping()
                } else {
                    None
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
                    self.check_snapping()
                } else {
                    None
                }
            }
        }
    }

    pub fn mouse_input(
        &mut self,
        event: input::MouseEvent,
    ) -> (Option<RoadMesh>, Option<RoadMesh>) {
        use input::MouseEvent;

        match event {
            MouseEvent::Click(button) if button == input::Mouse::Left => self.left_click(),
            MouseEvent::Click(button) if button == input::Mouse::Right => self.right_click(),
            _ => (None, None),
        }
    }

    fn left_click(&mut self) -> (Option<RoadMesh>, Option<RoadMesh>) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos => match self.snapped_node.clone() {
                Some(snapped_node) => {
                    let road_mesh = self.select_node(snapped_node);
                    (None, road_mesh)
                }
                None => {
                    self.road_generator =
                        RoadGeneratorTool::new(self.ground_pos, None, self.sel_road_type, false);
                    let road_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::SelectDir;
                    (None, road_mesh)
                }
            },
            Mode::SelectDir => match self.sel_road_type.curve_type {
                CurveType::Straight => self.build_road(),
                CurveType::Curved => {
                    self.road_generator.lock_dir(self.ground_pos);
                    self.road_generator.update_pos(self.ground_pos);
                    let road_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::Build;
                    (None, road_mesh)
                }
            },
            Mode::Build => self.build_road(),
            Mode::Bulldoze => {
                let segment_id = self.road_graph.get_segment_inside(self.ground_pos);
                let road_mesh = segment_id.map(|id| self.road_graph.remove_segment(id));
                (road_mesh.flatten(), None)
            }
        }
    }

    fn right_click(&mut self) -> (Option<RoadMesh>, Option<RoadMesh>) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos | Mode::Bulldoze => {
                #[cfg(debug_assertions)]
                {
                    self.road_graph.debug_segment_from_pos(self.ground_pos);
                    self.road_graph.debug_node_from_pos(self.ground_pos);
                }
                (None, None)
            }
            Mode::SelectDir => (None, self.reset_snap(Mode::SelectPos)),
            Mode::Build => match (self.sel_road_type.curve_type, self.sel_node.clone()) {
                (CurveType::Curved, None) => {
                    self.road_generator.unlock_dir();
                    (None, self.reset_snap(Mode::SelectDir))
                }
                _ => (None, self.reset_snap(Mode::SelectPos)),
            },
        }
    }

    fn select_node(&mut self, snapped_node: network::SnapConfig) -> Option<RoadMesh> {
        let node = self.road_graph.get_node(snapped_node.node_id);

        self.road_generator = generator::RoadGeneratorTool::new(
            snapped_node.pos,
            Some(node.get_dir()),
            self.sel_road_type,
            snapped_node.reverse,
        );
        self.road_generator.update_pos(self.ground_pos);
        let road_mesh = self.road_generator.get_mesh();

        self.sel_node = Some(snapped_node);
        self.snapped_node = None;
        self.mode = Mode::Build;

        road_mesh
    }

    fn build_road(&mut self) -> (Option<RoadMesh>, Option<RoadMesh>) {
        let (road_mesh, new_node) = self.road_graph.add_road(
            self.road_generator.extract(),
            self.sel_node.clone(),
            self.snapped_node.clone(),
        );
        // TODO have add_road return new_node in such a way that is not necessary to check snapped_node
        if self.snapped_node.is_some() {
            (Some(road_mesh), self.reset_snap(Mode::SelectPos))
        } else if let Some(new_node) = new_node {
            let road_generator_mesh = self.select_node(new_node);
            (Some(road_mesh), road_generator_mesh)
        } else {
            (Some(road_mesh), self.reset_snap(Mode::SelectPos))
        }
    }

    fn reset_snap(&mut self, new_mode: Mode) -> Option<RoadMesh> {
        if let Mode::SelectPos = new_mode {
            self.road_generator = RoadGeneratorTool::default();
        };
        self.sel_node = None;
        self.snapped_node = None;
        self.mode = new_mode;
        self.check_snapping()
    }

    fn update_no_snap(&mut self) -> Option<RoadMesh> {
        self.snapped_node = None;
        let empty_mesh = Some(generator::empty_mesh());

        match self.mode {
            Mode::SelectPos | Mode::Bulldoze => empty_mesh,
            Mode::SelectDir => {
                self.road_generator.update_pos(self.ground_pos);
                self.road_generator.get_mesh()
            }
            Mode::Build => {
                self.road_generator.update_pos(self.ground_pos);
                self.road_generator.get_mesh()
            }
        }
    }

    fn update_snap(&mut self, snap_configs: Vec<network::SnapConfig>) -> Option<RoadMesh> {
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
                road_generator.get_mesh()
            }
            Mode::SelectDir | Mode::Build => {
                for snap_config in snap_configs {
                    if self
                        .road_generator
                        .try_snap(snap_config.clone(), self.sel_node.is_some())
                        .is_some()
                    {
                        self.snapped_node = Some(snap_config);
                        return self.road_generator.get_mesh();
                    }
                }
                self.update_no_snap()
            }
            Mode::Bulldoze => Some(generator::empty_mesh()),
        }
    }

    fn check_snapping(&mut self) -> Option<RoadMesh> {
        if let Some((_snap_id, snap_configs)) = self
            .road_graph
            .get_node_snap_configs(self.ground_pos, self.sel_road_type.no_lanes)
        {
            if snap_configs.is_empty() {
                self.update_no_snap()
            } else {
                self.update_snap(snap_configs)
                // match self.snapped_node.clone() {
                //     Some(snapped_node) => {
                //         if snapped_node == snap_configs[0] {
                //             None
                //         } else {
                //             self.update_snap(snap_configs)
                //         }
                //     }
                //     None => self.update_snap(snap_configs),
                // }
            }
        } else {
            self.update_no_snap()
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: Vec3) -> Option<RoadMesh> {
        self.ground_pos = ground_pos;
        self.check_snapping()
    }
}
