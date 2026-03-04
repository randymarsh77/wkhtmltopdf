//! Visual regression test harness.
//!
//! Each HTML fixture in the `fixtures/` directory is rendered to a PNG image
//! by the Rust `ImageConverter`.  The output is compared pixel-by-pixel against
//! a stored reference image under `fixtures/references/`.
//!
//! # First-run / reference generation
//!
//! When no reference image exists for a fixture the current rendering is saved
//! as the new reference and the test passes immediately.  Set the env var
//! `VISUAL_UPDATE_REFS=true` to force regeneration of **all** reference images
//! regardless of whether they already exist.
//!
//! # Configurable threshold
//!
//! The maximum allowed visual diff percentage is read from the env var
//! `VISUAL_DIFF_THRESHOLD` (a floating-point number, e.g. `"5.0"`).  It
//! defaults to `5.0` when the variable is absent or unparsable.
//!
//! # Diff image output
//!
//! Diff images are written to `VISUAL_REGRESSION_OUTPUT_DIR` when that env
//! var is set, otherwise to a `visual_regression_diffs/` sub-directory of
//! `CARGO_TARGET_TMPDIR` (or the system temp dir as a fallback).  Diff images
//! are always written, even for passing fixtures, so they can be uploaded as
//! CI artifacts for inspection.
//!
//! # Headless browser availability
//!
//! If the headless Chromium backend is unavailable (e.g. Chrome is not installed
//! on the host) each test is skipped with an informational message rather than
//! failing, so the suite remains green in environments without a browser.

use std::fs;
use std::path::{Path, PathBuf};

use wkhtmltopdf_core::{ConvertError, Converter as _};
use wkhtmltopdf_diff::{diff_images, DiffOptions};
use wkhtmltopdf_image::ImageConverter;
use wkhtmltopdf_settings::ImageGlobal;

// ---------------------------------------------------------------------------
// Configuration helpers
// ---------------------------------------------------------------------------

/// Maximum allowed diff percentage.  Reads `VISUAL_DIFF_THRESHOLD`; defaults
/// to `5.0`.
fn threshold() -> f64 {
    std::env::var("VISUAL_DIFF_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(5.0)
}

/// Whether to unconditionally regenerate all reference images.
fn update_refs() -> bool {
    std::env::var("VISUAL_UPDATE_REFS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
}

/// Directory where diff images are written.
fn output_dir() -> PathBuf {
    let base = std::env::var("VISUAL_REGRESSION_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("CARGO_TARGET_TMPDIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("visual_regression_diffs")
        });
    fs::create_dir_all(&base).expect("create visual regression output dir");
    base
}

/// Directory containing reference PNG images.  Reads
/// `VISUAL_REGRESSION_REFS_DIR`; defaults to `fixtures/references/` relative
/// to this crate's manifest directory.
fn references_dir() -> PathBuf {
    std::env::var("VISUAL_REGRESSION_REFS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("fixtures")
                .join("references")
        })
}

/// Directory containing HTML fixture files.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

// ---------------------------------------------------------------------------
// Rendering helper
// ---------------------------------------------------------------------------

/// Render `<fixture>.html` to PNG bytes using [`ImageConverter`].
///
/// Returns `None` when the headless browser backend is unavailable so the
/// caller can skip the test gracefully.
fn try_render_fixture(name: &str) -> Option<Vec<u8>> {
    let html_path = fixtures_dir().join(format!("{}.html", name));
    assert!(
        html_path.exists(),
        "fixture not found: {:?}",
        html_path
    );

    let mut settings = ImageGlobal::default();
    settings.page = Some(html_path.to_string_lossy().to_string());

    let converter = ImageConverter::new(settings);
    match converter.convert() {
        Ok(bytes) => Some(bytes),
        // ConvertError::Render wraps RenderError::BackendUnavailable when the
        // headless Chrome binary cannot be located or launched.  Treat this as
        // a graceful skip so the suite stays green in environments without a
        // browser (e.g. plain Rust CI without Chrome installed).
        Err(ConvertError::Render(ref msg))
            if msg.contains("browser backend unavailable") =>
        {
            eprintln!(
                "SKIP '{}': headless browser unavailable – {}",
                name, msg
            );
            None
        }
        Err(e) => panic!("ImageConverter failed for fixture '{}': {}", name, e),
    }
}

// ---------------------------------------------------------------------------
// Core harness
// ---------------------------------------------------------------------------

