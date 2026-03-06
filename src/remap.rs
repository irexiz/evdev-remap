use crate::config::RuleConfig;
use crate::device;
use crate::focus::{self, FocusTracker, HyprEnv};
use crate::input::{Binding, Event, Mapping, Modifier, ScrollAxis};
use anyhow::{Context, Result, bail};
use evdev::{Device, EventType, InputEvent, Key};

const PRESS: i32 = 1;
const RELEASE: i32 = 0;

const SYN_REPORT_CODE: u16 = 0;
const SYN_REPORT_VAL: i32 = 0;

/// What to do with an event after attempting a remap.
enum Action {
    Passthrough,
    Consumed,
}

const MODIFIER_KEYS: &[(Key, Key, Modifier)] = &[
    (Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL, Modifier::Ctrl),
    (Key::KEY_LEFTALT, Key::KEY_RIGHTALT, Modifier::Alt),
    (Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT, Modifier::Shift),
];

/// Query the keyboard for the currently held modifier. Ctrl > Alt > Shift priority.
fn read_modifier(keyboard: Option<&Device>) -> Modifier {
    let keys = keyboard
        .and_then(|kb| kb.get_key_state().ok())
        .unwrap_or_default();

    MODIFIER_KEYS
        .iter()
        .find(|(l, r, _)| keys.contains(*l) || keys.contains(*r))
        .map(|(_, _, m)| *m)
        .unwrap_or(Modifier::None)
}

fn lookup<'a>(mappings: &'a [Mapping], binding: &Binding) -> Option<&'a Mapping> {
    mappings.iter().find(|m| m.binding == *binding)
}

/// Emit a button press + release with SYN_REPORT after each.
fn emit(virt: &mut evdev::uinput::VirtualDevice, key: Key) -> Result<()> {
    let press = InputEvent::new(EventType::KEY, key.code(), PRESS);
    let release = InputEvent::new(EventType::KEY, key.code(), RELEASE);
    let syn = InputEvent::new(EventType::SYNCHRONIZATION, SYN_REPORT_CODE, SYN_REPORT_VAL);

    virt.emit(&[press, syn])?;
    virt.emit(&[release, syn])?;

    Ok(())
}

/// Try to remap a scroll event. Returns Consumed if matched (or hi-res swallowed).
fn try_remap(
    ev: &InputEvent,
    mappings: &[Mapping],
    keyboard: Option<&Device>,
    virt: &mut evdev::uinput::VirtualDevice,
) -> Result<Action> {
    let Some(axis) = ScrollAxis::from_code(ev.code()) else {
        return Ok(Action::Passthrough);
    };

    let direction = if ev.value().is_positive() {
        Event::ScrollUp
    } else {
        Event::ScrollDown
    };

    let binding = Binding {
        modifier: read_modifier(keyboard),
        input: direction,
    };

    let Some(m) = lookup(mappings, &binding) else {
        return Ok(Action::Passthrough);
    };

    if matches!(axis, ScrollAxis::HiRes) {
        return Ok(Action::Consumed);
    }

    eprintln!("{:?} -> {:?}", m.binding, m.output);

    emit(virt, m.output.to_evdev())?;

    Ok(Action::Consumed)
}

/// Grab device, create virtual mirror, enter event loop.
/// Matched events get remapped, everything else passes through.
pub fn run(rule: &RuleConfig, hypr_env: Option<HyprEnv>) -> Result<()> {
    let mappings = rule.mappings();

    if mappings.is_empty() {
        bail!("no valid mappings");
    }

    for m in &mappings {
        eprintln!("  {:?} -> {:?}", m.binding, m.output);
    }

    let mut dev = device::find_mouse(rule.device.as_deref()).context("no suitable device found")?;

    eprintln!("grabbing: {}", dev.name().unwrap_or("?"));

    dev.grab()?;

    let provider = focus::provider(hypr_env);
    let keyboard = device::find_keyboard();

    let mut virt = device::mirror(&dev, &mappings)?;
    let mut window = FocusTracker::new(&rule.window_class, provider);

    loop {
        for event in dev.fetch_events()? {
            let action = match event.event_type() {
                EventType::RELATIVE if window.is_focused() => {
                    try_remap(&event, &mappings, keyboard.as_ref(), &mut virt)?
                }
                _ => Action::Passthrough,
            };

            if matches!(action, Action::Passthrough) {
                virt.emit(&[event])?;
            }
        }
    }
}
