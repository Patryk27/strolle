{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    alsa-lib
    libxkbcommon
    udev
    vulkan-loader
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  hardeningDisable = [
    "fortify"
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ [ pkgs.stdenv.cc.cc.lib ]);
}
