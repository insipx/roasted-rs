{
  lib,
  python3,
  craneLib,
  bootstrapToolchain,
  rustSrc,
  compilerVersion,
  target ? "xtensa-esp32s3-none-elf",
}:
assert lib.assertMsg (
  compilerVersion == rustSrc.version
) "Xtensa rustc and rust-src must come from the same Espressif release";
assert lib.assertMsg (builtins.elem target [
  "xtensa-esp32-none-elf"
  "xtensa-esp32s2-none-elf"
  "xtensa-esp32s3-none-elf"
]) "This component builds core/alloc for bare-metal Xtensa targets only";
let
  cargoVendorDir = craneLib.vendorMultipleCargoDeps {
    cargoLockList = [
      ./dummy/Cargo.lock
      "${rustSrc}/lib/rustlib/src/rust/library/Cargo.lock"
    ];
  };
in
craneLib.mkCargoDerivation {
  pname = "rust-std-${target}";
  version = compilerVersion;
  src = ./dummy;
  inherit cargoVendorDir;
  cargoArtifacts = null;
  doInstallCargoArtifacts = false;
  doIncludeCrossToolchainEnv = false;
  strictDeps = true;
  doCheck = false;

  dontFixup = true;
  nativeBuildInputs = [ python3 ];

  CARGO_BUILD_TARGET = target;
  CARGO_NET_OFFLINE = "true";

  buildPhaseCargoCommand = ''
    cargo build --frozen --release --lib \
      --target ${lib.escapeShellArg target} \
      -Z build-std=core,alloc \
      --message-format=json > cargo-artifacts.json || {
        cat cargo-artifacts.json
        exit 1
      }
  '';

  installPhaseCommand = ''
    python3 ${./install-artifacts.py} \
      cargo-artifacts.json \
      "''${CARGO_TARGET_DIR:-target}/${target}/release" \
      "$out/lib/rustlib/${target}/lib"
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    runHook preInstallCheck
    rustc --sysroot "$out" \
      --target ${lib.escapeShellArg target} \
      --crate-type rlib --edition 2021 \
      -C panic=abort ${./consumer.rs} -o consumer.rlib
    runHook postInstallCheck
  '';

  passthru = {
    inherit target bootstrapToolchain;
  };

  meta.description = "Precompiled core and alloc for ${target}";
}
