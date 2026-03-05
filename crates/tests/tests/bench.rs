//! Performance benchmarking harness for wkhtmltopdf.
//!
//! Measures rendering throughput (pages/second) and memory usage (peak RSS) for
//! the Rust implementation across the standard fixture set, and – when the
//! legacy `wkhtmltopdf` binary is available on `$PATH` – also for the C++
//! baseline so the two can be compared side-by-side.
//!
//! # Running
//!
//! Most benchmark tests are marked `#[ignore]` to avoid slowing down the
//! regular test suite.  Run them explicitly with:
//!
//! ```text
//! cargo test -p wkhtmltopdf-tests -- --ignored --nocapture bench
//! ```
//!
//! A `bench_sanity` test (not ignored) verifies that the timing infrastructure
//! itself is functional, so it always runs with `cargo test`.
//!
//! # Environment variables
//!
//! | Variable              | Effect                                         |
//! |-----------------------|------------------------------------------------|
//! | `BENCH_ITERATIONS`    | Number of renders per fixture (default: 5)     |
//! | `WKHTMLTOPDF_BINARY`  | Override path to the legacy `wkhtmltopdf` binary |

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use wkhtmltopdf_core::Converter as _;
use wkhtmltopdf_pdf::PdfConverter;
use wkhtmltopdf_settings::{PdfGlobal, PdfObject};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Number of render iterations used to compute throughput averages.
fn bench_iterations() -> u32 {
    std::env::var("BENCH_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

// ---------------------------------------------------------------------------
// File-system helpers
// ---------------------------------------------------------------------------

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn output_dir() -> PathBuf {
    let base = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("bench_outputs");
    std::fs::create_dir_all(&dir).expect("create bench output dir");
    dir
}

// ---------------------------------------------------------------------------
// Memory measurement
// ---------------------------------------------------------------------------

/// Sample the current process's resident-set size in kilobytes.
///
/// On Linux this reads `/proc/self/status`.  On other platforms the function
/// always returns `None` (memory comparisons are skipped but throughput
/// figures are still reported).
fn sample_rss_kb() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kb);
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

// ---------------------------------------------------------------------------
// Benchmark result type
// ---------------------------------------------------------------------------

/// Timing and memory statistics for a single fixture benchmark run.
#[derive(Debug)]
struct BenchResult {
    /// Name of the HTML fixture (without extension).
    fixture: String,
    /// Number of render iterations completed.
    iterations: u32,
    /// Total wall-clock time for all iterations.
    total_duration: Duration,
    /// Average render time per page.
    avg_per_page: Duration,
    /// Throughput in pages per second.
    pages_per_second: f64,
    /// Peak RSS observed during the benchmark run (kB), if measurable.
    peak_rss_kb: Option<u64>,
}

impl BenchResult {
    fn new(fixture: &str, iterations: u32, total: Duration, peak_rss_kb: Option<u64>) -> Self {
        assert!(iterations > 0, "benchmark requires at least one iteration");
        let avg = total / iterations;
        let pps = if total.as_secs_f64() > 0.0 {
            iterations as f64 / total.as_secs_f64()
        } else {
            f64::INFINITY
        };
        BenchResult {
            fixture: fixture.to_string(),
            iterations,
            total_duration: total,
            avg_per_page: avg,
            pages_per_second: pps,
            peak_rss_kb,
        }
    }
}

// ---------------------------------------------------------------------------
// Rust implementation benchmark
// ---------------------------------------------------------------------------

/// Render `fixture_name` through the Rust PDF converter `iterations` times.
///
/// Returns timing/memory statistics and writes the last rendered PDF to the
/// output directory so it can be inspected afterwards.
fn benchmark_rust(fixture_name: &str, iterations: u32) -> BenchResult {
    let html_path = fixtures_dir().join(format!("{}.html", fixture_name));
    assert!(
        html_path.exists(),
        "fixture not found: {}",
        html_path.display()
    );

    let rss_before = sample_rss_kb().unwrap_or(0);
    let start = Instant::now();

    let mut last_bytes: Vec<u8> = Vec::new();
    for _ in 0..iterations {
        let mut global = PdfGlobal::default();
        global.document_title = Some(fixture_name.to_string());
        let mut converter = PdfConverter::new(global);
        let mut obj = PdfObject::default();
        obj.page = Some(html_path.to_string_lossy().to_string());
        converter.add_object(obj);
        last_bytes = converter
            .convert()
            .unwrap_or_else(|e| panic!("Rust conversion failed for {}: {}", fixture_name, e));
    }

    let total = start.elapsed();
    let rss_after = sample_rss_kb().unwrap_or(0);
    let peak_rss = if rss_before > 0 || rss_after > 0 {
        Some(rss_after.max(rss_before))
    } else {
        None
    };

    // Persist last output for inspection.
    let out = output_dir().join(format!("{}.rust.pdf", fixture_name));
    std::fs::write(&out, &last_bytes).expect("write bench output");

    BenchResult::new(fixture_name, iterations, total, peak_rss)
}

// ---------------------------------------------------------------------------
// Legacy (C++) binary benchmark
// ---------------------------------------------------------------------------

/// Return the path to the legacy `wkhtmltopdf` binary, if discoverable.
fn find_legacy_binary() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("WKHTMLTOPDF_BINARY") {
        let pb = PathBuf::from(&p);
        if pb.is_file() {
            return Some(pb);
        }
    }
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

/// Render `fixture_name` using the legacy binary `iterations` times.
///
/// Returns `None` when the binary is not available (the caller should skip
/// the legacy half of the comparison).
fn benchmark_legacy(
    binary: &Path,
    fixture_name: &str,
    iterations: u32,
) -> Option<BenchResult> {
    let html_path = fixtures_dir().join(format!("{}.html", fixture_name));
    if !html_path.exists() {
        return None;
    }

    let rss_before = sample_rss_kb().unwrap_or(0);
    let start = Instant::now();

    for _ in 0..iterations {
        let tmp = tempfile::NamedTempFile::new().expect("tmp file");
        let status = Command::new(binary)
            .arg("--quiet")
            .arg(&html_path)
            .arg(tmp.path())
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                eprintln!(
                    "legacy wkhtmltopdf exited {} for {}",
                    s, fixture_name
                );
            }
            Err(e) => {
                eprintln!("failed to run legacy binary for {}: {}", fixture_name, e);
                return None;
            }
        }
    }

    let total = start.elapsed();
    let rss_after = sample_rss_kb().unwrap_or(0);
    let peak_rss = if rss_before > 0 || rss_after > 0 {
        Some(rss_after.max(rss_before))
    } else {
        None
    };

    Some(BenchResult::new(fixture_name, iterations, total, peak_rss))
}

