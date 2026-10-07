{ inputs, ... }:
{
  imports = [
    inputs.pkgs-by-name.flakeModule
  ];
  config.perSystem =
    {
      system,
      config,
      pkgs,
      lib,
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
          (lib.forEach targets (target: p.fenix.targets."${target}".latest.rust-std))
        ];
      rust-toolchain = target: p: mkToolchain p [ target ];
      # Make a toolchain for a single target with the x-compile pkgs
      mkToolchainFor =
        final: target: (inputs.crane.mkLib final).overrideToolchain (rust-toolchain target);
      nativeToolchain = mkToolchainFor pkgs pkgs.stdenv.buildPlatform.rust.rustcTarget;
      commonOverlays = [
        inputs.fenix.overlays.default
        inputs.esp-qemu.overlays.default
        (final: prev: {
          # legacyPackages exposes packages as well as derivations
          local = config.legacyPackages;
          inherit mkToolchain nativeToolchain;
          mkToolchainFor = mkToolchainFor final;
        })
      ];
      riscvPkgs = import inputs.nixpkgs {
        inherit system;
        overlays = commonOverlays;
        crossSystem = lib.systems.examples.riscv32-embedded // {
          rustc.config = "riscv32imc-unknown-none-elf";
        };
      };
    in
    {
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = commonOverlays ++ [
          (final: prev: {
            inherit riscvPkgs;
          })
        ];
      };
      pkgsDirectory = ./_pkgs;
    };
}
