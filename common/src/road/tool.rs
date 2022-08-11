use crate::input;
use super::generator;
use super::network;
use cgmath::*;

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
        ground_pos: Vector3<f32>,
    ) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        use network::CurveType;
        use input::MouseEvent;
        use generator::RoadGenerator;

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
                    let node = self.road_graph.get_node(snapped_node);

                    self.road_generator = RoadGenerator::new(node.pos, self.sel_road_type, Some(node.dir));
                    self.road_generator.update_pos(ground_pos);
                    let road_mesh = self.road_generator.get_mesh();

                    self.sel_node = Some(snapped_node);
                    self.snapped_node = None;
                    self.mode = Mode::Build;
                    (None, road_mesh)
                }
                None => {
                    self.road_generator = RoadGenerator::new(ground_pos, self.sel_road_type, None);
                    let road_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::SelectDir;
                    (None, road_mesh)
                }
            }
            (Mode::SelectDir, MouseEvent::LeftClick) => match self.sel_road_type.curve_type {
                CurveType::Straight => {
                    let road_mesh = self.build_road();
                    (Some(road_mesh), empty_mesh)
                }
                CurveType::Curved => {
                    self.road_generator.lock();
                    self.road_generator.update_pos(ground_pos);
                    let road_mesh = self.road_generator.get_mesh();

                    self.mode = Mode::Build; 
                    (None, road_mesh)
                }
            }
            (Mode::Build, MouseEvent::LeftClick) => match self.sel_road_type.curve_type {
                CurveType::Straight => {
                    let road_mesh = self.build_road();
                    (Some(road_mesh), None)
                }
                CurveType::Curved => {
                    let road_mesh = self.build_road();
                    (Some(road_mesh), None)
                }
            }
            (Mode::SelectDir, MouseEvent::RightClick) => {
                self.road_generator = generator::RoadGenerator::default();
                self.sel_node = None;
                self.snapped_node = None;
                self.update_ground_pos(ground_pos);
                self.mode = Mode::SelectPos;
                (None, empty_mesh)
            }
            (Mode::Build, MouseEvent::RightClick) => {
                match self.sel_road_type.curve_type {
                    CurveType::Straight => {
                        self.road_generator = generator::RoadGenerator::default();
                        self.sel_node = None;
                        self.snapped_node = None;
                        self.update_ground_pos(ground_pos);
                        self.mode = Mode::SelectPos;
                        (None, empty_mesh)
                    }
                    CurveType::Curved => {
                        // TODO should return to dir mode
                        self.road_generator = generator::RoadGenerator::default();
                        self.sel_node = None;
                        self.snapped_node = None;
                        self.update_ground_pos(ground_pos);
                        self.mode = Mode::SelectPos;
                        (None, empty_mesh)
                    }
                }
            }
            (_, _) => (None, None),
        }
    }

    fn build_road(&mut self) -> network::RoadMesh {
        let road_mesh = self.road_graph.add_road(self.road_generator.clone(), None, None);
        self.road_generator = generator::RoadGenerator::default();
        self.sel_node = None;
        self.mode = Mode::SelectPos;
        road_mesh
    }

    fn check_snapping(&mut self, ground_pos: Vector3<f32>) {
        // check for node within ground_pos
        // pass to road_generator to see if a road can be generated
        let possible_snap = self.road_graph.get_node_from_pos(ground_pos);
        match (possible_snap, self.sel_node) {
            (Some(possible_snap), None) => {
                // TODO check if lanes match the type we are connecting to
                self.snapped_node = Some(possible_snap);
            }
            (Some(possible_snap), Some(sel_node)) => {
                // TODO check if we can connect to the road
                // temp
                self.snapped_node = None;
            }
            _ => {
                self.snapped_node = None;
            }
        };
    }

    pub fn update_ground_pos(&mut self, ground_pos: Vector3<f32>) -> Option<network::RoadMesh> {
        use generator::RoadGenerator;

        // returned when road_generator is set to None
        let empty_mesh = Some(generator::empty_mesh());

        self.check_snapping(ground_pos);
        match self.mode {
            Mode::SelectPos => match self.snapped_node {
                Some(snapped_node) => {
                    let node = self.road_graph.get_node(snapped_node);
                    self.road_generator = RoadGenerator::new(node.pos, self.sel_road_type, Some(node.dir));
                    self.road_generator.get_mesh()
                }
                None => empty_mesh,
            },
            Mode::SelectDir => {
                // for now we are not allowed to snap in dir mode
                self.road_generator.update_pos(ground_pos);
                self.road_generator.get_mesh()
            },
            Mode::Build => {
                // for now we are not allowed to snap in build mode
                self.road_generator.update_pos(ground_pos);
                self.road_generator.get_mesh()
            },
        }
    }
}
