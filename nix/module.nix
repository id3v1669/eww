self: {
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.eww;
  ewwCmd = "${lib.getExe cfg.package}";
  inherit (pkgs.stdenv.hostPlatform) system;

  inherit (lib) types literalExpression;
  inherit (lib.modules) mkIf;
  inherit (lib.options) mkOption mkEnableOption;
in
{
  disabledModules = [ "programs/eww.nix" ];
  options.programs.eww = {
    enable = mkEnableOption "Wacky Widgets";

    package = mkOption {
      type = types.package;
      description = "The eww package to install.";
      default = self.packages.${system}.default;
    };
    
    configDir = mkOption {
      type = types.nullOr types.path;
      default = null;
      example = literalExpression "./eww-config-dir";
      description = ''
        The directory that gets symlinked to
        {file}`$XDG_CONFIG_HOME/eww`.
      '';
    };

    enableBashIntegration = mkEnableOption "Bash integration" // {
      default = true;
    };

    enableZshIntegration = mkEnableOption "Zsh integration" // {
      default = true;
    };

    enableFishIntegration = mkEnableOption "Fish integration" // {
      default = true;
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile = mkIf (cfg.configDir != null) {
      "eww".source = cfg.configDir;
    };

    programs.bash.initExtra = mkIf cfg.enableBashIntegration ''
      if [[ $TERM != "dumb" ]]; then
        eval "$(${ewwCmd} shell-completions --shell bash)"
      fi
    '';

    programs.zsh.initExtra = mkIf cfg.enableZshIntegration ''
      if [[ $TERM != "dumb" ]]; then
        eval "$(${ewwCmd} shell-completions --shell zsh)"
      fi
    '';

    programs.fish.interactiveShellInit = mkIf cfg.enableFishIntegration ''
      if test "$TERM" != "dumb"
        eval "$(${ewwCmd} shell-completions --shell fish)"
      end
    '';
  };
}
