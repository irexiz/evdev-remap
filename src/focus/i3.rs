use super::FocusProvider;
use serde::Deserialize;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::{env, fs};

// i3/sway IPC: 6-byte magic + u32 payload length + u32 message type
const I3_IPC_MAGIC: &[u8] = b"i3-ipc";
// GET_TREE returns the whole layout tree
const GET_TREE: u32 = 4;

pub struct I3 {
    socket: String,
    buf: Vec<u8>,
}

#[derive(Deserialize)]
struct Node {
    #[serde(default)]
    focused: bool,
    window_properties: Option<WindowProperties>,
    #[serde(default)]
    nodes: Vec<Node>,
    #[serde(default)]
    floating_nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct WindowProperties {
    class: Option<String>,
}

impl I3 {
    pub fn new(socket: String) -> Self {
        Self {
            socket,
            buf: Vec::with_capacity(8192),
        }
    }
}

impl FocusProvider for I3 {
    fn active_window_class(&mut self) -> Option<String> {
        let mut stream = UnixStream::connect(&self.socket).ok()?;

        // magic + payload_len=0 + type=GET_TREE
        let mut req = [0u8; 14];
        req[..6].copy_from_slice(I3_IPC_MAGIC);
        req[10..14].copy_from_slice(&GET_TREE.to_ne_bytes());
        stream.write_all(&req).ok()?;

        // Response header: magic, payload_len, type
        let mut header = [0u8; 14];

        stream.read_exact(&mut header).ok()?;
        let (_, rest) = header.split_first_chunk::<6>()?;
        let (len_bytes, _) = rest.split_first_chunk::<4>()?;
        let len = u32::from_ne_bytes(*len_bytes) as usize;

        self.buf.resize(len, 0);
        stream.read_exact(&mut self.buf).ok()?;

        let tree: Node = serde_json::from_slice(&self.buf).ok()?;
        find_focused(&tree)
    }
}

fn find_focused(node: &Node) -> Option<String> {
    if node.focused {
        if let Some(class) = node
            .window_properties
            .as_ref()
            .and_then(|w| w.class.clone())
        {
            return Some(class);
        }
    }
    for child in node.nodes.iter().chain(&node.floating_nodes) {
        if let Some(class) = find_focused(child) {
            return Some(class);
        }
    }
    None
}

pub fn socket() -> Option<String> {
    if let Ok(path) = env::var("I3SOCK") {
        if fs::metadata(&path).is_ok() {
            eprintln!("i3: {path}");
            return Some(path);
        }
    }

    if let Ok(path) = env::var("SWAYSOCK") {
        if fs::metadata(&path).is_ok() {
            eprintln!("sway: {path}");
            return Some(path);
        }
    }

    for entry in fs::read_dir("/run/user").ok()? {
        let user_dir = entry.ok()?.path();
        let i3_dir = user_dir.join("i3");

        let Ok(entries) = fs::read_dir(&i3_dir) else {
            continue;
        };

        for e in entries.filter_map(|e| e.ok()) {
            let name = e.file_name();
            if name.to_string_lossy().starts_with("ipc-socket") {
                let path = e.path().to_string_lossy().to_string();
                eprintln!("i3: {path}");
                return Some(path);
            }
        }
    }

    None
}
