use super::generator;
use super::network;
use crate::input;
use generator::RoadGeneratorTool;
use glam::*;

#[derive(Clone)]
pub enum Mode {
    SelectPos,
    SelectDir,
    Build,
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

impl ToolState {
    pub fn new() -> Self {
        ToolState {
            road_graph: network::RoadGraph::new(),
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

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        use network::CurveType::*;
        let (action, pressed) = key;
        if !pressed {
            match action {
                CycleRoadType => match self.sel_road_type.curve_type {
                    Straight => self.sel_road_type.curve_type = Curved,
                    Curved => self.sel_road_type.curve_type = Straight,
                },
                OneLane => self.sel_road_type.no_lanes = 1,
                TwoLane => self.sel_road_type.no_lanes = 2,
                ThreeLane => self.sel_road_type.no_lanes = 3,
                FourLane => self.sel_road_type.no_lanes = 4,
                FiveLane => self.sel_road_type.no_lanes = 5,
                SixLane => self.sel_road_type.no_lanes = 6,
                _ => {}
            };
        }
    }

    pub fn mouse_input(
        &mut self,
        event: input::MouseEvent,
    ) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        use input::MouseEvent;

        match event {
            MouseEvent::LeftClick => self.left_click(),
            MouseEvent::RightClick => self.right_click(),
            _ => (None, None),
        }
    }

    fn left_click(&mut self) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
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
        }
    }

    fn right_click(&mut self) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        use network::CurveType;
        match self.mode {
            Mode::SelectPos => {
                dbg!(self.road_graph.get_segment_inside(self.ground_pos));
                match self.road_graph.get_node_id_from_pos(self.ground_pos) {
                    Some(node_id) => {
                        let node = self.road_graph.get_node(node_id);
                        dbg!(node);
                        (None, None)
                    }
                    None => (None, None),
                }
            }
            Mode::SelectDir => {
                self.sel_node = None;
                self.snapped_node = None;
                let mesh = self.update_ground_pos(self.ground_pos);
                self.mode = Mode::SelectPos;
                (None, mesh)
            }
            Mode::Build => {
                match (self.sel_road_type.curve_type, self.sel_node.clone()) {
                    (CurveType::Curved, None) => {
                        self.road_generator.unlock_dir();
                        self.mode = Mode::SelectDir;
                    }
                    _ => {
                        self.mode = Mode::SelectPos;
                    }
                };
                self.sel_node = None;
                self.snapped_node = None;
                let mesh = self.update_ground_pos(self.ground_pos);
                (None, mesh)
            }
        }
    }

    fn select_node(&mut self, snapped_node: network::SnapConfig) -> Option<network::RoadMesh> {
        let node = self.road_graph.get_node(snapped_node.node_id);

        self.road_generator = generator::RoadGeneratorTool::new(
            snapped_node.pos,
            Some(node.dir),
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

    fn build_road(&mut self) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        let (road_mesh, new_node) = self.road_graph.add_road(
            self.road_generator.extract(),
            self.sel_node.clone(),
            self.snapped_node.clone(),
        );
        // TODO have add_road return new_node in such a way that is not necessary to check snapped_node
        if self.snapped_node.is_some() {
            self.sel_node = None;
            self.snapped_node = None;
            self.mode = Mode::SelectPos;
            (Some(road_mesh), Some(generator::empty_mesh()))
        } else if let Some(new_node) = new_node {
            let road_generator_mesh = self.select_node(new_node);
            (Some(road_mesh), road_generator_mesh)
        } else {
            self.sel_node = None;
            self.snapped_node = None;
            self.mode = Mode::SelectPos;
            (Some(road_mesh), Some(generator::empty_mesh()))
        }
    }

    fn update_no_snap(&mut self) -> Option<network::RoadMesh> {
        self.snapped_node = None;
        let empty_mesh = Some(generator::empty_mesh());

        match self.mode {
            Mode::SelectPos => empty_mesh,
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

    fn update_snap(&mut self, snap_configs: Vec<network::SnapConfig>) -> Option<network::RoadMesh> {
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
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: Vec3) -> Option<network::RoadMesh> {
        self.ground_pos = ground_pos;

        if let Some((_snap_id, snap_configs)) = self
            .road_graph
            .get_node_snap_configs(self.ground_pos, self.sel_road_type.no_lanes)
        {
            if snap_configs.is_empty() {
                self.update_no_snap()
            } else {
                match self.snapped_node.clone() {
                    Some(snapped_node) => {
                        if snapped_node == snap_configs[0] {
                            None
                        } else {
                            self.update_snap(snap_configs)
                        }
                    }
                    None => self.update_snap(snap_configs),
                }
            }
        } else {
            self.update_no_snap()
        }
    }
}
