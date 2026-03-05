# wkhtmltopdf and wkhtmltoimage

> **Project status: Active** — This project has been rewritten in Rust.

`wkhtmltopdf` and `wkhtmltoimage` are command-line tools that render HTML into
PDF and various image formats.  The rendering is performed by the Qt WebKit
engine, so no display server is required.

This repository contains a complete Rust reimplementation of the original
`wkhtmltopdf` / `wkhtmltoimage` tooling.  The C FFI layer (`libwkhtmltox`) is
also re-exported so that existing applications that link against the original
shared library can continue to work without source changes.

---

## Architecture

The workspace is organised into focused crates under the `crates/` directory:

| Crate | Description |
|---|---|
| `settings` (`wkhtmltopdf-settings`) | Strongly-typed settings structs and enums for every PDF and image option. |
| `core` (`wkhtmltopdf-core`) | `Renderer` trait, `HeadlessRenderer` (Qt WebKit engine), shared error types. |
| `pdf` (`wkhtmltopdf-pdf`) | `PdfConverter` — multi-page PDF pipeline: margins, headers/footers, TOC, PDF/A, metadata. |
| `image` (`wkhtmltopdf-image`) | `ImageConverter` — PNG/JPEG/BMP/SVG capture with crop, resize, and DPI support. |
| `ffi` (`wkhtmltox`) | `cdylib` that re-exports the original `libwkhtmltox` C ABI using the Rust implementation. |
| `cli` | Two binaries: `wkhtmltopdf` and `wkhtmltoimage` (Clap 4 derive-style CLIs). |
| `diff` (`wkhtmltopdf-diff`) | Perceptual image diffing and PDF structural diffing for regression testing. |
| `tests` | Integration and visual-regression test harness. |

### Key design choices

* **Qt WebKit** is used for all HTML rendering, matching the original
  wkhtmltopdf behaviour.  Enable with the `qt-webkit` feature flag.
* **`printpdf`** assembles multi-page PDF documents.
* **`ureq`** fetches remote HTML resources.
* **`image` crate** handles image encode/decode and transforms.
* All settings are serialisable (`serde`) so they can be embedded in larger
  application configurations.

---

## System dependencies

### Linux (Debian / Ubuntu)

```bash
# Rust toolchain
curl https://sh.rustup.rs -sSf | sh

# Qt 5 development libraries (required for the qt-webkit rendering backend)
sudo apt-get install -y qt5-default libqt5webkit5-dev libqt5webenginewidgets5

# Optional: fonts for full Unicode / CJK support
sudo apt-get install -y fonts-noto fonts-noto-cjk
```

### macOS

```bash
# Rust toolchain
curl https://sh.rustup.rs -sSf | sh

# Qt 5 via Homebrew (includes WebKit)
brew install qt@5

# Optional: additional fonts
brew install --cask font-noto-sans
```

### Windows

1. Install the [Rust toolchain](https://rustup.rs/).
2. Install [Qt 5](https://www.qt.io/download) including the `Qt WebKit` or
   `Qt WebEngine` module.
3. Ensure the Qt `bin/` directory is on your `PATH`.

---

## Building

```bash
# Clone the repository
git clone https://github.com/randymarsh77/wkhtmltopdf.git
cd wkhtmltopdf

# Build all workspace crates (debug)
cargo build

# Build with the Qt WebKit rendering backend enabled
cargo build --features qt-webkit

# Build release binaries
cargo build --release
```

The compiled binaries are placed in `target/release/`:

* `target/release/wkhtmltopdf`
* `target/release/wkhtmltoimage`

### Building the C FFI library

```bash
cargo build --release -p wkhtmltox
# Produces target/release/libwkhtmltox.so (Linux) / .dylib (macOS) / .dll (Windows)
```

---

## Usage

### wkhtmltopdf

```bash
# Convert a URL to PDF
wkhtmltopdf https://example.com output.pdf

# Convert a local HTML file
wkhtmltopdf page.html output.pdf

# With header, footer, and margins
wkhtmltopdf \
  --header-left "[title]" \
  --footer-right "[page] / [toPage]" \
  --margin-top 20mm \
  page.html output.pdf

# Generate a table of contents
wkhtmltopdf --toc --toc-depth 3 page.html output.pdf
```

### wkhtmltoimage

```bash
# Capture a page as PNG
wkhtmltoimage https://example.com screenshot.png

# Specify format and dimensions
wkhtmltoimage --format jpg --width 1280 page.html screenshot.jpg

# Crop a region
wkhtmltoimage --crop-x 10 --crop-y 10 --crop-w 800 --crop-h 600 page.html out.png
```

---

## Running the tests

```bash
# Unit and integration tests
cargo test

# Visual regression tests only
cargo test -p wkhtmltopdf-tests visual_regression

# Force regeneration of all reference images
VISUAL_UPDATE_REFS=true cargo test -p wkhtmltopdf-tests visual_regression
```

### Visual regression test suite

The visual regression suite lives in `crates/tests/tests/visual_regression.rs`.
It covers 15 HTML fixtures (simple layout, styled content, headings, tables,
images, flexbox, CSS grid, print-media queries, page breaks, headers/footers,
multi-page documents, TOC, JavaScript-rendered content, Unicode/RTL text, and
edge cases).

How it works:

1. Each HTML fixture under `crates/tests/fixtures/` is rendered to PNG by
   `ImageConverter`.
2. On the first run (or when `VISUAL_UPDATE_REFS=true`) the PNG is saved as the
   reference image in `crates/tests/fixtures/references/`.
3. On subsequent runs the new PNG is compared pixel-by-pixel against the stored
   reference using the `wkhtmltopdf-diff` crate.
4. The test fails if the diff percentage exceeds the threshold set by
   `VISUAL_DIFF_THRESHOLD` (default `5.0`%).
5. An annotated diff image is always written to the output directory
   (`VISUAL_REGRESSION_OUTPUT_DIR`) so failing pixels can be inspected as CI
   artefacts.
6. If the Qt WebKit backend is unavailable the test is **skipped**
   (rather than failed) so the suite stays green in minimal CI environments.

Environment variables:

| Variable | Default | Description |
|---|---|---|
| `VISUAL_UPDATE_REFS` | `false` | Set to `true` to regenerate all reference images. |
| `VISUAL_DIFF_THRESHOLD` | `5.0` | Maximum allowed diff percentage before a test fails. |
| `VISUAL_REGRESSION_OUTPUT_DIR` | `$CARGO_TARGET_TMPDIR/visual_regression_diffs` | Where diff PNG images are written. |
| `VISUAL_REGRESSION_REFS_DIR` | `crates/tests/fixtures/references` | Where reference PNG images are stored. |

---

## License

Licensed under the GNU Lesser General Public License v3.0 — see [LICENSE](LICENSE) for details.
