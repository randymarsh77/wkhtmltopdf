fn main() {
    // When the `qt-webkit` feature is enabled, invoke cxx-qt-build to compile
    // the Qt C++ bridge.  Qt 5 or Qt 6 (plus cmake) must be installed on the
    // build host for this to succeed.
    #[cfg(feature = "qt-webkit")]
    {
        cxx_qt_build::CxxQtBuilder::new().build();
    }
}
