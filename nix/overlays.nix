{ self
, lib
}:
{
  default = lib.composeManyExtensions [
    (final: prev: {
      eww = final.callPackage ./package.nix { };
      eww-wayland = final.lib.warn "`eww-wayland` is deprecated. Wayland is support by default. Use `eww` instead." final.eww;
    })
  ];
}