{ pkgs, lib, config, inputs, ... }:

let
  # MyGo's Linux backend opens every library it uses by name at run time
  # (purego, no cgo): libglib-2.0.so.0 and libgdk-3.so.0 as much as
  # libgtk-3.so.0. Each one has to be on the search path, and nix keeps a lib
  # directory per package, so they are named one by one: GTK 3 holds the
  # window and GtkGLArea draws it with OpenGL, and the rest is text,
  # accessibility, the tray, secrets and sound. A window of web pages, or the
  # updater's, also needs webkitgtk_4_1.
  gui = with pkgs; [
    glib                    # libglib-2.0, libgobject-2.0, libgio-2.0
    gtk3                    # libgtk-3, libgdk-3
    gdk-pixbuf              # libgdk_pixbuf-2.0
    pango                   # libpango-1.0, libpangocairo, libpangoft2
    cairo
    harfbuzz
    fontconfig              # libfontconfig
    atk                     # accessibility
    libepoxy                # OpenGL for GtkGLArea
    libGL                   # libGL, libEGL
    gtk-layer-shell         # panels and launchers
    libayatana-appindicator # the tray icon
    libsecret               # secrets
    alsa-lib                # sound, raw ALSA
    libpulseaudio           # sound, through the desktop
  ];
in
{
  packages = [
    pkgs.git
    # MyGo needs Go 1.27 or later; nixpkgs' default is 1.26.
    pkgs.go_1_27
    # fc-list and fc-match, for the text the app draws.
    pkgs.fontconfig
    # React+Tailwind design prototype (prototype/), not the product.
    pkgs.nodejs_22
    pkgs.bun
  ];

  env = {
    # `mygo doctor` looks for GTK with `ldconfig -p`, which on NixOS has no
    # cache to read and so reports it missing; the app finds it through the
    # search path below, and this is the one it uses.
    LD_LIBRARY_PATH = lib.makeLibraryPath gui;
    XDG_DATA_DIRS = lib.makeSearchPath "share" gui;
    # Text goes through the fontconfig the app loads, which reads its config
    # from these paths, not the system's, and the config names the fonts it
    # finds beside it.
    FONTCONFIG_FILE = "${pkgs.fontconfig.out}/etc/fonts/fonts.conf";
    FONTCONFIG_PATH = "${pkgs.fontconfig.out}/etc/fonts";
    # Build with the pinned Go, and never download another toolchain.
    GOTOOLCHAIN = "local";
  };

  enterShell = ''
    go version
  '';
}
