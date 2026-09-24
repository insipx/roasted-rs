{ lib, ... }:
{
  options.flake.lib.overlays = lib.mkOption {
    type = lib.types.listOf (
      lib.types.functionTo (lib.types.functionTo (lib.types.attrsOf lib.types.raw))
    );
    default = [ ];
  };
}
