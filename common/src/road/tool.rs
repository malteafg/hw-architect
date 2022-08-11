use crate::input;
use super::generator;
use super::network;
use cgmath::*;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    SelectPos,
    SelectDir,
    Build,
    // SelectDir {
    //     pos: Vector3<f32>,
    // },
    // Build {
    //     pos: Vector3<f32>,
    //     dir: Vector3<f32>,
    // },
}

pub struct ToolState {
    road_generator: Option<generator::RoadGenerator>,
    road_graph: network::RoadGraph,
    selected_road: network::RoadType,
    mode: Mode,
}

impl ToolState {
    pub fn new() -> Self {
        ToolState {
            road_generator: None,
            road_graph: network::RoadGraph::new(),
            selected_road: network::RoadType {
                no_lanes: 3,
                curve_type: network::CurveType::Straight,
            },
            mode: Mode::SelectPos,
        }
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        use network::CurveType::*;
        let (action, pressed) = key;
        if !pressed {
            match action {
                CycleRoadType => match self.selected_road.curve_type {
                    Straight => self.selected_road.curve_type = Curved,
                    Curved => self.selected_road.curve_type = Straight,
                },
                OneLane => self.selected_road.no_lanes = 1,
                TwoLane => self.selected_road.no_lanes = 2,
                ThreeLane => self.selected_road.no_lanes = 3,
                FourLane => self.selected_road.no_lanes = 4,
                FiveLane => self.selected_road.no_lanes = 5,
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
        
        // returned when road_generator is set to None
        let empty_mesh = Some(generator::empty_mesh());

        match (self.mode, event) {
            (Mode::SelectPos, MouseEvent::Left { .. }) => {
                // if snapped node add Some(dir) instead of None
                let road_generator = RoadGenerator::new(ground_pos, self.selected_road, None);
                let road_mesh = road_generator.get_mesh();
                self.road_generator = Some(road_generator);
                self.mode = Mode::SelectDir;
                (None, Some(road_mesh))
            }
            (Mode::SelectDir, MouseEvent::Left { .. }) => {
                match self.road_generator.as_mut() {
                    Some(road_generator) => match self.selected_road.curve_type {
                        CurveType::Straight => {
                            let road_mesh = self.road_graph.add_road(road_generator.clone(), None, None);
                            self.road_generator = None;
                            self.mode = Mode::SelectPos;
                            (Some(road_mesh), empty_mesh)
                        }
                        CurveType::Curved => {
                            road_generator.lock();
                            road_generator.update_pos(ground_pos);
                            self.mode = Mode::Build; 
                            (None, Some(road_generator.get_mesh()))
                        }
                    },
                    None => {
                        //fail
                        (None, None)
                    }
                }
            }
            (Mode::Build, MouseEvent::Left { .. }) => {
                match self.road_generator.clone() {
                    Some(road_generator) => match self.selected_road.curve_type {
                        CurveType::Straight => {
                            let road_mesh = self.road_graph.add_road(road_generator, None, None);
                            self.road_generator = None;
                            self.mode = Mode::SelectPos;
                            (Some(road_mesh), None)
                        }
                        CurveType::Curved => {
                            let road_mesh = self.road_graph.add_road(road_generator, None, None);
                            self.road_generator = None;
                            self.mode = Mode::SelectPos;
                            (Some(road_mesh), None)
                        }
                    },
                    None => {
                        //fail
                        (None, None)
                    }
                }
            }
            (Mode::SelectDir, MouseEvent::Right { .. }) => {
                self.road_generator = None;
                self.mode = Mode::SelectPos;
                (None, empty_mesh)
            }
            (Mode::Build, MouseEvent::Right { .. }) => {
                match self.selected_road.curve_type {
                    CurveType::Straight => {
                        self.road_generator = None;
                        self.mode = Mode::SelectPos;
                        (None, empty_mesh)
                    }
                    CurveType::Curved => {
                        // TODO should return to dir mode
                        self.road_generator = None;
                        self.mode = Mode::SelectPos;
                        (None, empty_mesh)
                    }
                }
            }
            (_, _) => (None, None),
        }
    }

    pub fn update_ground_pos(&mut self, ground_pos: Vector3<f32>) -> Option<network::RoadMesh> {
        // do snapping stuff
        match self.road_generator.as_mut() {
            Some(road_generator) => {
                road_generator.update_pos(ground_pos);
                Some(road_generator.get_mesh())
            }
            None => None,
        }
    }
}
