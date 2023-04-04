use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

/// Maybe make IdSize generic over integers using the num crate.
type IdSize = u16;

pub const MAX_NUM_ID: usize = 65536;

pub type NodeId = Id<NodeMarker>;
pub type SegmentId = Id<SegmentMarker>;
pub type TreeId = Id<TreeMarker>;
pub type VehicleId = Id<VehicleMarker>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SegmentMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TreeMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VehicleMarker;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Id<A> {
    id: IdSize,
    marker: PhantomData<A>,
}

impl<A> Id<A> {
    pub fn internal(&self) -> IdSize {
        self.id
    }

    pub fn usize(&self) -> usize {
        self.id as usize
    }
}

impl<A> FromIdSize for Id<A> {
    fn from_id_size(id_size: IdSize) -> Self {
        Self {
            id: id_size,
            marker: PhantomData,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdManager<A: PartialEq + FromIdSize> {
    counter: IdSize,
    state: PhantomData<A>,
}

pub trait FromIdSize {
    fn from_id_size(id_size: IdSize) -> Self;
}

impl<Id: PartialEq + FromIdSize> IdManager<Id> {
    pub fn new() -> Self {
        IdManager {
            counter: 0,
            state: PhantomData::<Id>,
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
