use utils::id::{NodeId, SegmentId, TreeId};

use glam::Vec3;

use crate::{
    nature::Tree,
    roads::{LRoadBuilder, NodeType, Side, SnapConfig},
};

pub trait RoadManipulator {
    fn get_node_pos(&self, node: NodeId) -> Vec3;
    fn get_node_dir(&self, node: NodeId) -> Vec3;
    fn get_node_positions(&self) -> Vec<Vec3>;
    fn get_segment_inside(&self, pos: Vec3) -> Option<SegmentId>;

    /// The node_type parameter is temporary until implementation of transition segments.
    fn add_road(
        &mut self,
        road: LRoadBuilder,
        sel_node_type: NodeType,
    ) -> (Option<SnapConfig>, Vec<SegmentId>);
    /// The return bool signals whether the segment was allowed to be removed or not.
    fn remove_segment(&mut self, segment_id: SegmentId) -> bool;

    /// Returns a list of node id's that have an open slot for the selected road type to snap to.
    /// If reverse parameter is set to {`None`}, then no direction is checked when matching nodes.
    fn get_possible_snap_nodes(&self, side: Option<Side>, node_type: NodeType) -> Vec<NodeId>;
    /// If no node is within range of pos, then this function returns {`None`}. Otherwise it
    /// returns the closest node to pos, and all its possible {`SnapConfig`}'s.
    fn get_snap_configs_closest_node(
        &self,
        ground_pos: Vec3,
        node_type: NodeType,
    ) -> Option<(NodeId, Vec<SnapConfig>)>;

    fn debug_node_from_pos(&self, pos: Vec3);
    fn debug_segment_from_pos(&self, pos: Vec3);
}

pub trait TreeManipulator {
    fn add_tree(&mut self, tree: Tree, id: TreeId);
    fn remove_tree(&mut self, pos: Vec3);
    fn get_trees(&self, id: TreeId) -> &Vec<Tree>;
}
