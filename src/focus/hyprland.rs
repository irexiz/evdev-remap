use super::FocusProvider;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

pub struct Hyprland {
    socket: String,
    buf: Vec<u8>,
}

#[derive(Deserialize)]
struct ActiveWindow {
    class: Option<String>,
}

impl Hyprland {
    pub fn new(socket: String) -> Self {
        Self {
            socket,
            buf: Vec::with_capacity(512),
        }
    }
}

impl FocusProvider for Hyprland {
    fn active_window_class(&mut self) -> Option<String> {
        let mut stream = UnixStream::connect(&self.socket).ok()?;
        stream.write_all(b"j/activewindow").ok()?;
        self.buf.clear();
        stream.read_to_end(&mut self.buf).ok()?;
        let win: ActiveWindow = serde_json::from_slice(&self.buf).ok()?;
        win.class
    }
}

pub fn socket() -> Option<String> {
    // checks /run/user/*/hypr/*/.socket.sock for hyprland instances
    // should work regardless of uid - hacky, but the alternatives with libc are flakier
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
