mod id_manager;
mod id_map;

use id_manager::{Id, IdBehaviour};

pub use id_manager::IdManager;
pub use id_map::IdMap;

use serde::{Deserialize, Serialize};

/// TODO this should be deprecated and removed
pub const MAX_NUM_ID: usize = 65536;

pub type NodeId = Id<NodeMarker, u16>;
pub type SegmentId = Id<SegmentMarker, u16>;
pub type TreeId = Id<TreeMarker, u16>;
pub type VehicleId = Id<VehicleMarker, u32>;

/// It is dum to hash ids, make IdMap using Vec
/// Maybe all of these traits are dum to implement, they should just use the underlying integer to
/// do all these things.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SegmentMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TreeMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VehicleMarker;
