_: {
  config.perSystem = { pkgs, ... }: {
    packages.daemon = pkgs.callPackage ./../crates/daemon { };
  };
}
