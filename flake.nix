{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @
  { self
  , nixpkgs
  , systems
  }:
  let
    inherit (nixpkgs) lib;
    pkgsFor = system: import nixpkgs { inherit system; };
    eachSystem = nixpkgs.lib.genAttrs (import systems);
  in 
  {
    packages = eachSystem (system: rec 
    {
      eww = nixpkgs.legacyPackages.${system}.callPackage ./nix/package.nix { };
      eww-wayland = nixpkgs.lib.warn "`eww-wayland` is deprecated. Wayland is support by default. Use `eww` instead." eww;
      default = eww;
    });

    overlays = import ./nix/overlays.nix {inherit self lib;};

    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix { };
    });

    formatter = eachSystem (system: (pkgsFor system).nixfmt);
  };
}
