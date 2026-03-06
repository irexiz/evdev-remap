# evdev-remap

Per-window mouse and scroll remapping for Linux/Wayland. Remap scroll wheel to clicks, rebind mouse buttons, all scoped to a specific window. Built for repetitive games (Path of Exile, Diablo, Last Epoch) and RSI prevention.

Grabs a physical device via evdev, remaps matched events when the target window is focused, passes everything else through untouched. No X11 dependency, works natively on Wayland.

Currently supports Hyprland for focus detection. The compositor backend is pluggable via a trait.

## Why?

There's plenty of alternatives, but I always found it annoying that I can't specify which window the biding should be active in. I want to use my scroll-wheel in game to click items, but I want the scroll-wheel in my browser on the other screen to just scroll. Also, RSI and carpal tunnel aren't fun to have.

## How it works

1. Grabs exclusive access to your mouse via evdev
2. Creates a virtual device (uinput) that mirrors its capabilities
3. Polls the compositor for the focused window class
4. When the target window is focused, matched inputs are remapped and emitted on the virtual device
5. Unmatched events pass through as-is

No daemon, no root (just `input` group), no latency. One binary, one config file. Strict 1:1 input remapping - one action in, one action out.

## Install

```
cargo install --git https://github.com/irexiz/evdev-remap
```

Requires access to `/dev/input/` and `/dev/uinput` (user in `input` group, `uinput` kernel module loaded).

## Usage

```
evdev-remap /path/to/config.toml

# or from source
cargo run -- /path/to/config.toml
```

## Config

```toml
[[rule]]
window_class = "steam_app_238960"
device = "Logitech USB Receiver"  # optional, auto-detects if omitted

[[rule.remap]]
input = "scroll_up"
output = "mouse_left"

[[rule.remap]]
modifier = "ctrl"
input = "scroll_down"
output = "mouse_left"
```

Inputs: `scroll_up`, `scroll_down`, `mouse_left`, `mouse_right`, `mouse_middle`, `mouse_side`, `mouse_extra`.

Outputs: `mouse_left`, `mouse_right`, `mouse_middle`, `mouse_side`, `mouse_extra`.

Modifiers: `ctrl`, `alt`, `shift` (optional field, omit for no modifier).

Device accepts a name, a substring, a `/dev/input/eventN` path, or omit to auto-detect.

To find your window class (Hyprland):
```
hyprctl activewindow -j | jq .class
```

To find your device name:
```
evtest  # lists all input devices with names and paths
```

## NixOS

Add as a flake input:

```nix
# flake.nix
inputs.evdev-remap = {
  url = "github:irexiz/evdev-remap";
  flake = false;
};
```

Import the module and configure:

```nix
# configuration.nix
imports = [ inputs.evdev-remap + "/module.nix" ];

services.evdev-remap = {
  enable = true;
  rules = [
    {
      windowClass = "steam_app_238960";
      device = "Logitech USB Receiver";
      remap = [
        {
          input = "scroll_up";
          output = "mouse_left";
        }
        {
          input = "scroll_down";
          output = "mouse_left";
        }
        {
          modifier = "ctrl";
          input = "scroll_up";
          output = "mouse_left";
        }
        {
          modifier = "ctrl";
          input = "scroll_down";
          output = "mouse_left";
        }
        {
          modifier = "alt";
          input = "scroll_up";
          output = "mouse_left";
        }
        {
          modifier = "alt";
          input = "scroll_down";
          output = "mouse_left";
        }
      ];
    }
  ];
};
```

The module handles udev rules, kernel modules, and the systemd service.

## Alternatives

- [makima](https://github.com/cyber-sushi/makima) - per-app remapping daemon for keyboards, mice and controllers
- [input-remapper](https://github.com/sezanzeb/input-remapper) - GUI-based, global remapping, no per-window support
- [keyd](https://github.com/rvaiya/keyd) - keyboard-focused, no window-aware remapping
- [xdotool](https://github.com/jordansissel/xdotool) - X11 only, scripting approach
- AutoHotkey (Windows) - this is the Linux equivalent for per-window mouse remapping
