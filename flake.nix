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
      default = eww;
    });

    overlays = import ./nix/overlays.nix {inherit self lib;};

    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix { };
    });
    
    homeManagerModules.default = import ./nix/module.nix self;
    
    formatter = eachSystem (system: (pkgsFor system).nixfmt);
  };
}
