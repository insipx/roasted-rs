{
  makeRustPlatform,
  fenix,
  fetchCrate,
  pkg-config,
  lib,
  stdenv,
  udev,
  espflash,
  makeBinaryWrapper,
}:
let

  rust-toolchain = fenix.minimal.toolchain;
  rustPlatform = makeRustPlatform {
    cargo = rust-toolchain;
    rustc = rust-toolchain;
  };
  espflash-common = {
    pname = "cargo-espflash";
    version = "4.6.0";
  };
in
rustPlatform.buildRustPackage {
  inherit (espflash-common) pname version;
  nativeBuildInputs = [
    pkg-config
    makeBinaryWrapper
  ];

  buildInputs = [ espflash ] ++ lib.optionals stdenv.hostPlatform.isLinux [ udev ];

  src = fetchCrate {
    inherit (espflash-common) pname version;
    hash = "sha256-dLtEd1pc0MPlWnuVkT6tdf1oDtG4VQrEiAEpg8ai5CE=";
  };
  cargoHash = "sha256-egPIBJ+TgAR21Pw1WQzvYLmrQdZW3gmmt8b+kf2B4QU=";
}
