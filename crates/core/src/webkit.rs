//! WebKit rendering backend using native platform WebKit.
//!
//! - **macOS**: WKWebView (WebKit.framework + AppKit.framework)
//! - **Linux**: WebKitGTK (webkit2gtk + GTK 3)
//!
//! The native implementations live in `webkit_render_macos.m` and
//! `webkit_render_linux.c`; this module provides safe Rust wrappers
//! around the C FFI.
//!
//! When `has_webkit` is not set at build time (native libraries were not
//! found), every public function still exists but returns a descriptive
//! [`RenderError::BackendUnavailable`] error so that downstream crates
//! can compile unconditionally.

use crate::renderer::{HtmlInput, RenderError, RenderedPage, Renderer};

// ---------------------------------------------------------------------------
// FFI declarations — only compiled when native WebKit is available
// ---------------------------------------------------------------------------

#[cfg(has_webkit)]
mod ffi {
    use std::os::raw::{c_char, c_int, c_uchar};

    #[repr(C)]
    pub(super) struct WkPdfOptions {
        pub page_width_mm: f64,
        pub page_height_mm: f64,
        pub margin_top_mm: f64,
        pub margin_bottom_mm: f64,
        pub margin_left_mm: f64,
        pub margin_right_mm: f64,
        pub print_backgrounds: c_int,
        pub js_delay_ms: c_int,
    }

    extern "C" {
        pub fn wk_render_pdf(
            url: *const c_char,
            opts: *const WkPdfOptions,
            out_data: *mut *mut c_uchar,
            out_len: *mut usize,
        ) -> c_int;

        pub fn wk_render_png(
            url: *const c_char,
            viewport_width: c_int,
            viewport_height: c_int,
            js_delay_ms: c_int,
            out_data: *mut *mut c_uchar,
            out_len: *mut usize,
        ) -> c_int;

        pub fn wk_last_error() -> *const c_char;

        pub fn wk_free(ptr: *mut c_uchar);
    }
}

/// Read the thread-local error message from the C side.
#[cfg(has_webkit)]
fn last_error() -> String {
    unsafe {
        let ptr = ffi::wk_last_error();
        if ptr.is_null() {
            return "unknown error".into();
        }
        std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned()
    }
}

/// Message returned when WebKit was not found at build time.
#[cfg(not(has_webkit))]
const NOT_AVAILABLE: &str =
    "WebKit backend is not available: the native WebKit libraries were not \
     found at build time. Install webkit2gtk-4.1-dev (Linux) or use macOS, \
     then rebuild.";

// ---------------------------------------------------------------------------
// WebkitPdfOptions — Rust mirror of WkPdfOptions for public API
// ---------------------------------------------------------------------------

/// Options for WebKit PDF rendering.
pub struct WebkitPdfOptions {
    pub page_width_mm: f64,
    pub page_height_mm: f64,
    pub margin_top_mm: f64,
    pub margin_bottom_mm: f64,
    pub margin_left_mm: f64,
    pub margin_right_mm: f64,
    pub print_backgrounds: bool,
    pub js_delay_ms: u32,
}

impl Default for WebkitPdfOptions {
    fn default() -> Self {
        // A4 in mm: 210 × 297
        Self {
            page_width_mm: 210.0,
            page_height_mm: 297.0,
            margin_top_mm: 10.0,
            margin_bottom_mm: 10.0,
            margin_left_mm: 10.0,
            margin_right_mm: 10.0,
            print_backgrounds: true,
            js_delay_ms: 200,
        }
    }
}

// ---------------------------------------------------------------------------
// webkit_print_to_pdf — high-level PDF rendering function
// ---------------------------------------------------------------------------

