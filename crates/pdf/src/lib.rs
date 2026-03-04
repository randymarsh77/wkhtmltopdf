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

use lopdf::Bookmark as LopdfBookmark;
use printpdf::{
    conformance::PdfConformance, deserialize::PdfWarnMsg, GeneratePdfOptions, PdfDocument,
    serialize::PdfSaveOptions,
};
use thiserror::Error;
use wkhtmltopdf_core::{ConvertError, Converter};
use wkhtmltopdf_settings::{
    HeaderFooter, Orientation, PageSize, PdfAConformance, PdfGlobal, PdfObject, TableOfContent,
    Unit, UnitReal,
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
    /// pages.  Objects with `is_table_of_content = true` have their HTML
    /// auto-generated from the headings found in the other page objects.
    /// The resulting pages are assembled into a single document with the
    /// settings from [`PdfGlobal`].
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

        // -----------------------------------------------------------------------
        // TOC support: pre-fetch HTML and collect headings from non-TOC objects.
        // -----------------------------------------------------------------------
        let has_toc = self.objects.iter().any(|o| o.is_table_of_content);

        // Find TOC settings from the first TOC object (used for back-link injection).
        let toc_settings = self.objects.iter().find(|o| o.is_table_of_content).map(|o| &o.toc);
        let toc_depth = toc_settings.map(|t| t.depth).unwrap_or(3);
        let needs_back_links = toc_settings.map(|t| t.back_links).unwrap_or(false);

        // Pre-fetch HTML for all page objects (non-TOC only); TOC objects get None.
        let mut fetched: Vec<Option<String>> = Vec::new();
        for object in &self.objects {
            if object.is_table_of_content {
                fetched.push(None);
            } else if let Some(ref src) = object.page {
                fetched.push(Some(fetch_html(src)?));
            } else {
                fetched.push(None);
            }
        }

        // Collect headings from all non-TOC pages.
        let all_headings: Vec<HeadingEntry> = if has_toc {
            fetched
                .iter()
                .filter_map(|h| h.as_ref())
                .flat_map(|html| extract_headings(html, toc_depth))
                .collect()
        } else {
            Vec::new()
        };

        // Render each page object.  While doing so, track (0-based first page,
        // headings) for objects that participate in the PDF outline.
        let mut combined: Option<PdfDocument> = None;
        let mut outline_entries: Vec<(usize, Vec<HeadingEntry>)> = Vec::new();

        for (object, pre_html) in self.objects.iter().zip(fetched.iter()) {
            let html = if object.is_table_of_content {
                // Generate the TOC HTML from the collected headings.
                generate_toc_html(&all_headings, &object.toc)
            } else {
                match pre_html {
                    Some(h) => {
                        if has_toc && needs_back_links {
                            inject_heading_anchors(h, &all_headings, toc_depth)
                        } else {
                            h.clone()
                        }
                    }
                    None => continue,
                }
            };

            let page_src = object.page.as_deref().unwrap_or("<toc>");

            // Inject header/footer bands if any settings are active.
            let header_band = build_band_html(&object.header, true, &date, &title, page_src)?;
            let footer_band = build_band_html(&object.footer, false, &date, &title, page_src)?;
            let html = if !header_band.is_empty() || !footer_band.is_empty() {
                inject_header_footer(
                    &html,
                    &header_band,
                    &footer_band,
                    object.header.spacing,
                    object.footer.spacing,
                )
            } else {
                html
            };

            // Record the first page index of this object (0-based) for outline use.
            let page_offset = combined.as_ref().map(|c| c.page_count()).unwrap_or(0);

            // Collect headings for the PDF outline from non-TOC objects that opt in.
            if self.global.outline && object.include_in_outline && !object.is_table_of_content {
                if let Some(html_src) = pre_html {
                    let headings = extract_headings(html_src, self.global.outline_depth);
                    if !headings.is_empty() {
                        outline_entries.push((page_offset, headings));
                    }
                }
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
        let pdf_bytes = doc.save(&save_opts, &mut save_warnings);

        // -----------------------------------------------------------------------
        // Outline / bookmarks: embed hierarchical PDF bookmarks when enabled.
        // -----------------------------------------------------------------------
        if self.global.outline && !outline_entries.is_empty() {
            // Re-parse the PDF with lopdf so we can embed a proper outline tree.
            let mut lopdf_doc = lopdf::Document::load_mem(&pdf_bytes)
                .map_err(|e| ConvertError::Render(format!("failed to re-parse PDF for outline: {e}")))?;

            add_outline_to_lopdf(&mut lopdf_doc, &outline_entries);

            // Write XML dump if requested.
            if let Some(ref outline_path) = self.global.dump_outline {
                let xml = build_outline_xml(&outline_entries);
                std::fs::write(outline_path, xml).map_err(ConvertError::Io)?;
            }

            // Re-serialize (lopdf preserves existing streams; no extra compression needed).
            let mut buf: Vec<u8> = Vec::new();
            lopdf_doc
                .save_to(&mut std::io::Cursor::new(&mut buf))
                .map_err(|e| ConvertError::Render(format!("failed to save PDF with outline: {e}")))?;
            return Ok(buf);
        }

        // No outline: also write dump XML if requested (empty outline).
        if let Some(ref outline_path) = self.global.dump_outline {
            let xml = build_outline_xml(&[]);
            std::fs::write(outline_path, xml).map_err(ConvertError::Io)?;
        }

        Ok(pdf_bytes)
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
// TOC generation helpers
// ---------------------------------------------------------------------------

/// A single heading entry extracted from an HTML document.
#[derive(Debug, Clone)]
pub struct HeadingEntry {
    /// Heading level (1 = `<h1>`, 2 = `<h2>`, …, 6 = `<h6>`).
    pub level: u32,
    /// Plain-text content of the heading.
    pub text: String,
    /// Anchor id used for linking (either from an existing `id=""` attribute
    /// or auto-generated from the heading text).
    pub anchor: String,
}

/// Extract heading elements from `html` up to `max_depth` levels (1–6).
///
/// Headings are returned in document order.  If a heading tag already has
/// an `id` attribute, that value is used as the anchor; otherwise a
/// URL-safe slug is derived from the heading text and made unique within
/// the returned list.
pub fn extract_headings(html: &str, max_depth: u32) -> Vec<HeadingEntry> {
    let mut entries: Vec<HeadingEntry> = Vec::new();
    let lower = html.to_ascii_lowercase();
    let max_depth = max_depth.min(6);
    let mut pos = 0;

    while pos < lower.len() {
        // Find the earliest heading tag (h1…h{max_depth}) starting at `pos`.
        let mut best: Option<(usize, u32)> = None;
        for level in 1..=max_depth {
            let tag = format!("<h{}", level);
            if let Some(rel) = lower[pos..].find(&tag) {
                let abs = pos + rel;
                // The character immediately after the tag name must be a
                // delimiter (space, '>', newline) to avoid matching e.g. <h10>.
                let after = lower.as_bytes().get(abs + tag.len()).copied();
                if matches!(
                    after,
                    Some(b'>') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | None
                ) {
                    if best.map_or(true, |(bp, _)| abs < bp) {
                        best = Some((abs, level));
                    }
                }
            }
        }

        let (tag_start, level) = match best {
            Some(b) => b,
            None => break,
        };

        // Find the end of the opening tag.
        let gt_rel = match lower[tag_start..].find('>') {
            Some(o) => o,
            None => break,
        };
        let tag_end = tag_start + gt_rel;
        let opening_tag = &html[tag_start..=tag_end];

        // Find the closing tag and extract inner HTML.
        let close_tag = format!("</h{}>", level);
        let content_start = tag_end + 1;
        if let Some(close_rel) = lower[content_start..].find(&close_tag) {
            let content_end = content_start + close_rel;
            let inner = &html[content_start..content_end];
            let text = strip_html_tags(inner).trim().to_string();

            if !text.is_empty() {
                let anchor = extract_id_attr(opening_tag).unwrap_or_else(|| {
                    make_unique_anchor(&slugify_text(&text), &entries)
                });
                entries.push(HeadingEntry { level, text, anchor });
            }

            pos = content_end + close_tag.len();
        } else {
            pos = tag_start + 1;
        }
    }

    entries
}

/// Inject `id` attributes into heading tags (up to `max_depth`) that do not
/// already have one, using the anchors from `headings` (in document order).
///
/// This ensures that TOC forward-links (`href="#anchor"`) resolve correctly
/// when the TOC and the content are rendered together.
pub fn inject_heading_anchors(html: &str, headings: &[HeadingEntry], max_depth: u32) -> String {
    if headings.is_empty() {
        return html.to_string();
    }

    let lower = html.to_ascii_lowercase();
    let max_depth = max_depth.min(6);
    let mut result = String::with_capacity(html.len() + headings.len() * 24);
    let mut src_pos = 0;
    let mut entry_idx = 0;

    while src_pos < lower.len() && entry_idx < headings.len() {
        // Find the next '<'.
        let rel_lt = match lower[src_pos..].find('<') {
            Some(o) => o,
            None => break,
        };
        let abs_lt = src_pos + rel_lt;

        // Determine whether this '<' opens a heading tag h1…h{max_depth}.
        let mut found_level: Option<u32> = None;
        for level in 1..=max_depth {
            let tag_name = format!("h{}", level);
            let tag_end = abs_lt + 1 + tag_name.len();
            if tag_end <= lower.len() && &lower[abs_lt + 1..tag_end] == tag_name.as_str() {
                let after = lower.as_bytes().get(tag_end).copied();
                if matches!(
                    after,
                    Some(b'>') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | None
                ) {
                    found_level = Some(level);
                    break;
                }
            }
        }

        if let Some(_level) = found_level {
            // Find end of the opening tag.
            let rel_gt = match lower[abs_lt..].find('>') {
                Some(o) => o,
                None => break,
            };
            let abs_gt = abs_lt + rel_gt;
            let opening_tag = &html[abs_lt..=abs_gt];

            // Flush content before this tag.
            result.push_str(&html[src_pos..abs_lt]);

            // Inject `id` only when the tag has none yet.
            if extract_id_attr(opening_tag).is_none() {
                // Push everything up to (not including) the closing '>'.
                result.push_str(&html[abs_lt..abs_gt]);
                result.push_str(&format!(" id=\"{}\"", headings[entry_idx].anchor));
                result.push('>');
            } else {
                result.push_str(opening_tag);
            }

            src_pos = abs_gt + 1;
            entry_idx += 1;
        } else {
            // Not a heading tag – copy the '<' and advance.
            result.push_str(&html[src_pos..=abs_lt]);
            src_pos = abs_lt + 1;
        }
    }

    // Flush any remaining content.
    result.push_str(&html[src_pos..]);
    result
}

/// Generate a self-contained HTML page that renders as a Table of Contents.
///
/// Each entry in `headings` becomes one row with the heading text, an
/// optional dotted fill line, and an indented layout based on the heading
/// level.  When `toc.forward_links` is `true` each entry is wrapped in an
/// `<a href="#anchor">` link.
pub fn generate_toc_html(headings: &[HeadingEntry], toc: &TableOfContent) -> String {
    let indent_per_level: f32 = toc
        .indentation
        .trim_end_matches("em")
        .parse()
        .unwrap_or(1.0);

    let mut rows = String::new();
    for entry in headings {
        let indent = (entry.level.saturating_sub(1) as f32) * indent_per_level;
        // font_scale is applied once per level below h1.
        let scale = toc.font_scale.powi(entry.level.saturating_sub(1) as i32);

        let text_html = escape_html(&entry.text);
        let content = if toc.forward_links {
            format!(
                "<a href=\"#{anchor}\">{text}</a>",
                anchor = escape_html_attr(&entry.anchor),
                text = text_html,
            )
        } else {
            text_html
        };

        let dot_style = if toc.use_dotted_lines { "" } else { "visibility:hidden;" };
        rows.push_str(&format!(
            "<div class=\"_wk_toc_row\" style=\"margin-left:{indent:.2}em;font-size:{scale:.3}em;\">\
             <span class=\"_wk_toc_text\">{content}</span>\
             <span class=\"_wk_toc_dots\" style=\"{dot_style}\"></span>\
             </div>\n",
        ));
    }

    format!(
        "<!DOCTYPE html>\
         <html><head><meta charset=\"UTF-8\"><style>\
         body{{font-family:serif;margin:2em;}}\
         h1._wk_toc_title{{text-align:center;font-size:1.4em;margin-bottom:1em;}}\
         ._wk_toc_row{{display:flex;align-items:baseline;margin-bottom:0.3em;}}\
         ._wk_toc_text{{white-space:nowrap;}}\
         ._wk_toc_dots{{flex:1;border-bottom:1px dotted #000;margin:0 0.3em 0.15em;}}\
         a{{color:inherit;text-decoration:none;}}\
         </style></head><body>\
         <h1 class=\"_wk_toc_title\" id=\"_wk_toc\">{title}</h1>\
         {rows}\
         </body></html>",
        title = escape_html(&toc.caption_text),
        rows = rows,
    )
}

/// Return the default XSLT stylesheet used to render the TOC XML document.
///
/// This is the same default stylesheet that the original wkhtmltopdf uses.
/// It can be printed to stdout via the `--dump-default-toc-xsl` CLI flag so
/// that users can customise it and pass it back with `--xsl-style-sheet`.
pub fn default_toc_xsl() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="2.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:outline="http://wkhtmltopdf.org/outline"
  xmlns="http://www.w3.org/1999/xhtml">
  <xsl:output doctype-public="-//W3C//DTD XHTML 1.0 Strict//EN"
    doctype-system="http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"
    indent="yes" />
  <xsl:template match="outline:outline">
    <html>
      <head>
        <title>Table of Contents</title>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8" />
        <style>
          h1 { text-align: center; font-size: 1.4em; }
          div { margin-top: 1em; }
          div div { margin-left: 1em; font-size: 0.8em; margin-top: 0; }
          span.dotfill {
            display: inline-block;
            border-bottom: 1px dotted black;
            flex: 1;
            margin: 0 0.3em 0.15em;
          }
          a { color: black; text-decoration: none; }
          div.toc-entry { display: flex; align-items: baseline; }
          div.toc-entry a { white-space: nowrap; }
          div.toc-entry span.page { white-space: nowrap; }
        </style>
      </head>
      <body>
        <h1><xsl:value-of select="@title"/></h1>
        <xsl:apply-templates select="outline:item/outline:item"/>
      </body>
    </html>
  </xsl:template>
  <xsl:template match="outline:item">
    <xsl:if test="@title!=''">
      <div class="toc-entry">
        <a>
          <xsl:if test="@link">
            <xsl:attribute name="href"><xsl:value-of select="@link"/></xsl:attribute>
          </xsl:if>
          <xsl:value-of select="@title"/>
        </a>
        <span class="dotfill"></span>
        <span class="page"><xsl:value-of select="@page"/></span>
      </div>
    </xsl:if>
    <div>
      <xsl:apply-templates select="outline:item"/>
    </div>
  </xsl:template>
</xsl:stylesheet>
"#
}

// ---------------------------------------------------------------------------
// TOC generation internal helpers
// ---------------------------------------------------------------------------

/// Strip HTML tags from a string, returning plain text.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

/// Extract the value of the `id` attribute from an HTML opening-tag string.
fn extract_id_attr(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let id_pos = lower.find("id=")?;
    let rest = &tag[id_pos + 3..];
    let value = if rest.starts_with('"') {
        let inner = &rest[1..];
        let end = inner.find('"').unwrap_or(inner.len());
        &inner[..end]
    } else if rest.starts_with('\'') {
        let inner = &rest[1..];
        let end = inner.find('\'').unwrap_or(inner.len());
        &inner[..end]
    } else {
        let end = rest
            .find(|c: char| c.is_ascii_whitespace() || c == '>')
            .unwrap_or(rest.len());
        &rest[..end]
    };
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Convert heading text to a URL-safe slug (lowercase, hyphens).
fn slugify_text(text: &str) -> String {
    let raw: String = text
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    // Collapse consecutive hyphens and strip leading/trailing ones.
    let slug = raw
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "heading".to_string()
    } else {
        slug
    }
}

/// Return a slug that is unique among the anchors already in `existing`.
fn make_unique_anchor(base: &str, existing: &[HeadingEntry]) -> String {
    if !existing.iter().any(|e| e.anchor == base) {
        return base.to_string();
    }
    let mut i = 2usize;
    loop {
        let candidate = format!("{}-{}", base, i);
        if !existing.iter().any(|e| e.anchor == candidate) {
            return candidate;
        }
        i += 1;
    }
}

/// HTML-escape special characters for text content.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// HTML-escape for use inside attribute values.
fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// PDF outline / bookmark helpers
// ---------------------------------------------------------------------------

/// Embed a hierarchical PDF outline (bookmarks) into `lopdf_doc`.
///
/// `outline_entries` is a list of `(first_page_0indexed, headings)` pairs, one
/// entry per rendered page-object that participates in the outline.  Each
/// heading in the list is mapped to a lopdf [`Bookmark`] with the correct
/// parent so that the hierarchy mirrors the HTML heading levels (H1 → H2 →
/// H3, etc.).  After all bookmarks have been added, [`Document::build_outline`]
/// is called to build the PDF outline dictionary.
pub fn add_outline_to_lopdf(
    lopdf_doc: &mut lopdf::Document,
    outline_entries: &[(usize, Vec<HeadingEntry>)],
) {
    // Build a BTreeMap of 1-based page number → lopdf ObjectId.
    let pages: std::collections::BTreeMap<u32, lopdf::ObjectId> = lopdf_doc.get_pages();

    // A stack of `(heading_level, lopdf_bookmark_id)` used to determine the
    // parent of each new bookmark.
    let mut level_stack: Vec<(u32, u32)> = Vec::new();

    for (page_offset, headings) in outline_entries {
        for heading in headings {
            // Convert 0-based page_offset to 1-based lopdf page number.
            let page_num = (*page_offset as u32) + 1;
            let page_obj_id = match pages.get(&page_num) {
                Some(&id) => id,
                None => continue,
            };

            // Pop any stack entries at the same or deeper level so we find the
            // correct parent.
            while level_stack.last().map_or(false, |(l, _)| *l >= heading.level) {
                level_stack.pop();
            }

            let parent_id = level_stack.last().map(|(_, id)| *id);

            let bookmark = LopdfBookmark::new(
                heading.text.clone(),
                [0.0, 0.0, 0.0], // black
                0,               // normal (non-bold, non-italic)
                page_obj_id,
            );
            let bm_id = lopdf_doc.add_bookmark(bookmark, parent_id);
            level_stack.push((heading.level, bm_id));
        }
    }

    lopdf_doc.build_outline();
}

/// Build an XML string describing the PDF outline, compatible with the format
/// expected by the wkhtmltopdf XSL stylesheet (`default_toc_xsl()`).
///
/// The XML uses the `outline` namespace and produces nested
/// `<outline:item>` elements that mirror the heading hierarchy.
pub fn build_outline_xml(outline_entries: &[(usize, Vec<HeadingEntry>)]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <outline xmlns:outline=\"http://wkhtmltopdf.org/outline\">\n",
    );

    // Collect all entries into a flat list of (level, title, page, anchor).
    let flat: Vec<(u32, &str, usize, &str)> = outline_entries
        .iter()
        .flat_map(|(page_offset, headings)| {
            headings
                .iter()
                .map(move |h| (h.level, h.text.as_str(), page_offset + 1, h.anchor.as_str()))
        })
        .collect();

    // Write nested XML by tracking an indent stack.
    let mut open_levels: Vec<u32> = Vec::new();

    for (level, title, page, anchor) in &flat {
        // Close any open levels deeper than the current heading level.
        while open_levels.last().map_or(false, |l| *l >= *level) {
            let indent = "  ".repeat(open_levels.len());
            xml.push_str(&format!("{}</outline:item>\n", indent));
            open_levels.pop();
        }

        let indent = "  ".repeat(open_levels.len() + 1);
        xml.push_str(&format!(
            "{}<outline:item title=\"{}\" page=\"{}\" link=\"#{}\">",
            indent,
            escape_xml_attr(title),
            page,
            escape_xml_attr(anchor),
        ));

        open_levels.push(*level);

        // Peek at the next item: if it's a child, leave this tag open.
        // Otherwise close it immediately (we'll close via the stack above on
        // the next iteration).
        xml.push('\n');
    }

    // Close any remaining open levels.
    while !open_levels.is_empty() {
        open_levels.pop();
        let indent = "  ".repeat(open_levels.len() + 1);
        xml.push_str(&format!("{}</outline:item>\n", indent));
    }

    xml.push_str("</outline>\n");
    xml
}

/// XML-attribute-safe escaping (subset needed for outline XML output).
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

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

    // -----------------------------------------------------------------------
    // TOC generation tests
    // -----------------------------------------------------------------------

    #[test]
    fn extract_headings_basic() {
        let html = "<h1>Chapter One</h1><h2>Section 1.1</h2><h1>Chapter Two</h1>";
        let entries = extract_headings(html, 3);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, 1);
        assert_eq!(entries[0].text, "Chapter One");
        assert_eq!(entries[1].level, 2);
        assert_eq!(entries[1].text, "Section 1.1");
        assert_eq!(entries[2].level, 1);
        assert_eq!(entries[2].text, "Chapter Two");
    }

    #[test]
    fn extract_headings_uses_existing_id() {
        let html = "<h1 id=\"intro\">Introduction</h1>";
        let entries = extract_headings(html, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].anchor, "intro");
    }

    #[test]
    fn extract_headings_generates_anchor_when_no_id() {
        let html = "<h2>Hello World</h2>";
        let entries = extract_headings(html, 3);
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].anchor.is_empty());
        assert_eq!(entries[0].anchor, "hello-world");
    }

    #[test]
    fn extract_headings_depth_limit() {
        let html = "<h1>One</h1><h2>Two</h2><h3>Three</h3><h4>Four</h4>";
        let entries = extract_headings(html, 2);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, 1);
        assert_eq!(entries[1].level, 2);
    }

    #[test]
    fn extract_headings_unique_anchors_for_same_text() {
        let html = "<h1>Section</h1><h2>Section</h2><h3>Section</h3>";
        let entries = extract_headings(html, 6);
        assert_eq!(entries.len(), 3);
        let anchors: Vec<&str> = entries.iter().map(|e| e.anchor.as_str()).collect();
        assert_eq!(anchors[0], "section");
        assert_eq!(anchors[1], "section-2");
        assert_eq!(anchors[2], "section-3");
    }

    #[test]
    fn extract_headings_strips_inner_html() {
        let html = "<h1><strong>Bold <em>Title</em></strong></h1>";
        let entries = extract_headings(html, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "Bold Title");
    }

    #[test]
    fn extract_headings_empty_html() {
        let entries = extract_headings("", 3);
        assert!(entries.is_empty());
    }

    #[test]
    fn extract_headings_no_headings() {
        let html = "<p>Just a paragraph</p><div>Some content</div>";
        let entries = extract_headings(html, 3);
        assert!(entries.is_empty());
    }

    #[test]
    fn inject_heading_anchors_adds_id_where_missing() {
        let html = "<h1>Title</h1>";
        let headings = extract_headings(html, 3);
        let injected = inject_heading_anchors(html, &headings, 3);
        assert!(injected.contains("id=\"title\""), "should contain id='title', got: {injected}");
    }

    #[test]
    fn inject_heading_anchors_preserves_existing_id() {
        let html = "<h1 id=\"my-section\">Title</h1>";
        let headings = extract_headings(html, 3);
        let injected = inject_heading_anchors(html, &headings, 3);
        assert!(injected.contains("id=\"my-section\""));
        // Should not have a duplicate id attribute
        let count = injected.matches("id=").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn inject_heading_anchors_empty_headings_is_noop() {
        let html = "<h1>Title</h1>";
        let result = inject_heading_anchors(html, &[], 3);
        assert_eq!(result, html);
    }

    #[test]
    fn generate_toc_html_contains_title() {
        let toc = wkhtmltopdf_settings::TableOfContent {
            caption_text: "My TOC".into(),
            ..Default::default()
        };
        let html = generate_toc_html(&[], &toc);
        assert!(html.contains("My TOC"));
    }

    #[test]
    fn generate_toc_html_contains_headings() {
        let headings = vec![
            HeadingEntry { level: 1, text: "Chapter 1".into(), anchor: "chapter-1".into() },
            HeadingEntry { level: 2, text: "Section 1.1".into(), anchor: "section-1-1".into() },
        ];
        let toc = wkhtmltopdf_settings::TableOfContent::default();
        let html = generate_toc_html(&headings, &toc);
        assert!(html.contains("Chapter 1"));
        assert!(html.contains("Section 1.1"));
    }

    #[test]
    fn generate_toc_html_dotted_lines_enabled() {
        let headings = vec![HeadingEntry {
            level: 1,
            text: "Title".into(),
            anchor: "title".into(),
        }];
        let toc = wkhtmltopdf_settings::TableOfContent {
            use_dotted_lines: true,
            ..Default::default()
        };
        let html = generate_toc_html(&headings, &toc);
        assert!(
            html.contains("_wk_toc_dots"),
            "dotted-line element missing"
        );
        // The dotted-line span should NOT be hidden
        assert!(!html.contains("visibility:hidden"));
    }

    #[test]
    fn generate_toc_html_dotted_lines_disabled() {
        let headings = vec![HeadingEntry {
            level: 1,
            text: "Title".into(),
            anchor: "title".into(),
        }];
        let toc = wkhtmltopdf_settings::TableOfContent {
            use_dotted_lines: false,
            ..Default::default()
        };
        let html = generate_toc_html(&headings, &toc);
        assert!(html.contains("visibility:hidden"), "dots should be hidden");
    }

    #[test]
    fn generate_toc_html_forward_links_enabled() {
        let headings = vec![HeadingEntry {
            level: 1,
            text: "Intro".into(),
            anchor: "intro".into(),
        }];
        let toc = wkhtmltopdf_settings::TableOfContent {
            forward_links: true,
            ..Default::default()
        };
        let html = generate_toc_html(&headings, &toc);
        assert!(html.contains("href=\"#intro\""));
    }

    #[test]
    fn generate_toc_html_forward_links_disabled() {
        let headings = vec![HeadingEntry {
            level: 1,
            text: "Intro".into(),
            anchor: "intro".into(),
        }];
        let toc = wkhtmltopdf_settings::TableOfContent {
            forward_links: false,
            ..Default::default()
        };
        let html = generate_toc_html(&headings, &toc);
        assert!(!html.contains("href=\"#intro\""));
        assert!(html.contains("Intro"));
    }

    #[test]
    fn generate_toc_html_indentation_increases_per_level() {
        let headings = vec![
            HeadingEntry { level: 1, text: "H1".into(), anchor: "h1".into() },
            HeadingEntry { level: 2, text: "H2".into(), anchor: "h2".into() },
            HeadingEntry { level: 3, text: "H3".into(), anchor: "h3".into() },
        ];
        let toc = wkhtmltopdf_settings::TableOfContent {
            indentation: "1em".into(),
            ..Default::default()
        };
        let html = generate_toc_html(&headings, &toc);
        // h1 has 0 indentation, h2 has 1em, h3 has 2em
        assert!(html.contains("margin-left:0.00em"), "h1 should have 0em indent");
        assert!(html.contains("margin-left:1.00em"), "h2 should have 1em indent");
        assert!(html.contains("margin-left:2.00em"), "h3 should have 2em indent");
    }

    #[test]
    fn generate_toc_html_toc_anchor_in_title() {
        let toc = wkhtmltopdf_settings::TableOfContent::default();
        let html = generate_toc_html(&[], &toc);
        assert!(html.contains("id=\"_wk_toc\""), "TOC title needs back-link anchor");
    }

    #[test]
    fn default_toc_xsl_is_non_empty_xml() {
        let xsl = default_toc_xsl();
        assert!(!xsl.is_empty());
        assert!(xsl.contains("<?xml"), "should start with XML declaration");
        assert!(xsl.contains("xsl:stylesheet"), "should contain stylesheet element");
        assert!(xsl.contains("outline:item"), "should handle outline items");
    }

    #[test]
    fn table_of_content_default_depth_is_3() {
        let toc = wkhtmltopdf_settings::TableOfContent::default();
        assert_eq!(toc.depth, 3);
    }

    #[test]
    fn slugify_text_basic() {
        assert_eq!(slugify_text("Hello World"), "hello-world");
        assert_eq!(slugify_text("  spaces  "), "spaces");
        assert_eq!(slugify_text("Special & Chars!"), "special-chars");
        assert_eq!(slugify_text(""), "heading");
    }

    #[test]
    fn make_unique_anchor_no_conflict() {
        let existing: Vec<HeadingEntry> = vec![];
        assert_eq!(make_unique_anchor("section", &existing), "section");
    }

    #[test]
    fn make_unique_anchor_with_conflict() {
        let existing = vec![
            HeadingEntry { level: 1, text: "S".into(), anchor: "section".into() },
        ];
        assert_eq!(make_unique_anchor("section", &existing), "section-2");
    }

    #[test]
    fn escape_html_escapes_special_chars() {
        assert_eq!(escape_html("<b>Hello & \"World\"</b>"), "&lt;b&gt;Hello &amp; &quot;World&quot;&lt;/b&gt;");
    }

    // -----------------------------------------------------------------------
    // PDF outline / bookmark helper tests
    // -----------------------------------------------------------------------

    #[test]
    fn build_outline_xml_empty() {
        let xml = build_outline_xml(&[]);
        assert!(xml.contains("<?xml"), "should start with XML declaration");
        assert!(xml.contains("<outline"), "should contain outline element");
        assert!(xml.contains("</outline>"), "should close outline element");
        assert!(!xml.contains("outline:item"), "no items for empty input");
    }

    #[test]
    fn build_outline_xml_flat_entries() {
        let entries = vec![(
            0,
            vec![
                HeadingEntry { level: 1, text: "Chapter 1".into(), anchor: "chapter-1".into() },
                HeadingEntry { level: 1, text: "Chapter 2".into(), anchor: "chapter-2".into() },
            ],
        )];
        let xml = build_outline_xml(&entries);
        assert!(xml.contains("title=\"Chapter 1\""));
        assert!(xml.contains("title=\"Chapter 2\""));
        assert!(xml.contains("link=\"#chapter-1\""));
        assert!(xml.contains("page=\"1\""));
    }

    #[test]
    fn build_outline_xml_nested_entries() {
        let entries = vec![(
            0,
            vec![
                HeadingEntry { level: 1, text: "Ch1".into(), anchor: "ch1".into() },
                HeadingEntry { level: 2, text: "Sec1".into(), anchor: "sec1".into() },
                HeadingEntry { level: 1, text: "Ch2".into(), anchor: "ch2".into() },
            ],
        )];
        let xml = build_outline_xml(&entries);
        // All three titles should appear.
        assert!(xml.contains("title=\"Ch1\""));
        assert!(xml.contains("title=\"Sec1\""));
        assert!(xml.contains("title=\"Ch2\""));
        // Sec1 should appear after Ch1 (nested inside).
        let ch1_pos = xml.find("title=\"Ch1\"").unwrap();
        let sec1_pos = xml.find("title=\"Sec1\"").unwrap();
        let ch2_pos = xml.find("title=\"Ch2\"").unwrap();
        assert!(ch1_pos < sec1_pos);
        assert!(sec1_pos < ch2_pos);
    }

    #[test]
    fn build_outline_xml_escapes_special_chars() {
        let entries = vec![(
            0,
            vec![HeadingEntry {
                level: 1,
                text: "Title & <More>".into(),
                anchor: "title-more".into(),
            }],
        )];
        let xml = build_outline_xml(&entries);
        assert!(xml.contains("&amp;"), "ampersand should be escaped");
        assert!(xml.contains("&lt;"), "< should be escaped");
        assert!(xml.contains("&gt;"), "> should be escaped");
    }

    #[test]
    fn build_outline_xml_page_numbers_offset_by_object() {
        // Two objects: first starts at page 0, second at page 5.
        let entries = vec![
            (0, vec![HeadingEntry { level: 1, text: "A".into(), anchor: "a".into() }]),
            (5, vec![HeadingEntry { level: 1, text: "B".into(), anchor: "b".into() }]),
        ];
        let xml = build_outline_xml(&entries);
        assert!(xml.contains("page=\"1\""), "first object page should be 1");
        assert!(xml.contains("page=\"6\""), "second object page should be 6");
    }
}
