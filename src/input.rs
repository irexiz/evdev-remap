use evdev::{Key, RelativeAxisType};
use serde::Deserialize;

/// Keyboard modifier held during an input event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    #[default]
    None,
    Ctrl,
    Alt,
    Shift,
}

/// A remappable event: scroll direction or mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    ScrollUp,
    ScrollDown,
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseSide,
    MouseExtra,
}

/// A modifier + event pair that triggers a remap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Binding {
    pub modifier: Modifier,
    pub input: Event,
}

/// A binding mapped to its replacement output event.
#[derive(Debug, Clone, Copy)]
pub struct Mapping {
    pub binding: Binding,
    pub output: Event,
}

/// Classifies REL_WHEEL vs REL_WHEEL_HI_RES axis codes.
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

impl Event {
    /// Convert to evdev key code. Only valid for button events.
    pub fn to_evdev(self) -> Key {
        match self {
            Self::MouseLeft => Key::BTN_LEFT,
            Self::MouseRight => Key::BTN_RIGHT,
            Self::MouseMiddle => Key::BTN_MIDDLE,
            Self::MouseSide => Key::BTN_SIDE,
            Self::MouseExtra => Key::BTN_EXTRA,
            Self::ScrollUp | Self::ScrollDown => unreachable!("scroll events have no key code"),
        }
    }

    pub fn is_button(self) -> bool {
        !matches!(self, Self::ScrollUp | Self::ScrollDown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct W<T> {
        v: T,
    }

    fn deser<T: for<'de> Deserialize<'de>>(val: &str) -> T {
        toml::from_str::<W<T>>(&format!("v = \"{val}\"")).unwrap().v
    }

    #[test]
    fn event_deser() {
        assert_eq!(deser::<Event>("scroll_up"), Event::ScrollUp);
        assert_eq!(deser::<Event>("mouse_left"), Event::MouseLeft);
        assert_eq!(deser::<Event>("mouse_side"), Event::MouseSide);
    }

    #[test]
    fn modifier_deser() {
        assert_eq!(deser::<Modifier>("ctrl"), Modifier::Ctrl);
        assert_eq!(deser::<Modifier>("none"), Modifier::None);
    }
}
