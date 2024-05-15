{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, wrapGAppsHook
, gtk3
, librsvg
, gtk-layer-shell
, stdenv
, libdbusmenu-gtk3
}:
let
  curversion = (builtins.fromTOML (builtins.readFile ./crates/eww/Cargo.toml)).package.version;
in
rustPlatform.buildRustPackage rec {
  version = "${curversion}-dirty";
  pname = "eww";

  src = lib.cleanSource ./.;

  cargoLock = { lockFile = "${src}/Cargo.lock"; };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook
  ];

  buildInputs = [
    gtk3
    gtk-layer-shell
    libdbusmenu-gtk3
    librsvg
  ];

  cargoBuildFlags = [
    "--bin"
    "eww"
  ];

  cargoTestFlags = cargoBuildFlags;

  RUSTC_BOOTSTRAP = 1;

}
