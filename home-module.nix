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
    home.packages = [ cfg.package ];
  };
}
