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
    pkgsFor = system: import nixpkgs { inherit system; }; #overlays
    
    eachSystem = nixpkgs.lib.genAttrs (import systems);
  in 
  {
    packages = eachSystem (system: rec 
    {
      eww = nixpkgs.legacyPackages.${system}.callPackage ./eww.nix { };

      eww-wayland = nixpkgs.lib.warn
        "`eww-wayland` is deprecated due to eww building with both X11 and wayland support by default. Use `eww` instead."
        eww;
        
      default = eww;
    });


    devShells = eachSystem (system:
    let
      pkgs = pkgsFor system;
    in
    {
      default = pkgs.mkShell {
        name = "eww-devel";

        nativeBuildInputs = with pkgs; [
          # Compilers
          cargo
          rustc
          scdoc

          # Req
          gtk3
          gtk-layer-shell
          libdbusmenu-gtk3
          librsvg

          # Tools
          pkg-config
          gdb
          gnumake
          rust-analyzer
          rustfmt
          strace
        ];
      };
    });

    formatter = eachSystem (system: (pkgsFor system).nixfmt);
  };
}
