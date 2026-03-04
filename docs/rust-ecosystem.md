# Rust Ecosystem Equivalents

This document maps each major subsystem of the wkhtmltopdf C++/Qt codebase to the best available Rust crate alternatives. For each category, crates are evaluated on maturity, maintenance status, and fitness for purpose.

---

## 1. HTML/CSS Rendering

The existing codebase relies on QtWebKit's `QWebPage` / `QWebPrinter` to render HTML and CSS into a printable representation. The Rust ecosystem offers several alternatives at different layers:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`servo`](https://crates.io/crates/servo) | Full browser engine (Gecko-derived layout engine written in Rust) | Most complete Rust-native HTML/CSS renderer; complex integration; useful if embedding a full layout engine |
| [`webrender`](https://crates.io/crates/webrender) | GPU-accelerated rendering backend (used by Firefox, Servo) | Renders already-laid-out content; needs a layout engine upstream |
| [`stylo`](https://crates.io/crates/style) | CSS style computation from Servo | Handles CSS cascading, specificity, and computed values; must be paired with a layout engine |
| [`gosub_engine`](https://crates.io/crates/gosub_engine) | Pure-Rust experimental browser engine | Early stage; useful for learning/prototyping |
| [`html5ever`](https://crates.io/crates/html5ever) + [`selectors`](https://crates.io/crates/selectors) | HTML5 parsing + CSS selector matching | Does not perform layout or paint; use as parsing primitives |
| [`scraper`](https://crates.io/crates/scraper) | HTML parsing + CSS selection (thin wrapper around `html5ever`/`selectors`) | Convenient for DOM traversal; no rendering |

**Recommendation:** For a full HTML-to-PDF pipeline that needs accurate CSS layout and rendering, the most practical near-term approach is to delegate to a headless browser (see §4) rather than embedding a standalone layout engine. If a pure-Rust renderer is required, `servo` (or its component crates `html5ever` + `stylo`) is the closest native option, though integration is non-trivial.

---

## 2. PDF Generation

The existing codebase uses `QPrinter`/`QPainter` to produce PDFs. Rust has several crates for generating or manipulating PDF documents:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`printpdf`](https://crates.io/crates/printpdf) | Pure-Rust PDF creation | Generates PDF/A-compliant files; supports text, images, vector graphics, and custom page sizes; actively maintained |
| [`lopdf`](https://crates.io/crates/lopdf) | Pure-Rust PDF reading and writing | Low-level PDF object model; suitable for reading, modifying, and merging existing PDFs |
| [`pdf-writer`](https://crates.io/crates/pdf-writer) | Pure-Rust low-level PDF content stream builder | Lightweight, alloc-free API; good for generating PDF content streams from scratch |
| [`pdf`](https://crates.io/crates/pdf) | Pure-Rust PDF reader | Primarily for reading/decoding; limited write support |
| [`pdfium-render`](https://crates.io/crates/pdfium-render) | Rust bindings to Google's PDFium C library | Full-featured rendering and editing; depends on the PDFium native library |
| [`wasm-pdf`](https://crates.io/crates/wasm-pdf) | WASM-compatible PDF generator | Suitable for browser targets |

**Recommendation:** Use `printpdf` for programmatic PDF creation (text, images, page layout) and `lopdf` when merging or post-processing existing PDF documents.

---

## 3. Image Output

The existing codebase uses Qt's `QImage`/`QImageWriter` to encode raster output (PNG, JPEG, BMP, etc.). Rust has a mature image ecosystem:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`image`](https://crates.io/crates/image) | Pure-Rust image encoding/decoding | Supports PNG, JPEG, GIF, BMP, ICO, TIFF, WEBP, and more; the de-facto standard image crate |
| [`png`](https://crates.io/crates/png) | Pure-Rust PNG encoder/decoder | Lower-level alternative to `image` for PNG only |
| [`jpeg-decoder`](https://crates.io/crates/jpeg-decoder) | Pure-Rust JPEG decoder | Complement to `image` for JPEG-only pipelines |
| [`resvg`](https://crates.io/crates/resvg) | Pure-Rust SVG renderer to raster | Renders SVG files to `image::RgbaImage`; useful for form-element SVGs (checkboxes, radio buttons) |
| [`tiny-skia`](https://crates.io/crates/tiny-skia) | Pure-Rust 2D graphics (Skia-based) | Used internally by `resvg`; provides rasterisation of paths, text, and images |

**Recommendation:** Use the `image` crate as the primary image I/O layer. Use `resvg` + `tiny-skia` to replace the SVG form-element rendering currently handled by `MyLooksStyle`.

---

## 4. Headless Browser Automation

The existing codebase embeds QtWebKit directly as a library. The most practical Rust-native path for accurate HTML rendering is to drive a headless browser via an automation protocol:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`chromiumoxide`](https://crates.io/crates/chromiumoxide) | Async Rust CDP (Chrome DevTools Protocol) client | Drives Chromium/Chrome headlessly; supports full page screenshots, PDF printing, JS execution, network interception; actively maintained with `tokio` integration |
| [`headless_chrome`](https://crates.io/crates/headless_chrome) | Synchronous CDP client | Simpler API; good for basic screenshot and PDF tasks; less actively maintained |
| [`fantoccini`](https://crates.io/crates/fantoccini) | Async WebDriver (W3C spec) client | Works with any WebDriver-compatible browser (Firefox via geckodriver, Chrome via chromedriver); standard W3C protocol |
| [`thirtyfour`](https://crates.io/crates/thirtyfour) | Async WebDriver client | Feature-rich WebDriver client; good for test automation scenarios |
| [`playwright-rust`](https://crates.io/crates/playwright) | Rust bindings for Microsoft Playwright | Wraps the Playwright Node.js library; requires Node.js runtime |

**Recommendation:** Use `chromiumoxide` as the primary headless browser driver. It provides native async Rust (`tokio`), CDP-level access for PDF printing (`Page.printToPDF`) and screenshots (`Page.captureScreenshot`), JS evaluation, and network request interception—closely matching the feature surface of the current QtWebKit integration.

---

## 5. CLI Parsing

The existing codebase implements a custom `CommandLineParserBase` hierarchy with bespoke argument registration, help generation, and man-page output. Rust's CLI ecosystem is mature:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`clap`](https://crates.io/crates/clap) | Full-featured CLI argument parser | Supports derive macros, subcommands, environment variable fallback, shell completions, and rich help formatting; the dominant choice in the Rust ecosystem; actively maintained |
| [`clap_mangen`](https://crates.io/crates/clap_mangen) | Generates man pages from `clap` definitions | Direct replacement for the manual man-page generation in `pdfdocparts.cc` |
| [`clap_complete`](https://crates.io/crates/clap_complete) | Generates shell completion scripts from `clap` | Bash, Zsh, Fish, PowerShell completions |
| [`argh`](https://crates.io/crates/argh) | Derive-based CLI parser (Google style) | Lighter weight than `clap`; less feature-rich |
| [`lexopt`](https://crates.io/crates/lexopt) | Low-level option tokeniser | Minimal; suitable if complete control over parsing is needed |

**Recommendation:** Use `clap` (v4, derive API) as the CLI parser, `clap_mangen` for man-page output, and `clap_complete` for shell completions. This replaces `CommandLineParserBase`, `PdfCommandLineParser`, `ImageCommandLineParser`, and the associated argument registration infrastructure.

---

## 6. HTTP Client / Network Loading

The existing codebase uses `QNetworkAccessManager` with custom subclasses for proxies, SSL client certificates, authentication, cookies, custom headers, and local-file access control. Rust equivalents:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`reqwest`](https://crates.io/crates/reqwest) | High-level async (and blocking) HTTP client | Built on `hyper`; supports HTTP/1.1 and HTTP/2, TLS via `rustls` or `native-tls`, cookie jars, proxy configuration, custom headers, multipart forms, streaming; widely used |
| [`hyper`](https://crates.io/crates/hyper) | Low-level async HTTP client and server | Used by `reqwest` internally; suitable for fine-grained control |
| [`ureq`](https://crates.io/crates/ureq) | Lightweight synchronous HTTP client | Minimal dependencies; suitable when async is not required |
| [`rustls`](https://crates.io/crates/rustls) | Pure-Rust TLS implementation | Replaces Qt's OpenSSL integration; used by `reqwest` when the `rustls-tls` feature is enabled |
| [`cookie_store`](https://crates.io/crates/cookie_store) | RFC-compliant cookie storage | Integrates with `reqwest` to persist cookies across requests (replaces Qt's cookie jar) |
| [`http-auth`](https://crates.io/crates/http-auth) | HTTP authentication (Basic, Digest) | Handles the auth-challenge flow currently in `ResourceObject` |

**Recommendation:** Use `reqwest` (async, with `rustls-tls` and `cookies` features) as the primary HTTP layer. Use `cookie_store` for the persistent cookie jar and `rustls` for TLS. For proxy support, `reqwest` exposes a `Proxy` builder that covers the `MyNetworkProxyFactory` functionality.

---

## 7. JavaScript Execution

The existing codebase relies on QtWebKit's embedded JavaScriptCore engine (via `QWebFrame::evaluateJavaScript`). Rust options for embedding or invoking a JS engine:

| Crate | Approach | Notes |
|-------|----------|-------|
| [`deno_core`](https://crates.io/crates/deno_core) | Embeds V8 via the Deno runtime core | Full ES2022+ support; async, module-aware; larger binary size; used by the Deno project |
| [`rusty_v8`](https://crates.io/crates/rusty_v8) | Low-level V8 bindings | Direct access to V8 API; basis for `deno_core` |
| [`rquickjs`](https://crates.io/crates/rquickjs) | Bindings to QuickJS (lightweight JS engine) | ES2020 support; small footprint; suitable for script injection without a full browser stack |
| [`boa_engine`](https://crates.io/crates/boa_engine) | Pure-Rust JavaScript engine | ECMAScript implementation in Rust; no native dependency; less complete ES spec coverage than V8 |
| [`quickjs-rs`](https://crates.io/crates/quickjs-rs) | Alternative QuickJS bindings | Simpler API than `rquickjs`; useful for running JS snippets |

**Recommendation:** If using the `chromiumoxide`-based approach (§4), JavaScript execution is provided automatically by the embedded V8 engine inside the headless browser—no separate JS crate is needed. For cases where only script injection or `window.status` polling is required without a full browser, `rquickjs` offers a lightweight, embeddable JS engine.

---

## 8. Subsystem Mapping Summary

The table below maps each wkhtmltopdf subsystem (from `docs/architecture.md`) to the recommended Rust crate(s):

| wkhtmltopdf Subsystem | C++/Qt Component | Recommended Rust Crate(s) |
|----------------------|-----------------|--------------------------|
| HTML/CSS rendering | `QWebPage` / QtWebKit | `chromiumoxide` (via headless Chromium); `html5ever` + `stylo` (pure Rust, limited) |
| PDF generation | `QPrinter` / `QPainter` | `printpdf`, `lopdf`, `pdf-writer` |
| Image output | `QImage` / `QImageWriter` | `image`, `resvg` (SVG form elements) |
| Headless page loading | `MultiPageLoader` / `MyQWebPage` | `chromiumoxide`, `fantoccini` |
| CLI parsing | `CommandLineParserBase` / `PdfCommandLineParser` | `clap` (v4), `clap_mangen`, `clap_complete` |
| HTTP networking | `QNetworkAccessManager` / `MyNetworkAccessManager` | `reqwest`, `rustls`, `cookie_store` |
| Proxy support | `MyNetworkProxyFactory` | `reqwest::Proxy` |
| JavaScript execution | `QWebFrame::evaluateJavaScript` / JavaScriptCore | V8 via `chromiumoxide`; `rquickjs` (lightweight) |
| Settings / configuration | Settings structs + reflection (`reflect.hh`) | `serde` + `serde_json`/`toml`; `config` crate |
| TOC / Outline | `Outline` / XSLT (`tocstylesheet.cc`) | `lopdf` (bookmarks); `quick-xml` (XSLT output); custom tree structure |
| SVG form element rendering | `MyLooksStyle` | `resvg` + `tiny-skia` |
| Async event loop | Qt event loop (`QApplication`) | `tokio` |

---

## 9. Additional Supporting Crates

| Purpose | Crate | Notes |
|---------|-------|-------|
| Async runtime | [`tokio`](https://crates.io/crates/tokio) | Standard async executor; required by `chromiumoxide` and `reqwest` |
| Serialization / settings | [`serde`](https://crates.io/crates/serde) + [`serde_json`](https://crates.io/crates/serde_json) | Derive-based serialization; replaces the reflection system in `reflect.hh` |
| Error handling | [`anyhow`](https://crates.io/crates/anyhow) / [`thiserror`](https://crates.io/crates/thiserror) | `thiserror` for library error types; `anyhow` for application-level error propagation |
| Logging | [`tracing`](https://crates.io/crates/tracing) + [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) | Structured, async-aware logging; replaces Qt's debug/info/warning/error signals |
| Temporary files | [`tempfile`](https://crates.io/crates/tempfile) | Replaces `TempFile` class used for TOC XSLT output |
| XML / XSLT | [`quick-xml`](https://crates.io/crates/quick-xml) | Fast XML reading/writing; replaces Qt's XML classes used in TOC generation |
| URL handling | [`url`](https://crates.io/crates/url) | WHATWG URL parsing; replaces `QUrl` and `guessUrlFromString` |
| Path / file I/O | std (`std::fs`, `std::path`) | Replaces Qt file utilities |
