use serde::{Deserialize, Serialize};
use winit::event::ElementState;
use winit::keyboard::{Key, NamedKey};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Copy)]
pub enum KeyCode {
    Escape,
    Left,
    Up,
    Right,
    Down,
    Any,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space,
    Tab,
    LShift,
    RShift,
    Apostrophe,
    BackSpace,
    Enter,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Slash
}

impl KeyCode{
    pub(crate) fn to_char(&self) -> Option<char>{
        match self{
            KeyCode::A => Some('a'),
            KeyCode::B => Some('b'),
            KeyCode::C => Some('c'),
            KeyCode::D => Some('d'),
            KeyCode::E => Some('e'),
            KeyCode::F => Some('f'),
            KeyCode::G => Some('g'),
            KeyCode::H => Some('h'),
            KeyCode::I => Some('i'),
            KeyCode::J => Some('j'),
            KeyCode::K => Some('k'),
            KeyCode::L => Some('l'),
            KeyCode::M => Some('m'),
            KeyCode::N => Some('n'),
            KeyCode::O => Some('o'),
            KeyCode::P => Some('p'),
            KeyCode::Q => Some('q'),
            KeyCode::R => Some('r'),
            KeyCode::S => Some('s'),
            KeyCode::T => Some('t'),
            KeyCode::U => Some('u'),
            KeyCode::V => Some('v'),
            KeyCode::W => Some('w'),
            KeyCode::X => Some('x'),
            KeyCode::Y => Some('y'),
            KeyCode::Z => Some('z'),
            KeyCode::Space => Some(' '),
            KeyCode::Apostrophe => Some('\''),
            KeyCode::Slash => Some('/'),
            _ => None
        }
    }
}

impl From<&Key> for KeyCode {
    fn from(vkc: &Key) -> Self {
        match vkc.as_ref() {
            Key::Named(NamedKey::Escape) => KeyCode::Escape,
            Key::Named(NamedKey::ArrowLeft) => KeyCode::Left,
            Key::Named(NamedKey::ArrowUp) => KeyCode::Up,
            Key::Named(NamedKey::ArrowRight) => KeyCode::Right,
            Key::Named(NamedKey::ArrowDown) => KeyCode::Down,
            Key::Named(NamedKey::F1) => KeyCode::F1,
            Key::Named(NamedKey::F2) => KeyCode::F2,
            Key::Named(NamedKey::F3) => KeyCode::F3,
            Key::Named(NamedKey::F4) => KeyCode::F4,
            Key::Named(NamedKey::F5) => KeyCode::F5,
            Key::Named(NamedKey::F6) => KeyCode::F6,
            Key::Named(NamedKey::F7) => KeyCode::F7,
            Key::Named(NamedKey::F8) => KeyCode::F8,
            Key::Named(NamedKey::F9) => KeyCode::F9,
            Key::Named(NamedKey::F10) => KeyCode::F10,
            Key::Named(NamedKey::F11) => KeyCode::F11,
            Key::Named(NamedKey::F12) => KeyCode::F12,
            Key::Character("A") => KeyCode::A,
            Key::Character("B") => KeyCode::B,
            Key::Character("C") => KeyCode::C,
            Key::Character("D") => KeyCode::D,
            Key::Character("E") => KeyCode::E,
            Key::Character("F") => KeyCode::F,
            Key::Character("G") => KeyCode::G,
            Key::Character("H") => KeyCode::H,
            Key::Character("I") => KeyCode::I,
            Key::Character("J") => KeyCode::J,
            Key::Character("K") => KeyCode::K,
            Key::Character("L") => KeyCode::L,
            Key::Character("M") => KeyCode::M,
            Key::Character("N") => KeyCode::N,
            Key::Character("O") => KeyCode::O,
            Key::Character("P") => KeyCode::P,
            Key::Character("Q") => KeyCode::Q,
            Key::Character("R") => KeyCode::R,
            Key::Character("S") => KeyCode::S,
            Key::Character("T") => KeyCode::T,
            Key::Character("U") => KeyCode::U,
            Key::Character("V") => KeyCode::V,
            Key::Character("W") => KeyCode::W,
            Key::Character("X") => KeyCode::X,
            Key::Character("Y") => KeyCode::Y,
            Key::Character("Z") => KeyCode::Z,
            Key::Character("a") => KeyCode::A,
            Key::Character("b") => KeyCode::B,
            Key::Character("c") => KeyCode::C,
            Key::Character("d") => KeyCode::D,
            Key::Character("e") => KeyCode::E,
            Key::Character("f") => KeyCode::F,
            Key::Character("g") => KeyCode::G,
            Key::Character("h") => KeyCode::H,
            Key::Character("i") => KeyCode::I,
            Key::Character("j") => KeyCode::J,
            Key::Character("k") => KeyCode::K,
            Key::Character("l") => KeyCode::L,
            Key::Character("m") => KeyCode::M,
            Key::Character("n") => KeyCode::N,
            Key::Character("o") => KeyCode::O,
            Key::Character("p") => KeyCode::P,
            Key::Character("q") => KeyCode::Q,
            Key::Character("r") => KeyCode::R,
            Key::Character("s") => KeyCode::S,
            Key::Character("t") => KeyCode::T,
            Key::Character("u") => KeyCode::U,
            Key::Character("v") => KeyCode::V,
            Key::Character("w") => KeyCode::W,
            Key::Character("x") => KeyCode::X,
            Key::Character("y") => KeyCode::Y,
            Key::Character("z") => KeyCode::Z,
            Key::Character("\'") => KeyCode::Apostrophe,
            Key::Character("/") => KeyCode::Slash,
            Key::Named(NamedKey::Space) => KeyCode::Space,
            Key::Named(NamedKey::Shift) => KeyCode::LShift,
            Key::Named(NamedKey::Tab) => KeyCode::Tab,
            Key::Named(NamedKey::Backspace) => KeyCode::BackSpace,
            Key::Named(NamedKey::Enter) => KeyCode::Enter,
            _ => KeyCode::Any,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, Copy)]
pub enum InputState {
    Pressed,
    Released,
}

impl From<ElementState> for InputState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => InputState::Pressed,
            ElementState::Released => InputState::Released,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyboardEvent {
    pub keycode: KeyCode,
    pub state: InputState,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Input {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl From<KeyCode> for Input {
    fn from(val: KeyCode) -> Self {
        Input::Key(val)
    }
}

impl From<MouseButton> for Input {
    fn from(val: MouseButton) -> Self {
        Input::Mouse(val)
    }
}

pub type Shortcut = Vec<Input>;

#[cfg(test)]
mod tests {
    use crate::core::resources::inputs::inputs_controller::InputsController;

    #[test]
    fn shortcut_test() {
        let controller = InputsController::default();
        let pressed = controller.all_pressed();
        controller.shortcut_pressed(&pressed);
    }
}
