//! PDF structural diffing.
//!
//! This module compares two PDF documents beyond visual appearance by examining
//! their structural properties: metadata (title, author, subject, …), page
//! count, per-page text content, and bookmark outline.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use wkhtmltopdf_diff::pdf_diff::{extract_pdf_structure, diff_pdf_structure};
//!
//! let reference_pdf: Vec<u8> = std::fs::read("reference.pdf").unwrap();
//! let actual_pdf: Vec<u8>    = std::fs::read("actual.pdf").unwrap();
//!
//! let reference = extract_pdf_structure(&reference_pdf).unwrap();
//! let actual    = extract_pdf_structure(&actual_pdf).unwrap();
//!
//! let result = diff_pdf_structure(&reference, &actual);
//!
//! println!("Page count matches: {}", result.page_count_matches);
//! println!("Metadata diffs: {}", result.metadata_diffs.len());
//! println!("Outline matches: {}", result.outline_matches);
//! ```

use thiserror::Error;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// Errors returned by [`extract_pdf_structure`].
#[derive(Debug, Error)]
pub enum PdfDiffError {
    /// Failed to parse the PDF document.
    #[error("failed to parse PDF: {0}")]
    Parse(String),
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single entry from the PDF outline (bookmarks / table-of-contents).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    /// Nesting depth, starting from 1 for top-level entries.
    pub level: usize,
    /// The visible title of the bookmark.
    pub title: String,
    /// 1-based page number the bookmark points to.
    pub page: usize,
}

/// Structural information extracted from a PDF document.
#[derive(Debug, Clone)]
pub struct PdfStructure {
    // ---- metadata fields ----
    /// Document title (from the PDF Info dictionary).
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Document subject.
    pub subject: Option<String>,
    /// Document keywords.
    pub keywords: Option<String>,
    /// Application that originally created the document.
    pub creator: Option<String>,
    /// PDF producer (library/tool that wrote the file).
    pub producer: Option<String>,
    /// PDF creation date string (raw PDF date format).
    pub creation_date: Option<String>,
    /// PDF modification date string (raw PDF date format).
    pub modification_date: Option<String>,
    /// PDF version string (e.g. `"1.4"`).
    pub version: String,

    // ---- structural fields ----
    /// Total number of pages.
    pub page_count: u32,
    /// Per-page extracted text content (index 0 → page 1).
    ///
    /// Text extraction is best-effort; an empty string is stored for pages
    /// where extraction fails.
    pub page_texts: Vec<String>,
    /// Flattened list of outline (bookmark) entries in document order.
    ///
    /// Empty when the document has no outline.
    pub outline: Vec<OutlineEntry>,
}

/// A difference in a single metadata field between the reference and actual
/// PDF documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiff {
    /// Name of the field that differs (e.g. `"title"`, `"author"`).
    pub field: &'static str,
    /// Value in the reference document (`None` means the field was absent).
    pub reference: Option<String>,
    /// Value in the actual document (`None` means the field was absent).
    pub actual: Option<String>,
}

/// A difference in the text content of a single page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageTextDiff {
    /// 1-based page number.
    pub page: u32,
    /// Extracted text from the reference document.
    pub reference_text: String,
    /// Extracted text from the actual document.
    pub actual_text: String,
}

/// The result of comparing two [`PdfStructure`] values.
#[derive(Debug, Clone)]
pub struct PdfStructureDiffResult {
    /// Metadata fields that differ between the two documents.
    ///
    /// Fields that are identical in both documents are *not* included.
    pub metadata_diffs: Vec<FieldDiff>,

    /// `true` when both documents have the same page count.
    pub page_count_matches: bool,
    /// Page count of the reference document.
    pub reference_page_count: u32,
    /// Page count of the actual document.
    pub actual_page_count: u32,

    /// Pages whose extracted text content differs.
    ///
    /// Pages that share identical text are *not* included.  When the two
    /// documents have different page counts only the overlapping range is
    /// compared.
    pub page_text_diffs: Vec<PageTextDiff>,

    /// `true` when both documents have identical outline structures.
    pub outline_matches: bool,
    /// Outline entries from the reference document.
    pub reference_outline: Vec<OutlineEntry>,
    /// Outline entries from the actual document.
    pub actual_outline: Vec<OutlineEntry>,
}

