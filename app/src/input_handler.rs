//! Handles the input system for highway architect, including translating winit
//! events into the desired hw-architect events.

use std::collections::BTreeMap;
use utils::input::*;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::{Key, ModifiersKeyState};
use winit::window::WindowId;

/// Represents which key combinations from the keyboard are associated with each of the
/// hw-architect actions. Note that a keybinding can be associated to multiple actions, and the
/// input handler will send out all actions once such a keybinding is pressed. It is the
/// configuration loader's responsibility to make sure that no keybindings are conflicting.
pub type KeyMap = BTreeMap<(Key, ModifierState), Vec<Action>>;

pub enum InputEvent {
    /// Signals a key event. The winit event should not be further processed.
    KeyActions(Vec<KeyAction>),
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
    /// only sent to the most recently pressed set of actions.
    pressed_actions: Vec<Vec<Action>>,
}

impl InputHandler {
    pub fn new(key_map: KeyMap) -> Self {
        InputHandler {
            key_map,
            modifiers: ModifierState::default(),
            mouse_pos: MousePos { x: 0.0, y: 0.0 },
            pressed_buttons: Vec::new(),
            pressed_actions: Vec::new(),
        }
    }

    /// Processes scrolling. If a scroll consuming key is pressed, then a key event with scroll is
    /// sent instead of a mouse event.
    fn process_scrolling(&mut self, scroll: f32) -> InputEvent {
        use Action::*;
        let Some(pressed_actions) = self.pressed_actions.last() else {
            return InputEvent::MouseEvent(MouseEvent::Scrolled(scroll));
        };

        let mut new_actions = vec![];
        for pressed_action in pressed_actions {
            match pressed_action {
                CycleCurveType | CycleLaneWidth | CycleNoLanes => {
                    let state = if scroll < 0.0 {
                        KeyState::Scroll(ScrollState::Up)
                    } else {
                        KeyState::Scroll(ScrollState::Down)
                    };
                    new_actions.push((*pressed_action, state))
                }
                _ => {}
            }
        }

        if new_actions.is_empty() {
            InputEvent::MouseEvent(MouseEvent::Scrolled(scroll))
        } else {
            InputEvent::KeyActions(new_actions)
        }
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
            InputEvent::MouseEvent(MouseEvent::Dragged(button, self.mouse_pos, delta))
        } else {
            InputEvent::MouseEvent(MouseEvent::Moved(self.mouse_pos, delta))
        }
    }

    /// Processes mouse press and release events.
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
    fn process_keyboard_input(&mut self, key: Key, state: ElementState) -> InputEvent {
        let actions = self.key_map.get(&(key, self.modifiers));
        let Some(actions) = actions else {
            return InputEvent::Absorb;
        };
        if let ElementState::Released = state {
            self.pressed_actions.retain(|a| a != actions);
            let new_actions = actions
                .into_iter()
                .map(|a| (*a, KeyState::Release))
                .collect();
            return InputEvent::KeyActions(new_actions);
        }
        if !self.pressed_actions.contains(actions) {
            self.pressed_actions.push(actions.clone());
            let new_actions = actions.into_iter().map(|a| (*a, KeyState::Press)).collect();
            return InputEvent::KeyActions(new_actions);
        }
        if self.pressed_actions.last() == Some(actions) {
            let new_actions = actions
                .into_iter()
                .map(|a| (*a, KeyState::Repeat))
                .collect();
            return InputEvent::KeyActions(new_actions);
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
                let ctrl = match m.lcontrol_state() {
                    ModifiersKeyState::Pressed => true,
                    ModifiersKeyState::Unknown => match m.rcontrol_state() {
                        ModifiersKeyState::Pressed => true,
                        ModifiersKeyState::Unknown => false,
                    },
                };
                let alt = match m.lalt_state() {
                    ModifiersKeyState::Pressed => true,
                    ModifiersKeyState::Unknown => match m.ralt_state() {
                        ModifiersKeyState::Pressed => true,
                        ModifiersKeyState::Unknown => false,
                    },
                };
                let shift = match m.lshift_state() {
                    ModifiersKeyState::Pressed => true,
                    ModifiersKeyState::Unknown => match m.rshift_state() {
                        ModifiersKeyState::Pressed => true,
                        ModifiersKeyState::Unknown => false,
                    },
                };
                self.modifiers = ModifierState { ctrl, alt, shift };
                InputEvent::Absorb
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key, state, ..
                },
                ..
            } => self.process_keyboard_input(logical_key.clone(), *state),
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
        MouseButton::Other(_) | MouseButton::Back | MouseButton::Forward => Mouse::Other,
    }
}
