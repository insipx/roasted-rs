_:
let
  shell =
    {
      stdenv,
      darwin,
      lib,
      fenix,
      pkg-config,
      avrdude,
      pkgsCross,
      ravedude,
      mkShell,
      cargo-generate,
      local,
      capnproto,
    }:

    let
      inherit (stdenv) isDarwin;
      rust-toolchain = fenix.default.withComponents [
        "rustc"
        "cargo"
      ];

    in
    mkShell {
      nativeBuildInputs = [ pkg-config ];
      buildInputs = [
        rust-toolchain
        fenix.rust-analyzer
        ravedude
        avrdude
        pkgsCross.avr.buildPackages.gcc
        cargo-generate
        local.cargo-ravedude
        capnproto
      ]
      ++ lib.optionals isDarwin [
        darwin.cctools
      ];
    };
in
{
  config.perSystem =
    { pkgs, ... }:
    {
      devShells.default = pkgs.callPackage shell { };
    };
}
