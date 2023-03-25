use utils::input::ScrollState;

pub trait CycleSelection {
    fn prev(&self) -> Self;
    fn next(&self) -> Self;
}

pub fn scroll_mut<A: CycleSelection + Copy>(elem: &mut A, scroll_state: ScrollState) -> A {
    match scroll_state {
        ScrollState::Up => {
            *elem = elem.prev();
            *elem
        }
        ScrollState::Down => {
            *elem = elem.next();
            *elem
        }
    }
}

impl CycleSelection for world::roads::LaneWidth {
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

impl CycleSelection for world::roads::CurveType {
    fn prev(&self) -> Self {
        match self {
            Self::Straight => Self::Curved,
            Self::Curved => Self::Straight,
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Straight => Self::Curved,
            Self::Curved => Self::Straight,
        }
    }
}
