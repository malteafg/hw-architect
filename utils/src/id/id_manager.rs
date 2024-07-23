use num::{Bounded, FromPrimitive, Integer, ToPrimitive};
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeMarker;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct SegmentMarker;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TreeMarker;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct VehicleMarker;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]

/// TODO maybe ensure that Int is nonnegative?
pub struct Id<A, Int: Integer + Bounded + FromPrimitive + ToPrimitive> {
    id: Int,
    marker: PhantomData<A>,
}

/// All behaviour regarding id's should only be contained within this module so all functions
/// should be private
pub trait IdBehaviour {
    /// TODO benchmark how much this casting affects performance. If counter in IdManager was
    /// generic as well, there would probaly be no performance hit because no casting.
    fn from_usize(val: usize) -> Self;
    fn to_usize(&self) -> usize;
}

impl<A, Int: Integer + Bounded + FromPrimitive + ToPrimitive> IdBehaviour for Id<A, Int> {
    fn from_usize(val: usize) -> Self {
        Self {
            id: Int::from_usize(val).unwrap(),
            marker: PhantomData,
        }
    }

    fn to_usize(&self) -> usize {
        self.id.to_usize().unwrap()
    }
}

#[derive(Serialize, Deserialize)]
// pub struct IdManager<A = Id<M, Int>> {
pub struct IdManager<A: IdBehaviour> {
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
        let id = A::from_usize(self.counter);
        self.update_state();
        id
    }

    fn update_state(&mut self) {
        self.counter += 1;
    }
}
