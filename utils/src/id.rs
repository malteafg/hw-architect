use serde::{Deserialize, Serialize};

/// Maybe make IdSize generic over integers using the num crate.
type IdSize = u16;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(IdSize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentId(IdSize);

/// TreeId's are used for each model or type of tree, and not an id for each tree.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeId(IdSize);

#[derive(Serialize, Deserialize)]
pub struct IdManager<A: PartialEq + FromIdSize> {
    counter: IdSize,
    state: std::marker::PhantomData<A>,
}

pub trait FromIdSize {
    fn from_id_size(id_size: IdSize) -> Self;
}

impl<Id: PartialEq + FromIdSize> IdManager<Id> {
    pub fn new() -> Self {
        IdManager {
            counter: 0,
            state: std::marker::PhantomData::<Id>,
        }
    }

    fn update_state(&mut self) {
        self.counter += 1;
    }

    pub fn gen(&mut self) -> Id {
        self.update_state();
        Id::from_id_size(self.counter)
    }
}

impl FromIdSize for NodeId {
    fn from_id_size(id_size: IdSize) -> Self {
        NodeId(id_size)
    }
}

impl FromIdSize for SegmentId {
    fn from_id_size(id_size: IdSize) -> Self {
        SegmentId(id_size)
    }
}

impl FromIdSize for TreeId {
    fn from_id_size(id_size: IdSize) -> Self {
        TreeId(id_size)
    }
}
