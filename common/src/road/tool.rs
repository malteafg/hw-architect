use crate::input;
use crate::road::generator;
use crate::road::network;
use cgmath::*;

pub struct ToolState {
    road_generator: Option<network::RoadGenerator>,
    road_graph: network::RoadGraph,
}

impl ToolState {
    pub fn new() -> Self {
        ToolState {
            road_generator: None,
            road_graph: network::RoadGraph::new(),
        }
    }

    pub fn mouse_input(
        &mut self,
        event: input::MouseEvent,
        ground_pos: Vector3<f32>,
    ) -> (Option<network::RoadMesh>, Option<network::RoadMesh>) {
        match event {
            input::MouseEvent::Left { pos, .. } => match self.road_generator.clone() {
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
                    let mesh = generator::generate_mesh(start_pos, end_pos);
                    self.road_generator = Some(network::RoadGenerator::new(
                        start_node,
                        start_pos,
                        end_node,
                        mesh.clone(),
                    ));
                    (None, Some(mesh))
                }
            },
            input::MouseEvent::Moved { pos, .. } => match &mut self.road_generator.clone() {
                Some(road) => {
                    let end_pos = ground_pos;
                    let end_node = network::NodeDescriptor::NEW(network::Node::new(
                        end_pos,
                        Vector3::new(1.0, 0.0, 0.0),
                    ));
                    let mesh = generator::generate_mesh(road.start_pos, end_pos);

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
