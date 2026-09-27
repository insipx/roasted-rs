{
  mkToolchainFor,
  stdenv,
  cacert,
  pkg-config,
  local,
}:
let
  rust = mkToolchainFor stdenv.hostPlatform.rust.rustcTarget;
  commonArgs = {
    src = local.filesets.workspace;
    buildInputs = [ cacert ];
    nativeBuildInputs = [ pkg-config ];
    strictDeps = true;
    CARGO_BUILD_TARGET = stdenv.hostPlatform.rust.rustcTarget;
  };
  cargoArtifacts = rust.buildDepsOnly commonArgs;
in
rust.buildPackage commonArgs
// {
  pname = "roasted-zippy";
  version = "0.1.0";
  src = local.filesets.forCrate /crates/zippy;
  inherit cargoArtifacts;
  cargoExtraArgs = "-p roasted-zippy";
}
