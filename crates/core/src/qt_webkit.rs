//! Qt WebKit/WebEngine rendering backend.
//!
//! This module is compiled only when the `qt-webkit` Cargo feature is enabled.
//! It exposes a single function, [`render_url`], which delegates to the C++
//! implementation in `webkit_renderer.cpp` via a `cxx` FFI bridge.
//!
//! # Feature flag
//!
//! Enable the backend by activating the `qt-webkit` feature:
//!
//! ```toml
//! [dependencies]
//! wkhtmltopdf-core = { version = "0.13", features = ["qt-webkit"] }
//! ```
//!
//! Qt 5.6+ or Qt 6 with the `WebEngineWidgets` module (plus `cmake`) must be
//! installed on the build host.

#[cxx_qt::bridge(namespace = "wkhtmltopdf")]
mod ffi {
    unsafe extern "C++" {
        include!("wkhtmltopdf/webkit_renderer.h");

        /// Render the HTML page at `url` using Qt WebEngine and return the
        /// result as raw PNG bytes.
        ///
        /// `run_scripts` contains JavaScript snippets that are evaluated in
        /// the page context after the page has loaded and the settle delay has
        /// elapsed.  Each script runs sequentially and the call blocks until
        /// each one completes before the next starts.
        ///
        /// # Errors
        ///
        /// Propagates any C++ exception thrown by the implementation as a Rust
        /// `Err` (e.g. when the page fails to load).
        fn render_url(
            url: &str,
            js_enabled: bool,
            js_delay_ms: u32,
            run_scripts: &[&str],
        ) -> Result<Vec<u8>>;
    }
}

pub use ffi::render_url;
