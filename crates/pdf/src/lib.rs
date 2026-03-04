//! PDF output for wkhtmltopdf.
//!
//! This crate implements the `Converter` trait from `wkhtmltopdf-core` to
//! produce PDF documents from HTML sources, mirroring the role of
//! `PdfConverter` in the C++ codebase.
//!
//! HTML content is fetched from each [`PdfObject`]'s `page` field (either a
//! local file path or an HTTP/HTTPS URL) and rendered to PDF pages using
//! [`printpdf`].  The pages are assembled into a single multi-page document
//! with the size, orientation, margins, compression, and metadata specified
//! in the [`PdfGlobal`] settings.

use std::collections::BTreeMap;

use printpdf::{
    conformance::PdfConformance, deserialize::PdfWarnMsg, GeneratePdfOptions, PdfDocument,
    serialize::PdfSaveOptions,
};
use thiserror::Error;
use wkhtmltopdf_core::{ConvertError, Converter};
use wkhtmltopdf_settings::{
    Orientation, PageSize, PdfAConformance, PdfGlobal, PdfObject, Unit, UnitReal,
};

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// Errors specific to PDF conversion.
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("conversion failed: {0}")]
    Conversion(#[from] ConvertError),
}

// ---------------------------------------------------------------------------
// PdfConverter
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Converter implementation
// ---------------------------------------------------------------------------

impl Converter for PdfConverter {
    /// Render all added page objects to a PDF and return the raw PDF bytes.
    ///
    /// Each [`PdfObject`] whose `page` field is `Some` is fetched (from a
    /// local file or an HTTP/HTTPS URL) and rendered to one or more PDF
    /// pages.  The resulting pages are assembled into a single document with
    /// the settings from [`PdfGlobal`].
    fn convert(&self) -> Result<Vec<u8>, ConvertError> {
        // Determine page dimensions in mm, accounting for orientation.
        let (page_w, page_h) = page_dimensions_mm(&self.global);

        // Build rendering options.
        let opts = GeneratePdfOptions {
            page_width: Some(page_w),
            page_height: Some(page_h),
            margin_top: Some(unit_real_to_mm(&self.global.margin.top)),
            margin_right: Some(unit_real_to_mm(&self.global.margin.right)),
            margin_bottom: Some(unit_real_to_mm(&self.global.margin.bottom)),
            margin_left: Some(unit_real_to_mm(&self.global.margin.left)),
            font_embedding: Some(true),
            ..Default::default()
        };

        // Render each page object that has a URL/path specified.
        let mut combined: Option<PdfDocument> = None;
        for object in &self.objects {
            let page_src = match &object.page {
                Some(p) => p,
                None => continue,
            };

            let html = fetch_html(page_src)?;
            let mut warnings = Vec::new();
            let doc = PdfDocument::from_html(
                &html,
                &BTreeMap::new(),
                &BTreeMap::new(),
                &opts,
                &mut warnings,
            )
            .map_err(|e| ConvertError::Render(e))?;

            match combined.take() {
                None => combined = Some(doc),
                Some(mut existing) => {
                    existing.append_document(doc);
                    combined = Some(existing);
                }
            }
        }

        let mut doc = combined.ok_or_else(|| {
            ConvertError::Render("no pages to convert: add at least one PdfObject with a page URL".into())
        })?;

        // Apply document metadata.
        doc.metadata.info.document_title =
            self.global.document_title.clone().unwrap_or_default();
        doc.metadata.info.author = self.global.author.clone().unwrap_or_default();
        doc.metadata.info.subject = self.global.subject.clone().unwrap_or_default();

        // Apply PDF/A conformance.
        doc.metadata.info.conformance =
            pdf_a_conformance_to_printpdf(self.global.pdf_a_conformance);

        // Serialize to bytes with optional stream compression.
        let save_opts = PdfSaveOptions {
            optimize: self.global.use_compression,
            subset_fonts: true,
            secure: false,
            image_optimization: None,
        };
        let mut save_warnings: Vec<PdfWarnMsg> = Vec::new();
        Ok(doc.save(&save_opts, &mut save_warnings))
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Fetch HTML content from a local file path or an HTTP/HTTPS URL.
fn fetch_html(source: &str) -> Result<String, ConvertError> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let mut response = ureq::get(source)
            .call()
            .map_err(|e| ConvertError::Render(format!("HTTP request failed: {e}")))?;
        response
            .body_mut()
            .read_to_string()
            .map_err(|e| ConvertError::Render(format!("failed to read HTTP response: {e}")))
    } else {
        std::fs::read_to_string(source).map_err(ConvertError::Io)
    }
}

