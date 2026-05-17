use evdev::{Key, RelativeAxisType};
use serde::Deserialize;

pub trait ToEvdev {
    fn to_evdev(self) -> evdev::Key;
    fn is_button(self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Mouse(MouseEvent),
    Keyboard(KeyboardEvent),
}

impl ToEvdev for Event {
    fn to_evdev(self) -> evdev::Key {
        match self {
            Event::Mouse(mouse_event) => mouse_event.to_evdev(),
            Event::Keyboard(keyboard_event) => keyboard_event.to_evdev(),
        }
    }

    fn is_button(self) -> bool {
        match self {
            Event::Mouse(mouse_event) => mouse_event.is_button(),
            Event::Keyboard(keyboard_event) => keyboard_event.is_button(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    #[default]
    None,
    Ctrl,
    Alt,
    Shift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseEvent {
    ScrollUp,
    ScrollDown,
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseSide,
    MouseExtra,
}

impl ToEvdev for MouseEvent {
    fn to_evdev(self) -> Key {
        match self {
            Self::MouseLeft => Key::BTN_LEFT,
            Self::MouseRight => Key::BTN_RIGHT,
            Self::MouseMiddle => Key::BTN_MIDDLE,
            Self::MouseSide => Key::BTN_SIDE,
            Self::MouseExtra => Key::BTN_EXTRA,
            Self::ScrollUp | Self::ScrollDown => unreachable!("scroll events have no key code"),
        }
    }

    fn is_button(self) -> bool {
        !matches!(self, Self::ScrollUp | Self::ScrollDown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyboardEvent {
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
    W,
    V,
    X,
    Y,
    Z,
}

impl ToEvdev for KeyboardEvent {
    fn to_evdev(self) -> Key {
        match self {
            KeyboardEvent::A => Key::KEY_A,
            KeyboardEvent::B => Key::KEY_B,
            KeyboardEvent::C => Key::KEY_C,
            KeyboardEvent::D => Key::KEY_D,
            KeyboardEvent::E => Key::KEY_E,
            KeyboardEvent::F => Key::KEY_F,
            KeyboardEvent::G => Key::KEY_G,
            KeyboardEvent::H => Key::KEY_H,
            KeyboardEvent::I => Key::KEY_I,
            KeyboardEvent::J => Key::KEY_J,
            KeyboardEvent::K => Key::KEY_K,
            KeyboardEvent::L => Key::KEY_L,
            KeyboardEvent::M => Key::KEY_M,
            KeyboardEvent::N => Key::KEY_N,
            KeyboardEvent::O => Key::KEY_O,
            KeyboardEvent::P => Key::KEY_P,
            KeyboardEvent::Q => Key::KEY_Q,
            KeyboardEvent::R => Key::KEY_R,
            KeyboardEvent::S => Key::KEY_S,
            KeyboardEvent::T => Key::KEY_T,
            KeyboardEvent::U => Key::KEY_U,
            KeyboardEvent::W => Key::KEY_W,
            KeyboardEvent::V => Key::KEY_V,
            KeyboardEvent::X => Key::KEY_X,
            KeyboardEvent::Y => Key::KEY_Y,
            KeyboardEvent::Z => Key::KEY_Z,
        }
    }

    fn is_button(self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Binding {
    pub modifier: Modifier,
    pub input: Event,
}

#[derive(Debug, Clone, Copy)]
pub struct Mapping {
    pub binding: Binding,
    pub output: Event,
}

pub enum ScrollAxis {
    Standard,
    HiRes,
}

impl ScrollAxis {
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            c if c == RelativeAxisType::REL_WHEEL.0 => Some(Self::Standard),
            c if c == RelativeAxisType::REL_WHEEL_HI_RES.0 => Some(Self::HiRes),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    // im starting to thing toml was a mistake lol
    // TODO: change configs to json maybe?
    #[derive(Deserialize)]
    struct W<T> {
        v: T,
    }

    fn deser<T: for<'de> Deserialize<'de>>(val: &str) -> T {
        toml::from_str::<W<T>>(&format!("v = \"{val}\"")).unwrap().v
    }

    #[test]
    fn event_deser() {
        assert_eq!(
            deser::<Event>("scroll_up"),
            Event::Mouse(MouseEvent::ScrollUp)
        );
        assert_eq!(
            deser::<Event>("mouse_left"),
            Event::Mouse(MouseEvent::MouseLeft)
        );
        assert_eq!(
            deser::<Event>("mouse_side"),
            Event::Mouse(MouseEvent::MouseSide)
        );
    }

    #[test]
    fn modifier_deser() {
        assert_eq!(deser::<Modifier>("ctrl"), Modifier::Ctrl);
        assert_eq!(deser::<Modifier>("none"), Modifier::None);
    }
}
