{ inputs, lib, ... }:
{
  config.flake.lib.overlays = [
    inputs.fenix.overlays.default
  ];
  imports = [
    inputs.pkgs-by-name.flakeModule
  ];
  config.perSystem =
    {
      system,
      config,
      pkgs,
      ...
    }:
    let
      # this p is incredibly important for correct cross-compilation behavior
      # crane splices pkgs according to buildInputs/nativeBuildInputs
      # and the toolchain must be built with the correct package set according to target.
      toolchain = p: p.fenix.minimal.toolchain;
      # make a toolchain for each target in `targets`
      mkToolchain =
        p: targets:
        p.fenix.combine [
          (toolchain p)
          (lib.forEach targets (target: p.fenix.targets."${target}".minimal.rust-std))
        ];
      rust-toolchain = target: p: mkToolchain p [ target ];
      # Make a toolchain for a single target with the x-compile pkgs
      mkToolchainFor =
        final: target: (inputs.crane.mkLib final).overrideToolchain (rust-toolchain target);
      nativeToolchain = mkToolchainFor pkgs pkgs.stdenv.buildPlatform.rust.rustcTarget;

    in
    {

      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [
          inputs.fenix.overlays.default
          (final: prev: {
            # legacyPackages exposes packages as well as derivations
            local = config.legacyPackages;
            inherit mkToolchain nativeToolchain;
            mkToolchainFor = mkToolchainFor final;
          })
        ];
      };
      pkgsDirectory = ./_pkgs;
    };
}
