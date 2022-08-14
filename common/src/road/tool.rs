use super::curves;
use super::generator;
use super::network;
use crate::input;
use glam::*;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    SelectPos,
    SelectDir,
    Build,
}

pub struct ToolState {
    road_generator: generator::RoadGenerator,
    road_graph: network::RoadGraph,
    sel_road_type: network::RoadType,
    sel_node: Option<network::NodeId>,
    snapped_node: Option<network::NodeId>,
    ground_pos: Vec3,
    mode: Mode,
}

impl ToolState {
    pub fn new() -> Self {
        ToolState {
            road_generator: generator::RoadGenerator::default(),
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
        use generator::RoadGenerator;
        use input::MouseEvent;
        use network::CurveType;

        // match event {
        //     MouseEvent::LeftClick | MouseEvent::RightClick => {
        //         dbg!(self.sel_road_type, self.sel_node, self.snapped_node, self.mode);
        //     },
        //     _ => {},
        // };

        // returned when road_generator is set to None
        let empty_mesh = Some(generator::empty_mesh());

        match (self.mode, event) {
            (Mode::SelectPos, MouseEvent::LeftClick) => match self.snapped_node {
                Some(snapped_node) => {
                    let road_mesh = self.select_node(snapped_node);
                    (None, road_mesh)
                }
                None => {
                    self.road_generator =
                        RoadGenerator::new(self.ground_pos, self.sel_road_type, None);
                    let road_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::SelectDir;
                    (None, road_mesh)
                }
            },
            (Mode::SelectDir, MouseEvent::LeftClick) => match self.sel_road_type.curve_type {
                CurveType::Straight => self.build_road(),
                CurveType::Curved => {
                    self.road_generator.lock();
                    self.road_generator.update_pos(self.ground_pos);
                    let road_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::Build;
                    (None, road_mesh)
                }
            },
            (Mode::Build, MouseEvent::LeftClick) => self.build_road(),
            (_, MouseEvent::RightClick) => {
                match self.snapped_node {
                    Some(node_id) => {
                        let snapped_node = self.road_graph.get_node(node_id);
                        dbg!(snapped_node);
                        (None, None)
                    }
                    None => match self.mode {
                        Mode::SelectDir => {
                            self.road_generator = generator::RoadGenerator::default();
                            self.sel_node = None;
                            self.snapped_node = None;
                            self.update_ground_pos(self.ground_pos);
                            self.mode = Mode::SelectPos;
                            (None, empty_mesh)
                        }
                        Mode::Build => {
                            match (self.sel_road_type.curve_type, self.sel_node) {
                                (CurveType::Curved, None) => {
                                    self.road_generator.unlock();
                                    self.mode = Mode::SelectDir;
                                }
                                _ => {
                                    self.road_generator = generator::RoadGenerator::default();
                                    self.mode = Mode::SelectPos;
                                }
                            };
                            self.sel_node = None;
                            self.snapped_node = None;
                            self.update_ground_pos(self.ground_pos);
                            (None, empty_mesh)
                        }
                        _ => (None, None)
                    }
                }
            }
            (_, _) => (None, None),
        }
    }

    fn select_node(&mut self, snapped_node: network::NodeId) -> Option<network::RoadMesh> {
        let node = self.road_graph.get_node(snapped_node);

        self.road_generator =
            generator::RoadGenerator::new(node.pos, self.sel_road_type, Some(node.dir));
        self.road_generator.update_pos(self.ground_pos);
        let road_mesh = self.road_generator.get_mesh();

        self.sel_node = Some(snapped_node);
        self.snapped_node = None;
        self.mode = Mode::Build;

        road_mesh
    }

    fn build_road(&mut self) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        let (road_mesh, new_node) =
            self.road_graph
                .add_road(self.road_generator.clone(), None, None);
        let road_generator_mesh = self.select_node(new_node);
        (Some(road_mesh), road_generator_mesh)
    }

    fn check_snapping(&mut self, ground_pos: Vec3) -> Option<curves::DoubleSnapCurveCase> {
        // check for node within ground_pos
        // pass to road_generator to see if a road can be generated
        let possible_snap = self.road_graph.get_node_from_pos(ground_pos);
        match (possible_snap, self.sel_node) {
            (Some(possible_snap), None) => {
                // TODO check if lanes match the type we are connecting to
                self.snapped_node = Some(possible_snap);
                None
            }
            (Some(possible_snap), Some(sel_node)) => {
                use curves::DoubleSnapCurveCase::*;
                let start_node = self.road_graph.get_node(sel_node);
                let end_node = self.road_graph.get_node(possible_snap);
                if possible_snap == sel_node {
                    None
                } else {
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

        let snap_case = self.check_snapping(ground_pos);
        match self.mode {
            Mode::SelectPos => match self.snapped_node {
                Some(snapped_node) => {
                    let node = self.road_graph.get_node(snapped_node);
                    self.road_generator =
                        RoadGenerator::new(node.pos, self.sel_road_type, Some(node.dir));
                    self.road_generator.get_mesh()
                }
                None => empty_mesh,
            },
            Mode::SelectDir => {
                // for now we are not allowed to snap in dir mode
                self.road_generator.update_pos(ground_pos);
                self.road_generator.get_mesh()
            }
            Mode::Build => {
                match (snap_case, self.snapped_node) {
                    (Some(snap_case), Some(snapped_node)) => {
                        println!("building with double snap");
                        let snapped_node = self.road_graph.get_node(snapped_node);
                        self.road_generator.double_snap(snap_case, snapped_node.pos, snapped_node.dir);
                        self.road_generator.get_mesh()
                    }
                    _ => {
                        self.road_generator.update_pos(ground_pos);
                        self.road_generator.get_mesh()
                    }
                }
            }
        }
    }
}
