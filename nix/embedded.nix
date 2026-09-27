# cross packages for riscv
_: {

  config.perSystem =
    { pkgs, ... }:
    {
      packages.zippy = pkgs.riscvPkgs.callPackage ./../crates/zippy { };
    };
}
