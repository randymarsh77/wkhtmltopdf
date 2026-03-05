fn main() {
    // When the `qt-webkit` feature is enabled, invoke cxx-qt-build to compile
    // the Qt C++ bridge.  Qt 5.6+ or Qt 6 with the WebEngineWidgets module
    // (plus cmake) must be installed on the build host for this to succeed.
    #[cfg(feature = "qt-webkit")]
    {
        cxx_qt_build::CxxQtBuilder::new()
            // C++ source file that implements render_url() via QWebEngineView.
            .file("src/webkit_renderer.cpp")
            // Add the crate-local include directory so that the C++ compiler
            // can find "wkhtmltopdf/webkit_renderer.h".
            .cc_builder(|cc| {
                cc.include("include");
            })
            // Link against the Qt WebEngineWidgets module.
            .qt_module("WebEngineWidgets")
            .build();
    }
}
