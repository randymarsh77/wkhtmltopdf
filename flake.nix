{
  description = "wkhtmltopdf – HTML-to-PDF/image converter (Rust rewrite)";

  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable"; };

  outputs = { self, nixpkgs }:
    let
      supportedSystems =
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          # Wrapper scripts that forward to `cargo run` so the dev-built
          # binaries are always available on $PATH inside the shell.
          wkhtmltopdf-wrapper = pkgs.writeShellScriptBin "wkhtmltopdf" ''
            exec cargo run --quiet --bin wkhtmltopdf -- "$@"
          '';

          wkhtmltoimage-wrapper = pkgs.writeShellScriptBin "wkhtmltoimage" ''
            exec cargo run --quiet --bin wkhtmltoimage -- "$@"
          '';

          # Chromium package: available on Linux; on macOS users must install
          # Chrome themselves (the wrapper will search common paths).
          chromiumPkg = if pkgs.stdenv.isLinux then [ pkgs.chromium ] else [ ];

          # WebKitGTK + GTK 3 (Linux only; macOS uses WebKit.framework from
          # the SDK which is always present).
          webkitPkgs = if pkgs.stdenv.isLinux then [
            pkgs.webkitgtk_4_1 # webkit2gtk-4.1
            pkgs.gtk3
            pkgs.glib
            pkgs.cairo
            pkgs.gdk-pixbuf
            pkgs.pango
          ] else
            [ ];
        in {
          default = pkgs.mkShell {
            packages = [
              # ── Rust toolchain ──────────────────────────────────
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt

              # ── Build tools ─────────────────────────────────────
              pkgs.pkg-config

              # ── CLI wrappers (cargo run) ────────────────────────
              wkhtmltopdf-wrapper
              wkhtmltoimage-wrapper

              # ── Useful extras ───────────────────────────────────
              pkgs.rust-analyzer
              pkgs.cacert
            ] ++ chromiumPkg ++ webkitPkgs;

            shellHook = ''
              ${if pkgs.stdenv.isLinux then ''
                export CHROME_PATH="${pkgs.chromium}/bin/chromium"
              '' else ''
                # macOS: set CHROME_PATH if Chrome is installed in the standard location.
                if [ -z "$CHROME_PATH" ] && [ -x "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" ]; then
                  export CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
                fi
              ''}
              echo "🔧 wkhtmltopdf dev shell"
              echo "   rustc   $(rustc --version)"
              echo "   cargo   $(cargo --version)"
              echo ""
              echo "   wkhtmltopdf  → cargo run --bin wkhtmltopdf"
              echo "   wkhtmltoimage → cargo run --bin wkhtmltoimage"
              echo ""
              echo "   Backends: --backend webkit (default) | chrome | printpdf"
              if [ -n "$CHROME_PATH" ]; then
                echo "   CHROME_PATH=$CHROME_PATH"
              fi
            '';
          };
        });
    };
}