// ---------------------------------------------------------------------------
// Reporting
// ---------------------------------------------------------------------------

/// Print a human-readable comparison table to stdout.
fn print_report(rust: &BenchResult, legacy: Option<&BenchResult>) {
    println!("\n┌─────────────────────────────────────────────────────────────────────┐");
    println!("│  Benchmark: {:<57}│", rust.fixture);
    println!("├───────────────────┬─────────────┬──────────────┬────────────────────┤");
    println!("│ Implementation    │ Avg/page    │ pages/sec    │ Peak RSS (kB)      │");
    println!("├───────────────────┼─────────────┼──────────────┼────────────────────┤");

    let rss_str = |r: &BenchResult| {
        r.peak_rss_kb
            .map(|kb| format!("{kb}"))
            .unwrap_or_else(|| "n/a".to_string())
    };

    println!(
        "│ Rust              │ {:>9.1} ms │ {:>10.2} │ {:>18} │",
        rust.avg_per_page.as_secs_f64() * 1000.0,
        rust.pages_per_second,
        rss_str(rust),
    );

    if let Some(leg) = legacy {
        let speedup = leg.avg_per_page.as_secs_f64() / rust.avg_per_page.as_secs_f64();
        println!(
            "│ C++ (legacy)      │ {:>9.1} ms │ {:>10.2} │ {:>18} │",
            leg.avg_per_page.as_secs_f64() * 1000.0,
            leg.pages_per_second,
            rss_str(leg),
        );
        println!(
            "│ Rust speedup      │ {:>10.2}x │              │                    │",
            speedup
        );
    } else {
        println!("│ C++ (legacy)      │ n/a (binary not found)                          │");
    }

    println!("└───────────────────┴─────────────┴──────────────┴────────────────────┘");

    println!(
        "  iterations={}, total={:.2}s",
        rust.iterations,
        rust.total_duration.as_secs_f64()
    );
}

// ---------------------------------------------------------------------------
// Aggregate summary
// ---------------------------------------------------------------------------

