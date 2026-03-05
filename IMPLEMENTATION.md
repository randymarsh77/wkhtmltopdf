# Implementation Notes

This document describes the internal architecture of the Rust reimplementation
of `wkhtmltopdf` / `wkhtmltoimage`, focusing on the Qt WebKit rendering stack
and the LGPL-3.0 licensing requirements.

---

## Rendering stack

The rendering engine is **Qt WebKit** (or Qt WebEngine on Qt 6).  The relevant
code lives in:

| File | Purpose |
|------|---------|
| `crates/core/src/webkit_renderer.cpp` | C++ glue: `QApplication` lifecycle, `QWebPage`/`QWebEnginePage` page load, PNG capture. |
| `crates/core/src/qt_webkit.rs` | Rust side of the `cxx-qt` FFI bridge to the C++ backend. |
| `crates/core/src/renderer.rs` | `Renderer` trait, `HeadlessRenderer` struct, feature dispatch. |

### Feature flag

The Qt WebKit backend is compiled in only when the `qt-webkit` Cargo feature is
enabled:

```toml
# Cargo.toml of the crate that depends on wkhtmltopdf-core
wkhtmltopdf-core = { version = "...", features = ["qt-webkit"] }
```

Without the feature `HeadlessRenderer::render` returns
`RenderError::BackendUnavailable` immediately after validating the URL scheme,
so the rest of the codebase compiles and links cleanly even without Qt present
on the build host.

### Render flow (with `qt-webkit` feature)

```
HeadlessRenderer::render(input)
  └─ crate::qt_webkit::render_url(url, enable_js, js_delay, scripts)   [Rust FFI]
       └─ webkit_render_url(…)                                          [C++ via cxx-qt]
            ├─ QApplication::instance() or new QApplication
            ├─ QWebPage / QWebEnginePage::load(url)
            ├─ QEventLoop (runs until loadFinished)
            ├─ optional JS injection + QTimer(js_delay)
            └─ QWebFrame::render / QWebEnginePage::grab → PNG bytes
```

The C++ function writes PNG bytes into a `std::vector<uint8_t>` that is passed
back through the FFI boundary and returned as `RenderedPage { bytes, mime_type:
"image/png" }`.

---

## Crate structure

```
crates/
  settings/   Strongly-typed settings structs (PdfGlobal, ImageGlobal, …).
  core/       Renderer trait + Qt WebKit backend + shared error types.
  pdf/        PdfConverter: multi-page PDF pipeline using printpdf + ureq.
  image/      ImageConverter: PNG/JPEG/BMP/SVG capture using the image crate.
  ffi/        libwkhtmltox cdylib — re-exports the original C ABI.
  cli/        wkhtmltopdf and wkhtmltoimage binaries (Clap 4).
  diff/       Perceptual image diffing and PDF structural diffing.
  tests/      Integration and visual-regression test harness.
```

### Data flow — PDF conversion

```
CLI args
  └─ PdfGlobal + Vec<PdfObject>
       └─ PdfConverter::convert()
            ├─ fetch HTML (ureq, for remote URLs)
            ├─ inject_heading_anchors()   [for TOC]
            ├─ generate_toc_html()        [if --toc]
            ├─ inject_header_footer()     [if header/footer set]
            └─ printpdf::PdfDocument::from_html()  → Vec<u8>
```

### Data flow — image conversion

```
CLI args
  └─ ImageGlobal
       └─ ImageConverter::convert()
            ├─ HeadlessRenderer::render(input)   [Qt WebKit → PNG bytes]
            └─ image crate: crop / resize / re-encode (PNG/JPEG/BMP/SVG)
                 → Vec<u8>
```

---

## License

This project is licensed under the **GNU Lesser General Public License v3.0**
(LGPL-3.0), the same license used by the original `wkhtmltopdf` project.

Key points for downstream users:

* You may link against `libwkhtmltox` (the `ffi` crate) from a proprietary
  application **without** being required to open-source your application,
  provided you comply with the LGPL-3.0 terms (dynamic linking, providing
  object files to allow relinking, etc.).
* Any modifications to the library itself must be distributed under LGPL-3.0.
* Qt libraries are licensed under LGPL-3.0 as well (for the open-source
  edition), which is compatible with this project's licensing.

See [LICENSE](LICENSE) for the full license text.
