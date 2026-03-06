use crate::config::RuleConfig;
use crate::device;
use crate::focus::{self, Tracker};
use crate::input::{Binding, Event, Modifier, ScrollAxis};
use anyhow::{Context, Result, bail};
use evdev::{Device, EventType, InputEvent, Key};
use std::collections::HashMap;

const PRESS: i32 = 1;
const RELEASE: i32 = 0;

const SYN_REPORT_CODE: u16 = 0;
const SYN_REPORT_VAL: i32 = 0;

enum Action {
    Passthrough,
    Consumed,
}

const MODIFIER_KEYS: &[(Key, Key, Modifier)] = &[
    (Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL, Modifier::Ctrl),
    (Key::KEY_LEFTALT, Key::KEY_RIGHTALT, Modifier::Alt),
    (Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT, Modifier::Shift),
];

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

type Mappings = HashMap<Binding, Event>;

fn emit(virt: &mut evdev::uinput::VirtualDevice, key: Key) -> Result<()> {
    let press = InputEvent::new(EventType::KEY, key.code(), PRESS);
    let release = InputEvent::new(EventType::KEY, key.code(), RELEASE);
    let syn = InputEvent::new(EventType::SYNCHRONIZATION, SYN_REPORT_CODE, SYN_REPORT_VAL);

    virt.emit(&[press, syn])?;
    virt.emit(&[release, syn])?;

    Ok(())
}

fn try_remap(
    ev: &InputEvent,
    mappings: &Mappings,
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

    let Some(&output) = mappings.get(&binding) else {
        return Ok(Action::Passthrough);
    };

    if matches!(axis, ScrollAxis::HiRes) {
        return Ok(Action::Consumed);
    }

    eprintln!("{binding:?} -> {output:?}");
    emit(virt, output.to_evdev())?;

    Ok(Action::Consumed)
}

pub fn run(rule: &RuleConfig, socket_path: Option<String>) -> Result<()> {
    let mappings: Mappings = rule
        .mappings()
        .into_iter()
        .map(|m| (m.binding, m.output))
        .collect();

    if mappings.is_empty() {
        bail!("no valid mappings");
    }

    if rule.window_class.is_empty() {
        eprintln!("target: global (all windows)");
    } else {
        eprintln!("target: [{}]", rule.window_class.join(", "));
    }

    for (binding, output) in &mappings {
        eprintln!("  {binding:?} -> {output:?}");
    }

    let mut dev = device::find_mouse(rule.device.as_deref()).context("no suitable device found")?;
    eprintln!("grabbing: {}", dev.name().unwrap_or("?"));
    dev.grab()?;

    let provider = focus::provider(socket_path);
    let keyboard = device::find_keyboard();
    let extra_keys: Vec<_> = mappings.values().collect();

    let mut virt = device::mirror(&dev, &extra_keys)?;
    let mut window = Tracker::new(&rule.window_class, provider);

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
