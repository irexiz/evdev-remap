{rustPlatform}:
rustPlatform.buildRustPackage {
  pname = "evdev-remap";
  version = "0.2.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "Per-window evdev input remapper for Wayland (Hyprland, i3, sway)";
    mainProgram = "evdev-remap";
  };
}
