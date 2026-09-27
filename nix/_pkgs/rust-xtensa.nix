# This is unused
# i will put the rustc xtensa toolchain in nix in the future, however.
{
  lib,
  stdenv,
  fetchurl,
}:

let
  version = "1.97.0.0";
  host = stdenv.buildPlatform.rust.rustcTarget;
  base = "https://github.com/esp-rs/rust-build/releases/download/v${version}";

  compiler = fetchurl {
    url = "${base}/rust-${version}-${host}.tar.xz";
    sha256 = "430fcbf54967e99d16debe48926a1df558ab8b67af3c060a658d25ce752bd790";
  };

  sources = fetchurl {
    url = "${base}/rust-src-${version}.tar.xz";
    sha256 = "568d688b9f8f332ec4d04657544fad23e99ce11e9e6cf5835979e68a28c68b73";
  };
in
stdenv.mkDerivation {
  pname = "rust-xtensa";
  inherit version;

  dontUnpack = true;
  dontConfigure = true;
  dontBuild = true;
  dontStrip = true;

  installPhase = ''
    runHook preInstall

    tar -xf ${compiler}
    tar -xf ${sources}

    bash rust-nightly-${host}/install.sh \
      --prefix="$out" \
      --without=rust-docs,rust-docs-json-preview \
      --disable-ldconfig

    bash rust-src-nightly/install.sh \
      --prefix="$out" \
      --disable-ldconfig

    runHook postInstall
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    "$out/bin/rustc" --version
    "$out/bin/cargo" --version
    test -f "$out/lib/rustlib/src/rust/library/core/src/lib.rs"
  '';

  meta.platforms = [
    "aarch64-darwin"
    "x86_64-linux"
    "aarch64-linux"
  ];
}
