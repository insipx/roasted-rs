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
    (nativeToolchain.fileset.commonCargoSources (root + /crates/db))
    (nativeToolchain.fileset.commonCargoSources (root + /crates/db-schema))
    (nativeToolchain.fileset.commonCargoSources (root + /crates/data-streams))
    (lib.fileset.fileFilter (file: file.hasExt "snap" || file.hasExt "json") (root + /crates/types))
  ];
}
