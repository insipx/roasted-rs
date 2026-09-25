{ inputs, ... }: {
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  config.perSystem =
    { pkgs, ... }:
    let
      rustfmt = pkgs.fenix.minimal.rustfmt;
    in
    {
      treefmt = {
        flakeFormatter = true;
        flakeCheck = true;
        projectRootFile = "flake.nix";
        nixfmt.enable = true;
        rustfmt = {
          enable = true;
          package = rustfmt;
        };
        taplo.enable = true;
      };
    };
}
