{
  mkToolchainFor,
  stdenv,
  lib,
  cacert,
  pkg-config,
  local,
}:
let
  rust = mkToolchainFor stdenv.hostPlaform.rust.rustcTarget;
  commonArgs =
    lib.optionalAttrs stdenv.hostPlatform.isMusl {
      RUSTFLAGS = "-C target-feature=+crt-static";
      doCheck = false;
    }
    // {
      src = local.filesets.workspace;
      buildInputs = [ cacert ];
      nativeBuildInputs = [ pkg-config ];
      strictDeps = true;
      CARGO_BUILD_TARGET = stdenv.hostPlatform.rust.rustcTarget;
    };
  cargoArtifacts = rust.buildDepsOnly commonArgs;
in
rust.buildPackage {
  pname = "roasted-daemon";
  src = local.filesets.forCrate ./crates/daemon;
  version = "0.1.0";
  inherit cargoArtifacts;
  cargoExtraArgs = "-p roasted-daemon";
}
