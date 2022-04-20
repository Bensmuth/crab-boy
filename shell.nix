{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.hello
    pkgs.cargo
    pkgs.rustc
    pkgs.rust-analyzer
    pkgs.gtk4
    pkgs.pkgconfig
    pkgs.gdb

    # keep this line if you use bash
    # pkgs.bashInteractive
  ];
}
