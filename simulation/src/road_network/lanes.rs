use super::snap::SnapRange;
use std::collections::VecDeque;
use utils::id::SegmentId;

/// Defines an order of {`SegmentId`}'s that an {`LNode`} uses to keep track of which segments are
/// incoming and outgoing to and from itself. Thus an {`LNode`} has two {`LaneMap`}s. None is used
/// to signal that there is no segment present at this lane
#[derive(Debug, Clone, PartialEq)]
pub(super) struct LaneMap(VecDeque<Option<SegmentId>>);

impl core::ops::Deref for LaneMap {
    type Target = VecDeque<Option<SegmentId>>;

    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for LaneMap {
    fn deref_mut(self: &'_ mut Self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

impl LaneMap {
    fn from_vec(lane_map: VecDeque<Option<SegmentId>>) -> Self {
        LaneMap(lane_map)
    }

    pub(super) fn create(no_lanes: u8, id: Option<SegmentId>) -> Self {
        let mut vec = VecDeque::new();
        for _ in 0..no_lanes {
            vec.push_back(id)
        }
        LaneMap::from_vec(vec)
    }

    pub(super) fn contains_none(&self) -> bool {
        let mut contains_none = false;
        for seg in self.iter() {
            if seg.is_none() {
                contains_none = true;
            }
        }
        contains_none
    }

    pub(super) fn contains_some(&self) -> bool {
        let mut contains_some = false;
        for seg in self.iter() {
            if seg.is_some() {
                contains_some = true;
            }
        }
        contains_some
    }

    fn contains_some_in_range(&self, range: SnapRange) -> bool {
        let mut contains_some = false;
        for i in range.iter() {
            if self[*i as usize].is_some() {
                contains_some = true;
            }
        }
        contains_some
    }

    // fn contains_different_somes(&self) -> bool {
    //     let mut temp: Option<SegmentId> = None;
    //     for ele in self.iter() {
    //         match (temp, ele) {
    //             (Some(a), Some(b)) => {
    //                 if a != *b {
    //                     return true;
    //                 }
    //             }
    //             (None, _) => temp = *ele,
    //             _ => {}
    //         }
    //     }
    //     false
    // }

    pub(super) fn is_same(&self) -> bool {
        let temp = self[0];
        for seg in self.iter() {
            if *seg != temp {
                return false;
            }
        }
        true
    }

    fn get_range_of_segment(&self, segment_id: SegmentId) -> SnapRange {
        let mut snap_range = vec![];
        for (i, id) in self.iter().enumerate() {
            if let Some(id) = id {
                if *id == segment_id {
                    snap_range.push(i as i8);
                }
            }
        }
        SnapRange::from_vec(snap_range)
    }

    pub(super) fn is_middle_segment(&self, segment_id: SegmentId) -> bool {
        let segment_range = self.get_range_of_segment(segment_id);
        let bottom_range = SnapRange::create(0, segment_range[0]);
        let top_range = SnapRange::create(segment_range[0] + 1, self.len() as i8);
        self.contains_some_in_range(bottom_range) && self.contains_some_in_range(top_range)
    }

    pub(super) fn is_continuous(&self) -> bool {
        let mut some_seen = false;
        let mut none_seen = false;
        let mut some = None;
        for &seg in self.iter() {
            if seg.is_some() && !none_seen {
                some_seen = true;
                some = seg;
            } else if seg.is_some() && some_seen && seg != some {
                return false;
            } else if seg.is_none() && some_seen {
                none_seen = true
            }
        }
        true
    }

    pub(super) fn remove_segment(&mut self, segment_id: SegmentId) {
        for seg in self.iter_mut() {
            if let Some(id) = seg {
                if *id == segment_id {
                    *seg = None;
                }
            }
        }
    }

    pub(super) fn update(&mut self, snap_range: &SnapRange, segment_id: SegmentId) {
        for i in snap_range.iter() {
            if self[*i as usize].replace(segment_id).is_some() {
                panic!("Some segment was overriden in an update of a nodes lane map")
            }
        }
    }

    pub(super) fn expand(&mut self, snap_range: &SnapRange, segment_id: Option<SegmentId>) {
        let len = self.len() as i8;
        for i in snap_range.iter() {
            if *i < 0 {
                self.push_front(segment_id);
            }
            if *i >= len {
                self.push_back(segment_id);
            }
        }
    }
}
