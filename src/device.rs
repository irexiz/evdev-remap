use crate::input::Mapping;
use anyhow::{Context, Result};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, Key, RelativeAxisType};

// Software-created devices that should never be grabbed (feedback loops).
const SKIP_NAMES: &[&str] = &["virtual", "ydotool", "synergy", "barrier", "evdev-remap"];

/// Priority rank for device selection. Lower = better match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Exact = 0,
    Fuzzy = 1,
}

/// How to locate a device: by path, name substring, or auto-detect.
enum DeviceFilter<'a> {
    Path(&'a str),
    Name(&'a str),
    Auto,
}

impl<'a> DeviceFilter<'a> {
    fn parse(s: Option<&'a str>) -> Self {
        match s {
            Some(s) if s.starts_with("/dev/") => Self::Path(s),
            Some(s) => Self::Name(s),
            None => Self::Auto,
        }
    }
}

/// Find a device by path, name, or auto-detect.
pub fn find_mouse(filter: Option<&str>) -> Option<Device> {
    match DeviceFilter::parse(filter) {
        DeviceFilter::Path(p) => {
            eprintln!("opening: {p}");
            Device::open(p).ok()
        }
        DeviceFilter::Name(n) => by_name(n).or_else(|| {
            eprintln!("'{n}' not found, trying auto-detect");
            auto_detect()
        }),
        DeviceFilter::Auto => auto_detect(),
    }
}

/// Prefers exact match over substring.
fn by_name(filter: &str) -> Option<Device> {
    let filter = filter.to_lowercase();

    evdev::enumerate()
        .filter(|(_, dev)| is_mouse(dev))
        .filter_map(|(path, dev)| {
            let name = dev.name()?.to_lowercase();
            let score = if name == filter {
                Rank::Exact
            } else if name.contains(&filter) && !is_keyboard(&name) {
                Rank::Fuzzy
            } else {
                return None;
            };
            Some((score, path))
        })
        .min_by_key(|(score, _)| *score)
        .and_then(|(_, path)| {
            eprintln!("found: {path:?}");
            Device::open(path).ok()
        })
}

/// Pick the best mouse. Prefers devices with "mouse" or "receiver" in the name.
fn auto_detect() -> Option<Device> {
    evdev::enumerate()
        .filter(|(_, dev)| {
            let name = dev.name().unwrap_or_default().to_lowercase();

            !SKIP_NAMES.iter().any(|s| name.contains(s))
                && !is_keyboard(&name)
                && is_mouse(dev)
                && dev.supported_keys().is_some()
        })
        .min_by_key(|(_, dev)| {
            let name = dev.name().unwrap_or_default().to_lowercase();

            if name.contains("mouse") || name.contains("receiver") {
                Rank::Exact
            } else {
                Rank::Fuzzy
            }
        })
        .and_then(|(path, _)| {
            eprintln!("auto-detected: {path:?}");
            Device::open(&path).ok()
        })
}

fn is_keyboard(name: &str) -> bool {
    name.contains("keyboard") || name.contains("kbd")
}

/// Find a keyboard for reading modifier state (not grabbed).
/// Prefers keyd's virtual keyboard since it reflects remapped state.
pub fn find_keyboard() -> Option<Device> {
    evdev::enumerate()
        .filter_map(|(path, dev)| {
            let name = dev.name()?.to_lowercase();

            if name.contains("keyd virtual keyboard") {
                return Some((Rank::Exact, path));
            }

            if SKIP_NAMES.iter().any(|s| name.contains(s)) && !name.contains("keyd") {
                return None;
            }

            let keys = dev.supported_keys()?;
            if keys.contains(Key::KEY_LEFTCTRL) && keys.contains(Key::KEY_LEFTALT) {
                return Some((Rank::Fuzzy, path));
            }

            None
        })
        .min_by_key(|(score, _)| *score)
        .and_then(|(_, path)| {
            eprintln!("modifier source: {path:?}");
            Device::open(&path).ok()
        })
}

const MOUSE_AXES: &[RelativeAxisType] = &[
    RelativeAxisType::REL_X,
    RelativeAxisType::REL_Y,
    RelativeAxisType::REL_WHEEL,
];

/// A mouse has relative X/Y axes and a scroll wheel.
fn is_mouse(dev: &Device) -> bool {
    dev.supported_relative_axes()
        .is_some_and(|axes| MOUSE_AXES.iter().all(|a| axes.contains(*a)))
}

/// Create a virtual device mirroring the source's capabilities,
/// plus any extra keys needed by remap outputs.
pub fn mirror(source: &Device, mappings: &[Mapping]) -> Result<VirtualDevice> {
    let mut keys = AttributeSet::<Key>::new();
    let mut axes = AttributeSet::<RelativeAxisType>::new();

    source
        .supported_keys()
        .into_iter()
        .flat_map(|s| s.iter())
        .for_each(|k| {
            keys.insert(k);
        });

    mappings.iter().for_each(|m| {
        keys.insert(m.output.to_evdev());
    });

    source
        .supported_relative_axes()
        .into_iter()
        .flat_map(|s| s.iter())
        .for_each(|a| {
            axes.insert(a);
        });

    VirtualDeviceBuilder::new()
        .context("failed to create uinput builder")?
        .name("evdev-remap virtual mouse")
        .with_keys(&keys)?
        .with_relative_axes(&axes)?
        .build()
        .context("failed to build virtual device")
}
