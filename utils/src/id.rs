#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(pub u32);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SegmentId(pub u32);

// Write something that manages id's, and make the value inside Id's private again
// probably use a trait

