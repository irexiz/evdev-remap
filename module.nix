{
  config,
  lib,
  pkgs,
  inputs,
  ...
}:
with lib; let
  cfg = config.services.evdev-remap;

  evdev-remap = pkgs.callPackage ./default.nix {
    hyprland = inputs.hyprland.packages.${pkgs.system}.hyprland;
  };

  remapEntryToToml = entry: ''
    [[rule.remap]]
    ${optionalString (entry.modifier != null) ''modifier = "${entry.modifier}"''}
    input = "${entry.input}"
    output = "${entry.output}"
  '';

  windowClassToToml = classes:
    if classes == []
    then ""
    else ''window_class = [${concatMapStringsSep ", " (c: ''"${c}"'') classes}]'';

  ruleToToml = rule: ''
    [[rule]]
    ${windowClassToToml rule.windowClass}
    ${optionalString (rule.device != null) ''device = "${rule.device}"''}

    ${concatMapStringsSep "\n" remapEntryToToml rule.remap}
  '';

  configFile = pkgs.writeText "evdev-remap.toml" (
    concatMapStringsSep "\n" ruleToToml cfg.rules
  );

  remapEntrySubmodule = types.submodule {
    options = {
      modifier = mkOption {
        type = types.nullOr (types.enum ["ctrl" "alt" "shift"]);
        default = null;
        example = "ctrl";
      };
      input = mkOption {
        type = types.str;
        example = "scroll_up";
      };
      output = mkOption {
        type = types.str;
        example = "mouse_left";
      };
    };
  };

  ruleSubmodule = types.submodule {
    options = {
      windowClass = mkOption {
        type = types.listOf types.str;
        default = [];
        example = ["steam_app_238960"];
        description = "Window classes to match. Empty list means always active.";
      };
      device = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "Logitech USB Receiver";
      };
      remap = mkOption {
        type = types.listOf remapEntrySubmodule;
        default = [];
      };
    };
  };
in {
  options.services.evdev-remap = {
    enable = mkEnableOption "evdev-remap per-window input remapper";

    rules = mkOption {
      type = types.listOf ruleSubmodule;
      default = [];
    };
  };

  config = mkIf cfg.enable {
    services.udev.extraRules = ''
      KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"
      KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input"
    '';

    users.users.izanagi.extraGroups = ["input"];
    boot.kernelModules = ["uinput"];

    systemd.services.evdev-remap = {
      description = "Per-window evdev input remapper";
      after = ["graphical.target"];
      wantedBy = ["graphical.target"];

      serviceConfig = {
        ExecStart = "${evdev-remap}/bin/evdev-remap ${configFile}";
        Restart = "on-failure";
        RestartSec = 3;
      };
    };

    environment.systemPackages = [evdev-remap];
  };
}