/// Return the page width and height in millimetres, swapping the axes when
/// the orientation is `Landscape`.
fn page_dimensions_mm(global: &PdfGlobal) -> (f32, f32) {
    let (w, h) = named_page_size_mm(global);
    match global.orientation {
        Orientation::Portrait => (w, h),
        Orientation::Landscape => (h, w),
    }
}

/// Return the (width, height) in mm for the named page size, or the custom
/// dimensions when `PageSize::Custom` is selected.
fn named_page_size_mm(global: &PdfGlobal) -> (f32, f32) {
    // If the caller has supplied explicit custom dimensions, honour them.
    if let (Some(w), Some(h)) = (&global.size.width, &global.size.height) {
        return (unit_real_to_mm(w), unit_real_to_mm(h));
    }

    // Standard ISO/ANSI paper sizes (portrait, width × height in mm).
    match global.size.page_size {
        PageSize::A0 => (841.0, 1189.0),
        PageSize::A1 => (594.0, 841.0),
        PageSize::A2 => (420.0, 594.0),
        PageSize::A3 => (297.0, 420.0),
        PageSize::A4 => (210.0, 297.0),
        PageSize::A5 => (148.0, 210.0),
        PageSize::A6 => (105.0, 148.0),
        PageSize::A7 => (74.0, 105.0),
        PageSize::A8 => (52.0, 74.0),
        PageSize::A9 => (37.0, 52.0),
        PageSize::B0 => (1000.0, 1414.0),
        PageSize::B1 => (707.0, 1000.0),
        PageSize::B2 => (500.0, 707.0),
        PageSize::B3 => (353.0, 500.0),
        PageSize::B4 => (250.0, 353.0),
        PageSize::B5 => (176.0, 250.0),
        PageSize::B6 => (125.0, 176.0),
        PageSize::B7 => (88.0, 125.0),
        PageSize::B8 => (62.0, 88.0),
        PageSize::B9 => (44.0, 62.0),
        PageSize::B10 => (31.0, 44.0),
        PageSize::Letter => (215.9, 279.4),
        PageSize::Legal => (215.9, 355.6),
        PageSize::Executive => (184.2, 266.7),
        PageSize::Tabloid | PageSize::Ledger => (279.4, 431.8),
        // Custom without explicit dimensions defaults to A4.
        PageSize::Custom => (210.0, 297.0),
    }
}

/// Convert a [`UnitReal`] length to millimetres.
fn unit_real_to_mm(ur: &UnitReal) -> f32 {
    let v = ur.value as f32;
    match ur.unit {
        Unit::Millimeter => v,
        Unit::Centimeter => v * 10.0,
        Unit::Inch => v * 25.4,
        Unit::Point => v * (25.4 / 72.0),
        Unit::Pica => v * (25.4 / 6.0),
        // Pixels are device-dependent; use 96 dpi as a reasonable default.
        Unit::Pixel => v * (25.4 / 96.0),
    }
}

