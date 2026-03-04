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
use std::time::{SystemTime, UNIX_EPOCH};

use printpdf::{
    conformance::PdfConformance, deserialize::PdfWarnMsg, GeneratePdfOptions, PdfDocument,
    serialize::PdfSaveOptions,
};
use thiserror::Error;
use wkhtmltopdf_core::{ConvertError, Converter};
use wkhtmltopdf_settings::{
    HeaderFooter, Orientation, PageSize, PdfAConformance, PdfGlobal, PdfObject, Unit, UnitReal,
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

        // Shared template context for all page objects.
        let date = current_date_string();
        let title = self.global.document_title.clone().unwrap_or_default();

        // Render each page object that has a URL/path specified.
        let mut combined: Option<PdfDocument> = None;
        for object in &self.objects {
            let page_src = match &object.page {
                Some(p) => p,
                None => continue,
            };

            let mut html = fetch_html(page_src)?;

            // Inject header/footer bands if any settings are active.
            let header_band = build_band_html(&object.header, true, &date, &title, page_src)?;
            let footer_band = build_band_html(&object.footer, false, &date, &title, page_src)?;
            if !header_band.is_empty() || !footer_band.is_empty() {
                html = inject_header_footer(
                    &html,
                    &header_band,
                    &footer_band,
                    object.header.spacing,
                    object.footer.spacing,
                );
            }

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
// Header / footer rendering helpers
// ---------------------------------------------------------------------------

/// Return the current date formatted as `YYYY-MM-DD` (UTC) using only `std`.
fn current_date_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Gregorian calendar conversion from Unix timestamp (UTC).
    let mut days = secs / 86400;
    let mut year = 1970u32;
    loop {
        let days_in_year: u64 = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u64; 12] = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u32;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = days + 1;
    format!("{year:04}-{month:02}-{day:02}")
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Substitute template variables in a header/footer text string.
///
/// The following variables are recognised:
///
/// | Variable   | Replacement                                          |
/// |------------|------------------------------------------------------|
/// | `[page]`   | A CSS-counter `<span>` (rendered as current page #). |
/// | `[toPage]` | A CSS-counter `<span>` (rendered as total pages).    |
/// | `[date]`   | Current date in `YYYY-MM-DD` format.                 |
/// | `[title]`  | The document title from [`PdfGlobal`].               |
/// | `[url]`    | The URL/path of the page being rendered.             |
pub fn substitute_vars(text: &str, date: &str, title: &str, url: &str) -> String {
    text.replace("[page]", "<span class='_wk_page'></span>")
        .replace("[toPage]", "<span class='_wk_topage'></span>")
        .replace("[date]", date)
        .replace("[title]", title)
        .replace("[url]", url)
}

/// Build the HTML fragment for a header or footer band.
///
/// Returns an empty string when the band has no visible content (no text,
/// no HTML URL, and no separator line).
pub fn build_band_html(
    hf: &HeaderFooter,
    is_header: bool,
    date: &str,
    title: &str,
    url: &str,
) -> Result<String, ConvertError> {
    let has_text = hf.left.is_some() || hf.center.is_some() || hf.right.is_some();
    let has_html = hf.html_url.is_some();

    if !has_text && !has_html && !hf.line {
        return Ok(String::new());
    }

    let position = if is_header { "top:0" } else { "bottom:0" };
    let border = if hf.line {
        if is_header {
            "border-bottom:1px solid black;"
        } else {
            "border-top:1px solid black;"
        }
    } else {
        ""
    };

    let band_html = if let Some(ref html_url) = hf.html_url {
        // Fetch the HTML-based header/footer, apply variable substitution.
        let raw = fetch_html(html_url)?;
        substitute_vars(&raw, date, title, url)
    } else {
        let left = substitute_vars(hf.left.as_deref().unwrap_or(""), date, title, url);
        let center = substitute_vars(hf.center.as_deref().unwrap_or(""), date, title, url);
        let right = substitute_vars(hf.right.as_deref().unwrap_or(""), date, title, url);
        format!(
            "<div style=\"display:flex;justify-content:space-between;width:100%;\">\
             <span style=\"text-align:left;\">{left}</span>\
             <span style=\"flex:1;text-align:center;\">{center}</span>\
             <span style=\"text-align:right;\">{right}</span>\
             </div>"
        )
    };

    Ok(format!(
        "<div class=\"_wk_band\" style=\"position:fixed;left:0;right:0;{position};\
         {border}font-family:{font};font-size:{size}pt;\
         padding:1mm 5mm;box-sizing:border-box;\">{inner}</div>",
        font = hf.font_name,
        size = hf.font_size,
        inner = band_html,
    ))
}

/// Inject header and footer bands into an HTML document.
///
/// The bands are rendered as `position:fixed` elements so they appear on
/// every printed page.  Body margins are extended by `header_spacing` and
/// `footer_spacing` millimetres respectively to prevent content from being
/// obscured by the bands.
pub fn inject_header_footer(
    html: &str,
    header_band: &str,
    footer_band: &str,
    header_spacing: f32,
    footer_spacing: f32,
) -> String {
    if header_band.is_empty() && footer_band.is_empty() {
        return html.to_string();
    }

    // Estimate a reasonable band height so the body margin clears the band.
    // 8 mm comfortably fits a single text line at the default 12 pt font size
    // (~4.2 mm cap-height + padding).  The caller can widen it via `*_spacing`.
    const BAND_HEIGHT_MM: f32 = 8.0;
    let top_margin = if header_band.is_empty() {
        0.0
    } else {
        BAND_HEIGHT_MM + header_spacing
    };
    let bottom_margin = if footer_band.is_empty() {
        0.0
    } else {
        BAND_HEIGHT_MM + footer_spacing
    };

    let extra = format!(
        "<style>\
         ._wk_page::before{{content:counter(page);}}\
         ._wk_topage::before{{content:counter(pages);}}\
         @page{{counter-increment:page;}}\
         body{{margin-top:{top_margin}mm!important;margin-bottom:{bottom_margin}mm!important;}}\
         </style>\
         {header_band}{footer_band}"
    );

    // Inject immediately after the opening <body …> tag when present.
    if let Some(body_start) = html.find("<body") {
        if let Some(tag_end) = html[body_start..].find('>') {
            let insert_at = body_start + tag_end + 1;
            let mut result = html.to_string();
            result.insert_str(insert_at, &extra);
            return result;
        }
    }

    // Fallback: prepend before </body> if present.
    if let Some(pos) = html.find("</body>") {
        let mut result = html.to_string();
        result.insert_str(pos, &extra);
        return result;
    }

    // Last resort: wrap the whole content.
    format!("<html><head></head><body>{extra}{html}</body></html>")
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

    // -----------------------------------------------------------------------
    // Header / footer helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn substitute_vars_replaces_date_title_url() {
        let result = substitute_vars("Date: [date] Title: [title] URL: [url]", "2024-01-15", "My Doc", "http://example.com");
        assert_eq!(result, "Date: 2024-01-15 Title: My Doc URL: http://example.com");
    }

    #[test]
    fn substitute_vars_replaces_page_with_css_span() {
        let result = substitute_vars("Page [page] of [toPage]", "", "", "");
        assert!(result.contains("_wk_page"), "should contain _wk_page span");
        assert!(result.contains("_wk_topage"), "should contain _wk_topage span");
    }

    #[test]
    fn substitute_vars_leaves_unrecognised_vars_intact() {
        let result = substitute_vars("[unknown]", "d", "t", "u");
        assert_eq!(result, "[unknown]");
    }

    #[test]
    fn build_band_html_empty_when_no_content() {
        let hf = HeaderFooter::default(); // no text, no html_url, line=false
        let result = build_band_html(&hf, true, "d", "t", "u").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn build_band_html_returns_html_when_left_set() {
        let mut hf = HeaderFooter::default();
        hf.left = Some("Left text".into());
        let result = build_band_html(&hf, true, "d", "t", "u").unwrap();
        assert!(result.contains("Left text"));
        assert!(result.contains("position:fixed"));
        assert!(result.contains("top:0"));
    }

    #[test]
    fn build_band_html_footer_positions_at_bottom() {
        let mut hf = HeaderFooter::default();
        hf.right = Some("Footer".into());
        let result = build_band_html(&hf, false, "d", "t", "u").unwrap();
        assert!(result.contains("bottom:0"));
    }

    #[test]
    fn build_band_html_line_adds_border() {
        let mut hf = HeaderFooter::default();
        hf.center = Some("Title".into());
        hf.line = true;
        let header = build_band_html(&hf, true, "d", "t", "u").unwrap();
        assert!(header.contains("border-bottom"));
        let footer = build_band_html(&hf, false, "d", "t", "u").unwrap();
        assert!(footer.contains("border-top"));
    }

    #[test]
    fn build_band_html_applies_font_settings() {
        let mut hf = HeaderFooter::default();
        hf.left = Some("x".into());
        hf.font_name = "Times New Roman".into();
        hf.font_size = 10;
        let result = build_band_html(&hf, true, "d", "t", "u").unwrap();
        assert!(result.contains("Times New Roman"));
        assert!(result.contains("10pt"));
    }

    #[test]
    fn build_band_html_only_line_produces_output() {
        let mut hf = HeaderFooter::default();
        hf.line = true;
        let result = build_band_html(&hf, true, "d", "t", "u").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn inject_header_footer_noop_when_both_empty() {
        let html = "<html><body>content</body></html>";
        let result = inject_header_footer(html, "", "", 0.0, 0.0);
        assert_eq!(result, html);
    }

    #[test]
    fn inject_header_footer_inserts_after_body_tag() {
        let html = "<html><body>content</body></html>";
        let result = inject_header_footer(html, "<div id='hdr'>H</div>", "", 0.0, 0.0);
        // Header should appear inside <body>
        let body_pos = result.find("<body>").unwrap();
        let hdr_pos = result.find("id='hdr'").unwrap();
        assert!(hdr_pos > body_pos, "header should be after <body>");
    }

    #[test]
    fn inject_header_footer_includes_counter_css() {
        let html = "<html><body>content</body></html>";
        let result = inject_header_footer(html, "<div>H</div>", "", 0.0, 0.0);
        assert!(result.contains("_wk_page"));
        assert!(result.contains("_wk_topage"));
    }

    #[test]
    fn inject_header_footer_adjusts_body_margin() {
        let html = "<html><body>content</body></html>";
        let result = inject_header_footer(html, "<div>H</div>", "<div>F</div>", 2.0, 3.0);
        assert!(result.contains("margin-top:"), "should set top margin");
        assert!(result.contains("margin-bottom:"), "should set bottom margin");
    }

    #[test]
    fn inject_header_footer_fallback_without_body_tag() {
        let html = "<p>plain content</p>";
        let result = inject_header_footer(html, "<div>H</div>", "", 0.0, 0.0);
        assert!(result.contains("<p>plain content</p>"));
        assert!(result.contains("<div>H</div>"));
    }

    #[test]
    fn current_date_string_format() {
        let date = current_date_string();
        // Format: YYYY-MM-DD  (10 characters, digits and hyphens)
        assert_eq!(date.len(), 10);
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 4); // year
        assert_eq!(parts[1].len(), 2); // month
        assert_eq!(parts[2].len(), 2); // day
    }
}
