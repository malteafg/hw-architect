mod builder;
mod road_builder;
mod road_type;
mod snap;
mod tree;

pub use builder::*;
pub use road_builder::*;
pub use road_type::*;
pub use snap::*;
pub use tree::*;

use glam::Vec3;
use utils::id::{NodeId, SegmentId, TreeId};

pub trait WorldManipulator: RoadManipulator + TreeManipulator + IdGetter {}

pub trait RoadManipulator {
    /// The node_type parameter is temporary until implementation of transition segments.
    fn add_road(
        &mut self,
        road: LRoadBuilder,
        sel_node_type: NodeType,
    ) -> (Option<SnapConfig>, Vec<SegmentId>);

    /// The return bool signals whether the segment was allowed to be removed or not.
    fn remove_segment(&mut self, segment_id: SegmentId) -> bool;

    /// Returns a list of node id's that have an open slot for the selected road type to snap to
    /// together with that nodes pos and dir.
    /// If side parameter is set to {`None`}, then no direction is checked when matching nodes.
    fn get_possible_snap_nodes(
        &self,
        side: Option<Side>,
        node_type: NodeType,
    ) -> Vec<(NodeId, Vec3, Vec3)>;

    /// If no node is within range of pos, then this function returns {`None`}. Otherwise it
    /// returns the closest node to pos, and all its possible {`SnapConfig`}'s.
    fn get_snap_configs_closest_node(
        &self,
        ground_pos: Vec3,
        node_type: NodeType,
    ) -> Option<(NodeId, Vec<SnapConfig>)>;

    #[cfg(debug_assertions)]
    fn debug_node(&self, id: NodeId);
    #[cfg(debug_assertions)]
    fn debug_segment(&self, id: SegmentId);
}

pub trait TreeManipulator {
    fn add_tree(&mut self, tree: Tree, model_id: u128) -> TreeId;
    /// Returns the model_id of the tree that has been removed.
    fn remove_tree(&mut self, tree_id: TreeId) -> u128;
    fn get_tree_pos(&self, id: TreeId) -> Vec3;
}

pub trait IdGetter {
    /// Returns the first node found that contains the given position.
    fn get_node_from_pos(&self, pos: Vec3) -> Option<NodeId>;
    /// Returns the first segment found that contains the given position.
    fn get_segment_from_pos(&self, pos: Vec3) -> Option<SegmentId>;
    /// Returns the first tree found that contains the given position.
    fn get_tree_from_pos(&self, pos: Vec3) -> Option<TreeId>;
}
