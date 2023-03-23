//! This crate defines all the static data of the world, and how other crates are allowed to
//! manipulate this data such that the world is always in a valid configuration. Note that this
//! crate does not care about constraints such as road curvature, it only concerns itself with the
//! logical state of the world. For stuff like road curvature the tool crate is intended to enforce
//! it.
pub mod nature;
pub mod roads;

use nature::TreeMap;
use roads::RoadGraph;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct World {
    road_graph: RoadGraph,
    tree_map: TreeMap,
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

pub trait TreeManipulator {
    fn get_tree_map(&self) -> &TreeMap;
    fn mut_tree_map(&mut self) -> &mut TreeMap;
}

impl RoadManipulator for World {
    fn get_road_graph(&self) -> &RoadGraph {
        &self.road_graph
    }

    fn mut_road_graph(&mut self) -> &mut RoadGraph {
        &mut self.road_graph
    }
}

impl TreeManipulator for World {
    fn get_tree_map(&self) -> &TreeMap {
        &self.tree_map
    }

    fn mut_tree_map(&mut self) -> &mut TreeMap {
        &mut self.tree_map
    }
}
