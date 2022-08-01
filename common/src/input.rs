use std::collections::BTreeMap;
use winit::event::*;
use winit::event::{MouseButton, VirtualKeyCode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

pub type KeyMap = BTreeMap<(VirtualKeyCode, ModifierState), Action>;

pub struct InputHandler {
    key_map: KeyMap,
    modifiers: ModifierState,
}

impl InputHandler {
    pub fn new(key_map: KeyMap) -> Self {
        InputHandler {
            key_map,
            modifiers: ModifierState::default(),
        }
    }

    pub fn process_input(&mut self, event: &WindowEvent) -> Option<KeyAction> {
        match event {
            WindowEvent::ModifiersChanged(m) => {
                self.modifiers = ModifierState {
                    ctrl: m.ctrl(),
                    alt: m.alt(),
                    shift: m.shift(),
                };
                None
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
                .get(&(key.clone(), self.modifiers.clone()))
                .map(|action| (action.clone(), state == &ElementState::Pressed)),
            _ => None,
        }
    }
}

pub fn parse_key_code(key: &String) -> anyhow::Result<VirtualKeyCode> {
    match key.as_str() {
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
        _ => Err(anyhow::anyhow!(format!("could not parse key: {}", key))),
    }
}

pub type KeyAction = (Action, bool);

#[derive(EnumString, Display, PartialEq, Debug, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    // #[strum(serialize = "camera_left")]
    CameraLeft,
    // #[strum(serialize = "camera_right")]
    CameraRight,
    // #[strum(serialize = "camera_up")]
    CameraUp,
    // #[strum(serialize = "camera_down")]
    CameraDown,
    // #[strum(serialize = "camera_rotate_left")]
    CameraRotateLeft,
    // #[strum(serialize = "camera_rotate_right")]
    CameraRotateRight,
}

// #[derive(EnumString, Display, PartialEq, Debug, Clone)]
// #[strum(serialize_all = "snake_case")]
// pub enum KeyMap {
//     Qwerty,
//     Wokmok,
// }

// impl fmt::Display for KeyAction {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             KeyAction::CameraLeft(_) => write!(f, "camera_left"),
//             KeyAction::CameraRight(_) => write!(f, "camera_right"),
//             KeyAction::CameraUp(_) => write!(f, "camera_up"),
//             KeyAction::CameraDown(_) => write!(f, "camera_down"),
//             KeyAction::CameraRotateLeft(_) => write!(f, "camera_rotate_left"),
//             KeyAction::CameraRotateRight(_) => write!(f, "camera_rotate_right"),
//         }
//     }
// }
