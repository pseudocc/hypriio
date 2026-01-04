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
    systemd.services.hypriio = {
      description = "Hypriio automatic screen rotation daemon";
      documentation = [ "https://github.com/pseudoc/hypriio" ];

      wants = [ "iio-sensor-proxy.service" ];
      after = [ "iio-sensor-proxy.service" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/hypriio";
        Restart = "on-failure";
        RestartSec = 5;

        # Security hardening
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;

        # Allow access to system bus for iio-sensor-proxy
        # and Hyprland socket
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        RestrictRealtime = true;
      };

      wantedBy = [ "multi-user.target" ];
    };

    # Ensure iio-sensor-proxy is available
    hardware.sensor.iio.enable = lib.mkDefault true;
    environment.systemPackages = [ cfg.package ];
  };
}
