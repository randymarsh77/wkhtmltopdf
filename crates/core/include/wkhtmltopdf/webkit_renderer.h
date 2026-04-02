#pragma once

// webkit_renderer.h – C++ interface for the Qt WebEngine rendering backend.
//
// This header is consumed by the cxx bridge generated from `qt_webkit.rs`.
// It declares a single free function that accepts a URL string and rendering
// options, loads the page via QWebEngineView, executes any requested
// JavaScript snippets, and returns a PNG screenshot as a byte vector that
// cxx can safely hand to Rust.

#include "rust/cxx.h"

#include <cstdint>

namespace wkhtmltopdf {

/// Render the HTML page at `url` using Qt WebEngine and return the result as
/// raw PNG image bytes.
///
/// @param url          HTTP/HTTPS/file URL to render.
/// @param js_enabled   Whether JavaScript execution is enabled on the page.
/// @param js_delay_ms  Milliseconds to wait after the initial `loadFinished`
///                     signal before capturing the screenshot.  Useful when
///                     pages use JS to populate content after load.
/// @param run_scripts  JavaScript snippets to evaluate in the page context
///                     after the page has loaded (and after the settle delay).
///                     Each script is run sequentially and the call blocks
///                     until the script finishes before the next one starts.
/// @return             Raw PNG bytes of the rendered viewport screenshot.
/// @throws std::runtime_error  When the page fails to load or Qt WebEngine is
///                             not available.
rust::Vec<uint8_t> render_url(rust::Str url, bool js_enabled,
                               uint32_t js_delay_ms,
                               rust::Slice<const rust::Str> run_scripts);

} // namespace wkhtmltopdf
