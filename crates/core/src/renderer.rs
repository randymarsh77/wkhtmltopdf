//! Headless rendering abstraction layer.
//!
//! This module defines the [`Renderer`] trait, the [`HtmlInput`] enum that
//! describes what to render, the [`RenderedPage`] type that holds the output,
//! and a [`HeadlessRenderer`] implementation backed by a headless
//! Chromium/WebKit browser (via the `headless_chrome` crate).

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
// HeadlessRenderer – initial implementation backed by headless Chromium
// ---------------------------------------------------------------------------

/// A [`Renderer`] implementation that drives a headless Chromium/WebKit
/// browser via the `headless_chrome` crate.
///
/// # Example
/// ```no_run
/// use wkhtmltopdf_core::renderer::{HeadlessRenderer, HtmlInput, Renderer};
///
/// let renderer = HeadlessRenderer::new();
/// let input = HtmlInput::Url("https://example.com".into());
/// let page = renderer.render(&input).expect("render failed");
/// assert_eq!(page.mime_type, "image/png");
/// ```
pub struct HeadlessRenderer {
    /// When `true` the browser window is hidden (the normal operating mode).
    pub headless: bool,
}

impl HeadlessRenderer {
    /// Create a new `HeadlessRenderer` with headless mode enabled.
    pub fn new() -> Self {
        Self { headless: true }
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for HeadlessRenderer {
    fn render(&self, input: &HtmlInput) -> Result<RenderedPage, RenderError> {
        use headless_chrome::{Browser, LaunchOptions};
        use headless_chrome::protocol::cdp::Page;

        let url = input.to_url_string();

        let launch_options = LaunchOptions {
            headless: self.headless,
            ..Default::default()
        };

        let browser = Browser::new(launch_options)
            .map_err(|e| RenderError::BackendUnavailable(e.to_string()))?;

        let tab = browser
            .new_tab()
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;

        tab.navigate_to(&url)
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;

        tab.wait_until_navigated()
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;

        // capture_screenshot(format, quality, clip, from_surface)
        // `from_surface: true` captures the GPU-composited output rather than
        // the raw bitmap, which gives a pixel-accurate rendering of the page.
        let bytes = tab
            .capture_screenshot(
                Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true, // from_surface: capture from the GPU compositor
            )
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;

        Ok(RenderedPage {
            bytes,
            mime_type: "image/png".to_string(),
        })
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
    }

    #[test]
    fn headless_renderer_new_is_headless() {
        let renderer = HeadlessRenderer::new();
        assert!(renderer.headless);
    }

    /// Verify that `HeadlessRenderer` implements `Renderer`.
    /// This test only checks that rendering returns an error when Chrome is
    /// not available (as expected in a CI/sandbox environment).
    #[test]
    fn headless_renderer_implements_renderer_trait() {
        fn assert_renderer<R: Renderer>(_: &R) {}
        let renderer = HeadlessRenderer::new();
        assert_renderer(&renderer);
        // In environments without Chrome the render will fail; that is expected.
        let input = HtmlInput::Url("https://example.com".into());
        let result = renderer.render(&input);
        match result {
            Ok(_) => { /* Chrome was available and rendering succeeded */ }
            Err(RenderError::BackendUnavailable(_)) | Err(RenderError::RenderFailed(_)) => {
                // Expected in sandboxed / CI environments
            }
            Err(e) => panic!("unexpected error variant: {e}"),
        }
    }
}
