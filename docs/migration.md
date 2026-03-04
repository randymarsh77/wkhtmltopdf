# Migration Guide — v0.12.x (C++) → v0.13.0 (Rust)

This document describes everything you need to know when upgrading from the
original C++/Qt-based wkhtmltopdf (≤ 0.12.6) to the new Rust-based release
(0.13.0).

---

## Table of Contents

1. [System requirements](#system-requirements)
2. [CLI users](#cli-users)
3. [libwkhtmltox C API users](#libwkhtmltox-c-api-users)
4. [Changed and removed features](#changed-and-removed-features)
5. [Rendering differences](#rendering-differences)
6. [Rust library users (new)](#rust-library-users-new)

---

## System requirements

| | v0.12.x (C++) | v0.13.0 (Rust) |
|---|---|---|
| **Runtime deps** | Qt 4.8 / bundled QtWebKit, OpenSSL | Chromium or Google Chrome, OpenSSL |
| **Display server** | Required (X11) on Linux | **Not required** — headless Chromium |
| **Build deps** | Qt 4.8 source tree, C++ toolchain | Rust ≥ 1.80, Cargo |

### Installing Chromium

**Debian / Ubuntu:**
```bash
sudo apt-get install -y chromium-browser
# or
sudo apt-get install -y chromium
```

**macOS (Homebrew):**
```bash
brew install --cask chromium
# or install Google Chrome from https://www.google.com/chrome/
```

**Windows:** Install [Google Chrome](https://www.google.com/chrome/) or
[Chromium](https://www.chromium.org/getting-involved/download-chromium/) and
ensure the executable is on your `PATH`.

---

## CLI users

The command-line interface is **fully compatible** with v0.12.x.  All flags
accepted by the original `wkhtmltopdf` and `wkhtmltoimage` binaries are
supported with identical names and semantics unless noted in
[Changed and removed features](#changed-and-removed-features).

### Drop-in replacement

```bash
# No changes required — existing invocations continue to work:
wkhtmltopdf https://example.com output.pdf
wkhtmltopdf --margin-top 20mm --header-left "[title]" page.html output.pdf
wkhtmltoimage https://example.com screenshot.png
```

### New flags available in v0.13.0

| Flag | Description |
|---|---|
| `--toc-depth N` | Maximum heading depth included in the table of contents (default: 3) |
| `--pdf-a` | Produce a PDF/A-1b conformant file |
| `--author TEXT` | Set the PDF Author metadata field |
| `--subject TEXT` | Set the PDF Subject metadata field |
| `--dpi N` | Rendering DPI for image capture (wkhtmltoimage) |

---

## libwkhtmltox C API users

The `libwkhtmltox` shared library produced by this release exports the **same
C ABI** as v0.12.x.  All public symbols are preserved:

* `wkhtmltopdf_init` / `wkhtmltopdf_deinit`
* `wkhtmltopdf_create_global_settings` / `wkhtmltopdf_destroy_global_settings`
* `wkhtmltopdf_set_global_setting` / `wkhtmltopdf_get_global_setting`
* `wkhtmltopdf_create_object_settings` / `wkhtmltopdf_destroy_object_settings`
* `wkhtmltopdf_set_object_setting` / `wkhtmltopdf_get_object_setting`
* `wkhtmltopdf_create_converter` / `wkhtmltopdf_destroy_converter`
* `wkhtmltopdf_add_object`
* `wkhtmltopdf_convert`
* `wkhtmltopdf_get_output`
* `wkhtmltopdf_set_warning_callback` / `wkhtmltopdf_set_error_callback`
* `wkhtmltopdf_set_phase_changed_callback` / `wkhtmltopdf_set_progress_changed_callback`
* `wkhtmltopdf_current_phase` / `wkhtmltopdf_phase_count` / `wkhtmltopdf_phase_description`
* `wkhtmltopdf_progress_string` / `wkhtmltopdf_http_error_code`
* `wkhtmltopdf_version`
* All corresponding `wkhtmltoimage_*` variants

### What changed in the shared library

* `wkhtmltopdf_version()` now returns `"0.13.0"` instead of `"0.12.6"`.
* The library is built from pure Rust; it no longer depends on Qt or OpenSSL
  at link time.  It does require Chromium to be available in `PATH` at
  **runtime** (the same machine running the conversion).
* The `use_graphics` parameter passed to `wkhtmltopdf_init` / `wkhtmltoimage_init`
  is accepted but ignored — the Rust backend is always headless.

### Binary compatibility

Applications that currently `dlopen` or dynamically link `libwkhtmltox.so`
(Linux), `libwkhtmltox.dylib` (macOS), or `wkhtmltox.dll` (Windows) can
**replace the library in-place** without recompiling.

---

## Changed and removed features

### Supported (and working)

| Feature | Notes |
|---|---|
| URL / local file input | Fully supported |
| Page size and orientation | Fully supported |
| Margins | Fully supported |
| Headers and footers (text) | Fully supported; variables `[page]`, `[toPage]`, `[date]`, `[title]`, `[url]` |
| Headers and footers (HTML) | Fully supported via `--header-html` / `--footer-html` |
| Table of contents (`--toc`) | Fully supported; depth configurable via `--toc-depth` |
| Cookies, custom HTTP headers | Fully supported |
| Basic authentication | Fully supported |
| `--grayscale` | Fully supported |
| `--copies` | Fully supported |
| `--page-offset` | Fully supported |
| Image formats (PNG/JPEG/BMP) | Fully supported |
| SVG output (`wkhtmltoimage`) | Fully supported |
| Crop / resize (`wkhtmltoimage`) | Fully supported via `--crop-*` flags |

### Changed behaviour

| Feature | Change |
|---|---|
| **Rendering engine** | Qt WebKit → headless Chromium.  Visual output may differ for some pages. |
| **JavaScript** | Always executed (Chromium); `--disable-javascript` is accepted but has no effect. |
| **`--window-status`** | Accepted for compatibility; Chromium uses `--javascript-delay` instead. |
| **DPI** | Default DPI is inherited from Chromium (96); use `--dpi` to override for image capture. |
| **Font rendering** | System fonts are used directly; Qt's bundled fonts are no longer included. |

### Removed features

| Feature | Status |
|---|---|
| PostScript output | Removed in v0.12.1; not supported. |
| `--use-xserver` | No longer needed — rendering is always headless. |
| Qt-specific CSS extensions | Removed with the Qt rendering engine. |

---

## Rendering differences

Because the rendering engine has changed from Qt WebKit to headless Chromium,
some pages may render differently:

* **Modern CSS**: Chromium has broader support for modern CSS features
  (Grid, custom properties, `@container`, etc.) that were unsupported in Qt
  WebKit.
* **Font stack**: ensure the fonts your pages depend on are installed on the
  host system.  The Qt build shipped bundled fonts which are no longer present.
* **`print` media queries**: Chromium applies `@media print` rules; behaviour
  should be the same or better than Qt WebKit.
* **`page-break-inside: avoid`**: honoured by Chromium; the Qt WebKit
  patch for this feature is no longer required.

If you encounter rendering regressions, please open an issue with a minimal
HTML reproducer.

---

## Rust library users (new)

v0.13.0 publishes the following crates that can be consumed directly from
Rust applications:

| Crate | Description |
|---|---|
| `wkhtmltopdf-settings` | Strongly-typed settings structs (`PdfGlobal`, `PdfObject`, `ImageGlobal`, …) |
| `wkhtmltopdf-core` | `Renderer` trait, `HeadlessRenderer` backed by headless Chromium |
| `wkhtmltopdf-pdf` | `PdfConverter` — full PDF pipeline |
| `wkhtmltopdf-image` | `ImageConverter` — full image capture pipeline |

Example:

```rust
use wkhtmltopdf_settings::{PdfGlobal, PdfObject};
use wkhtmltopdf_pdf::PdfConverter;

let global = PdfGlobal {
    document_title: Some("My Document".into()),
    ..Default::default()
};
let object = PdfObject {
    page: Some("https://example.com".into()),
    ..Default::default()
};

let pdf_bytes = PdfConverter::convert(global, vec![object])?;
std::fs::write("output.pdf", pdf_bytes)?;
```
