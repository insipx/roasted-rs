{ lib, nativeToolchain }:
let
  root = ./../../..;
in
lib.fileset.toSource {
  inherit root;
  fileset = lib.fileset.unions [
    (root + /Cargo.toml)
    (root + /Cargo.lock)
    (nativeToolchain.fileset.commonCargoSources (root + /crates/types))
    (nativeToolchain.fileset.commonCargoSources (root + /crates/audio))
    (nativeToolchain.fileset.commonCargoSources (root + /crates/daemon))
  ];
}
