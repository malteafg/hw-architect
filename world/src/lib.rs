//! This crate defines all the static data of the world, and how other crates are allowed to
//! manipulate this data such that the world is always in a valid configuration. Note that this
//! crate does not care about constraints such as road curvature, it only concerns itself with the
//! logical state of the world. For stuff like road curvature the tool crate is intended to enforce
//! it.
pub mod nature;
pub mod roads;

mod api;
pub use api::{RoadManipulator, TreeManipulator, WorldManipulator};

use nature::{Tree, TreeMap};
use roads::{LRoadBuilder, NodeType, RoadGraph, Side, SnapConfig};

use utils::id::{NodeId, SegmentId, TreeId};

use glam::Vec3;
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

impl api::WorldManipulator for World {}

impl api::RoadManipulator for World {
    fn get_node_from_pos(&self, pos: Vec3) -> Option<NodeId> {
        self.road_graph.get_node_from_pos(pos)
    }

    fn get_segment_from_pos(&self, pos: Vec3) -> Option<SegmentId> {
        self.road_graph.get_segment_from_pos(pos)
    }

    fn add_road(
        &mut self,
        road: LRoadBuilder,
        sel_node_type: NodeType,
    ) -> (Option<SnapConfig>, Vec<SegmentId>) {
        self.road_graph.add_road(road, sel_node_type)
    }

    fn remove_segment(&mut self, segment_id: SegmentId) -> bool {
        self.road_graph.remove_segment(segment_id)
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

impl api::TreeManipulator for World {
    fn add_tree(&mut self, tree: Tree, id: TreeId) {
        self.tree_map.add_tree(tree, id)
    }

    fn remove_tree(&mut self, pos: Vec3) {
        self.tree_map.remove_tree(pos)
    }

    fn get_trees(&self, id: TreeId) -> &Vec<Tree> {
        self.tree_map.get_trees(id)
    }
}
