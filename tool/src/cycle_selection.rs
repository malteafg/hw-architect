use utils::input::ScrollState;

pub trait CycleSelection {
    fn prev(&self) -> Self;
    fn next(&self) -> Self;
}

pub fn scroll_mut<A: CycleSelection>(elem: &mut A, scroll_state: ScrollState) {
    match scroll_state {
        ScrollState::Up => *elem = elem.prev(),
        ScrollState::Down => *elem = elem.next(),
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
