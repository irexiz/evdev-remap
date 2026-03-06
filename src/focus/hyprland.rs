use super::FocusProvider;
use serde::Deserialize;
use std::process::{Command, Stdio};
use std::{env, fs};

/// Focus provider for the Hyprland compositor.
/// Queries window focus via `hyprctl activewindow -j`.
pub struct HyprlandFocus {
    env: Option<HyprEnv>,
}

/// Pre-resolved socket env vars for when the process doesn't inherit them
/// (e.g. running as a systemd service).
#[derive(Clone)]
pub struct HyprEnv {
    pub signature: String,
    pub runtime_dir: String,
}

#[derive(Deserialize)]
struct ActiveWindow {
    class: Option<String>,
}

impl HyprlandFocus {
    pub fn new(env: Option<HyprEnv>) -> Self {
        Self { env }
    }
}

impl FocusProvider for HyprlandFocus {
    fn active_window_class(&mut self) -> Option<String> {
        let mut cmd = Command::new("hyprctl");
        cmd.args(["activewindow", "-j"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        if let Some(env) = &self.env {
            cmd.env("HYPRLAND_INSTANCE_SIGNATURE", &env.signature);
            cmd.env("XDG_RUNTIME_DIR", &env.runtime_dir);
        }

        let out = cmd.output().ok()?;
        let win: ActiveWindow = serde_json::from_slice(&out.stdout).ok()?;
        win.class
    }
}

/// Locate the Hyprland instance signature from the filesystem.
/// Only needed when HYPRLAND_INSTANCE_SIGNATURE isn't already in the environment.
pub fn resolve_env() -> Option<HyprEnv> {
    if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return None;
    }

    let uid = unsafe { libc::getuid() };
    let runtime_dir = format!("/run/user/{uid}");
    let hypr_dir = format!("{runtime_dir}/hypr");

    let signature = fs::read_dir(&hypr_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.file_name().to_string_lossy().to_string())?;

    Some(HyprEnv {
        signature,
        runtime_dir,
    })
}
