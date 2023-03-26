//! Defines all types associated with different types of input events.

/// A state of the modifier keys, shift, ctrl and alt. Does not distinguish
/// between right and left modifier keys.
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Defines an Action that has happened. The bool tells if the key is currently
/// pressed. If key is pressed event is sent in each input update, followed by
/// one event signalling that key is now unpressed.
pub type KeyAction = (Action, KeyState);

/// Enum containing all possible actions that a user can do with a keyboard.
#[derive(EnumString, Display, PartialEq, Eq, Debug, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    CameraLeft,
    CameraRight,
    CameraUp,
    CameraDown,
    CameraRotateLeft,
    CameraRotateRight,
    CameraReturn,

    ToggleSnapping,
    ToggleReverse,

    CycleCurveType,
    CycleLaneWidth,
    CycleNoLanes,

    EnterBulldoze,
    EnterConstruct,
    EnterTreePlopper,

    Exit,
    Esc,
}

/// Defines the modes that a scroll can be in. For now this is up or down, corresponding to exactly
/// one roll of the mouse wheel either up or down.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum ScrollState {
    Up,
    Down,
}

/// Defines the state of the key that an event is regarding.
/// TODO maybe have two different release events? One is sent if no scrolling has been sent.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum KeyState {
    /// The key has just been pressed.
    Press,
    /// The key is being held down and is the most recent key to have been pressed of those keys
    /// that are pressed.
    Repeat,
    /// The key has just been released.
    Release,
    /// The mouse wheel have been scrolled whilst this key has been held down and this is the
    /// most recent key to have been pressed down of those keys that are pressed.
    Scroll(ScrollState),
}

/// Position of mouse given in pixels from top left corner of window.
#[derive(Debug, Clone, Copy)]
pub struct MousePos {
    pub x: f64,
    pub y: f64,
}

/// Mouse movement since last input update given in pixels.
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
    // TODO add functionality for other buttons possibly.
    Other,
}

#[derive(Clone, Copy)]
pub enum MouseEvent {
    Press(Mouse),
    Release(Mouse),
    Moved(MouseDelta),
    Dragged(Mouse, MouseDelta),
    Scrolled(f32),
}
