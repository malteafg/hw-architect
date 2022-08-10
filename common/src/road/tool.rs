use crate::input;
use crate::road::generator;
use crate::road::network;
use cgmath::*;

use super::curves;

#[derive(Debug, Clone, Copy)]
pub enum CurveType {
    Straight,
    Curved,
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    SelectPos,
    SelectDir {
        pos: Vector3<f32>,
    },
    Build {
        pos: Vector3<f32>,
        dir: Vector3<f32>,
    },
}

#[derive(Debug, Clone)]
struct SelectedRoad {
    no_lanes: u32,
    curve_type: CurveType,
}

pub struct ToolState {
    road_generator: Option<network::RoadGenerator>,
    road_graph: network::RoadGraph,
    selected_road: SelectedRoad,
    mode: Mode,
}

impl ToolState {
    pub fn new() -> Self {
        ToolState {
            road_generator: None,
            road_graph: network::RoadGraph::new(),
            selected_road: SelectedRoad {
                no_lanes: 3,
                curve_type: CurveType::Straight,
            },
            mode: Mode::SelectPos,
        }
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        let (action, pressed) = key;
        if !pressed {
            match action {
                CycleRoadType => match self.selected_road.curve_type {
                    CurveType::Straight => self.selected_road.curve_type = CurveType::Curved,
                    CurveType::Curved => self.selected_road.curve_type = CurveType::Straight,
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
        match (self.mode, event) {
            (Mode::SelectPos, input::MouseEvent::Left { .. }) => {
                let start_pos = ground_pos;
                let end_pos = ground_pos + Vector3::new(10.0, 0.0, 0.0);
                let start_node = network::NodeDescriptor::NEW(network::Node::new(
                    ground_pos,
                    Vector3::new(1.0, 0.0, 0.0),
                ));
                let end_node = network::NodeDescriptor::NEW(network::Node::new(
                    ground_pos + Vector3::new(10.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                ));
                let mesh = generator::generate_mesh(
                    start_pos,
                    end_pos,
                    self.selected_road.no_lanes,
                    self.selected_road.curve_type,
                    None,
                );
                self.road_generator = Some(network::RoadGenerator::new(
                    start_node,
                    start_pos,
                    end_node,
                    mesh.clone(),
                ));
                self.mode = Mode::SelectDir { pos: start_pos };
                (None, Some(mesh))
            }
            (Mode::SelectDir { pos }, input::MouseEvent::Left { .. }) => {
                match self.road_generator.clone() {
                    Some(road) => match self.selected_road.curve_type {
                        CurveType::Straight => {
                            let road_mesh = self.road_graph.add_road(road);
                            self.road_generator = None;
                            self.mode = Mode::SelectPos;
                            (Some(road_mesh), None)
                        }
                        CurveType::Curved => {
                            // self.mode = Mode::Build {};
                            // (Some(road_mesh), None)
                            self.mode = Mode::Build {
                                pos,
                                dir: ground_pos - pos,
                            };
                            (None, None)
                        }
                    },
                    None => {
                        //fail
                        (None, None)
                    }
                }
            }
            (Mode::Build { pos, dir }, input::MouseEvent::Left { .. }) => {
                match self.road_generator.clone() {
                    Some(road) => match self.selected_road.curve_type {
                        CurveType::Straight => {
                            let road_mesh = self.road_graph.add_road(road);
                            self.road_generator = None;
                            self.mode = Mode::SelectPos;
                            (Some(road_mesh), None)
                        }
                        CurveType::Curved => {
                            let road_mesh = self.road_graph.add_road(road);
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
            (Mode::SelectDir { pos }, input::MouseEvent::Right { .. }) => {
                self.road_generator = None;
                self.mode = Mode::SelectPos;
                (None, None)
            }
            (Mode::Build { pos, dir }, input::MouseEvent::Right { .. }) => {
                match self.selected_road.curve_type {
                    CurveType::Straight => {
                        self.road_generator = None;
                        self.mode = Mode::SelectPos;
                        (None, None)
                    }
                    CurveType::Curved => {
                        self.road_generator = None;
                        self.mode = Mode::SelectDir { pos };
                        (None, None)
                    }
                }
            }
            (Mode::SelectDir { .. }, input::MouseEvent::Moved { .. }) => {
                match &mut self.road_generator.clone() {
                    Some(road) => {
                        let end_pos = ground_pos;
                        let end_node = network::NodeDescriptor::NEW(network::Node::new(
                            end_pos,
                            Vector3::new(1.0, 0.0, 0.0),
                        ));
                        let mesh = generator::generate_mesh(
                            road.start_pos,
                            end_pos,
                            self.selected_road.no_lanes,
                            self.selected_road.curve_type,
                            None,
                        );
                        road.update(end_node, mesh.clone());
                        self.road_generator = Some(road.clone());
                        (None, Some(mesh))
                    }
                    None => (None, None),
                }
            }
            (Mode::Build { pos, dir }, input::MouseEvent::Moved { .. }) => {
                match &mut self.road_generator.clone() {
                    Some(road) => {
                        let (g_points, end_dir) = curves::circle(road.start_pos, dir, ground_pos);
                        //dbg!(g_points.clone());
                        let end_node =
                            network::NodeDescriptor::NEW(network::Node::new(pos, end_dir));
                        let mesh = generator::generate_mesh(
                            road.start_pos,
                            pos,
                            self.selected_road.no_lanes,
                            self.selected_road.curve_type,
                            Some(g_points),
                        );
                        road.update(end_node, mesh.clone());
                        self.road_generator = Some(road.clone());
                        (None, Some(mesh))
                    }
                    None => (None, None),
                }
            }
            (_, _) => (None, None),
        }
    }
}
