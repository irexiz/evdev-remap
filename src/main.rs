mod config;
mod device;
mod focus;
mod input;
mod remap;

use anyhow::{Context, Result, bail};
use std::{env, fs, thread};

fn main() {
    if let Err(e) = run() {
        eprintln!("{e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/etc/evdev-remap.toml".into());

    let raw = fs::read_to_string(&path).context(path)?;
    let config: config::Config = toml::from_str(&raw).context("config")?;

    if config.rule.is_empty() {
        bail!("no rules defined");
    }

    let socket = focus::resolve_socket();

    if config.rule.len() == 1 {
        let rule = config.rule.into_iter().next().expect("checked non-empty");
        return remap::run(&rule, socket);
    }

    let handles: Vec<_> = config
        .rule
        .into_iter()
        .enumerate()
        .map(|(i, rule)| {
            let socket = socket.clone();
            thread::spawn(move || {
                if let Err(e) = remap::run(&rule, socket) {
                    eprintln!("rule {i}: {e:#}");
                }
            })
        })
        .collect();

    for h in handles {
        if let Err(e) = h.join() {
            eprintln!("thread panicked: {e:?}");
        }
    }

    Ok(())
}
