# Embedded packages and toolchain components.
{ inputs, ... }: {

  config.perSystem =
    { pkgs, inputs', ... }:
    let
      bootstrapToolchain = pkgs.fenix.combine [
        inputs'.esp-nix.packages.esp-rustc
        inputs'.esp-nix.packages.esp-rustc-src
        pkgs.fenix.minimal.cargo
      ];

      bootstrapCrane = (inputs.crane.mkLib pkgs).overrideToolchain bootstrapToolchain;

      xtensaStd = pkgs.callPackage ./_support/xtensa-rust-std {
        craneLib = bootstrapCrane;
        inherit bootstrapToolchain;
        rustSrc = inputs'.esp-nix.packages.esp-rustc-src;
        compilerVersion = inputs'.esp-nix.packages.esp-rustc.version;
      };
      xtensaToolchain = pkgs.fenix.combine [
        xtensaStd
        bootstrapToolchain
      ];
    in
    {
      packages.xtensa-rust-std = xtensaStd;
      packages.zippy = pkgs.riscvPkgs.callPackage ./../crates/zippy { };
      devShells.xtensa = pkgs.mkShell {
        packages = [
          xtensaToolchain
          inputs'.esp-nix.packages.esp-idf-xtensa.tools.xtensa-esp-elf
          pkgs.espflash
          pkgs.qemu-esp32
        ];
      };
    };
}