/// Run the visual regression harness for a single HTML fixture.
///
/// Steps:
/// 1. Render the fixture to PNG (skip gracefully when Chrome is absent).
/// 2. If `VISUAL_UPDATE_REFS=true` **or** no reference exists → save as new
///    reference and return (test passes).
/// 3. Otherwise compare against the stored reference with [`diff_images`].
/// 4. Write the annotated diff image to the output directory.
/// 5. Fail if `diff_percentage > threshold`.
fn run_visual_fixture(name: &str) {
    let actual_png = match try_render_fixture(name) {
        Some(bytes) => bytes,
        None => return, // skip — browser unavailable
    };

    let refs_dir = references_dir();
    let ref_path = refs_dir.join(format!("{}.png", name));

    // ------------------------------------------------------------------
    // Reference generation / update
    // ------------------------------------------------------------------
    if update_refs() || !ref_path.exists() {
        fs::create_dir_all(&refs_dir).expect("create references directory");
        fs::write(&ref_path, &actual_png).unwrap_or_else(|e| {
            panic!(
                "failed to write reference image for '{}' to {:?}: {}",
                name, ref_path, e
            )
        });
        println!(
            "Reference {} for '{}': {}",
            if update_refs() { "updated" } else { "generated" },
            name,
            ref_path.display()
        );
        return;
    }

    // ------------------------------------------------------------------
    // Load reference
    // ------------------------------------------------------------------
    let reference_png = fs::read(&ref_path).unwrap_or_else(|e| {
        panic!(
            "failed to read reference image for '{}' from {:?}: {}",
            name, ref_path, e
        )
    });

    // ------------------------------------------------------------------
    // Pixel diff
    // ------------------------------------------------------------------
    let opts = DiffOptions {
        // Use overlapping-region comparison rather than requiring identical
        // dimensions.  Viewport or rendering engine updates may cause the
        // captured image size to change slightly; comparing only the
        // overlapping area surfaces actual content changes without false-failing
        // on pure size differences.  The diff percentage is still reported
        // against the overlap pixel count, so the threshold check remains
        // meaningful.
        require_same_size: false,
        ..Default::default()
    };
    let result = diff_images(&reference_png, &actual_png, opts)
        .unwrap_or_else(|e| panic!("diff_images failed for '{}': {}", name, e));

    // ------------------------------------------------------------------
    // Write diff image
    // ------------------------------------------------------------------
    let out_dir = output_dir();
    let diff_path = out_dir.join(format!("{}.diff.png", name));
    fs::write(&diff_path, result.diff_image()).unwrap_or_else(|e| {
        panic!(
            "failed to write diff image for '{}' to {:?}: {}",
            name, diff_path, e
        )
    });

    println!(
        "Fixture '{}': {:.4}% diff ({}/{} pixels changed); diff → {}",
        name,
        result.diff_percentage(),
        result.different_pixels(),
        result.total_pixels(),
        diff_path.display(),
    );

    // ------------------------------------------------------------------
    // Threshold check
    // ------------------------------------------------------------------
    let max_diff = threshold();
    assert!(
        result.diff_percentage() <= max_diff,
        "visual diff for fixture '{}' is {:.4}% which exceeds the configured \
         threshold of {:.2}% (VISUAL_DIFF_THRESHOLD). \
         Inspect the diff image at: {}",
        name,
        result.diff_percentage(),
        max_diff,
        diff_path.display(),
    );
}

// ---------------------------------------------------------------------------
// Per-fixture tests
// ---------------------------------------------------------------------------

#[test]
fn visual_regression_simple() {
    run_visual_fixture("simple");
}

#[test]
fn visual_regression_styled() {
    run_visual_fixture("styled");
}

#[test]
fn visual_regression_headings() {
    run_visual_fixture("headings");
}

#[test]
fn visual_regression_tables() {
    run_visual_fixture("tables");
}

#[test]
fn visual_regression_images() {
    run_visual_fixture("images");
}

#[test]
fn visual_regression_flexbox() {
    run_visual_fixture("flexbox");
}

#[test]
fn visual_regression_grid() {
    run_visual_fixture("grid");
}

#[test]
fn visual_regression_print_media() {
    run_visual_fixture("print_media");
}

#[test]
fn visual_regression_page_breaks() {
    run_visual_fixture("page_breaks");
}

#[test]
fn visual_regression_header_footer() {
    run_visual_fixture("header_footer");
}

#[test]
fn visual_regression_multi_page() {
    run_visual_fixture("multi_page");
}

#[test]
fn visual_regression_toc() {
    run_visual_fixture("toc");
}

#[test]
fn visual_regression_javascript() {
    run_visual_fixture("javascript");
}

#[test]
fn visual_regression_unicode_rtl() {
    run_visual_fixture("unicode_rtl");
}

#[test]
fn visual_regression_edge_cases() {
    run_visual_fixture("edge_cases");
}
