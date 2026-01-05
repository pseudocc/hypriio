{ config, lib, pkgs, ... }:

let
  cfg = config.services.hypriio;
in {
  options.services.hypriio = {
    enable = lib.mkEnableOption "Hypriio automatic screen rotation daemon";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.hypriio;
      defaultText = lib.literalExpression "pkgs.hypriio";
      description = "The hypriio package to use.";
    };

    settings.restart-services = lib.mkOption {
      type = with lib.types; listOf str;
      default = [];
      description = "A list of services to restart when orientation changes.";
    };

    settings.output = lib.mkOption {
      type = lib.types.str;
      default = "auto";
      description = "The output to monitor for orientation changes.";
    };

    settings.transforms = lib.mkOption {
      type = with lib.types; listOf int;
      default = [ 0 1 2 3 ];
      description = "hyprland transforms to apply the rotation to.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.hypriio = {
      Unit = {
        Description = "Hypriio automatic screen rotation daemon";
        Documentation = "https://github.com/pseudoc/hypriio";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
      };

      Service = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/hypriio";
        Restart = "on-failure";
        RestartSec = 5;
      };

      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };

    xdg.configFile."hypriio/config.toml".text = let
      toList = list: "[ ${lib.concatStringsSep ", " list} ]";
      transforms = builtins.map toString cfg.settings.transforms;
      restart-services = builtins.map (s: "\"${s}\"") cfg.settings.restart-services;
    in lib.mkIf (cfg.settings != {}) ''
      output = "${cfg.settings.output}"
      transforms = ${toList transforms}
      restart-services = [ ${toList restart-services} ]
    '';

    home.packages = [ cfg.package ];
  };
}
