use std::collections::BTreeMap;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::event::{MouseButton, VirtualKeyCode};
use winit::window::WindowId;

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

pub type KeyMap = BTreeMap<(VirtualKeyCode, ModifierState), Action>;

pub struct InputHandler {
    key_map: KeyMap,
    modifiers: ModifierState,
    mouse_clicking: bool,
    mouse_pressed: bool,
    mouse_pos: MousePos,
}

impl InputHandler {
    pub fn new(key_map: KeyMap) -> Self {
        InputHandler {
            key_map,
            modifiers: ModifierState::default(),
            mouse_clicking: false,
            mouse_pressed: false,
            mouse_pos: MousePos { x: 0.0, y: 0.0 },
        }
    }

    pub fn process_input(&mut self, event: &Event<()>, this_window_id: WindowId) -> InputEvent {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                self.mouse_clicking = false;
                let pos = self.mouse_pos;
                let delta = MouseDelta { dx: delta.0, dy: delta.1};
                match self.mouse_pressed {
                    true => InputEvent::MouseEvent(MouseEvent::Dragged { pos, delta }),
                    false => InputEvent::MouseEvent(MouseEvent::Moved { pos, delta }),
                }
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == &this_window_id => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.mouse_pos.x = position.x;
                    self.mouse_pos.y = position.y;
                    InputEvent::Absorb
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    MouseScrollDelta::LineDelta(_, scroll) => InputEvent::MouseEvent(MouseEvent::Scrolled( -scroll * 0.5)),
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => InputEvent::MouseEvent(MouseEvent::Scrolled( -*scroll as f32)),
                }
                WindowEvent::MouseInput {
                    button,
                    state,
                    ..
                } => match *state {
                    ElementState::Pressed => {
                        if *button == MouseButton::Left {
                            self.mouse_pressed = true;
                        }
                        self.mouse_clicking = true;
                        InputEvent::Absorb
                    }
                    ElementState::Released => if self.mouse_clicking {
                        let pos = self.mouse_pos;
                        let mods = self.modifiers;
                        self.mouse_pressed = false;
                        match button {
                            MouseButton::Left => InputEvent::MouseEvent(MouseEvent::Left { pos, mods }),
                            MouseButton::Right => InputEvent::MouseEvent(MouseEvent::Right { pos, mods }),
                            MouseButton::Middle => InputEvent::MouseEvent(MouseEvent::Middle { pos, mods }),
                            MouseButton::Other(i) => { dbg!(i); InputEvent::Absorb}
                        }
                    } else {
                        self.mouse_pressed = false;
                        InputEvent::Absorb
                    }
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
                } => self
                    .key_map
                    .get(&(*key, self.modifiers))
                    .map(|action| {
                        InputEvent::KeyAction((*action, *state == ElementState::Pressed))
                    })
                    .unwrap_or(InputEvent::Absorb),
                _ => InputEvent::Proceed
            }
            _ => {
                InputEvent::Proceed
            }
        }
    }
    
    pub fn get_mouse_pos(&self) -> MousePos {
        self.mouse_pos
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
        "esc" => Ok(VirtualKeyCode::Escape),
        _ => Err(anyhow::anyhow!(format!("could not parse key: {}", key))),
    }
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
    CycleRoadType,
    OneLane,
    TwoLane,
    ThreeLane,
    FourLane,
    FiveLane,
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

#[derive(Clone, Copy)]
pub enum MouseEvent {
    Left { pos: MousePos, mods: ModifierState },
    Middle { pos: MousePos, mods: ModifierState },
    Right { pos: MousePos, mods: ModifierState },
    Moved { pos: MousePos, delta: MouseDelta },
    Dragged { pos: MousePos, delta: MouseDelta },
    Scrolled(f32),
}

pub enum InputEvent {
    KeyAction(KeyAction),
    MouseEvent(MouseEvent),
    Absorb,
    Proceed,
}
