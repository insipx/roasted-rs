{
  makeRustPlatform,
  fenix,
  fetchCrate,
  pkg-config,
  lib,
  stdenv,
  udev,
  avrdude,
  makeBinaryWrapper,
}:
let

  rust-toolchain = fenix.minimal.toolchain;
  rustPlatform = makeRustPlatform {
    cargo = rust-toolchain;
    rustc = rust-toolchain;
  };
  ravedude-common = {
    pname = "ravedude";
    version = "0.2.2";
  };
in
rustPlatform.buildRustPackage {
  inherit (ravedude-common) pname version;
  nativeBuildInputs = [
    pkg-config
    makeBinaryWrapper
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [ udev ];

  postInstall = ''
    wrapProgram $out/bin/ravedude --suffix PATH : ${lib.makeBinPath [ avrdude ]}
  '';

  src = fetchCrate {
    inherit (ravedude-common) pname version;
    hash = "sha256-Ar2oQx7dKKfzkM3FMcJXiPHxNa0KcMRht38q+NgowfU=";
  };
  cargoHash = "sha256-ME9egPOMTv/nEsmuxI+gJ6Tqa1Vqc/enlPttHXfTdBg=";
}
