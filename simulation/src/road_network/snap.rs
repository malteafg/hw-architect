use glam::Vec3;
use utils::id::NodeId;

/// Represents a continuous range of lane indexes. As an example, SnapRange might contain 2,3,4
/// representing lanes 2,3 and 4. Start index of lanes is 0.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapRange(Vec<i8>);

/// Represents a legal way (configuration) of snapping to a node.
#[derive(Debug, Clone)]
pub struct SnapConfig {
    node_id: NodeId,
    pos: Vec3,
    dir: Vec3,
    snap_range: SnapRange,
    // Reverse means that outgoing lanes exist, and incoming does not
    reverse: bool,
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
    pub fn empty() -> Self {
        SnapRange(vec![])
    }

    pub fn from_vec(snap_range: Vec<i8>) -> Self {
        SnapRange(snap_range)
    }

    pub fn create(start: i8, end: i8) -> Self {
        let mut snap_range = vec![];
        for i in 0..end - start {
            snap_range.push(i as i8 + start)
        }
        SnapRange(snap_range)
    }

    pub fn reduce_size(&self, end: u8) -> Self {
        let mut snap_range = vec![];
        for i in self.iter() {
            if *i >= 0 && *i < end as i8 {
                snap_range.push(*i)
            }
        }
        SnapRange(snap_range)
    }
}

// #################################################################################################
// Implementation of SnapConfig
// #################################################################################################
impl PartialEq for SnapConfig {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.snap_range == other.snap_range
            && self.reverse == other.reverse
    }
}

impl SnapConfig {
    pub(super) fn new(
        node_id: NodeId,
        pos: Vec3,
        dir: Vec3,
        snap_range: SnapRange,
        reverse: bool,
    ) -> Self {
        Self {
            node_id,
            pos,
            dir,
            snap_range,
            reverse,
        }
    }

    pub fn get_id(&self) -> NodeId {
        self.node_id
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

    pub fn is_reverse(&self) -> bool {
        self.reverse
    }
}
