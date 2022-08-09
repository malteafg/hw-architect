use crate::input;
use crate::road::generator;
use crate::road::network;
use cgmath::*;

#[derive(Debug, Clone)]
enum CurveType {
    STRAIGHT,
    CURVED,
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
}

impl ToolState {
    pub fn new() -> Self {
        ToolState {
            road_generator: None,
            road_graph: network::RoadGraph::new(),
            selected_road: SelectedRoad {
                no_lanes: 3,
                curve_type: CurveType::STRAIGHT,
            },
        }
    }

    pub fn process_keyboard(&mut self, key: input::KeyAction) {
        use input::Action::*;
        let (action, pressed) = key;
        if !pressed {
            match action {
                CycleRoadType => match self.selected_road.curve_type {
                    CurveType::STRAIGHT => self.selected_road.curve_type = CurveType::CURVED,
                    CurveType::CURVED => self.selected_road.curve_type = CurveType::STRAIGHT,
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
        match event {
            input::MouseEvent::Left { .. } => match self.road_generator.clone() {
                Some(road) => {
                    let road_mesh = self.road_graph.add_road(road);
                    self.road_generator = None;
                    (Some(road_mesh), None)
                }
                None => {
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
                    let mesh =
                        generator::generate_mesh(start_pos, end_pos, self.selected_road.no_lanes);
                    self.road_generator = Some(network::RoadGenerator::new(
                        start_node,
                        start_pos,
                        end_node,
                        mesh.clone(),
                    ));
                    (None, Some(mesh))
                }
            },
            input::MouseEvent::Moved { .. } => match &mut self.road_generator.clone() {
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
                    );

                    road.update(end_node, mesh.clone());
                    self.road_generator = Some(road.clone());
                    (None, Some(mesh))
                }
                None => (None, None),
            },
            _ => (None, None),
        }
    }
}
