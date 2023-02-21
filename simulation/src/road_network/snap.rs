use super::NodeType;
use super::Side;
use glam::Vec3;
use utils::id::NodeId;

/// Represents a continuous range of lane indexes. As an example, SnapRange might contain 2,3,4
/// representing lanes 2,3 and 4. Lane indexes can also be negative for use in a {`SnapConfig`}
/// where the node is expanded.
///
/// # INVARIANTS
/// A {`SnapRange`} must never be empty.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapRange(Vec<i8>);

/// Represents a legal way (configuration) of snapping to a node.
#[derive(Debug, Clone)]
pub struct SnapConfig {
    node_id: NodeId,
    node_type: NodeType,
    pos: Vec3,
    dir: Vec3,
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

impl SnapRange {
    // pub fn empty() -> Self {
    //     SnapRange(vec![])
    // }

    // pub fn from_vec(snap_range: Vec<i8>) -> Self {
    //     SnapRange(snap_range)
    // }

    pub fn create(start: i8, end: i8) -> Self {
        let mut snap_range = vec![];
        for i in 0..end - start {
            snap_range.push(i as i8 + start)
        }
        SnapRange(snap_range)
    }

    /// Removes indexes in the snap range that are smaller than 0 and larger than `end`.
    pub fn reduce_size(&self, end: u8) -> Self {
        let mut snap_range = vec![];
        for i in self.iter() {
            if *i >= 0 && *i < end as i8 {
                snap_range.push(*i)
            }
        }
        SnapRange(snap_range)
    }

    pub fn largest(&self) -> i8 {
        self[self.len() - 1]
    }

    pub fn smallest(&self) -> i8 {
        self[0]
    }

    pub fn get_no_negatives(&self) -> u8 {
        let result = 0;
        for i in self.iter() {
            if *i < 0 {
                result += 1;
            } else {
                break;
            }
        }
        result
    }

    pub fn shift(&mut self, amount: i8) {
        self.iter_mut().for_each(|i| *i = *i + amount)
    }

    pub fn trim(&mut self, amount: u8) {
        for i in 0..amount {
            self.pop();
        }
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
    pub(super) fn new(
        node_id: NodeId,
        node_type: NodeType,
        pos: Vec3,
        dir: Vec3,
        snap_range: SnapRange,
        side: Side,
    ) -> Self {
        Self {
            node_id,
            node_type,
            pos,
            dir,
            snap_range,
            side,
        }
    }

    pub fn get_id(&self) -> NodeId {
        self.node_id
    }

    pub fn get_node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn get_dir(&self) -> Vec3 {
        self.dir
    }

    pub(super) fn get_snap_range(&self) -> &SnapRange {
        &self.snap_range
    }

    // pub fn is_reverse(&self) -> bool {
    //     self.reverse
    // }
}
