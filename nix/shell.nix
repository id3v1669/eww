{pkgs}:
pkgs.mkShell {
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
    killall
    socat
    alsa-utils
    pkg-config
    cargo-audit
    cargo-deny
    cargo-tarpaulin
    clippy
    gdb
    gnumake
    rust-analyzer
    rustfmt
    strace
    zbus-xmlgen
    deno
    mdbook
  ];
}
