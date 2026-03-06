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
    target_class: Option<String>,
    active: bool,
    last_check: Instant,
    provider: Box<dyn FocusProvider>,
}

impl FocusTracker {
    pub fn new(window_class: Option<&str>, provider: Box<dyn FocusProvider>) -> Self {
        Self {
            target_class: window_class.map(|s| s.to_lowercase()),
            active: false,
            last_check: Instant::now() - POLL_INTERVAL,
            provider,
        }
    }

    pub fn is_focused(&mut self) -> bool {
        let Some(ref target) = self.target_class else {
            return true;
        };

        if self.last_check.elapsed() < POLL_INTERVAL {
            return self.active;
        }
        self.last_check = Instant::now();

        self.active = self
            .provider
            .active_window_class()
            .map(|c| c.to_lowercase().contains(target))
            .unwrap_or(false);

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
