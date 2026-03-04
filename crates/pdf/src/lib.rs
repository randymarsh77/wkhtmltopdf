//! PDF output for wkhtmltopdf.
//!
//! This crate implements the `Converter` trait from `wkhtmltopdf-core` to
//! produce PDF documents from HTML sources, mirroring the role of
//! `PdfConverter` in the C++ codebase.

use thiserror::Error;
use wkhtmltopdf_core::{ConvertError, Converter};
use wkhtmltopdf_settings::{PdfGlobal, PdfObject};

/// Errors specific to PDF conversion.
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("conversion failed: {0}")]
    Conversion(#[from] ConvertError),
}

/// Converts one or more HTML pages to a PDF document.
pub struct PdfConverter {
    global: PdfGlobal,
    objects: Vec<PdfObject>,
}

impl PdfConverter {
    /// Create a new `PdfConverter` with the given global settings.
    pub fn new(global: PdfGlobal) -> Self {
        Self {
            global,
            objects: Vec::new(),
        }
    }

    /// Add a page object (HTML source) to the conversion.
    pub fn add_object(&mut self, object: PdfObject) {
        self.objects.push(object);
    }

    /// Return the global settings.
    pub fn global(&self) -> &PdfGlobal {
        &self.global
    }
}

impl Converter for PdfConverter {
    fn convert(&self) -> Result<Vec<u8>, ConvertError> {
        // Placeholder – real rendering will delegate to the headless browser
        // backend (e.g. chromiumoxide) once the rendering subsystem is wired up.
        Err(ConvertError::Render(
            "PDF rendering not yet implemented".into(),
        ))
    }
}
