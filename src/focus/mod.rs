pub mod hyprland;
pub mod i3;

use std::time::{Duration, Instant};

pub trait FocusProvider {
    fn active_window_class(&mut self) -> Option<String>;
}

struct NoopProvider;

impl FocusProvider for NoopProvider {
    fn active_window_class(&mut self) -> Option<String> {
        None
    }
}

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct Tracker {
    targets: Vec<String>,
    active: bool,
    last_check: Instant,
    provider: Box<dyn FocusProvider>,
}

impl Tracker {
    pub fn new(targets: &[String], provider: Box<dyn FocusProvider>) -> Self {
        Self {
            targets: targets.iter().map(|s| s.to_lowercase()).collect(),
            active: false,
            last_check: Instant::now() - POLL_INTERVAL,
            provider,
        }
    }

    pub fn is_focused(&mut self) -> bool {
        if self.targets.is_empty() {
            return true;
        }

        if self.last_check.elapsed() < POLL_INTERVAL {
            return self.active;
        }

        self.last_check = Instant::now();

        let prev = self.active;

        self.active = self
            .provider
            .active_window_class()
            .map(|c| {
                let c = c.to_lowercase();
                self.targets.iter().any(|t| c.contains(t))
            })
            .unwrap_or(false);

        if self.active != prev {
            eprintln!("focus: {}", if self.active { "matched" } else { "lost" });
        }

        self.active
    }
}

#[derive(Clone)]
pub enum Socket {
    Hyprland(String),
    I3(String),
}

pub fn socket() -> Option<Socket> {
    hyprland::socket()
        .map(Socket::Hyprland)
        .or_else(|| i3::socket().map(Socket::I3))
}

pub fn provider(socket: Option<Socket>) -> Box<dyn FocusProvider> {
    match socket {
        Some(Socket::Hyprland(p)) => Box::new(hyprland::Hyprland::new(p)),
        Some(Socket::I3(p)) => Box::new(i3::I3::new(p)),
        None => Box::new(NoopProvider),
    }
}
