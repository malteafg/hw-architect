use super::NodeType;
use super::Side;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use utils::id::NodeId;
use utils::math::{DirXZ, Loc};

/// Represents a continuous range of lane indexes. As an example, SnapRange might contain 2,3,4
/// representing lanes 2,3 and 4. Lane indexes can also be negative for use in a {`SnapConfig`}
/// where the node is expanded.
///
/// # INVARIANTS
/// A {`SnapRange`} must never be empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapRange(Vec<i8>);

/// Represents a legal way (configuration) of snapping to a node.
#[derive(Debug, Clone)]
pub struct SnapConfig {
    node_id: NodeId,
    node_type: NodeType,
    loc: Loc,
    snap_range: SnapRange,
    side: Side,
}

// #################################################################################################
// Implementation of SnapRange
// #################################################################################################
impl core::ops::Deref for SnapRange {
    type Target = Vec<i8>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for SnapRange {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

impl From<SnapConfig> for Loc {
    fn from(value: SnapConfig) -> Self {
        value.loc
    }
}

impl From<&SnapConfig> for Loc {
    fn from(value: &SnapConfig) -> Self {
        value.loc
    }
}

impl SnapRange {
    /// Returns a new snap range with `size` where indexes start at 0.
    /// Should only be called by world. How to enforce?
    pub fn new(size: u8) -> Self {
        let mut snap_range = vec![];
        (0..size).for_each(|i| snap_range.push(i as i8));
        SnapRange(snap_range)
    }

    pub fn largest(&self) -> i8 {
        self[self.len() - 1]
    }

    pub fn smallest(&self) -> i8 {
        self[0]
    }

    pub fn no_negatives(&self) -> u8 {
        let mut result = 0;
        for i in self.iter() {
            if *i < 0 {
                result += 1;
            } else {
                break;
            }
        }
        result
    }

    pub fn contains(&self, snap: i8) -> bool {
        snap >= self.smallest() && snap <= self.largest()
    }

    pub fn shift(&mut self, amount: i8) {
        self.iter_mut().for_each(|i| *i = *i + amount)
    }
}

// #################################################################################################
// Implementation of SnapConfig
// #################################################################################################
impl PartialEq for SnapConfig {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.snap_range == other.snap_range
            && self.side == other.side
    }
}

impl SnapConfig {
    /// Should only be called by world. How to enforce?
    pub fn new(
        node_id: NodeId,
        node_type: NodeType,
        loc: Loc,
        snap_range: SnapRange,
        side: Side,
    ) -> Self {
        Self {
            node_id,
            node_type,
            loc,
            snap_range,
            side,
        }
    }

    pub fn id(&self) -> NodeId {
        self.node_id
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn pos(&self) -> Vec3 {
        self.loc.pos
    }

    pub fn dir(&self) -> DirXZ {
        self.loc.dir
    }

    pub fn pos_and_dir(&self) -> (Vec3, DirXZ) {
        (self.loc.pos, self.loc.dir)
    }

    pub fn get_snap_range(&self) -> &SnapRange {
        &self.snap_range
    }

    pub fn consume_snap_range(self) -> SnapRange {
        self.snap_range
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn is_reverse(&self) -> bool {
        self.side == Side::In
    }
}
