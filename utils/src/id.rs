use serde::{Deserialize, Serialize};

/// Maybe make IdSize generic over integers using the num crate.
type IdSize = u16;

#[derive(Serialize, Deserialize)]
pub struct IdManager<A: PartialEq> {
    counter: IdSize,
    state: std::marker::PhantomData<A>,
}

impl<A: PartialEq> IdManager<A> {
    pub fn new() -> Self {
        IdManager {
            counter: 0,
            state: std::marker::PhantomData::<A>,
        }
    }

    fn update_state(&mut self) {
        self.counter += 1;
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(IdSize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentId(IdSize);

impl IdManager<SegmentId> {
    pub fn gen(&mut self) -> SegmentId {
        self.update_state();
        SegmentId(self.counter)
    }
}

impl IdManager<NodeId> {
    pub fn gen(&mut self) -> NodeId {
        self.update_state();
        NodeId(self.counter)
    }
}
