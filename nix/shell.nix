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
    pkg-config
    gdb
    gnumake
    rust-analyzer
    rustfmt
    strace
  ];
}
