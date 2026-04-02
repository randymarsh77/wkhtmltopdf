fn main() {
    // Declare has_webkit as a valid cfg for downstream check-cfg.
    println!("cargo::rustc-check-cfg=cfg(has_webkit)");
    // When the `qt-webkit` feature is enabled, invoke cxx-qt-build to compile
    // the Qt C++ bridge.  Qt 5.6+ or Qt 6 with the WebEngineWidgets module
    // (plus cmake) must be installed on the build host for this to succeed.
    #[cfg(feature = "qt-webkit")]
    {
        // SAFETY: we only add an include path to the cc::Build; no ABI or
        // linkage invariants are violated.
        unsafe {
            cxx_qt_build::CxxQtBuilder::new()
                // Rust source file containing the #[cxx_qt::bridge] module.
                .file("src/qt_webkit.rs")
                // C++ source file that implements render_url() via QWebEngineView.
                .cpp_file("src/webkit_renderer.cpp")
                // Add the crate-local include directory so that the C++ compiler
                // can find "wkhtmltopdf/webkit_renderer.h".
                .cc_builder(|cc| {
                    cc.include("include");
                })
                // Link against required Qt modules:
                // - Gui: QImage, QPixmap
                // - Widgets: QApplication
                // - WebEngineCore: QWebEnginePage, QWebEngineSettings (Qt 6)
                // - WebEngineWidgets: QWebEngineView
                .qt_module("Gui")
                .qt_module("Widgets")
                .qt_module("WebEngineCore")
                .qt_module("WebEngineWidgets")
                .build();
        }
    }

    // ── Native WebKit rendering backend ──────────────────────────────────
    //
    // macOS:  Always available (WebKit.framework is part of the OS SDK).
    // Linux:  Available when pkg-config finds webkit2gtk-4.1 (or 4.0).
    //
    // Sets the `has_webkit` cfg flag so Rust code can conditionally include
    // the webkit module.

    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/webkit_render_macos.m")
            .include("include")
            .flag("-fobjc-arc")
            // Silence deprecation warnings from older Cocoa APIs.
            .flag("-Wno-deprecated-declarations")
            .compile("webkit_render");

        println!("cargo:rustc-link-lib=framework=WebKit");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-cfg=has_webkit");
    }

    #[cfg(target_os = "linux")]
    {
        // Try webkit2gtk-4.1 first (GNOME 42+), then fall back to 4.0.
        let webkit = pkg_config::Config::new()
            .probe("webkit2gtk-4.1")
            .or_else(|_| pkg_config::Config::new().probe("webkit2gtk-4.0"));

        let gtk = pkg_config::Config::new().probe("gtk+-3.0");

        if let (Ok(wk), Ok(gt)) = (webkit, gtk) {
            let mut build = cc::Build::new();
            build.file("src/webkit_render_linux.c");
            build.include("include");

            for path in wk.include_paths.iter().chain(gt.include_paths.iter()) {
                build.include(path);
            }

            build.compile("webkit_render");

            println!("cargo:rustc-cfg=has_webkit");
        } else {
            // WebKitGTK not found — the `has_webkit` cfg will not be set and
            // the Rust code will fall back to BackendUnavailable.
            println!(
                "cargo:warning=webkit2gtk not found; WebKit backend disabled. \
                 Install webkit2gtk-4.1-dev (or webkit2gtk-4.0-dev) and \
                 libgtk-3-dev to enable it."
            );
        }
    }
}
