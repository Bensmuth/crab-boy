{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.rust-analyzer
    pkgs.gtk4
    pkgs.pkgconfig
    pkgs.gdb

    # keep this line if you use bash
    # pkgs.bashInteractive
  ];
}
