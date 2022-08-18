use super::curves;
use super::generator;
use super::network;
use crate::input;
use glam::*;

#[derive(Clone)]
pub enum Mode {
    SelectPos,
    SelectDir(generator::RoadGenerator),
    Build(generator::RoadGenerator),
}

pub struct ToolState {
    road_graph: network::RoadGraph,
    sel_road_type: network::RoadType,
    sel_node: Option<network::SnapConfig>,
    snapped_node: Option<network::SnapConfig>,
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
        use generator::RoadGenerator;
        use network::CurveType;

        match self.mode.clone() {
            Mode::SelectPos => match self.snapped_node {
                Some(snapped_node) => {
                    let road_mesh = self.select_node(snapped_node);
                    (None, road_mesh)
                }
                None => {
                    let road_generator =
                        RoadGenerator::new(self.ground_pos, self.sel_road_type, None, false);
                    let road_mesh = road_generator.get_mesh();

                    self.mode = Mode::SelectDir(road_generator);
                    (None, road_mesh)
                }
            },
            Mode::SelectDir(ref mut road_generator) => match self.sel_road_type.curve_type {
                CurveType::Straight => self.build_road(road_generator.clone()),
                CurveType::Curved => {
                    road_generator.lock();
                    road_generator.update_pos(self.ground_pos);
                    let road_mesh = road_generator.get_mesh();

                    self.mode = Mode::Build(road_generator.clone());
                    (None, road_mesh)
                }
            },
            Mode::Build(ref mut road_generator) => self.build_road(road_generator.clone()),
        }
    }

    fn right_click(&mut self) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        use network::CurveType;

        // returned when road_generator is set to None
        let empty_mesh = Some(generator::empty_mesh());

        match self.mode.clone() {
            Mode::SelectPos => match self.road_graph.get_node_debug(self.ground_pos) {
                Some(snap_config) => {
                    let snapped_node = self.road_graph.get_node(snap_config.node_id);
                    dbg!(snapped_node);
                    (None, None)
                }
                None => (None, None),
            },
            Mode::SelectDir(_) => {
                self.sel_node = None;
                self.snapped_node = None;
                self.update_ground_pos(self.ground_pos);
                self.mode = Mode::SelectPos;
                (None, empty_mesh)
            }
            Mode::Build(mut road_generator) => {
                match (self.sel_road_type.curve_type, self.sel_node) {
                    (CurveType::Curved, None) => {
                        road_generator.unlock();
                        self.mode = Mode::SelectDir(road_generator);
                    }
                    _ => {
                        self.mode = Mode::SelectPos;
                    }
                };
                self.sel_node = None;
                self.snapped_node = None;
                self.update_ground_pos(self.ground_pos);
                (None, empty_mesh)
            }
        }
    }

    fn select_node(&mut self, snapped_node: network::SnapConfig) -> Option<network::RoadMesh> {
        let node = self.road_graph.get_node(snapped_node.node_id);

        let mut road_generator = generator::RoadGenerator::new(
            node.pos,
            self.sel_road_type,
            Some(node.dir),
            snapped_node.reverse,
        );
        road_generator.update_pos(self.ground_pos);
        let road_mesh = road_generator.get_mesh();

        self.sel_node = Some(snapped_node);
        self.snapped_node = None;
        self.mode = Mode::Build(road_generator);

        road_mesh
    }

    fn build_road(
        &mut self,
        road_generator: generator::RoadGenerator,
    ) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        let (road_mesh, new_node) =
            self.road_graph
                .add_road(road_generator, self.sel_node, self.snapped_node);
        if self.snapped_node.is_some() {
            self.sel_node = None;
            self.snapped_node = None;
            self.mode = Mode::SelectPos;
            (Some(road_mesh), Some(generator::empty_mesh()))
        } else {
            let road_generator_mesh = self.select_node(new_node);
            (Some(road_mesh), road_generator_mesh)
        }
    }

    fn check_snapping(&mut self) -> Option<curves::DoubleSnapCurveCase> {
        // check for node within ground_pos
        // pass to road_generator to see if a road can be generated
        let possible_snap = self.road_graph.get_node_from_pos(self.ground_pos);
        match (possible_snap, self.sel_node) {
            (Some(possible_snap), None) => {
                match self.mode {
                    Mode::SelectPos => {
                        self.snapped_node = Some(possible_snap);
                        None
                    }
                    Mode::SelectDir(ref mut road_generator) => {
                        self.snapped_node = None;
                        None
                    }
                    Mode::Build(ref mut road_generator) => {
                        use curves::DoubleSnapCurveCase::*;
                        let node = self.road_graph.get_node(possible_snap.node_id);
                        let ((start_pos, start_dir), (end_pos, end_dir)) = if possible_snap.reverse
                        {
                            (road_generator.get_start_node(), (node.pos, node.dir))
                        } else {
                            let (end_pos, end_dir) = road_generator.get_start_node();
                            ((node.pos, node.dir), (end_pos, -end_dir))
                        };
                        match curves::double_snap_curve_case(
                            start_pos,
                            start_dir,
                            end_pos,
                            end_dir,
                            self.sel_road_type.no_lanes,
                        ) {
                            ErrorTooSmall | ErrorSegmentAngle | ErrorCurveAngle
                            | ErrorUnhandled => {
                                self.snapped_node = None;
                                None
                            }
                            snap_case => {
                                self.snapped_node = Some(possible_snap);
                                Some(snap_case)
                            }
                        }
                    }
                }
                // TODO check if lanes match the type we are connecting to
            }
            (Some(possible_snap), Some(sel_node)) => {
                if possible_snap.node_id == sel_node.node_id
                    || possible_snap.reverse == sel_node.reverse
                {
                    self.snapped_node = None;
                    None
                } else {
                    use curves::DoubleSnapCurveCase::*;
                    let (start_node, end_node) = if sel_node.reverse {
                        (
                            self.road_graph.get_node(possible_snap.node_id),
                            self.road_graph.get_node(sel_node.node_id),
                        )
                    } else {
                        (
                            self.road_graph.get_node(sel_node.node_id),
                            self.road_graph.get_node(possible_snap.node_id),
                        )
                    };
                    match curves::double_snap_curve_case(
                        start_node.pos,
                        start_node.dir,
                        end_node.pos,
                        end_node.dir,
                        self.sel_road_type.no_lanes,
                    ) {
                        ErrorTooSmall | ErrorSegmentAngle | ErrorCurveAngle | ErrorUnhandled => {
                            self.snapped_node = None;
                            None
                        }
                        snap_case => {
                            self.snapped_node = Some(possible_snap);
                            Some(snap_case)
                        }
                    }
                }
            }
            _ => {
                self.snapped_node = None;
                None
            }
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: Vec3) -> Option<network::RoadMesh> {
        use generator::RoadGenerator;

        self.ground_pos = ground_pos;

        // returned when road_generator is set to None
        let empty_mesh = Some(generator::empty_mesh());

        let prev_snap = self.snapped_node;
        let snap_case = self.check_snapping();

        // node snap has not changed
        match (prev_snap, self.snapped_node) {
            (Some(a), Some(b)) => {
                if a == b {
                    return None;
                }
            }
            _ => {}
        }
        if let Some(s) = self.snapped_node {
            dbg!(s);
        }

        match self.mode {
            Mode::SelectPos => match self.snapped_node {
                Some(snapped_node) => {
                    let node = self.road_graph.get_node(snapped_node.node_id);
                    let road_generator = RoadGenerator::new(
                        node.pos,
                        self.sel_road_type,
                        Some(node.dir),
                        snapped_node.reverse,
                    );
                    road_generator.get_mesh()
                }
                None => empty_mesh,
            },
            Mode::SelectDir(ref mut road_generator) => {
                // for now we are not allowed to snap in dir mode
                road_generator.update_pos(ground_pos);
                road_generator.get_mesh()
            }
            Mode::Build(ref mut road_generator) => match (snap_case, self.snapped_node) {
                (Some(snap_case), Some(snapped_node)) => {
                    let snapped_node = self.road_graph.get_node(snapped_node.node_id);
                    road_generator.double_snap(snap_case, snapped_node.pos, snapped_node.dir);
                    road_generator.get_mesh()
                }
                _ => {
                    road_generator.update_pos(ground_pos);
                    road_generator.get_mesh()
                }
            },
        }
    }
}
