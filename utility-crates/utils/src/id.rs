use num::{Bounded, FromPrimitive, Integer};
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

pub const MAX_NUM_ID: usize = 65536;

pub type NodeId = Id<NodeMarker, u16>;
pub type SegmentId = Id<SegmentMarker, u16>;
pub type TreeId = Id<TreeMarker, u16>;
pub type VehicleId = Id<VehicleMarker, u32>;

/// It is dum to hash ids, make IdMap using Vec
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SegmentMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TreeMarker;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VehicleMarker;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
/// TODO maybe ensure that Int is nonnegative?
pub struct Id<A, Int: Integer + Bounded + FromPrimitive> {
    id: Int,
    marker: PhantomData<A>,
}

/// All behaviour regarding id's should only be contained within this module so all functions
/// should be private
pub trait IdBehaviour {
    /// TODO benchmark how much this casting affects performance. If counter in IdManager was
    /// generic as well, there would probaly be no performance because no casting.
    fn from_usize(val: usize) -> Self;
}

impl<A, Int: Integer + Bounded + FromPrimitive> IdBehaviour for Id<A, Int> {
    fn from_usize(val: usize) -> Self {
        Self {
            id: Int::from_usize(val).unwrap(),
            marker: PhantomData,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdManager<A: IdBehaviour> {
    // Maybe this should be larger than usize, ideally generic with the type from inside
    // IdBehaviour.
    counter: usize,
    state: PhantomData<A>,
}

impl<A: IdBehaviour> IdManager<A> {
    pub fn new() -> Self {
        IdManager {
            counter: 0,
            state: PhantomData::<A>,
        }
    }

    pub fn gen(&mut self) -> A {
        self.update_state();
        A::from_usize(self.counter)
    }

    fn update_state(&mut self) {
        self.counter += 1;
    }
}

// make IdSet using FixedBitSet and IdMap using Vec.
