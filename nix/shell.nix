_:
let
  shell =
    {
      stdenv,
      darwin,
      lib,
      fenix,
      pkg-config,
      mkShell,
      cargo-generate,
      local,
      diesel-cli,
      sqlite,
      espflash,
      ldproxy,
      picocom,
    }:

    let
      inherit (stdenv.hostPlatform) isDarwin;
      rust-toolchain = fenix.default.toolchain;
    in
    mkShell {
      nativeBuildInputs = [ pkg-config ];
      buildInputs = [
        rust-toolchain
        fenix.rust-analyzer
        cargo-generate
        local.cargo-espflash
        espflash
        ldproxy
        diesel-cli
        sqlite
        picocom
      ]
      ++ lib.optionals isDarwin [
        darwin.cctools
      ];
      shellHook = ''
        export TMP_DIR="$(mktemp -d /tmp/nix-shell-XXXXXX)"
        mkdir $TMP_DIR/sqlite
        export DATABASE_URL="$TMP_DIR/sqlite/roasted.db";
        echo "temporary diesel database directory at $DATABASE_URL"
      '';
    };
in
{
  config.perSystem =
    { pkgs, ... }:
    {
      devShells.default = pkgs.callPackage shell { };
    };
}
