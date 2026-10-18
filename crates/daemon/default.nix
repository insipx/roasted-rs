{
  mkToolchainFor,
  stdenv,
  lib,
  cacert,
  pkg-config,
  sqlite,
  local,
}:
let
  rust = mkToolchainFor stdenv.hostPlatform.rust.rustcTarget;
  commonArgs =
    lib.optionalAttrs stdenv.hostPlatform.isMusl {
      RUSTFLAGS = "-C target-feature=+crt-static";
      doCheck = false;
    }
    // {
      src = local.filesets.workspace;
      buildInputs = [
        cacert
        sqlite
      ];
      nativeBuildInputs = [ pkg-config ];
      strictDeps = true;
      CARGO_BUILD_TARGET = stdenv.hostPlatform.rust.rustcTarget;
      cargoExtraArgs = "--workspace --exclude roasted-zippy";
    };
  cargoArtifacts = rust.buildDepsOnly commonArgs;
in
rust.buildPackage (
  commonArgs
  // {
    pname = "roasted-daemon";
    src = local.filesets.forCrate /crates/daemon;
    version = "0.1.0";
    inherit cargoArtifacts;
    cargoExtraArgs = "-p roasted-daemon";
  }
)
