# Headless Chrome Audit

This document summarizes every place in the repository where Chrome headless
(via the [`headless_chrome`](https://crates.io/crates/headless_chrome) crate) is
declared as a dependency, imported, or otherwise relied upon.

---

## 1. Cargo.toml dependencies

| File | Dependency | Version |
|------|-----------|---------|
| `crates/core/Cargo.toml` | `headless_chrome` | `"1"` |

No other `Cargo.toml` file declares a direct dependency on `headless_chrome` or
any other headless-browser crate. The `image`, `pdf`, `cli`, `ffi`, `tests`, and
`settings` crates consume the `wkhtmltopdf-core` crate and therefore gain the
headless-browser capability transitively, but they carry no direct declaration.

---

## 2. Source-file usages

### 2.1 `crates/core/src/renderer.rs` — **primary integration point**

This file owns the entire headless-browser integration.  It defines:

* **`HtmlInput`** — an enum that represents the HTML source to render (a remote
  URL or a local file path).
* **`RenderedPage`** — a type holding the raw output bytes and MIME type returned
  by a render operation.
* **`RenderError`** — the error type (`BackendUnavailable`, `RenderFailed`, `Io`).
* **`Renderer`** — a trait with a single `render(&self, input: &HtmlInput) ->
  Result<RenderedPage, RenderError>` method.
* **`HeadlessRenderer`** — the `Renderer` implementation that launches and drives
  a headless Chromium/WebKit browser instance via the `headless_chrome` crate.

Key `headless_chrome` API surface used inside `HeadlessRenderer::render`:

```rust
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::protocol::cdp::{Emulation, Page};
```

Specific operations performed:

| Step | `headless_chrome` API |
|------|-----------------------|
| Launch browser | `Browser::new(LaunchOptions { headless, sandbox, .. })` |
| Open a tab | `browser.new_tab()` |
| Disable JavaScript | `tab.call_method(Emulation::SetScriptExecutionDisabled { value: true })` |
| Navigate | `tab.navigate_to(&url)` |
| Wait for load | `tab.wait_until_navigated()` |
| Execute scripts | `tab.evaluate(script, false)` |
| Capture screenshot | `tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, …)` |

`HeadlessRenderer` also exposes the following public fields that map directly to
`LaunchOptions` / browser behaviour:

| Field | Type | Default | Purpose |
|-------|------|---------|---------|
| `headless` | `bool` | `true` | Hide the browser window |
| `sandbox` | `bool` | `true` | Enable the OS sandbox |
| `enable_javascript` | `bool` | `true` | Allow JS execution |
| `js_delay` | `u32` | `200` ms | Pause after page load |
| `run_scripts` | `Vec<String>` | empty | Custom JS to execute |

### 2.2 `crates/core/src/lib.rs` — re-export

```rust
pub use renderer::{HeadlessRenderer, HtmlInput, RenderError, RenderedPage, Renderer};
```

`HeadlessRenderer` (and supporting types) are re-exported from the crate root so
downstream crates can import them without knowing the internal module path.

### 2.3 `crates/image/src/lib.rs` — consumer

The `ImageConverter::convert` method instantiates a `HeadlessRenderer` and calls
`render` to obtain a PNG screenshot, which it then post-processes (crop, resize,
re-encode):

```rust
use wkhtmltopdf_core::{ConvertError, Converter, HeadlessRenderer, HtmlInput, Renderer};
// …
let renderer = HeadlessRenderer::with_js_settings(
    self.settings.web.enable_javascript,
    self.settings.load_page.js_delay,
    self.settings.load_page.run_script.clone(),
);
let rendered = renderer.render(&input)
    .map_err(|e| ConvertError::Render(e.to_string()))?;
```

### 2.4 `crates/tests/tests/visual_regression.rs` — test harness

The visual-regression test suite renders HTML fixtures via `ImageConverter` (and
therefore via `HeadlessRenderer`) and compares the output against stored reference
images.  It gracefully skips each test when Chrome is absent so the suite stays
green in environments without a browser installed:

```rust
Err(ConvertError::Render(ref msg))
    if msg.contains("browser backend unavailable") =>
{
    eprintln!("SKIP '{}': headless browser unavailable – {}", name, msg);
    None
}
```

---

## 3. Integration-point summary

```
Cargo dependency
  └── crates/core/Cargo.toml
        headless_chrome = "1"

Primary implementation
  └── crates/core/src/renderer.rs
        struct HeadlessRenderer  (implements Renderer trait)
        fn render() — uses Browser, LaunchOptions, Tab, Page CDP types

Re-exported public API
  └── crates/core/src/lib.rs
        pub use renderer::HeadlessRenderer

Consumers
  ├── crates/image/src/lib.rs
  │     ImageConverter::convert() → HeadlessRenderer::with_js_settings()
  └── crates/tests/tests/visual_regression.rs
        visual_regression_* tests → ImageConverter → HeadlessRenderer
```

---

## 4. Runtime requirement

Because `headless_chrome` spawns a real Chrome/Chromium process, any environment
that runs code relying on `HeadlessRenderer` must have a compatible Chrome or
Chromium binary available on `PATH` (or at the path the crate auto-discovers).
The `BackendUnavailable` error variant is returned when no binary is found,
allowing callers to fail gracefully.
