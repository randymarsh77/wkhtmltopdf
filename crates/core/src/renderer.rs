//! Headless rendering abstraction layer.
//!
//! This module defines the [`Renderer`] trait, the [`HtmlInput`] enum that
//! describes what to render, the [`RenderedPage`] type that holds the output,
//! and [`HeadlessRenderer`] which implements the renderer interface.
//!
//! When compiled with the `qt-webkit` feature (and Qt WebEngine installed on
//! the build host), [`HeadlessRenderer::render`] delegates to the Qt
//! WebEngine C++ backend.  Without the feature the method returns
//! [`RenderError::BackendUnavailable`].

use std::path::PathBuf;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Input / output types
// ---------------------------------------------------------------------------

/// The HTML source to be rendered: either a remote URL or a local file path.
#[derive(Debug, Clone)]
pub enum HtmlInput {
    /// An HTTP or HTTPS URL.
    Url(String),
    /// A path to a local HTML file.
    File(PathBuf),
}

impl HtmlInput {
    /// Return the URL string that the browser should navigate to.
    ///
    /// For `Url` variants the string is returned as-is.  For `File` variants
    /// the path is canonicalized to an absolute path before being formatted as
    /// a `file://` URL; if canonicalization fails the original path is used as
    /// a best-effort fallback.
    pub fn to_url_string(&self) -> String {
        match self {
            HtmlInput::Url(url) => url.clone(),
            HtmlInput::File(path) => {
                let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
                format!("file://{}", abs.display())
            }
        }
    }
}

/// The output of a successful render operation.
#[derive(Debug, Clone)]
pub struct RenderedPage {
    /// Raw bytes of the rendered output (e.g. a PNG screenshot).
    pub bytes: Vec<u8>,
    /// MIME type of the rendered output (e.g. `"image/png"`).
    pub mime_type: String,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    /// The headless browser backend could not be launched or is unavailable.
    #[error("browser backend unavailable: {0}")]
    BackendUnavailable(String),
    /// The page could not be rendered.
    #[error("render failed: {0}")]
    RenderFailed(String),
    /// An I/O error occurred (e.g. reading a local file).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Renderer trait
// ---------------------------------------------------------------------------

/// Abstraction over a headless HTML rendering engine.
///
/// Implementors accept an [`HtmlInput`] (URL or local file) and return a
/// [`RenderedPage`] containing the raw output bytes together with their MIME
/// type.
pub trait Renderer {
    /// Render the given HTML input and return a page representation.
    fn render(&self, input: &HtmlInput) -> Result<RenderedPage, RenderError>;
}

// ---------------------------------------------------------------------------
// HeadlessRenderer – implementation backed by Qt WebKit/WebEngine
// ---------------------------------------------------------------------------

/// A [`Renderer`] implementation backed by Qt WebEngine when the `qt-webkit`
/// feature is enabled, or a no-op stub otherwise.
///
/// # Feature-gated behaviour
///
/// * **With `qt-webkit`**: [`Renderer::render`] calls into the Qt WebEngine
///   C++ backend (see `webkit_renderer.cpp`).  Qt 5.6+ or Qt 6 with the
///   `WebEngineWidgets` module must be installed on the build host.
/// * **Without `qt-webkit`** (default): [`Renderer::render`] always returns
///   [`RenderError::BackendUnavailable`] after validating the input URL scheme.
///
/// # Example
/// ```
/// use wkhtmltopdf_core::renderer::{HeadlessRenderer, HtmlInput, Renderer, RenderError};
///
/// let renderer = HeadlessRenderer::new();
/// let input = HtmlInput::Url("https://example.com".into());
/// let result = renderer.render(&input);
/// // Without the `qt-webkit` feature the backend is unavailable.
/// #[cfg(not(feature = "qt-webkit"))]
/// assert!(matches!(result, Err(RenderError::BackendUnavailable(_))));
/// ```
pub struct HeadlessRenderer {
    /// When `true` the browser window is hidden (the normal operating mode).
    pub headless: bool,
    /// When `true` the rendering process runs inside the OS sandbox
    /// (Qt's process isolation).  Defaults to `true`.  Only set this to
    /// `false` when running in an environment where the kernel namespaces
    /// required by Qt's sandbox are unavailable (e.g. some container
    /// runtimes); doing so is not recommended in production.
    pub sandbox: bool,
    /// When `false`, JavaScript execution is disabled in the rendered page.
    pub enable_javascript: bool,
    /// Milliseconds to wait after page load before capturing the screenshot.
    pub js_delay: u32,
    /// JavaScript snippets to execute after the page has loaded.
    pub run_scripts: Vec<String>,
}

impl HeadlessRenderer {
    /// Create a new `HeadlessRenderer` with headless mode and sandbox enabled.
    pub fn new() -> Self {
        Self {
            headless: true,
            sandbox: true,
            enable_javascript: true,
            js_delay: 200,
            run_scripts: Vec::new(),
        }
    }