/// Render an HTML page to PDF bytes via the native WebKit engine.
#[cfg(has_webkit)]
pub fn webkit_print_to_pdf(
    input: &HtmlInput,
    opts: &WebkitPdfOptions,
) -> Result<Vec<u8>, RenderError> {
    use std::ffi::CString;
    use std::os::raw::c_int;
    use std::ptr;

    let url_str = input.to_url_string();
    let c_url = CString::new(url_str.as_str()).map_err(|e| {
        RenderError::RenderFailed(format!("URL contains interior NUL byte: {e}"))
    })?;

    let c_opts = ffi::WkPdfOptions {
        page_width_mm: opts.page_width_mm,
        page_height_mm: opts.page_height_mm,
        margin_top_mm: opts.margin_top_mm,
        margin_bottom_mm: opts.margin_bottom_mm,
        margin_left_mm: opts.margin_left_mm,
        margin_right_mm: opts.margin_right_mm,
        print_backgrounds: opts.print_backgrounds as c_int,
        js_delay_ms: opts.js_delay_ms as c_int,
    };

    let mut data: *mut u8 = ptr::null_mut();
    let mut len: usize = 0;

    let rc = unsafe { ffi::wk_render_pdf(c_url.as_ptr(), &c_opts, &mut data, &mut len) };

    if rc != 0 {
        return Err(match rc {
            -1 => RenderError::BackendUnavailable(last_error()),
            -2 => RenderError::RenderFailed(format!("page load failed: {}", last_error())),
            -3 => RenderError::RenderFailed(last_error()),
            _ => RenderError::Io(std::io::Error::other(last_error())),
        });
    }

    if data.is_null() || len == 0 {
        return Err(RenderError::RenderFailed(
            "WebKit produced empty output".into(),
        ));
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, len).to_vec() };
    unsafe { ffi::wk_free(data) };

    Ok(bytes)
}

/// Stub when native WebKit was not found at build time.
#[cfg(not(has_webkit))]
pub fn webkit_print_to_pdf(
    _input: &HtmlInput,
    _opts: &WebkitPdfOptions,
) -> Result<Vec<u8>, RenderError> {
    Err(RenderError::BackendUnavailable(NOT_AVAILABLE.into()))
}

// ---------------------------------------------------------------------------
// WebkitRenderer — Renderer trait implementation (screenshots → PNG)
// ---------------------------------------------------------------------------

/// A [`Renderer`] that produces PNG screenshots via the native WebKit engine.
pub struct WebkitRenderer {
    /// Viewport width in pixels (0 = default 1280).
    pub viewport_width: i32,
    /// Viewport height in pixels (0 = default 960).
    pub viewport_height: i32,
    /// Milliseconds to wait after page load for JavaScript to settle.
    pub js_delay: u32,
}

impl WebkitRenderer {
    pub fn new() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 960,
            js_delay: 200,
        }
    }
}

impl Default for WebkitRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for WebkitRenderer {
    #[cfg(has_webkit)]
    fn render(&self, input: &HtmlInput) -> Result<RenderedPage, RenderError> {
        use std::ffi::CString;
        use std::os::raw::c_int;
        use std::ptr;

        let url_str = input.to_url_string();
        let c_url = CString::new(url_str.as_str()).map_err(|e| {
            RenderError::RenderFailed(format!("URL contains interior NUL byte: {e}"))
        })?;

        let mut data: *mut u8 = ptr::null_mut();
        let mut len: usize = 0;

        let rc = unsafe {
            ffi::wk_render_png(
                c_url.as_ptr(),
                self.viewport_width as c_int,
                self.viewport_height as c_int,
                self.js_delay as c_int,
                &mut data,
                &mut len,
            )
        };

        if rc != 0 {
            return Err(match rc {
                -1 => RenderError::BackendUnavailable(last_error()),
                -2 => RenderError::RenderFailed(format!(
                    "page load failed: {}",
                    last_error()
                )),
                -3 => RenderError::RenderFailed(last_error()),
                _ => RenderError::Io(std::io::Error::other(last_error())),
            });
        }

        if data.is_null() || len == 0 {
            return Err(RenderError::RenderFailed(
                "WebKit produced empty screenshot".into(),
            ));
        }

        let bytes = unsafe { std::slice::from_raw_parts(data, len).to_vec() };
        unsafe { ffi::wk_free(data) };

        Ok(RenderedPage {
            bytes,
            mime_type: "image/png".into(),
        })
    }

    #[cfg(not(has_webkit))]
    fn render(&self, _input: &HtmlInput) -> Result<RenderedPage, RenderError> {
        Err(RenderError::BackendUnavailable(NOT_AVAILABLE.into()))
    }
}