impl PdfStructureDiffResult {
    /// Returns `true` when every compared structural aspect is identical.
    pub fn is_identical(&self) -> bool {
        self.metadata_diffs.is_empty()
            && self.page_count_matches
            && self.page_text_diffs.is_empty()
            && self.outline_matches
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Parse a PDF document and extract its structural information.
///
/// `bytes` must be the raw bytes of a PDF file.
///
/// Text extraction is attempted for every page.  If extraction fails for a
/// page (e.g. the page contains only images), an empty string is stored for
/// that page and processing continues.
///
/// Outline extraction is attempted once.  If the document has no outline, or
/// if extraction fails, an empty outline is stored.
pub fn extract_pdf_structure(bytes: &[u8]) -> Result<PdfStructure, PdfDiffError> {
    // --- metadata (lightweight parse, no stream decoding) ---
    let meta = lopdf::Document::load_metadata_mem(bytes)
        .map_err(|e| PdfDiffError::Parse(format!("metadata parse error: {e}")))?;

    let title = meta.title;
    let author = meta.author;
    let subject = meta.subject;
    let keywords = meta.keywords;
    let creator = meta.creator;
    let producer = meta.producer;
    let creation_date = meta.creation_date;
    let modification_date = meta.modification_date;
    let version = meta.version;

    // --- full document load for text and outline ---
    let doc = lopdf::Document::load_mem(bytes)
        .map_err(|e| PdfDiffError::Parse(format!("document parse error: {e}")))?;

    // Derive the page count and page numbers from the document itself so that
    // `page_texts.len()` is always consistent with `page_count`.
    let doc_pages = doc.get_pages();
    let page_count = doc_pages.len() as u32;

    // Per-page text extraction (one call per page to keep per-page granularity).
    let page_texts = {
        let mut texts = Vec::with_capacity(page_count as usize);
        let mut page_numbers_sorted: Vec<u32> = doc_pages.keys().copied().collect();
        page_numbers_sorted.sort_unstable();
        for pn in page_numbers_sorted {
            let text = doc.extract_text(&[pn]).unwrap_or_default();
            texts.push(text);
        }
        texts
    };

    // Outline / bookmarks.
    let outline = match doc.get_toc() {
        Ok(toc) => toc
            .toc
            .into_iter()
            .map(|entry| OutlineEntry {
                level: entry.level,
                title: entry.title,
                page: entry.page,
            })
            .collect(),
        // No outline or parse error → empty outline.
        Err(_) => Vec::new(),
    };

    Ok(PdfStructure {
        title,
        author,
        subject,
        keywords,
        creator,
        producer,
        creation_date,
        modification_date,
        version,
        page_count,
        page_texts,
        outline,
    })
}

/// Compare two [`PdfStructure`] values and return a [`PdfStructureDiffResult`].
///
/// This function never fails: structural mismatches are recorded in the
/// returned result rather than returned as errors.
pub fn diff_pdf_structure(
    reference: &PdfStructure,
    actual: &PdfStructure,
) -> PdfStructureDiffResult {
    // --- metadata diffs ---
    let mut metadata_diffs = Vec::new();

    macro_rules! check_field {
        ($field:ident, $name:literal) => {
            if reference.$field != actual.$field {
                metadata_diffs.push(FieldDiff {
                    field: $name,
                    reference: reference.$field.clone(),
                    actual: actual.$field.clone(),
                });
            }
        };
    }

    check_field!(title, "title");
    check_field!(author, "author");
    check_field!(subject, "subject");
    check_field!(keywords, "keywords");
    check_field!(creator, "creator");
    check_field!(producer, "producer");

    // `version` is a non-optional String, so handle it separately.
    if reference.version != actual.version {
        metadata_diffs.push(FieldDiff {
            field: "version",
            reference: Some(reference.version.clone()),
            actual: Some(actual.version.clone()),
        });
    }

    // Dates are intentionally excluded from the diff because the PDF
    // creation/modification date is expected to change on every render.

    // --- page count ---
    let page_count_matches = reference.page_count == actual.page_count;

    // --- text diffs (overlapping pages only) ---
    let compared_pages = reference.page_count.min(actual.page_count) as usize;
    let mut page_text_diffs = Vec::new();
    for i in 0..compared_pages {
        let ref_text = reference
            .page_texts
            .get(i)
            .map(String::as_str)
            .unwrap_or("");
        let act_text = actual.page_texts.get(i).map(String::as_str).unwrap_or("");
        if normalize_text(ref_text) != normalize_text(act_text) {
            page_text_diffs.push(PageTextDiff {
                page: (i as u32) + 1,
                reference_text: ref_text.to_owned(),
                actual_text: act_text.to_owned(),
            });
        }
    }

    // --- outline ---
    let outline_matches = reference.outline == actual.outline;

    PdfStructureDiffResult {
        metadata_diffs,
        page_count_matches,
        reference_page_count: reference.page_count,
        actual_page_count: actual.page_count,
        page_text_diffs,
        outline_matches,
        reference_outline: reference.outline.clone(),
        actual_outline: actual.outline.clone(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalise extracted text for comparison by collapsing whitespace.
///
/// PDF text extraction can produce varying amounts of whitespace depending on
/// font metrics and spacing operators.  Normalizing before comparison reduces
/// false positives caused by cosmetic whitespace differences.
fn normalize_text(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // normalize_text
    // -----------------------------------------------------------------------

    #[test]
    fn normalize_collapses_whitespace() {
        assert_eq!(normalize_text("hello   world\n\t foo"), "hello world foo");
    }

    #[test]
    fn normalize_empty_string_stays_empty() {
        assert_eq!(normalize_text(""), "");
    }

    #[test]
    fn normalize_leading_trailing_whitespace() {
        assert_eq!(normalize_text("  hello  "), "hello");
    }

    // -----------------------------------------------------------------------
    // extract_pdf_structure – invalid input
    // -----------------------------------------------------------------------

    #[test]
    fn extract_structure_invalid_pdf_returns_error() {
        let garbage = b"this is not a PDF";
        let result = extract_pdf_structure(garbage);
        assert!(result.is_err(), "expected an error for invalid PDF bytes");
    }

    // -----------------------------------------------------------------------
    // diff_pdf_structure – direct struct construction
    // -----------------------------------------------------------------------

    fn make_structure(
        title: Option<&str>,
        author: Option<&str>,
        page_count: u32,
        page_texts: Vec<&str>,
        outline: Vec<OutlineEntry>,
    ) -> PdfStructure {
        PdfStructure {
            title: title.map(str::to_owned),
            author: author.map(str::to_owned),
            subject: None,
            keywords: None,
            creator: None,
            producer: None,
            creation_date: None,
            modification_date: None,
            version: "1.4".into(),
            page_count,
            page_texts: page_texts.into_iter().map(str::to_owned).collect(),
            outline,
        }
    }

    #[test]
    fn diff_identical_structures_is_identical() {
        let s = make_structure(Some("Title"), Some("Author"), 1, vec!["hello"], vec![]);
        let result = diff_pdf_structure(&s, &s);
        assert!(result.is_identical());
    }

    #[test]
    fn diff_detects_title_change() {
        let reference = make_structure(Some("Old Title"), None, 1, vec!["text"], vec![]);
        let actual = make_structure(Some("New Title"), None, 1, vec!["text"], vec![]);
        let result = diff_pdf_structure(&reference, &actual);
        assert!(result.metadata_diffs.iter().any(|d| d.field == "title"));
        let diff = result
            .metadata_diffs
            .iter()
            .find(|d| d.field == "title")
            .unwrap();
        assert_eq!(diff.reference.as_deref(), Some("Old Title"));
        assert_eq!(diff.actual.as_deref(), Some("New Title"));
    }

    #[test]
    fn diff_detects_author_change() {
        let reference = make_structure(None, Some("Alice"), 1, vec!["text"], vec![]);
        let actual = make_structure(None, Some("Bob"), 1, vec!["text"], vec![]);
        let result = diff_pdf_structure(&reference, &actual);
        assert!(result.metadata_diffs.iter().any(|d| d.field == "author"));
    }

    #[test]
    fn diff_no_false_positive_when_metadata_identical() {
        let s = make_structure(Some("Same"), Some("Same Author"), 1, vec!["text"], vec![]);
        let result = diff_pdf_structure(&s, &s);
        assert!(!result.metadata_diffs.iter().any(|d| d.field == "title"));
        assert!(!result.metadata_diffs.iter().any(|d| d.field == "author"));
    }

    #[test]
    fn diff_detects_page_count_mismatch() {
        let reference = make_structure(None, None, 1, vec!["p1"], vec![]);
        let actual = make_structure(None, None, 2, vec!["p1", "p2"], vec![]);
        let result = diff_pdf_structure(&reference, &actual);
        assert!(!result.page_count_matches);
        assert_eq!(result.reference_page_count, 1);
        assert_eq!(result.actual_page_count, 2);
    }

    #[test]
    fn diff_page_count_matches_for_same_page_count() {
        let s = make_structure(None, None, 2, vec!["p1", "p2"], vec![]);
        let result = diff_pdf_structure(&s, &s);
        assert!(result.page_count_matches);
    }

    #[test]
    fn diff_detects_text_change_on_page() {
        let reference = make_structure(None, None, 1, vec!["hello world"], vec![]);
        let actual = make_structure(None, None, 1, vec!["goodbye world"], vec![]);
        let result = diff_pdf_structure(&reference, &actual);
        assert_eq!(result.page_text_diffs.len(), 1);
        assert_eq!(result.page_text_diffs[0].page, 1);
    }

    #[test]
    fn diff_same_text_produces_no_text_diffs() {
        let s = make_structure(None, None, 1, vec!["identical content"], vec![]);
        let result = diff_pdf_structure(&s, &s);
        assert!(result.page_text_diffs.is_empty());
    }

    #[test]
    fn diff_text_whitespace_normalized() {
        // Texts that differ only in whitespace should not produce a diff.
        let reference = make_structure(None, None, 1, vec!["hello  world"], vec![]);
        let actual = make_structure(None, None, 1, vec!["hello world"], vec![]);
        let result = diff_pdf_structure(&reference, &actual);
        assert!(
            result.page_text_diffs.is_empty(),
            "whitespace-only text difference should be ignored"
        );
    }

    #[test]
    fn diff_text_comparison_uses_only_overlapping_pages() {
        // reference has 2 pages, actual has 1 – only page 1 is compared.
        let reference = make_structure(None, None, 2, vec!["same", "extra"], vec![]);
        let actual = make_structure(None, None, 1, vec!["same"], vec![]);
        let result = diff_pdf_structure(&reference, &actual);
        // Page count mismatches but text should not differ for the shared page.
        assert!(!result.page_count_matches);
        assert!(result.page_text_diffs.is_empty());
    }

    #[test]
    fn diff_detects_outline_change() {
        let outline_a = vec![OutlineEntry {
            level: 1,
            title: "Chapter 1".into(),
            page: 1,
        }];
        let outline_b = vec![OutlineEntry {
            level: 1,
            title: "Chapter 2".into(),
            page: 1,
        }];
        let reference = make_structure(None, None, 1, vec!["text"], outline_a);
        let actual = make_structure(None, None, 1, vec!["text"], outline_b);
        let result = diff_pdf_structure(&reference, &actual);
        assert!(!result.outline_matches);
    }

    #[test]
    fn diff_outline_matches_for_identical_outlines() {
        let outline = vec![OutlineEntry {
            level: 1,
            title: "Introduction".into(),
            page: 1,
        }];
        let s = make_structure(None, None, 1, vec!["text"], outline);
        let result = diff_pdf_structure(&s, &s);
        assert!(result.outline_matches);
    }

    #[test]
    fn diff_outline_populated_in_result() {
        let outline_a = vec![OutlineEntry {
            level: 1,
            title: "A".into(),
            page: 1,
        }];
        let outline_b: Vec<OutlineEntry> = vec![];
        let reference = make_structure(None, None, 1, vec!["text"], outline_a.clone());
        let actual = make_structure(None, None, 1, vec!["text"], outline_b);
        let result = diff_pdf_structure(&reference, &actual);
        assert_eq!(result.reference_outline, outline_a);
        assert!(result.actual_outline.is_empty());
    }

    // -----------------------------------------------------------------------
    // PdfStructureDiffResult::is_identical
    // -----------------------------------------------------------------------

    #[test]
    fn is_identical_true_when_all_match() {
        let result = PdfStructureDiffResult {
            metadata_diffs: vec![],
            page_count_matches: true,
            reference_page_count: 1,
            actual_page_count: 1,
            page_text_diffs: vec![],
            outline_matches: true,
            reference_outline: vec![],
            actual_outline: vec![],
        };
        assert!(result.is_identical());
    }

    #[test]
    fn is_identical_false_when_metadata_differs() {
        let result = PdfStructureDiffResult {
            metadata_diffs: vec![FieldDiff {
                field: "title",
                reference: Some("A".into()),
                actual: Some("B".into()),
            }],
            page_count_matches: true,
            reference_page_count: 1,
            actual_page_count: 1,
            page_text_diffs: vec![],
            outline_matches: true,
            reference_outline: vec![],
            actual_outline: vec![],
        };
        assert!(!result.is_identical());
    }

    #[test]
    fn is_identical_false_when_page_count_differs() {
        let result = PdfStructureDiffResult {
            metadata_diffs: vec![],
            page_count_matches: false,
            reference_page_count: 1,
            actual_page_count: 2,
            page_text_diffs: vec![],
            outline_matches: true,
            reference_outline: vec![],
            actual_outline: vec![],
        };
        assert!(!result.is_identical());
    }

    #[test]
    fn is_identical_false_when_text_differs() {
        let result = PdfStructureDiffResult {
            metadata_diffs: vec![],
            page_count_matches: true,
            reference_page_count: 1,
            actual_page_count: 1,
            page_text_diffs: vec![PageTextDiff {
                page: 1,
                reference_text: "hello".into(),
                actual_text: "world".into(),
            }],
            outline_matches: true,
            reference_outline: vec![],
            actual_outline: vec![],
        };
        assert!(!result.is_identical());
    }

    #[test]
    fn is_identical_false_when_outline_differs() {
        let result = PdfStructureDiffResult {
            metadata_diffs: vec![],
            page_count_matches: true,
            reference_page_count: 1,
            actual_page_count: 1,
            page_text_diffs: vec![],
            outline_matches: false,
            reference_outline: vec![OutlineEntry {
                level: 1,
                title: "Chapter 1".into(),
                page: 1,
            }],
            actual_outline: vec![],
        };
        assert!(!result.is_identical());
    }
}
