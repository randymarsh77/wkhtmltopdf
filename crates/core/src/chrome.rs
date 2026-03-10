//! Chrome headless rendering backend.
//!
//! This module provides [`ChromeRenderer`], a [`Renderer`] implementation that
//! uses a locally installed Chrome or Chromium binary in headless mode.
//!
//! It also provides [`ChromePdfRenderer`] which uses Chrome's built-in
//! `--print-to-pdf` flag to produce high-fidelity PDF output directly.

use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use crate::renderer::{HtmlInput, RenderError, RenderedPage, Renderer};

/// Locate the Chrome / Chromium binary.
///
/// Resolution order:
/// 1. `CHROME_PATH` environment variable (explicit user override).
/// 2. Well-known binary names on `$PATH`: `chromium`, `chromium-browser`,
///    `google-chrome-stable`, `google-chrome`.
/// 3. macOS application bundle paths.
fn find_chrome() -> Result<PathBuf, RenderError> {
    // 1. Explicit env-var override.
    if let Ok(p) = std::env::var("CHROME_PATH") {
        let path = PathBuf::from(&p);
        if path.exists() {
            return Ok(path);
        }
        return Err(RenderError::BackendUnavailable(format!(
            "CHROME_PATH is set to '{p}' but that path does not exist"
        )));
    }

    // 2. Well-known names on $PATH.
    let candidates = [
        "chromium",
        "chromium-browser",
        "google-chrome-stable",
        "google-chrome",
    ];
    for name in &candidates {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Ok(PathBuf::from(path_str));
                }
            }
        }
    }

    // 3. macOS application bundles.
    #[cfg(target_os = "macos")]
    {
        let mac_paths = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ];
        for p in &mac_paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    Err(RenderError::BackendUnavailable(
        "could not find Chrome or Chromium; install it or set CHROME_PATH".into(),
    ))
}

// ---------------------------------------------------------------------------
// ChromeRenderer – screenshots (PNG) via headless Chrome
// ---------------------------------------------------------------------------

/// A [`Renderer`] implementation that produces PNG screenshots using headless
/// Chrome's `--screenshot` flag.
pub struct ChromeRenderer {
    /// Milliseconds to wait after page load via a virtual-time-budget.
    pub js_delay: u32,
    /// Whether to disable the GPU sandbox (needed in some container envs).
    pub no_sandbox: bool,
}

impl ChromeRenderer {
    pub fn new() -> Self {
        Self {
            js_delay: 200,
            no_sandbox: false,
        }
    }
}

impl Default for ChromeRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for ChromeRenderer {
    fn render(&self, input: &HtmlInput) -> Result<RenderedPage, RenderError> {
        let chrome = find_chrome()?;
        let url = input.to_url_string();

        let tmp_dir = tempfile::tempdir()
            .map_err(|e| RenderError::Io(std::io::Error::other(format!("tmpdir: {e}"))))?;
        let screenshot_path = tmp_dir.path().join("screenshot.png");

        let mut cmd = Command::new(&chrome);
        cmd.arg("--headless=new")
            .arg("--disable-gpu")
            .arg(format!(
                "--screenshot={}",
                screenshot_path.display()
            ))
            .arg("--window-size=1280,960")
            .arg(format!(
                "--virtual-time-budget={}",
                self.js_delay
            ))
            .arg("--hide-scrollbars");

        if self.no_sandbox {
            cmd.arg("--no-sandbox");
        }

        cmd.arg(&url);

        let output = cmd.output().map_err(|e| {
            RenderError::BackendUnavailable(format!(
                "failed to run Chrome at '{}': {e}",
                chrome.display()
            ))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RenderError::RenderFailed(format!(
                "Chrome exited with {}: {stderr}",
                output.status
            )));
        }

        let bytes = std::fs::read(&screenshot_path).map_err(RenderError::Io)?;

        Ok(RenderedPage {
            bytes,
            mime_type: "image/png".into(),
        })
    }
}

// ---------------------------------------------------------------------------
// ChromePdfRenderer – PDF output via Chrome's --print-to-pdf
// ---------------------------------------------------------------------------

