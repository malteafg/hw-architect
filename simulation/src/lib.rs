use serde::{Deserialize, Serialize};

pub mod curves;
mod road_network;

pub use road_network::*;

#[derive(Serialize, Deserialize, Default)]
pub struct World {
    road_graph: RoadGraph,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}

pub trait RoadManipulator {
    fn get_road_graph(&self) -> &RoadGraph;
    fn mut_road_graph(&mut self) -> &mut RoadGraph;
}

impl RoadManipulator for World {
    fn get_road_graph(&self) -> &RoadGraph {
        &self.road_graph
    }

    fn mut_road_graph(&mut self) -> &mut RoadGraph {
        &mut self.road_graph
    }
}
