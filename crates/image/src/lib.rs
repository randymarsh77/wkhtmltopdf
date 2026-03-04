//! Image output for wkhtmltopdf.
//!
//! This crate implements the `Converter` trait from `wkhtmltopdf-core` to
//! produce raster images from HTML sources, mirroring the role of
//! `ImageConverter` in the C++ codebase.

use thiserror::Error;
use wkhtmltopdf_core::{ConvertError, Converter};
use wkhtmltopdf_settings::ImageGlobal;

/// Errors specific to image conversion.
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("conversion failed: {0}")]
    Conversion(#[from] ConvertError),
}

/// Converts an HTML page to a raster image.
pub struct ImageConverter {
    settings: ImageGlobal,
}

impl ImageConverter {
    /// Create a new `ImageConverter` with the given settings.
    pub fn new(settings: ImageGlobal) -> Self {
        Self { settings }
    }

    /// Return the image settings.
    pub fn settings(&self) -> &ImageGlobal {
        &self.settings
    }
}

impl Converter for ImageConverter {
    fn convert(&self) -> Result<Vec<u8>, ConvertError> {
        // Placeholder – real rendering will delegate to the headless browser
        // backend (e.g. chromiumoxide) once the rendering subsystem is wired up.
        Err(ConvertError::Render(
            "image rendering not yet implemented".into(),
        ))
    }
}
