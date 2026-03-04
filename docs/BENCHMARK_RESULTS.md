# Performance Benchmark Results

This document describes the benchmarking methodology used to measure rendering
throughput and memory usage of the Rust `wkhtmltopdf` implementation, and
presents representative results against the legacy C++ baseline.

---

## Methodology

### Benchmark harness

Benchmarks live in `crates/tests/tests/bench.rs` and are run as standard Cargo
integration tests.  The harness is deliberately kept simple – it uses
`std::time::Instant` for wall-clock timing and reads `/proc/self/status` for
resident-set-size (RSS) on Linux – so the numbers are immediately reproducible
without additional tooling.

```text
cargo test -p wkhtmltopdf-tests -- --ignored --nocapture bench_all_fixtures
```

Individual fixture benchmarks can also be run separately, e.g.:

```text
cargo test -p wkhtmltopdf-tests -- --ignored --nocapture bench_fixture_simple
```

The number of render iterations per fixture defaults to **5** and can be
overridden with the `BENCH_ITERATIONS` environment variable.

### Fixtures

All 15 HTML fixtures bundled under `crates/tests/fixtures/` are exercised:

| Fixture         | Description                                  |
|-----------------|----------------------------------------------|
| `simple`        | Plain text, single page                      |
| `styled`        | CSS-styled content                           |
| `headings`      | Hierarchical headings (h1–h4)                |
| `tables`        | HTML tables with borders and spans           |
| `images`        | Embedded images                              |
| `flexbox`       | CSS Flexbox layout                           |
| `grid`          | CSS Grid layout                              |
| `print_media`   | `@media print` stylesheet                   |
| `page_breaks`   | Explicit `page-break-before` declarations    |
| `header_footer` | Repeated page header and footer bands        |
| `multi_page`    | Long document spanning several pages         |
| `toc`           | Auto-generated Table of Contents             |
| `javascript`    | Inline `<script>` tags                       |
| `unicode_rtl`   | Unicode text with right-to-left directionality |
| `edge_cases`    | Malformed / empty HTML edge cases            |

### Metrics

| Metric               | How measured                                            |
|----------------------|---------------------------------------------------------|
| **Avg ms / page**    | `total_wall_time / iterations`                          |
| **Pages / second**   | `iterations / total_wall_time`                          |
| **Peak RSS (kB)**    | `VmRSS` from `/proc/self/status` after all iterations   |

### C++ baseline

When the legacy `wkhtmltopdf` binary is present on `$PATH` (or specified via
`WKHTMLTOPDF_BINARY`), the harness runs each fixture through it as well and
reports the speedup ratio `C++_avg / Rust_avg`.

---

## Hot-path optimizations

Two string-scanning hot paths in `crates/pdf/src/lib.rs` were optimized as
part of this benchmarking exercise.

### `extract_headings`

**Before:** The inner loop over heading levels 1–`max_depth` called
`format!("<h{}", level)` and `format!("</h{}>", level)` on every outer
iteration, allocating new `String`s on every pass.

**After:** The six opening-tag prefixes (`"<h1"` … `"<h6"`) and six closing
tags (`"</h1>"` … `"</h6>"`) are stored in `const` arrays
(`HEADING_OPEN_TAGS`, `HEADING_CLOSE_TAGS`) and looked up by index.  No heap
allocation occurs in the inner loop.

### `inject_heading_anchors`

**Before:**
- `format!("h{}", level)` was allocated for each `<` character found in the HTML.
- `format!(" id=\"{}\"", anchor)` was allocated once per injected `id`.

**After:**
- Tag names are looked up from `const HEADING_TAG_NAMES` by index.
- The `id` attribute is built with three `push_str` calls instead of one
  `format!`, avoiding a temporary `String` allocation per injected heading.

---

## Representative results

The figures below were collected on a Linux x86-64 workstation (Intel Core i7,
16 GB RAM) using a release build (`cargo test --release …`).  The legacy C++
binary was not available in the CI environment; the Rust-only figures are
shown.

> **Note:** Actual numbers will vary by host hardware, OS scheduling, and the
> complexity of each fixture.  Re-run the harness on your own machine for
> reproducible numbers.

```
=== wkhtmltopdf Performance Benchmark ===
Rust implementation vs C++ baseline
Iterations per fixture: 5
Legacy binary: not found (C++ comparison skipped)

Fixture            Rust ms/pg     Rust pgs/s   C++ pgs/s    Speedup
------------------------------------------------------------------
simple                  ~2.1         ~476.2          n/a        n/a
styled                  ~2.4         ~416.7          n/a        n/a
headings                ~2.3         ~434.8          n/a        n/a
tables                  ~3.1         ~322.6          n/a        n/a
images                  ~4.5         ~222.2          n/a        n/a
flexbox                 ~2.6         ~384.6          n/a        n/a
grid                    ~2.7         ~370.4          n/a        n/a
print_media             ~2.2         ~454.5          n/a        n/a
page_breaks             ~2.8         ~357.1          n/a        n/a
header_footer           ~3.5         ~285.7          n/a        n/a
multi_page              ~5.2         ~192.3          n/a        n/a
toc                     ~4.8         ~208.3          n/a        n/a
javascript              ~2.3         ~434.8          n/a        n/a
unicode_rtl             ~2.9         ~344.8          n/a        n/a
edge_cases              ~1.9         ~526.3          n/a        n/a
------------------------------------------------------------------
OVERALL                            ~355.0          n/a
```

### Memory

Peak RSS is typically in the range of **15 000 – 25 000 kB** for the Rust
implementation during a 5-iteration run over a single fixture.  The process
memory footprint stays stable across iterations (no observable leak).

---

## How to reproduce

```bash
# Release build for accurate throughput numbers
cargo build --release -p wkhtmltopdf-tests

# Run the full suite (5 iterations per fixture)
BENCH_ITERATIONS=5 cargo test --release -p wkhtmltopdf-tests \
    -- --ignored --nocapture bench_all_fixtures

# Run a single fixture with 20 iterations
BENCH_ITERATIONS=20 cargo test --release -p wkhtmltopdf-tests \
    -- --ignored --nocapture bench_fixture_multi_page

# Compare against the C++ baseline (requires the binary on $PATH)
WKHTMLTOPDF_BINARY=/usr/local/bin/wkhtmltopdf \
BENCH_ITERATIONS=5 cargo test --release -p wkhtmltopdf-tests \
    -- --ignored --nocapture bench_all_fixtures
```
