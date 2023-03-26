//! Handles the input system for highway architect, including translating winit
//! events into the desired hw-architect events.

use std::collections::BTreeMap;
use utils::input::*;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::window::WindowId;

pub type KeyMap = BTreeMap<(VirtualKeyCode, ModifierState), Action>;

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

pub struct InputHandler {
    /// The key map in use.
    key_map: KeyMap,
    /// Maintains the current state of ctrl, alt and shift. Does not distinguish
    /// between left and right modifiers.
    modifiers: ModifierState,
    /// Maintains the current mouse position on the window in pixels, relative
    /// to the top left corner.
    mouse_pos: MousePos,
    /// A list maintaining in which order the mouse buttons have been pressed.
    /// Once a mouse button has been released, it is removed from the list.
    pressed_buttons: Vec<Mouse>,
    /// A set maintaining the key actions that are currently pressed. Scroll and repeat states are
    /// only sent to the most recently pressed key.
    pressed_keys: Vec<Action>,
}

impl InputHandler {
    pub fn new(key_map: KeyMap) -> Self {
        InputHandler {
            key_map,
            modifiers: ModifierState::default(),
            mouse_pos: MousePos { x: 0.0, y: 0.0 },
            pressed_buttons: Vec::new(),
            pressed_keys: Vec::new(),
        }
    }

    /// Processes scrolling. If a scroll consuming key is pressed, then a key event with scroll is
    /// sent instead of a mouse event.
    fn process_scrolling(&mut self, scroll: f32) -> InputEvent {
        use Action::*;
        if let Some(action) = self.pressed_keys.last() {
            match action {
                CycleCurveType | CycleLaneWidth | CycleNoLanes => {
                    let state = if scroll < 0.0 {
                        KeyState::Scroll(ScrollState::Up)
                    } else {
                        KeyState::Scroll(ScrollState::Down)
                    };
                    return InputEvent::KeyAction((*action, state));
                }
                _ => {}
            }
        }
        InputEvent::MouseEvent(MouseEvent::Scrolled(scroll))
    }

    /// Processes mouse movement and returns either moved or dragged events.
    fn process_mouse_movement(&mut self, new_pos: PhysicalPosition<f64>) -> InputEvent {
        let old_pos = self.mouse_pos;
        let delta = MouseDelta {
            dx: new_pos.x - old_pos.x,
            dy: new_pos.y - old_pos.y,
        };
        self.mouse_pos.x = new_pos.x;
        self.mouse_pos.y = new_pos.y;
        if let Some(&button) = self.pressed_buttons.last() {
            InputEvent::MouseEvent(MouseEvent::Dragged(button, delta))
        } else {
            InputEvent::MouseEvent(MouseEvent::Moved(delta))
        }
    }

    /// Processes mouse click and release events.
    fn process_mouse_press(&mut self, button: MouseButton, state: ElementState) -> InputEvent {
        let button = translate_button(button);
        match state {
            ElementState::Pressed => {
                self.pressed_buttons.push(button);
                InputEvent::MouseEvent(MouseEvent::Press(button))
            }
            ElementState::Released => {
                self.pressed_buttons.retain(|&b| b != button);
                InputEvent::MouseEvent(MouseEvent::Release(button))
            }
        }
    }

    /// Release states are only sent if the key pressed is the most recent key to have been pressed
    /// of those keys that are pressed.
    fn process_keyboard_input(&mut self, key: VirtualKeyCode, state: ElementState) -> InputEvent {
        let action = self.key_map.get(&(key, self.modifiers));
        let Some(action) = action else {
            return InputEvent::Absorb;
        };
        if let ElementState::Released = state {
            self.pressed_keys.retain(|a| a != action);
            return InputEvent::KeyAction((*action, KeyState::Release));
        }
        if !self.pressed_keys.contains(action) {
            self.pressed_keys.push(*action);
            return InputEvent::KeyAction((*action, KeyState::Press));
        }
        if self.pressed_keys.last() == Some(action) {
            return InputEvent::KeyAction((*action, KeyState::Repeat));
        }
        InputEvent::Absorb
    }

    /// Takes a winit event and converts it to a hw-architect [`InputEvent`].
    pub fn process_input(&mut self, event: &Event<()>, this_window_id: WindowId) -> InputEvent {
        let Event::WindowEvent { window_id, event } = event else {
            return InputEvent::Proceed;
        };
        if *window_id != this_window_id {
            return InputEvent::Proceed;
        }

        match event {
            WindowEvent::CursorMoved {
                position: new_pos, ..
            } => self.process_mouse_movement(*new_pos),
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, scroll) => self.process_scrolling(-scroll * 0.5),
                MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                    self.process_scrolling(-*scroll as f32)
                }
            },
            WindowEvent::MouseInput { button, state, .. } => {
                self.process_mouse_press(*button, *state)
            }
            WindowEvent::ModifiersChanged(m) => {
                self.modifiers = ModifierState {
                    ctrl: m.ctrl(),
                    alt: m.alt(),
                    shift: m.shift(),
                };
                InputEvent::Absorb
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.process_keyboard_input(*key, *state),
            _ => InputEvent::Proceed,
        }
    }

    /// Returns the last recorded mouse position in pixels from the top left
    /// corner of the window
    pub fn get_mouse_pos(&self) -> MousePos {
        self.mouse_pos
    }
    // pub fn get_modifier_state(&self) -> ModifierState {
    //     self.modifiers
    // }
}

fn translate_button(button: MouseButton) -> Mouse {
    match button {
        MouseButton::Left => Mouse::Left,
        MouseButton::Middle => Mouse::Middle,
        MouseButton::Right => Mouse::Right,
        MouseButton::Other(_) => Mouse::Other,
    }
}
