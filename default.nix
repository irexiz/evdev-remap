{
  rustPlatform,
  makeWrapper,
  hyprland,
  lib,
}:
rustPlatform.buildRustPackage {
  pname = "evdev-remap";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [makeWrapper];

  postInstall = ''
    wrapProgram $out/bin/evdev-remap \
      --prefix PATH : ${lib.makeBinPath [hyprland]}
  '';

  meta = {
    description = "Per-window evdev input remapper for Wayland (Hyprland)";
    mainProgram = "evdev-remap";
  };
}
