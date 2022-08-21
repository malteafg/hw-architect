#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

pub type KeyAction = (Action, bool);

#[derive(EnumString, Display, PartialEq, Debug, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    CameraLeft,
    CameraRight,
    CameraUp,
    CameraDown,
    CameraRotateLeft,
    CameraRotateRight,
    CameraReturn,
    CycleRoadType,
    ToggleBulldoze,
    OneLane,
    TwoLane,
    ThreeLane,
    FourLane,
    FiveLane,
    SixLane,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub struct MousePos {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseDelta {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mouse {
    Left,
    Middle,
    Right,
    // TODO add functionality for other buttons possibly
    Other,
}

#[derive(Clone, Copy)]
pub enum MouseEvent {
    Click(Mouse),
    Release(Mouse),
    Moved(MouseDelta),
    Dragged(Mouse, MouseDelta),
    Scrolled(f32),
}

pub enum InputEvent {
    KeyAction(KeyAction),
    MouseEvent(MouseEvent),
    Absorb,
    Proceed,
}
