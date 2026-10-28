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

  # papago-nsh is the same prototype rewritten with Native Shell (nsh.dev).
  # The nsh CLI builds its Rust crate, and every runner it builds, with
  # Cargo from crates.io; on Linux the app draws with GTK 4, where mygo
  # uses GTK 3, over the text and graphics stack the list above already
  # names. Only what GTK 4 itself adds is listed here, and the runners
  # load it through the search path below.
  nsh-gui = with pkgs; [
    gtk4     # libgtk-4, the Native Shell Linux backend (GTK 4.14+)
    graphene # libgraphene-1.0, GTK 4's scene graph
    # What fontconfig itself loads, which both backends share.
    freetype # libfreetype, the fonts fontconfig reads
    expat    # libexpat, fontconfig's XML parser
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
    # papago-nsh: GTK 4, whose .pc files and the packages it propagates
    # (glib, pango, cairo, ...) pkg-config finds for gtk4-sys's build.
    pkgs.gtk4
    # papago-nsh: pkg-config finds GTK 4 for gtk4-sys, clang and libclang
    # bind the C libraries its crates wrap, and patchelf points the
    # prebuilt nsh CLI (a binary for generic Linux, glibc only) at this
    # system's loader.
    pkgs.pkg-config
    pkgs.patchelf
    pkgs.clang
    pkgs.libclang
  ];

  # papago-nsh's language: its crate, the runners and the headless test
  # engine nsh builds with them are Rust.
  languages.rust.enable = true;

  env = {
    # `mygo doctor` looks for GTK with `ldconfig -p`, which on NixOS has no
    # cache to read and so reports it missing; the app finds it through the
    # search path below, and this is the one it uses.
    LD_LIBRARY_PATH = lib.makeLibraryPath (gui ++ nsh-gui);
    XDG_DATA_DIRS = lib.makeSearchPath "share" (gui ++ nsh-gui);
    # Text goes through the fontconfig the app loads, which reads its config
    # from these paths, not the system's, and the config names the fonts it
    # finds beside it.
    FONTCONFIG_FILE = "${pkgs.fontconfig.out}/etc/fonts/fonts.conf";
    FONTCONFIG_PATH = "${pkgs.fontconfig.out}/etc/fonts";
    # Build with the pinned Go, and never download another toolchain.
    GOTOOLCHAIN = "local";
    # The nsh crates that bind C libraries look for libclang.so here.
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  };

  enterShell = ''
    go version
  '';
}
