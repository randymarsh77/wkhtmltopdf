//! Regression test harness.
//!
//! Each HTML fixture in the `fixtures/` directory is rendered to PDF by the
//! Rust implementation.  When the legacy `wkhtmltopdf` binary is present on
//! the host (checked via `$PATH`), the same fixture is also rendered by the
//! binary.  Both output files are written to a deterministic path under
//! `$CARGO_TARGET_TMPDIR` (or the system temp directory as a fallback) so
//! they can be inspected or compared manually after a test run.
//!
//! The tests never fail simply because the binary is absent – that case is
//! always treated as skipped (the Rust-only half still runs).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use wkhtmltopdf_core::Converter as _;
use wkhtmltopdf_pdf::PdfConverter;
use wkhtmltopdf_settings::{PdfGlobal, PdfObject};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Directory where regression outputs are written.
fn output_dir() -> PathBuf {
    // CARGO_TARGET_TMPDIR is set by Cargo for integration test crates.
    let base = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("regression_outputs");
    fs::create_dir_all(&dir).expect("create regression output dir");
    dir
}

/// Path to the `fixtures/` directory bundled with this crate.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Return `Some(path)` if the `wkhtmltopdf` binary can be found in `$PATH`,
/// `None` otherwise.
fn find_legacy_binary() -> Option<PathBuf> {
    // Allow the test suite to override the binary location.
    if let Ok(p) = std::env::var("WKHTMLTOPDF_BINARY") {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    // Walk every directory in $PATH.
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join("wkhtmltopdf");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Render `html_path` to PDF using the Rust implementation, write the bytes
/// to `out_path`, and assert the output is a valid PDF.
fn render_with_rust(html_path: &Path, out_path: &Path) {
    let mut global = PdfGlobal::default();
    global.document_title = Some(
        html_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );

    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(html_path.to_string_lossy().to_string());
    converter.add_object(obj);

    let bytes = converter
        .convert()
        .unwrap_or_else(|e| panic!("Rust converter failed for {:?}: {}", html_path, e));

    assert!(
        bytes.starts_with(b"%PDF-"),
        "output is not a valid PDF for {:?}",
        html_path
    );

    fs::write(out_path, &bytes)
        .unwrap_or_else(|e| panic!("failed to write Rust output to {:?}: {}", out_path, e));
}

/// Render `html_path` to PDF using the legacy binary, write the result to
/// `out_path`, and assert the output is a valid PDF.
///
/// Returns `false` (skip) when the binary is not available.
fn render_with_legacy(binary: &Path, html_path: &Path, out_path: &Path) -> bool {
    // Write a temp output file that the binary can target.
    let tmp_out = tempfile::NamedTempFile::new().expect("temp output file");
    let tmp_out_path = tmp_out.path().to_path_buf();

    let status = Command::new(binary)
        .arg("--quiet")
        .arg(html_path)
        .arg(&tmp_out_path)
        .status();

    match status {
        Err(e) => {
            eprintln!(
                "wkhtmltopdf binary {:?} could not be executed for {:?}: {}",
                binary, html_path, e
            );
            return false;
        }
        Ok(s) if !s.success() => {
            eprintln!(
                "wkhtmltopdf binary exited with {} for {:?} – copying partial output if any",
                s, html_path
            );
            // Still copy whatever was produced so the caller can inspect it.
            let _ = fs::copy(&tmp_out_path, out_path);
            return true;
        }
        Ok(_) => {}
    }

    let bytes = fs::read(&tmp_out_path).unwrap_or_else(|e| {
        panic!(
            "failed to read legacy output from {:?}: {}",
            tmp_out_path, e
        )
    });

    assert!(
        bytes.starts_with(b"%PDF-"),
        "legacy output is not a valid PDF for {:?}",
        html_path
    );

    fs::write(out_path, &bytes)
        .unwrap_or_else(|e| panic!("failed to write legacy output to {:?}: {}", out_path, e));

    true
}

// ---------------------------------------------------------------------------
// Per-fixture test driver
// ---------------------------------------------------------------------------

/// Run the full regression harness for a single HTML fixture.
///
/// 1. Render with the Rust implementation (always).
/// 2. Render with the legacy binary (when available).
fn run_fixture(name: &str) {
    let fixtures = fixtures_dir();
    let html_path = fixtures.join(format!("{}.html", name));
    assert!(html_path.exists(), "fixture not found: {:?}", html_path);

    let out = output_dir();

    // --- Rust implementation ---
    let rust_out = out.join(format!("{}.rust.pdf", name));
    render_with_rust(&html_path, &rust_out);
    println!("Rust output written to: {}", rust_out.display());

    // --- Legacy binary (optional) ---
    if let Some(binary) = find_legacy_binary() {
        let legacy_out = out.join(format!("{}.legacy.pdf", name));
        if render_with_legacy(&binary, &html_path, &legacy_out) {
            println!("Legacy output written to: {}", legacy_out.display());
        }
    } else {
        println!(
            "wkhtmltopdf binary not found – skipping legacy render for '{}'",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// Individual fixture tests
// ---------------------------------------------------------------------------

#[test]
fn regression_simple() {
    run_fixture("simple");
}

#[test]
fn regression_styled() {
    run_fixture("styled");
}

#[test]
fn regression_headings() {
    run_fixture("headings");
}

#[test]
fn regression_tables() {
    run_fixture("tables");
}

#[test]
fn regression_images() {
    run_fixture("images");
}

#[test]
fn regression_flexbox() {
    run_fixture("flexbox");
}

#[test]
fn regression_grid() {
    run_fixture("grid");
}

#[test]
fn regression_print_media() {
    run_fixture("print_media");
}

#[test]
fn regression_page_breaks() {
    run_fixture("page_breaks");
}

#[test]
fn regression_header_footer() {
    run_fixture("header_footer");
}

#[test]
fn regression_multi_page() {
    run_fixture("multi_page");
}

#[test]
fn regression_toc() {
    run_fixture("toc");
}

#[test]
fn regression_javascript() {
    run_fixture("javascript");
}

#[test]
fn regression_unicode_rtl() {
    run_fixture("unicode_rtl");
}

#[test]
fn regression_edge_cases() {
    run_fixture("edge_cases");
}

// ---------------------------------------------------------------------------
// PDF structural diff integration tests
// ---------------------------------------------------------------------------

use wkhtmltopdf_diff::pdf_diff::{diff_pdf_structure, extract_pdf_structure};

/// Build a PDF from an inline HTML string with optional metadata.
fn pdf_from_html(html: &str, title: Option<&str>, author: Option<&str>) -> Vec<u8> {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "{}", html).expect("write html");
    let path = tmp.path().to_string_lossy().to_string();

    let mut global = PdfGlobal::default();
    if let Some(t) = title {
        global.document_title = Some(t.to_owned());
    }
    if let Some(a) = author {
        global.author = Some(a.to_owned());
    }

    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    converter.add_object(obj);

    converter.convert().expect("PDF conversion failed")
}

#[test]
fn pdf_structural_diff_same_document_is_identical() {
    let bytes = pdf_from_html("<html><body><p>Hello World</p></body></html>", None, None);
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    let result = diff_pdf_structure(&s, &s);
    assert!(
        result.is_identical(),
        "the same PDF document compared to itself must be structurally identical"
    );
}

#[test]
fn pdf_structural_diff_extracts_correct_page_count() {
    let bytes = pdf_from_html("<html><body><p>Single page</p></body></html>", None, None);
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    assert!(
        s.page_count >= 1,
        "single-page HTML must produce at least one PDF page"
    );
    assert_eq!(
        s.page_texts.len(),
        s.page_count as usize,
        "page_texts length must equal page_count"
    );
}

#[test]
fn pdf_structural_diff_reads_title_metadata() {
    let bytes = pdf_from_html(
        "<html><body><p>content</p></body></html>",
        Some("Integration Test Title"),
        None,
    );
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    assert_eq!(
        s.title.as_deref(),
        Some("Integration Test Title"),
        "extracted title must match the one set during conversion"
    );
}

#[test]
fn pdf_structural_diff_reads_author_metadata() {
    let bytes = pdf_from_html(
        "<html><body><p>content</p></body></html>",
        None,
        Some("Test Author"),
    );
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    assert_eq!(
        s.author.as_deref(),
        Some("Test Author"),
        "extracted author must match the one set during conversion"
    );
}

#[test]
fn pdf_structural_diff_detects_title_change() {
    let ref_bytes = pdf_from_html(
        "<html><body><p>text</p></body></html>",
        Some("Old Title"),
        None,
    );
    let act_bytes = pdf_from_html(
        "<html><body><p>text</p></body></html>",
        Some("New Title"),
        None,
    );

    let reference = extract_pdf_structure(&ref_bytes).expect("extract reference");
    let actual = extract_pdf_structure(&act_bytes).expect("extract actual");

    let result = diff_pdf_structure(&reference, &actual);
    assert!(
        result.metadata_diffs.iter().any(|d| d.field == "title"),
        "title change must be reported in metadata_diffs"
    );
}

#[test]
fn pdf_structural_diff_page_text_non_empty() {
    let bytes = pdf_from_html(
        "<html><body><p>Hello PDF World</p></body></html>",
        None,
        None,
    );
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    // Text extraction is best-effort; we only assert the Vec is populated.
    assert!(
        !s.page_texts.is_empty(),
        "page_texts should contain at least one entry"
    );
}

#[test]
fn pdf_structural_diff_no_text_diffs_for_same_document() {
    let bytes = pdf_from_html(
        "<html><body><p>Identical content</p></body></html>",
        None,
        None,
    );
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    let result = diff_pdf_structure(&s, &s);
    assert!(
        result.page_text_diffs.is_empty(),
        "same document must produce no page text diffs"
    );
}

#[test]
fn pdf_structural_diff_outline_matches_for_same_document() {
    let bytes = pdf_from_html(
        "<html><body><h1>Section 1</h1><p>content</p></body></html>",
        None,
        None,
    );
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    let result = diff_pdf_structure(&s, &s);
    assert!(
        result.outline_matches,
        "same document must have matching outlines"
    );
}

#[test]
fn pdf_structural_diff_version_is_non_empty() {
    let bytes = pdf_from_html("<html><body><p>test</p></body></html>", None, None);
    let s = extract_pdf_structure(&bytes).expect("extract structure");
    assert!(
        !s.version.is_empty(),
        "PDF version string must be non-empty"
    );
}
