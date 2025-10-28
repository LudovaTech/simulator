# Easy way to enable rust support in NixOS

{  pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {

  packages = with pkgs; [
    cargo
    wayland
    libxkbcommon
    vulkan-loader
  ];

  LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib";

  shellHook = ''
    #export PATH=$PATH:~/.cargo/bin/ # add cargo applications (cross, cross-util...) to path
  '';
}