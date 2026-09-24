{ inputs, ... }:
{
  config.flake.lib.overlays = [
    inputs.fenix.overlays.default
  ];
  imports = [
    inputs.pkgs-by-name.flakeModule
  ];
  config.perSystem =
    { system, config, ... }:
    {

      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [
          inputs.fenix.overlays.default
          (final: prev: {
            local = config.packages;
          })
        ];
      };
      pkgsDirectory = ./_pkgs;
    };
}
