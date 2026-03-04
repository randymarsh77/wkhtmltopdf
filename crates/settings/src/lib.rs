//! Configuration and settings structs for wkhtmltopdf.
//!
//! This crate mirrors the settings subsystem from the C++ codebase
//! (`src/lib/*settings.*`). Settings are plain structs that are populated
//! by the CLI parser or the public API and then passed into the converters.

use serde::{Deserialize, Serialize};

/// Page margin values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Page orientation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

/// Global PDF conversion settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfGlobal {
    /// Path to the output PDF file.
    pub output: Option<String>,
    pub margin: Margin,
    pub orientation: Orientation,
    /// DPI to use for rendering.
    pub dpi: Option<u32>,
}

/// Per-page PDF settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PdfObject {
    /// URL or path of the HTML page to render.
    pub page: Option<String>,
}

/// Global image conversion settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageGlobal {
    /// URL or path of the HTML page to render.
    pub page: Option<String>,
    /// Path to the output image file.
    pub output: Option<String>,
    /// Output image format (e.g. "png", "jpg").
    pub format: Option<String>,
    /// Image quality (for JPEG output).
    pub quality: Option<u8>,
}

/// Global network/load settings shared by PDF and image converters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadGlobal {
    /// Path to a cookie jar file.
    pub cookie_jar: Option<String>,
}

/// Per-page network/load settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadPage {
    /// Additional milliseconds to wait after the page has loaded.
    pub js_delay: u32,
    /// Zoom factor applied to the page.
    pub zoom: f64,
}

impl Default for LoadPage {
    fn default() -> Self {
        Self {
            js_delay: 200,
            zoom: 1.0,
        }
    }
}
