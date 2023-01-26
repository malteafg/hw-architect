#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
/// A state of the modifier keys, shift, ctrl and alt. Does not distinguish
/// between right and left modifier keys.
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Defines an Action that has happened. The bool tells if the key is currently
/// pressed. If key is pressed event is sent in each input update, followed by
/// one event signalling that key is now unpressed.
pub type KeyAction = (Action, bool);

#[derive(EnumString, Display, PartialEq, Debug, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
/// Enum containing all possible actions that a user can do with a keyboard.
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
/// Position of mouse given in pixels from top left corner of window.
pub struct MousePos {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
/// Mouse movement since last input update given in pixels.
pub struct MouseDelta {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mouse {
    Left,
    Middle,
    Right,
    // TODO add functionality for other buttons possibly.
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
    /// Signals a key event. The winit event should not be further processed.
    KeyAction(KeyAction),
    /// Signals a mouse event. The winit event should not be further processed.
    MouseEvent(MouseEvent),
    /// Signals that the input system has used a given winit event and it should
    /// therefore, not be further processed.
    Absorb,
    /// Signals that the input system has not used a given winit event and
    /// therefore, the winit event should be further processed
    Proceed,
}
