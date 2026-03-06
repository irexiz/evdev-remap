use super::FocusProvider;
use serde::Deserialize;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::fs;

pub struct HyprlandFocus {
    socket_path: String,
}

#[derive(Deserialize)]
struct ActiveWindow {
    class: Option<String>,
}

impl HyprlandFocus {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }
}

impl FocusProvider for HyprlandFocus {
    fn active_window_class(&mut self) -> Option<String> {
        let mut stream = UnixStream::connect(&self.socket_path).ok()?;
        stream.write_all(b"j/activewindow").ok()?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).ok()?;
        let win: ActiveWindow = serde_json::from_slice(&buf).ok()?;
        win.class
    }
}

pub fn resolve_socket() -> Option<String> {
    // Scan /run/user/*/hypr/*/.socket.sock for any Hyprland instance.
    // Works regardless of uid (root systemd service, sudo, or normal user).
    for entry in fs::read_dir("/run/user").ok()? {
        let user_dir = entry.ok()?.path();
        let hypr_dir = user_dir.join("hypr");
        let Ok(instances) = fs::read_dir(&hypr_dir) else {
            continue;
        };
        for instance in instances.filter_map(|e| e.ok()) {
            let sock = instance.path().join(".socket.sock");
            if sock.exists() {
                let path = sock.to_string_lossy().to_string();
                eprintln!("hyprland: {path}");
                return Some(path);
            }
        }
    }
    None
}
