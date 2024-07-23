use utils::id::{IdMap, SegmentId, TreeId};
use utils::math::Loc;

use crate::GSegment;

pub enum Cmd {
    Add(Add),
    Remove(Remove),
}

pub enum Add {
    Segments(IdMap<SegmentId, GSegment>),
    Trees(u128, IdMap<TreeId, Loc>),
}

pub enum Remove {
    Segments(Vec<SegmentId>),
    Trees(Vec<TreeId>),
}
