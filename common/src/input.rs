use winit::event::{VirtualKeyCode, MouseButton};


#[derive(Copy, Clone)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

pub struct KeyInput {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub pressed: bool,
    pub key: VirtualKeyCode,
}

pub struct MouseInput {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub pressed: bool,
    pub key: MouseButton,
}