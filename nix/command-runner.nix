{
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib) mkOption types mkIf;
  cfg = config.services.command-runner;
in {
  options = {
    services.command-runner = {
      enable = lib.mkEnableOption "CommandRunner server for running commands over HTTP";

      package = lib.mkPackageOption pkgs "command-runner" {};

      host = mkOption {
        type = types.str;
        default = "0.0.0.0";
        description = "The host address the command-runner server should listen on.";
      };

      port = mkOption {
        type = types.port;
        default = 5599;
        description = "The port the command-runner server should listen on.";
      };

      openFirewall = mkOption {
        type = types.bool;
        default = false;
        description = "Open ports in the firewall for the command-runner server.";
      };

      commands = mkOption {
        type = lib.types.attrsOf (types.listOf types.str);
        default = {
          # "display_on" = ["hyprctl" "dispatch" "dpms" "on"];
          # "display_off" = ["hyprctl" "dispatch" "dpms"];
        };
        description = ''
          List of commands to register with the command-runner server.
          Each command should be a string in the format "name:command".
          For example: {
              "display_on" = ["hyprctl" "dispatch" "dpms" "on"];
              "display_off" = ["hyprctl" "dispatch" "dpms"];
          }
        '';
      };

      database = {
        path = mkOption {
          type = types.nullOr types.str;
          default = "/var/lib/command-runner/db";
          example = "/var/lib/command-runner/commands.db";
          description = ''
            URI to the database.
          '';
        };
      };
    };
  };

  config = mkIf cfg.enable {
    systemd.services.command-runner = {
      description = "command-runner server";
      after = [
        "network-online.target"
      ];
      wants = [
        "network-online.target"
      ];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package} run";
        RuntimeDirectory = "command-runner";
        RuntimeDirectoryMode = "0700";
        User = "command-runner";
        Group = "command-runner";

        # Hardening
        CapabilityBoundingSet = "";
        LockPersonality = true;
        NoNewPrivileges = true;
        MemoryDenyWriteExecute = true;
        PrivateDevices = true;
        PrivateMounts = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProcSubset = "pid";
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "full";
        RemoveIPC = true;
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
          # Required for connecting to database sockets,
          "AF_UNIX"
        ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
        ];
        UMask = "0077";
      };

      environment =
        {
          CMD_RUNNER_HOST = cfg.host;
          CMD_RUNNER_PORT = toString cfg.port;
          # CMD-RUNNER_MAX_HISTORY_LENGTH = toString cfg.maxHistoryLength;
          # CMD-RUNNER_OPEN_REGISTRATION = lib.boolToString cfg.openRegistration;
          # CMD-RUNNER_PATH = cfg.path;
          # CMD-RUNNER_CONFIG_DIR = "/run/command-runner"; # required to start, but not used as configuration is via environment variables
        }
        // lib.optionalAttrs (cfg.database.path != null) {CMD_RUNNER_DATABASE = cfg.database.path;};
    };
    systemd.services.command-runner-commands = {
      description = "command-runner commands";
      after = ["command-runner.service"];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        User = "command-runner";
        Group = "command-runner";
        RuntimeDirectory = "command-runner";
        RuntimeDirectoryMode = "0700";
      };

      script = let
        commands = lib.concatStringsSep "\n" (
          lib.mapAttrsToList (name: value: "${lib.getExe cfg.package} add ${name} ${lib.concatStringsSep " " value}") cfg.commands
        );
      in ''
        ${commands}
      '';
    };

    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [cfg.port];
  };
}
