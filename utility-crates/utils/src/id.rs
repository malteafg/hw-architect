mod id_manager;
mod id_map;
mod id_set;

use id_manager::{Id, IdBehaviour};

pub use id_manager::IdManager;
pub use id_map::IdMap;
pub use id_set::IdSet;

pub type NodeId = Id<id_manager::NodeMarker, u16>;
pub type SegmentId = Id<id_manager::SegmentMarker, u16>;
pub type TreeId = Id<id_manager::TreeMarker, u16>;
pub type VehicleId = Id<id_manager::VehicleMarker, u32>;
