{ lib
, config
, rustPlatform
, fetchFromGitHub
, pkg-config
, wrapGAppsHook3
, gtk3
, librsvg
, gtk-layer-shell
, stdenv
, libdbusmenu-gtk3
, nvidiaSupport ? config.cudaSupport
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
    wrapGAppsHook3
  ] ++ lib.optionals nvidiaSupport [
    autoAddDriverRunpath
  ];

  buildInputs = [
    gtk3
    gtk-layer-shell
    libdbusmenu-gtk3
    librsvg
  ];

  buildFeatures = [] ++ lib.optionals nvidiaSupport [
    "nvidia"
  ];

  cargoBuildFlags = [
    "--bin"
    "eww"
  ];

  cargoTestFlags = cargoBuildFlags;

  RUSTC_BOOTSTRAP = 1;

  meta.mainProgram = "eww";
}