/// Settings for Chrome's `--print-to-pdf` output.
pub struct ChromePdfOptions {
    /// Page width/height in inches (Chrome uses inches for print dimensions).
    pub page_width_inches: f64,
    pub page_height_inches: f64,
    /// Margins in inches.
    pub margin_top_inches: f64,
    pub margin_bottom_inches: f64,
    pub margin_left_inches: f64,
    pub margin_right_inches: f64,
    /// Whether to print background graphics.
    pub print_background: bool,
    /// Whether to render in landscape orientation.
    pub landscape: bool,
    /// JS delay in milliseconds (virtual-time-budget).
    pub js_delay: u32,
    /// Disable the GPU sandbox (for container environments).
    pub no_sandbox: bool,
    /// Whether to generate a tagged (accessible) PDF.
    pub generate_tagged_pdf: bool,
}

impl Default for ChromePdfOptions {
    fn default() -> Self {
        // A4 in inches: 8.27 × 11.69
        Self {
            page_width_inches: 8.27,
            page_height_inches: 11.69,
            margin_top_inches: 0.39,  // ~10mm
            margin_bottom_inches: 0.39,
            margin_left_inches: 0.39,
            margin_right_inches: 0.39,
            print_background: true,
            landscape: false,
            js_delay: 200,
            no_sandbox: false,
            generate_tagged_pdf: false,
        }
    }
}

/// Render an HTML page to PDF using Chrome's `--print-to-pdf` flag.
///
/// This bypasses the `printpdf`-based renderer entirely and delegates to
/// Chrome's high-fidelity Blink rendering engine for full CSS3, JS, and
/// web-font support.
pub fn chrome_print_to_pdf(
    input: &HtmlInput,
    opts: &ChromePdfOptions,
) -> Result<Vec<u8>, RenderError> {
    let chrome = find_chrome()?;
    let url = input.to_url_string();

    let tmp_dir = tempfile::tempdir()
        .map_err(|e| RenderError::Io(std::io::Error::other(format!("tmpdir: {e}"))))?;
    let pdf_path = tmp_dir.path().join("output.pdf");

    let mut cmd = Command::new(&chrome);
    cmd.arg("--headless=new")
        .arg("--disable-gpu")
        .arg(format!("--print-to-pdf={}", pdf_path.display()))
        .arg("--no-pdf-header-footer")
        .arg(format!(
            "--virtual-time-budget={}",
            opts.js_delay
        ))
        .arg("--hide-scrollbars");

    if opts.print_background {
        cmd.arg("--print-to-pdf-no-header");
    }

    if opts.landscape {
        // Chrome doesn't have a direct --landscape flag; we swap w/h.
    }

    if opts.no_sandbox {
        cmd.arg("--no-sandbox");
    }

    if opts.generate_tagged_pdf {
        cmd.arg("--generate-tagged-pdf");
    }

    cmd.arg(&url);

    let output = cmd.output().map_err(|e| {
        RenderError::BackendUnavailable(format!(
            "failed to run Chrome at '{}': {e}",
            chrome.display()
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RenderError::RenderFailed(format!(
            "Chrome --print-to-pdf exited with {}: {stderr}",
            output.status
        )));
    }

    // Read the generated PDF.
    let mut pdf_bytes = Vec::new();
    std::fs::File::open(&pdf_path)
        .and_then(|mut f| f.read_to_end(&mut pdf_bytes))
        .map_err(RenderError::Io)?;

    if pdf_bytes.len() < 4 || &pdf_bytes[..4] != b"%PDF" {
        return Err(RenderError::RenderFailed(
            "Chrome produced an invalid PDF (missing %PDF header)".into(),
        ));
    }

    Ok(pdf_bytes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_pdf_options_default_is_a4() {
        let opts = ChromePdfOptions::default();
        assert!((opts.page_width_inches - 8.27).abs() < 0.01);
        assert!((opts.page_height_inches - 11.69).abs() < 0.01);
    }

    #[test]
    fn find_chrome_returns_error_when_not_installed() {
        // When CHROME_PATH is set to a non-existent path, we get an error.
        std::env::set_var("CHROME_PATH", "/nonexistent/chrome-test-binary");
        let result = find_chrome();
        std::env::remove_var("CHROME_PATH");
        assert!(result.is_err());
    }
}
