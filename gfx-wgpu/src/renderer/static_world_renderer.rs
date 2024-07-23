use gfx_api::GSegment;
use utils::id::{IdMap, SegmentId, TreeId};
use utils::math::Loc;

pub struct StaticWorldState {
    segments: IdMap<SegmentId, GSegment>,
    trees: IdMap<TreeId, Loc>,
}