    /// Create a `HeadlessRenderer` with explicit JavaScript settings.
    pub fn with_js_settings(
        enable_javascript: bool,
        js_delay: u32,
        run_scripts: Vec<String>,
    ) -> Self {
        Self {
            headless: true,
            sandbox: true,
            enable_javascript,
            js_delay,
            run_scripts,
        }
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for HeadlessRenderer {
    fn render(&self, input: &HtmlInput) -> Result<RenderedPage, RenderError> {
        // Validate that network URLs use an allowed scheme (http or https).
        if let HtmlInput::Url(url) = input {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(RenderError::RenderFailed(format!(
                    "unsupported URL scheme; only http:// and https:// are allowed: {url}"
                )));
            }
        }

        // When the `qt-webkit` feature is enabled, delegate to the Qt
        // WebEngine backend.  The C++ implementation handles QApplication
        // lifecycle, page loading, and PNG capture internally.
        #[cfg(feature = "qt-webkit")]
        {
            let url_str = input.to_url_string();
            let scripts: Vec<&str> = self.run_scripts.iter().map(String::as_str).collect();
            return crate::qt_webkit::render_url(
                &url_str,
                self.enable_javascript,
                self.js_delay,
                &scripts,
            )
            .map(|bytes| RenderedPage {
                bytes,
                mime_type: "image/png".to_string(),
            })
            .map_err(|e| RenderError::RenderFailed(format!("Qt WebEngine error: {e}")));
        }

        // No browser backend compiled in.
        #[cfg(not(feature = "qt-webkit"))]
        Err(RenderError::BackendUnavailable(
            "headless browser rendering is not implemented; \
             no browser backend is available"
                .into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_input_url_to_url_string() {
        let input = HtmlInput::Url("https://example.com".into());
        assert_eq!(input.to_url_string(), "https://example.com");
    }

    #[test]
    fn html_input_file_to_url_string() {
        let input = HtmlInput::File(PathBuf::from("/tmp/test.html"));
        assert_eq!(input.to_url_string(), "file:///tmp/test.html");
    }

    #[test]
    fn rendered_page_holds_bytes_and_mime_type() {
        let page = RenderedPage {
            bytes: vec![1, 2, 3],
            mime_type: "image/png".to_string(),
        };
        assert_eq!(page.bytes, vec![1, 2, 3]);
        assert_eq!(page.mime_type, "image/png");
    }

    #[test]
    fn headless_renderer_default_is_headless() {
        let renderer = HeadlessRenderer::default();
        assert!(renderer.headless);
        assert!(renderer.sandbox);
        assert!(renderer.enable_javascript);
        assert_eq!(renderer.js_delay, 200);
        assert!(renderer.run_scripts.is_empty());
    }

    #[test]
    fn headless_renderer_new_is_headless() {
        let renderer = HeadlessRenderer::new();
        assert!(renderer.headless);
        assert!(renderer.sandbox);
        assert!(renderer.enable_javascript);
        assert_eq!(renderer.js_delay, 200);
        assert!(renderer.run_scripts.is_empty());
    }

    #[test]
    fn headless_renderer_with_js_settings() {
        let renderer = HeadlessRenderer::with_js_settings(
            false,
            500,
            vec!["console.log('test')".to_string()],
        );
        assert!(renderer.sandbox);
        assert!(!renderer.enable_javascript);
        assert_eq!(renderer.js_delay, 500);
        assert_eq!(renderer.run_scripts, vec!["console.log('test')"]);
    }

    #[test]
    fn headless_renderer_sandbox_can_be_disabled() {
        let mut renderer = HeadlessRenderer::new();
        renderer.sandbox = false;
        assert!(!renderer.sandbox);
    }

    #[test]
    fn render_rejects_non_http_url_scheme() {
        let renderer = HeadlessRenderer::new();
        let result = renderer.render(&HtmlInput::Url("ftp://example.com/file.html".into()));
        assert!(matches!(result, Err(RenderError::RenderFailed(_))));
    }

    #[test]
    fn render_rejects_javascript_url_scheme() {
        let renderer = HeadlessRenderer::new();
        let result = renderer.render(&HtmlInput::Url("javascript:alert(1)".into()));
        assert!(matches!(result, Err(RenderError::RenderFailed(_))));
    }

    /// Verify that `HeadlessRenderer` implements `Renderer`.
    /// This test only checks that rendering returns an error when Chrome is
    /// not available (as expected in a CI/sandbox environment).
    #[test]
    fn headless_renderer_implements_renderer_trait() {
        fn assert_renderer<R: Renderer>(_: &R) {}
        let renderer = HeadlessRenderer::new();
        assert_renderer(&renderer);
        // In environments without Qt WebKit the render will fail; that is expected.
        let input = HtmlInput::Url("https://example.com".into());
        let result = renderer.render(&input);
        match result {
            Ok(_) => { /* Qt WebKit was available and rendering succeeded */ }
            Err(RenderError::BackendUnavailable(_)) | Err(RenderError::RenderFailed(_)) => {
                // Expected in sandboxed / CI environments
            }
            Err(e) => panic!("unexpected error variant: {e}"),
        }
    }
}
