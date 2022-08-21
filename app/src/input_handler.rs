use std::collections::BTreeMap;
use utils::input::*;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::window::WindowId;

pub type KeyMap = BTreeMap<(VirtualKeyCode, ModifierState), Action>;

pub struct InputHandler {
    key_map: KeyMap,
    modifiers: ModifierState,
    mouse_pos: MousePos,
    pressed_buttons: Vec<Mouse>,
}

impl InputHandler {
    pub fn new(key_map: KeyMap) -> Self {
        InputHandler {
            key_map,
            modifiers: ModifierState::default(),
            mouse_pos: MousePos { x: 0.0, y: 0.0 },
            pressed_buttons: vec![],
        }
    }

    pub fn process_input(&mut self, event: &Event<()>, this_window_id: WindowId) -> InputEvent {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == this_window_id => match event {
                WindowEvent::CursorMoved {
                    position: new_pos, ..
                } => {
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
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_, scroll) => {
                        InputEvent::MouseEvent(MouseEvent::Scrolled(-scroll * 0.5))
                    }
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                        InputEvent::MouseEvent(MouseEvent::Scrolled(-*scroll as f32))
                    }
                },
                WindowEvent::MouseInput { button, state, .. } => match *state {
                    ElementState::Pressed => {
                        #[cfg(debug_assertions)]
                        if let MouseButton::Other(i) = button {
                            println!("Mouse button pressed: ");
                            dbg!(i);
                        }
                        self.pressed_buttons.push(translate_button(*button));
                        InputEvent::MouseEvent(MouseEvent::Click(translate_button(*button)))
                    }
                    ElementState::Released => {
                        self.pressed_buttons
                            .retain(|&b| b != translate_button(*button));
                        InputEvent::MouseEvent(MouseEvent::Release(translate_button(*button)))
                    }
                },
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
                } => self
                    .key_map
                    .get(&(*key, self.modifiers))
                    .map(|action| InputEvent::KeyAction((*action, *state == ElementState::Pressed)))
                    .unwrap_or(InputEvent::Absorb),
                _ => InputEvent::Proceed,
            },
            _ => InputEvent::Proceed,
        }
    }

    pub fn get_mouse_pos(&self) -> MousePos {
        self.mouse_pos
    }

    pub fn get_modifier_state(&self) -> ModifierState {
        self.modifiers
    }
}

pub fn parse_key_code(key: &String) -> anyhow::Result<VirtualKeyCode> {
    match key.to_lowercase().as_str() {
        "a" => Ok(VirtualKeyCode::A),
        "b" => Ok(VirtualKeyCode::B),
        "c" => Ok(VirtualKeyCode::C),
        "d" => Ok(VirtualKeyCode::D),
        "e" => Ok(VirtualKeyCode::E),
        "f" => Ok(VirtualKeyCode::F),
        "g" => Ok(VirtualKeyCode::G),
        "h" => Ok(VirtualKeyCode::H),
        "j" => Ok(VirtualKeyCode::J),
        "k" => Ok(VirtualKeyCode::K),
        "l" => Ok(VirtualKeyCode::L),
        "m" => Ok(VirtualKeyCode::M),
        "n" => Ok(VirtualKeyCode::N),
        "o" => Ok(VirtualKeyCode::O),
        "p" => Ok(VirtualKeyCode::P),
        "q" => Ok(VirtualKeyCode::Q),
        "r" => Ok(VirtualKeyCode::R),
        "s" => Ok(VirtualKeyCode::S),
        "t" => Ok(VirtualKeyCode::T),
        "u" => Ok(VirtualKeyCode::U),
        "v" => Ok(VirtualKeyCode::V),
        "w" => Ok(VirtualKeyCode::W),
        "x" => Ok(VirtualKeyCode::X),
        "y" => Ok(VirtualKeyCode::Y),
        "z" => Ok(VirtualKeyCode::Z),
        "1" => Ok(VirtualKeyCode::Key1),
        "2" => Ok(VirtualKeyCode::Key2),
        "3" => Ok(VirtualKeyCode::Key3),
        "4" => Ok(VirtualKeyCode::Key4),
        "5" => Ok(VirtualKeyCode::Key5),
        "6" => Ok(VirtualKeyCode::Key6),
        "7" => Ok(VirtualKeyCode::Key7),
        "8" => Ok(VirtualKeyCode::Key8),
        "9" => Ok(VirtualKeyCode::Key9),
        "esc" => Ok(VirtualKeyCode::Escape),
        "space" => Ok(VirtualKeyCode::Space),
        _ => Err(anyhow::anyhow!(format!("could not parse key: {}", key))),
    }
}

fn translate_button(button: MouseButton) -> Mouse {
    match button {
        MouseButton::Left => Mouse::Left,
        MouseButton::Middle => Mouse::Middle,
        MouseButton::Right => Mouse::Right,
        MouseButton::Other(_) => Mouse::Other,
    }
}
