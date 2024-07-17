use utils::{consts, input::ScrollState};
use world_api::LaneWidth;

use crate::tool_state::CurveType;

pub trait CycleSelection {
    fn prev(&self) -> Self;
    fn next(&self) -> Self;
}

pub fn scroll<A: CycleSelection + Copy>(elem: A, scroll_state: ScrollState) -> A {
    match scroll_state {
        ScrollState::Up => elem.prev(),
        ScrollState::Down => elem.next(),
    }
}

impl CycleSelection for LaneWidth {
    fn prev(&self) -> Self {
        match self {
            Self::Narrow => Self::Wide,
            Self::Standard => Self::Narrow,
            Self::Wide => Self::Standard,
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Narrow => Self::Standard,
            Self::Standard => Self::Wide,
            Self::Wide => Self::Narrow,
        }
    }
}

impl CycleSelection for CurveType {
    fn prev(&self) -> Self {
        match self {
            Self::Straight => Self::Circular,
            Self::Circular => Self::Straight,
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Straight => Self::Circular,
            Self::Circular => Self::Straight,
        }
    }
}

/// This implementation is for no lanes.
impl CycleSelection for u8 {
    fn prev(&self) -> Self {
        if *self == consts::MAX_NO_LANES {
            return 1;
        }
        *self + 1
    }

    fn next(&self) -> Self {
        if *self == 1 {
            return consts::MAX_NO_LANES;
        }
        *self - 1
    }
}
