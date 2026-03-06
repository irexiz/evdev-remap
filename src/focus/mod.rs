pub mod hyprland;

use std::time::{Duration, Instant};

pub use hyprland::HyprEnv;

/// Queries the compositor for the currently focused window class.
/// Implement this for each supported compositor.
pub trait FocusProvider {
    fn active_window_class(&mut self) -> Option<String>;
}

/// Wraps a FocusProvider with throttling and target class matching.
/// The underlying provider is only queried at most once per 100ms.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct FocusTracker {
    targets: Vec<String>,
    active: bool,
    last_check: Instant,
    provider: Box<dyn FocusProvider>,
}

impl FocusTracker {
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
            if self.active {
                eprintln!("focus: matched target window");
            } else {
                eprintln!("focus: lost target window");
            }
        }

        self.active
    }
}

pub fn hypr_env() -> Option<HyprEnv> {
    hyprland::resolve_env()
}

/// Auto-detect the compositor and return the appropriate FocusProvider.
pub fn provider(hypr_env: Option<HyprEnv>) -> Box<dyn FocusProvider> {
    // For now, only Hyprland is supported. Future: check $SWAYSOCK, $I3SOCK, etc.
    Box::new(hyprland::HyprlandFocus::new(hypr_env))
}