/// Run the full benchmark suite across all fixtures and print a summary table.
fn run_all_fixtures_benchmark() {
    let fixtures = &[
        "simple",
        "styled",
        "headings",
        "tables",
        "images",
        "flexbox",
        "grid",
        "print_media",
        "page_breaks",
        "header_footer",
        "multi_page",
        "toc",
        "javascript",
        "unicode_rtl",
        "edge_cases",
    ];

    let iters = bench_iterations();
    let legacy_bin = find_legacy_binary();

    println!("\n=== wkhtmltopdf Performance Benchmark ===");
    println!("Rust implementation vs C++ baseline");
    println!("Iterations per fixture: {}", iters);
    if let Some(ref b) = legacy_bin {
        println!("Legacy binary: {}", b.display());
    } else {
        println!("Legacy binary: not found (C++ comparison skipped)");
    }

    let mut rust_total = Duration::ZERO;
    let mut legacy_total = Duration::ZERO;
    let mut rust_count = 0u32;
    let mut legacy_count = 0u32;

    // Header for summary table.
    // Column widths: 18 + 1 + 10 + 1 + 12 + 1 + 12 + 1 + 10 = 66
    const SUMMARY_WIDTH: usize = 66;
    println!("\n{:<18} {:>10} {:>12} {:>12} {:>10}",
        "Fixture", "Rust ms/pg", "Rust pgs/s", "C++ pgs/s", "Speedup");
    println!("{}", "-".repeat(SUMMARY_WIDTH));

    for &name in fixtures {
        let rust = benchmark_rust(name, iters);
        let leg = legacy_bin
            .as_deref()
            .and_then(|b| benchmark_legacy(b, name, iters));

        let (leg_pps, speedup_str) = match &leg {
            Some(l) => {
                let sp = l.avg_per_page.as_secs_f64() / rust.avg_per_page.as_secs_f64();
                legacy_total += l.total_duration;
                legacy_count += iters;
                (format!("{:.2}", l.pages_per_second), format!("{:.2}x", sp))
            }
            None => ("n/a".to_string(), "n/a".to_string()),
        };

        println!(
            "{:<18} {:>10.1} {:>12.2} {:>12} {:>10}",
            name,
            rust.avg_per_page.as_secs_f64() * 1000.0,
            rust.pages_per_second,
            leg_pps,
            speedup_str,
        );

        rust_total += rust.total_duration;
        rust_count += iters;
    }

    println!("{}", "-".repeat(SUMMARY_WIDTH));

    let rust_overall_pps =
        rust_count as f64 / rust_total.as_secs_f64().max(f64::EPSILON);
    let legacy_overall_pps = if legacy_count > 0 {
        format!(
            "{:.2}",
            legacy_count as f64 / legacy_total.as_secs_f64().max(f64::EPSILON)
        )
    } else {
        "n/a".to_string()
    };

    println!(
        "{:<18} {:>10} {:>12.2} {:>12} {:>10}",
        "OVERALL",
        "",
        rust_overall_pps,
        legacy_overall_pps,
        "",
    );
    println!();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Sanity check: verify the benchmark infrastructure itself works correctly.
///
/// This test is *not* ignored so it always runs with `cargo test`.  It
/// renders a single fixture once and asserts that we got a non-zero duration
/// and a valid PDF back.
#[test]
fn bench_sanity() {
    let rust = benchmark_rust("simple", 1);
    assert!(
        rust.total_duration > Duration::ZERO,
        "benchmark duration must be positive"
    );
    assert!(
        rust.pages_per_second > 0.0,
        "throughput must be positive"
    );
    // Verify the output is a real PDF.
    let out = output_dir().join("simple.rust.pdf");
    let bytes = std::fs::read(&out).expect("bench output written");
    assert!(
        bytes.starts_with(b"%PDF-"),
        "bench output must be a valid PDF"
    );
}

/// Full benchmark suite across all standard fixtures (ignored by default).
///
/// Run with:
/// ```text
/// cargo test -p wkhtmltopdf-tests -- --ignored --nocapture bench_all_fixtures
/// ```
#[test]
#[ignore]
fn bench_all_fixtures() {
    run_all_fixtures_benchmark();
}

/// Per-fixture benchmarks (ignored by default).  Each can be run individually:
/// ```text
/// cargo test -p wkhtmltopdf-tests -- --ignored --nocapture bench_fixture_simple
/// ```
macro_rules! bench_fixture {
    ($name:ident, $fixture:expr) => {
        #[test]
        #[ignore]
        fn $name() {
            let iters = bench_iterations();
            let rust = benchmark_rust($fixture, iters);
            let legacy = find_legacy_binary()
                .as_deref()
                .and_then(|b| benchmark_legacy(b, $fixture, iters));
            print_report(&rust, legacy.as_ref());
        }
    };
}

bench_fixture!(bench_fixture_simple, "simple");
bench_fixture!(bench_fixture_styled, "styled");
bench_fixture!(bench_fixture_headings, "headings");
bench_fixture!(bench_fixture_tables, "tables");
bench_fixture!(bench_fixture_images, "images");
bench_fixture!(bench_fixture_flexbox, "flexbox");
bench_fixture!(bench_fixture_grid, "grid");
bench_fixture!(bench_fixture_print_media, "print_media");
bench_fixture!(bench_fixture_page_breaks, "page_breaks");
bench_fixture!(bench_fixture_header_footer, "header_footer");
bench_fixture!(bench_fixture_multi_page, "multi_page");
bench_fixture!(bench_fixture_toc, "toc");
bench_fixture!(bench_fixture_javascript, "javascript");
bench_fixture!(bench_fixture_unicode_rtl, "unicode_rtl");
bench_fixture!(bench_fixture_edge_cases, "edge_cases");
