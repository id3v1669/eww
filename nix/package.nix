{ lib
, config
, rustPlatform
, fetchFromGitHub
, pkg-config
, wrapGAppsHook
, gtk3
, librsvg
, gtk-layer-shell
, stdenv
, libdbusmenu-gtk3
, cudaSupport ? config.cudaSupport
, autoAddDriverRunpath
}:
let
  curversion = (builtins.fromTOML (builtins.readFile ./../crates/eww/Cargo.toml)).package.version;
in
rustPlatform.buildRustPackage rec {
  version = "${curversion}-dirty";
  pname = "eww";

  src = lib.cleanSource ./..;

  cargoLock = { lockFile = "${src}/Cargo.lock"; };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook
  ] ++ lib.optionals cudaSupport [
    autoAddDriverRunpath
  ];

  buildInputs = [
    gtk3
    gtk-layer-shell
    libdbusmenu-gtk3
    librsvg
  ];

  buildFeatures = [] ++ lib.optionals cudaSupport [
    "nvidia"
  ];

  cargoBuildFlags = [
    "--bin"
    "eww"
  ];

  cargoTestFlags = cargoBuildFlags;

  RUSTC_BOOTSTRAP = 1;

}
