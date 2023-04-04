//! This crate defines all the static data of the world, and how other crates are allowed to
//! manipulate this data such that the world is always in a valid configuration. Note that this
//! crate does not care about constraints such as road curvature, it only concerns itself with the
//! logical state of the world. For stuff like road curvature the tool crate is intended to enforce
//! it.
mod nature;
mod roads;
mod simulation;

use world_api::{
    IdGetter, RoadManipulator, SimController, SimData, TreeManipulator, WorldManipulator,
};
use world_api::{LRoadBuilder, NodeType, Side, SnapConfig, Tree};

use nature::Trees;
use roads::RoadGraph;
use simulation::SimHandler;

use utils::id::{NodeId, SegmentId, TreeId};

use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct World {
    road_graph: RoadGraph,
    sim_handler: SimHandler,
    trees: Trees,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}

impl WorldManipulator for World {}

impl RoadManipulator for World {
    fn add_road(
        &mut self,
        road: LRoadBuilder,
        sel_node_type: NodeType,
    ) -> (Option<SnapConfig>, Vec<SegmentId>) {
        let (snap, segments) = self.road_graph.add_road(road, sel_node_type);

        for s_id in segments.iter() {
            let no_lane_paths = self.road_graph.get_segment(*s_id).no_lane_paths();
            self.sim_handler.add_segment(*s_id, no_lane_paths);
        }

        (snap, segments)
    }

    fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        let result = self.road_graph.remove_segment(segment_id);
        if result {
            self.sim_handler.remove_segment(segment_id);
        }
        result
    }

    fn get_possible_snap_nodes(
        &self,
        side: Option<Side>,
        node_type: NodeType,
    ) -> Vec<(NodeId, Vec3, Vec3)> {
        self.road_graph.get_possible_snap_nodes(side, node_type)
    }

    fn get_snap_configs_closest_node(
        &self,
        ground_pos: Vec3,
        node_type: NodeType,
    ) -> Option<(NodeId, Vec<SnapConfig>)> {
        self.road_graph
            .get_snap_configs_closest_node(ground_pos, node_type)
    }

    fn debug_node(&self, id: NodeId) {
        self.road_graph.debug_node(id)
    }

    fn debug_segment(&self, id: SegmentId) {
        self.road_graph.debug_segment(id)
    }
}

impl TreeManipulator for World {
    fn add_tree(&mut self, tree: Tree, model_id: u128) -> TreeId {
        self.trees.add_tree(tree, model_id)
    }

    fn remove_tree(&mut self, tree_id: TreeId) -> u128 {
        self.trees.remove_tree(tree_id)
    }

    fn get_tree_pos(&self, id: TreeId) -> Vec3 {
        self.trees.get_tree_pos(id)
    }
}

impl IdGetter for World {
    fn get_node_from_pos(&self, pos: Vec3) -> Option<NodeId> {
        self.road_graph.get_node_from_pos(pos)
    }

    fn get_segment_from_pos(&self, pos: Vec3) -> Option<SegmentId> {
        self.road_graph.get_segment_from_pos(pos)
    }

    fn get_tree_from_pos(&self, pos: Vec3) -> Option<TreeId> {
        self.trees.get_tree_from_pos(pos)
    }
}

impl SimController for World {
    fn pause(&mut self) {}
    fn unpause(&mut self) {}
}

impl SimData for World {
    fn get_cars(&self) -> Vec<([f32; 3], f32)> {
        vec![]
    }
}
