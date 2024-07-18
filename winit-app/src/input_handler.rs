//! Handles the input system for highway architect, including translating winit
//! events into the desired hw-architect events.

use std::collections::BTreeMap;
use std::str::FromStr;

use anyhow::anyhow;

use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::{Key, ModifiersKeyState, NamedKey};
use winit::window::WindowId;

use utils::input::*;
use utils::loader;

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
        dbg!(key.clone());
        let actions = self.key_map.get(&(key, self.modifiers));
        dbg!(actions.clone());
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
}

fn translate_button(button: MouseButton) -> Mouse {
    match button {
        MouseButton::Left => Mouse::Left,
        MouseButton::Middle => Mouse::Middle,
        MouseButton::Right => Mouse::Right,
        MouseButton::Other(_) | MouseButton::Back | MouseButton::Forward => Mouse::Other,
    }
}

/// This type corresponds to the structure of the yaml files that define keybindings:
/// TODO: Right now the inner BTreeMap always only has one element, so maybe come up with a better
/// structure.
type KeyLoaderConfig = BTreeMap<String, Vec<BTreeMap<String, Vec<String>>>>;

/// Loads and returns the given keymap
///
/// # Arguments
///
/// * `key_map` - Default and ONLY (for now) options are "qwerty" "wokmok"
pub fn load_key_map(key_map: String) -> anyhow::Result<KeyMap> {
    let key_config_path = format!("config/{}.yml", &key_map);
    #[cfg(debug_assertions)]
    {
        dbg!(key_config_path.clone());
    }
    let key_config_file = loader::load_string(&key_config_path)?;
    let key_config: KeyLoaderConfig = serde_yaml::from_str(&key_config_file)?;

    let mut group_maps: BTreeMap<String, KeyMap> = BTreeMap::new();
    for (group, key_maps) in key_config.into_iter() {
        let mut group_map: KeyMap = BTreeMap::new();
        for key_map in key_maps.into_iter() {
            // this loop is silly as there is only one entry in the map
            for (action, keys) in key_map.into_iter() {
                let key_code = parse_key_code(&keys[0]).unwrap();
                let mod_state = keys
                    .iter()
                    .fold(ModifierState::default(), |mod_state, key| {
                        match key.as_str() {
                            "shift" => ModifierState {
                                shift: true,
                                ..mod_state
                            },
                            "ctrl" => ModifierState {
                                ctrl: true,
                                ..mod_state
                            },
                            "alt" => ModifierState {
                                alt: true,
                                ..mod_state
                            },
                            _ => mod_state,
                        }
                    });
                let action = Action::from_str(&action).unwrap();
                group_map.insert((key_code, mod_state), vec![action]);
            }
        }
        group_maps.insert(group, group_map);
    }

    // Merge group maps and check for conflicting keybindings
    let mut general_key_bindings: KeyMap = group_maps
        .remove("general")
        .ok_or(anyhow!("Could not find general key bindings"))?;

    let mut final_key_map: KeyMap = BTreeMap::new();
    for (_, group_map) in group_maps.into_iter() {
        for (key, mut action) in group_map.into_iter() {
            if general_key_bindings.contains_key(&key) {
                dbg!(key);
                return Err(anyhow!("Duplicate key binding"));
            }
            let Some(actions) = final_key_map.get_mut(&key) else {
                final_key_map.insert(key, action);
                continue;
            };
            actions.append(&mut action);
        }
    }

    // Add general_key_bindings to final_key_map
    final_key_map.append(&mut general_key_bindings);
    Ok(final_key_map)
}

/// Translates the keycode as it is written in the keymap config to a winit [`Key`]
fn parse_key_code(key: &String) -> anyhow::Result<Key> {
    match key.to_lowercase().as_str() {
        "esc" => Ok(Key::Named(NamedKey::Escape)),
        "space" => Ok(Key::Named(NamedKey::Space)),
        "a" => Ok(Key::Character("a".into())),
        "b" => Ok(Key::Character("b".into())),
        "c" => Ok(Key::Character("c".into())),
        "d" => Ok(Key::Character("d".into())),
        "e" => Ok(Key::Character("e".into())),
        "f" => Ok(Key::Character("f".into())),
        "g" => Ok(Key::Character("g".into())),
        "h" => Ok(Key::Character("h".into())),
        "j" => Ok(Key::Character("j".into())),
        "k" => Ok(Key::Character("k".into())),
        "l" => Ok(Key::Character("l".into())),
        "m" => Ok(Key::Character("m".into())),
        "n" => Ok(Key::Character("n".into())),
        "o" => Ok(Key::Character("o".into())),
        "p" => Ok(Key::Character("p".into())),
        "q" => Ok(Key::Character("q".into())),
        "r" => Ok(Key::Character("r".into())),
        "s" => Ok(Key::Character("s".into())),
        "t" => Ok(Key::Character("t".into())),
        "u" => Ok(Key::Character("u".into())),
        "v" => Ok(Key::Character("v".into())),
        "w" => Ok(Key::Character("w".into())),
        "x" => Ok(Key::Character("x".into())),
        "y" => Ok(Key::Character("y".into())),
        "z" => Ok(Key::Character("z".into())),
        "1" => Ok(Key::Character("1".into())),
        "2" => Ok(Key::Character("2".into())),
        "3" => Ok(Key::Character("3".into())),
        "4" => Ok(Key::Character("4".into())),
        "5" => Ok(Key::Character("5".into())),
        "6" => Ok(Key::Character("6".into())),
        "7" => Ok(Key::Character("7".into())),
        "8" => Ok(Key::Character("8".into())),
        "9" => Ok(Key::Character("9".into())),
        _ => Err(anyhow::anyhow!(format!("could not parse key: {}", key))),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::*;
    use std::io::prelude::*;

    // Run with cargo test write_keyconfig -- --ignored --nocapture in this crate
    #[test]
    #[ignore]
    fn write_keyconfig() {
        let mut key_map = std::collections::BTreeMap::new();
        key_map.insert(
            "camera_left".to_string(),
            vec!["a".to_string(), "shift".to_string()],
        );
        key_map.insert(
            "camera_right".to_string(),
            vec!["d".to_string(), "shift".to_string(), "ctrl".to_string()],
        );
        let keyconfigyaml = serde_yaml::to_string(&key_map).unwrap();
        println!("{}", keyconfigyaml);

        let mut file = File::create("../res/config/qwerty.yml").unwrap();
        file.write_all(keyconfigyaml.as_bytes()).unwrap();
    }
}