/// Map our [`PdfAConformance`] enum to printpdf's [`PdfConformance`].
fn pdf_a_conformance_to_printpdf(conformance: PdfAConformance) -> PdfConformance {
    match conformance {
        PdfAConformance::None => PdfConformance::Custom(
            printpdf::conformance::CustomPdfConformance::default(),
        ),
        PdfAConformance::A1b => PdfConformance::A1B_2005_PDF_1_4,
        PdfAConformance::A2b => PdfConformance::A2B_2011_PDF_1_7,
        PdfAConformance::A3b => PdfConformance::A3_2012_PDF_1_7,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wkhtmltopdf_settings::{UnitReal, Unit, PageSize, Orientation};

    #[test]
    fn pdf_converter_new_has_no_objects() {
        let conv = PdfConverter::new(PdfGlobal::default());
        assert!(conv.objects.is_empty());
    }

    #[test]
    fn pdf_converter_add_object_increases_count() {
        let mut conv = PdfConverter::new(PdfGlobal::default());
        conv.add_object(PdfObject::default());
        assert_eq!(conv.objects.len(), 1);
    }

    #[test]
    fn pdf_converter_global_returns_settings() {
        let mut g = PdfGlobal::default();
        g.document_title = Some("Test".into());
        let conv = PdfConverter::new(g);
        assert_eq!(conv.global().document_title.as_deref(), Some("Test"));
    }

    #[test]
    fn convert_returns_error_with_no_objects() {
        let conv = PdfConverter::new(PdfGlobal::default());
        let result = conv.convert();
        assert!(result.is_err());
    }

    #[test]
    fn convert_returns_error_with_object_missing_page() {
        let mut conv = PdfConverter::new(PdfGlobal::default());
        conv.add_object(PdfObject::default()); // page is None
        let result = conv.convert();
        assert!(result.is_err());
    }

    #[test]
    fn unit_real_to_mm_millimeter() {
        let ur = UnitReal { value: 10.0, unit: Unit::Millimeter };
        assert!((unit_real_to_mm(&ur) - 10.0).abs() < 0.001);
    }

    #[test]
    fn unit_real_to_mm_centimeter() {
        let ur = UnitReal { value: 1.0, unit: Unit::Centimeter };
        assert!((unit_real_to_mm(&ur) - 10.0).abs() < 0.001);
    }

    #[test]
    fn unit_real_to_mm_inch() {
        let ur = UnitReal { value: 1.0, unit: Unit::Inch };
        assert!((unit_real_to_mm(&ur) - 25.4).abs() < 0.001);
    }

    #[test]
    fn page_dimensions_portrait_a4() {
        let g = PdfGlobal::default(); // A4, Portrait
        let (w, h) = page_dimensions_mm(&g);
        assert!((w - 210.0).abs() < 0.1);
        assert!((h - 297.0).abs() < 0.1);
    }

    #[test]
    fn page_dimensions_landscape_a4() {
        let mut g = PdfGlobal::default();
        g.orientation = Orientation::Landscape;
        let (w, h) = page_dimensions_mm(&g);
        // Landscape swaps width and height.
        assert!((w - 297.0).abs() < 0.1);
        assert!((h - 210.0).abs() < 0.1);
    }

    #[test]
    fn page_dimensions_letter() {
        let mut g = PdfGlobal::default();
        g.size.page_size = PageSize::Letter;
        let (w, h) = page_dimensions_mm(&g);
        assert!((w - 215.9).abs() < 0.1);
        assert!((h - 279.4).abs() < 0.1);
    }

    #[test]
    fn page_dimensions_custom_explicit() {
        let mut g = PdfGlobal::default();
        g.size.page_size = PageSize::Custom;
        g.size.width = Some(UnitReal { value: 100.0, unit: Unit::Millimeter });
        g.size.height = Some(UnitReal { value: 200.0, unit: Unit::Millimeter });
        let (w, h) = page_dimensions_mm(&g);
        assert!((w - 100.0).abs() < 0.1);
        assert!((h - 200.0).abs() < 0.1);
    }

    #[test]
    fn pdf_a_conformance_none_maps_to_custom() {
        let c = pdf_a_conformance_to_printpdf(PdfAConformance::None);
        assert!(matches!(c, PdfConformance::Custom(_)));
    }

    #[test]
    fn pdf_a_conformance_a1b_maps_correctly() {
        let c = pdf_a_conformance_to_printpdf(PdfAConformance::A1b);
        assert!(matches!(c, PdfConformance::A1B_2005_PDF_1_4));
    }

    #[test]
    fn pdf_a_conformance_a2b_maps_correctly() {
        let c = pdf_a_conformance_to_printpdf(PdfAConformance::A2b);
        assert!(matches!(c, PdfConformance::A2B_2011_PDF_1_7));
    }

    #[test]
    fn pdf_a_conformance_a3b_maps_correctly() {
        let c = pdf_a_conformance_to_printpdf(PdfAConformance::A3b);
        assert!(matches!(c, PdfConformance::A3_2012_PDF_1_7));
    }
}
