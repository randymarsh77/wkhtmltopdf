//! C-compatible shared library API for wkhtmltopdf (libwkhtmltox replacement).
//!
//! This crate exposes a C ABI with the same function signatures as the original
//! `libwkhtmltox`, providing drop-in compatibility with existing language
//! bindings (Python, Ruby, Node.js, etc.) that load the library at runtime.
//!
//! # PDF API
//!
//! The PDF conversion workflow mirrors the original `libwkhtmltox`:
//! 1. Call [`wkhtmltopdf_init`].
//! 2. Create global settings with [`wkhtmltopdf_create_global_settings`] and
//!    set fields via [`wkhtmltopdf_set_global_setting`].
//! 3. Create one or more object settings with
//!    [`wkhtmltopdf_create_object_settings`] and populate them.
//! 4. Create a converter with [`wkhtmltopdf_create_converter`].
//! 5. Add objects with [`wkhtmltopdf_add_object`].
//! 6. Optionally register callbacks.
//! 7. Call [`wkhtmltopdf_convert`].
//! 8. Retrieve output bytes with [`wkhtmltopdf_get_output`].
//! 9. Destroy the converter and settings, then call [`wkhtmltopdf_deinit`].
//!
//! # Image API
//!
//! The image conversion workflow is analogous using `wkhtmltoimage_*` symbols.

#![allow(non_camel_case_types)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_long, c_uchar};
use std::ptr;

use wkhtmltopdf_core::Converter;
use wkhtmltopdf_image::ImageConverter;
use wkhtmltopdf_pdf::PdfConverter;
use wkhtmltopdf_settings::{
    ColorMode, ImageGlobal, LoadErrorHandling, Orientation, PdfGlobal, PdfObject, PageSize,
    Unit, UnitReal,
};

// ---------------------------------------------------------------------------
// Library version
// ---------------------------------------------------------------------------

const VERSION: &[u8] = b"0.12.6\0";

// ---------------------------------------------------------------------------
// PDF opaque types
// ---------------------------------------------------------------------------

/// Opaque global settings for the PDF converter.
pub struct wkhtmltopdf_global_settings {
    inner: PdfGlobal,
}

/// Opaque per-page object settings for the PDF converter.
pub struct wkhtmltopdf_object_settings {
    inner: PdfObject,
}

/// Opaque PDF converter state.
pub struct wkhtmltopdf_converter {
    global: PdfGlobal,
    /// Collected (object_settings, optional_inline_html) pairs.
    objects: Vec<(PdfObject, Option<String>)>,
    /// Raw PDF bytes after a successful [`wkhtmltopdf_convert`] call.
    output: Option<Vec<u8>>,
    /// Current conversion phase index (0-based).
    current_phase: c_int,
    /// HTTP error code from the last conversion (0 = none).
    http_error_code: c_int,
    /// Cached progress description string.
    progress_str: CString,
    /// Per-phase description strings.
    phase_descriptions: Vec<CString>,
    // Callbacks ---
    warning_cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, *const c_char)>,
    error_cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, *const c_char)>,
    phase_changed_cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter)>,
    progress_changed_cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, c_int)>,
    finished_cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, c_int)>,
}

// ---------------------------------------------------------------------------
// Image opaque types
// ---------------------------------------------------------------------------

/// Opaque global settings for the image converter.
pub struct wkhtmltoimage_global_settings {
    inner: ImageGlobal,
}

/// Opaque image converter state.
pub struct wkhtmltoimage_converter {
    settings: ImageGlobal,
    /// Inline HTML data supplied at converter-creation time (if any).
    inline_data: Option<String>,
    /// Raw image bytes after a successful [`wkhtmltoimage_convert`] call.
    output: Option<Vec<u8>>,
    /// Current conversion phase index (0-based).
    current_phase: c_int,
    /// HTTP error code from the last conversion (0 = none).
    http_error_code: c_int,
    /// Cached progress description string.
    progress_str: CString,
    /// Per-phase description strings.
    phase_descriptions: Vec<CString>,
    // Callbacks ---
    warning_cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, *const c_char)>,
    error_cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, *const c_char)>,
    phase_changed_cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter)>,
    progress_changed_cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, c_int)>,
    finished_cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, c_int)>,
}

// ---------------------------------------------------------------------------
// Helper: safe CStr → &str conversion
// ---------------------------------------------------------------------------

/// Convert a raw C string pointer to a Rust `&str`, returning `""` on null or
/// invalid UTF-8.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> &'a str {
    if ptr.is_null() {
        return "";
    }
    CStr::from_ptr(ptr).to_str().unwrap_or("")
}

/// Copy `value` into the caller-provided buffer `out` (length `vs`).
/// Returns 1 if the value fits (including NUL terminator), 0 otherwise.
unsafe fn write_setting_str(value: &str, out: *mut c_char, vs: c_int) -> c_int {
    if out.is_null() || vs <= 0 {
        return 0;
    }
    let capacity = vs as usize;
    let bytes = value.as_bytes();
    // We need space for the string plus a NUL terminator.
    if bytes.len() + 1 > capacity {
        return 0;
    }
    ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, out, bytes.len());
    *out.add(bytes.len()) = 0;
    1
}

// ---------------------------------------------------------------------------
// PDF global settings helpers
// ---------------------------------------------------------------------------

fn pdf_global_set(gs: &mut PdfGlobal, name: &str, value: &str) -> bool {
    match name {
        "out" => {
            gs.output = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        "dpi" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.dpi = Some(v);
            }
        }
        "imageDPI" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.image_dpi = v;
            }
        }
        "imageQuality" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.image_quality = v;
            }
        }
        "useCompression" => {
            gs.use_compression = !matches!(value, "false" | "0");
        }
        "pageOffset" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.page_offset = v;
            }
        }
        "copies" => {
            if let Ok(v) = value.parse::<u32>() {
                gs.copies = v;
            }
        }
        "collate" => {
            gs.collate = !matches!(value, "false" | "0");
        }
        "outline" => {
            gs.outline = !matches!(value, "false" | "0");
        }
        "outlineDepth" => {
            if let Ok(v) = value.parse::<u32>() {
                gs.outline_depth = v;
            }
        }
        "dumpOutline" => {
            gs.dump_outline = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        "title" => {
            gs.document_title = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        "author" => {
            gs.author = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        "subject" => {
            gs.subject = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        "colorMode" => {
            gs.color_mode = match value {
                "Grayscale" | "grayscale" => ColorMode::Grayscale,
                _ => ColorMode::Color,
            };
        }
        "orientation" => {
            gs.orientation = match value {
                "Landscape" | "landscape" => Orientation::Landscape,
                _ => Orientation::Portrait,
            };
        }
        "size.pageSize" => {
            gs.size.page_size = parse_page_size(value);
        }
        "size.width" => {
            if let Some(ur) = parse_unit_real(value) {
                gs.size.width = Some(ur);
            }
        }
        "size.height" => {
            if let Some(ur) = parse_unit_real(value) {
                gs.size.height = Some(ur);
            }
        }
        "margin.top" => {
            if let Some(ur) = parse_unit_real(value) {
                gs.margin.top = ur;
            }
        }
        "margin.right" => {
            if let Some(ur) = parse_unit_real(value) {
                gs.margin.right = ur;
            }
        }
        "margin.bottom" => {
            if let Some(ur) = parse_unit_real(value) {
                gs.margin.bottom = ur;
            }
        }
        "margin.left" => {
            if let Some(ur) = parse_unit_real(value) {
                gs.margin.left = ur;
            }
        }
        "viewportSize" => {
            gs.viewport_size = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        _ => return false,
    }
    true
}

fn pdf_global_get(gs: &PdfGlobal, name: &str) -> Option<String> {
    match name {
        "out" => Some(gs.output.clone().unwrap_or_default()),
        "dpi" => Some(gs.dpi.map(|v| v.to_string()).unwrap_or_default()),
        "imageDPI" => Some(gs.image_dpi.to_string()),
        "imageQuality" => Some(gs.image_quality.to_string()),
        "useCompression" => Some(bool_str(gs.use_compression)),
        "pageOffset" => Some(gs.page_offset.to_string()),
        "copies" => Some(gs.copies.to_string()),
        "collate" => Some(bool_str(gs.collate)),
        "outline" => Some(bool_str(gs.outline)),
        "outlineDepth" => Some(gs.outline_depth.to_string()),
        "dumpOutline" => Some(gs.dump_outline.clone().unwrap_or_default()),
        "title" => Some(gs.document_title.clone().unwrap_or_default()),
        "author" => Some(gs.author.clone().unwrap_or_default()),
        "subject" => Some(gs.subject.clone().unwrap_or_default()),
        "colorMode" => Some(
            match gs.color_mode {
                ColorMode::Grayscale => "Grayscale",
                ColorMode::Color => "Color",
            }
            .to_owned(),
        ),
        "orientation" => Some(
            match gs.orientation {
                Orientation::Landscape => "Landscape",
                Orientation::Portrait => "Portrait",
            }
            .to_owned(),
        ),
        "size.pageSize" => Some(format!("{:?}", gs.size.page_size)),
        "size.width" => Some(
            gs.size
                .width
                .as_ref()
                .map(unit_real_to_str)
                .unwrap_or_default(),
        ),
        "size.height" => Some(
            gs.size
                .height
                .as_ref()
                .map(unit_real_to_str)
                .unwrap_or_default(),
        ),
        "margin.top" => Some(unit_real_to_str(&gs.margin.top)),
        "margin.right" => Some(unit_real_to_str(&gs.margin.right)),
        "margin.bottom" => Some(unit_real_to_str(&gs.margin.bottom)),
        "margin.left" => Some(unit_real_to_str(&gs.margin.left)),
        "viewportSize" => Some(gs.viewport_size.clone().unwrap_or_default()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// PDF object settings helpers
// ---------------------------------------------------------------------------

fn pdf_object_set(obj: &mut PdfObject, name: &str, value: &str) -> bool {
    match name {
        "page" => {
            obj.page = if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            };
        }
        "useExternalLinks" => {
            obj.use_external_links = !matches!(value, "false" | "0");
        }
        "useLocalLinks" => {
            obj.use_local_links = !matches!(value, "false" | "0");
        }
        "produceForms" => {
            obj.produce_forms = !matches!(value, "false" | "0");
        }
        "includeInOutline" => {
            obj.include_in_outline = !matches!(value, "false" | "0");
        }
        "pagesCount" => {
            obj.pages_count = !matches!(value, "false" | "0");
        }
        "toc.useDottedLines" => {
            obj.toc.use_dotted_lines = !matches!(value, "false" | "0");
        }
        "toc.captionText" => {
            obj.toc.caption_text = value.to_owned();
        }
        "toc.forwardLinks" => {
            obj.toc.forward_links = !matches!(value, "false" | "0");
        }
        "toc.backLinks" => {
            obj.toc.back_links = !matches!(value, "false" | "0");
        }
        "toc.indentation" => {
            obj.toc.indentation = value.to_owned();
        }
        "toc.fontScale" => {
            if let Ok(v) = value.parse::<f32>() {
                obj.toc.font_scale = v;
            }
        }
        "toc.depth" => {
            if let Ok(v) = value.parse::<u32>() {
                obj.toc.depth = v;
            }
        }
        "header.fontSize" => {
            if let Ok(v) = value.parse::<i32>() {
                obj.header.font_size = v;
            }
        }
        "header.fontName" => {
            obj.header.font_name = value.to_owned();
        }
        "header.left" => {
            obj.header.left = opt_str(value);
        }
        "header.center" => {
            obj.header.center = opt_str(value);
        }
        "header.right" => {
            obj.header.right = opt_str(value);
        }
        "header.line" => {
            obj.header.line = !matches!(value, "false" | "0");
        }
        "header.htmlUrl" => {
            obj.header.html_url = opt_str(value);
        }
        "header.spacing" => {
            if let Ok(v) = value.parse::<f32>() {
                obj.header.spacing = v;
            }
        }
        "footer.fontSize" => {
            if let Ok(v) = value.parse::<i32>() {
                obj.footer.font_size = v;
            }
        }
        "footer.fontName" => {
            obj.footer.font_name = value.to_owned();
        }
        "footer.left" => {
            obj.footer.left = opt_str(value);
        }
        "footer.center" => {
            obj.footer.center = opt_str(value);
        }
        "footer.right" => {
            obj.footer.right = opt_str(value);
        }
        "footer.line" => {
            obj.footer.line = !matches!(value, "false" | "0");
        }
        "footer.htmlUrl" => {
            obj.footer.html_url = opt_str(value);
        }
        "footer.spacing" => {
            if let Ok(v) = value.parse::<f32>() {
                obj.footer.spacing = v;
            }
        }
        "web.background" => {
            obj.web.background = !matches!(value, "false" | "0");
        }
        "web.loadImages" => {
            obj.web.load_images = !matches!(value, "false" | "0");
        }
        "web.enableJavascript" | "web.enableJavaScript" => {
            obj.web.enable_javascript = !matches!(value, "false" | "0");
        }
        "web.enableIntelligentShrinking" => {
            obj.web.enable_intelligent_shrinking = !matches!(value, "false" | "0");
        }
        "web.minimumFontSize" => {
            if let Ok(v) = value.parse::<i32>() {
                obj.web.minimum_font_size = Some(v);
            } else if value.is_empty() {
                obj.web.minimum_font_size = None;
            }
        }
        "web.defaultEncoding" => {
            obj.web.default_encoding = opt_str(value);
        }
        "web.userStyleSheet" => {
            obj.web.user_style_sheet = opt_str(value);
        }
        "web.enablePlugins" => {
            obj.web.enable_plugins = !matches!(value, "false" | "0");
        }
        "load.username" => {
            obj.load.username = opt_str(value);
        }
        "load.password" => {
            obj.load.password = opt_str(value);
        }
        "load.jsDelay" => {
            if let Ok(v) = value.parse::<u32>() {
                obj.load.js_delay = v;
            }
        }
        "load.zoomFactor" => {
            if let Ok(v) = value.parse::<f64>() {
                obj.load.zoom = v;
            }
        }
        "load.blockLocalFileAccess" => {
            obj.load.block_local_file_access = !matches!(value, "false" | "0");
        }
        "load.stopSlowScripts" => {
            obj.load.stop_slow_scripts = !matches!(value, "false" | "0");
        }
        "load.debugJavascript" | "load.debugJavaScript" => {
            obj.load.debug_javascript = !matches!(value, "false" | "0");
        }
        "load.loadErrorHandling" => {
            obj.load.load_error_handling = parse_load_error_handling(value);
        }
        "load.mediaLoadErrorHandling" => {
            obj.load.media_load_error_handling = parse_load_error_handling(value);
        }
        "load.printMediaType" => {
            obj.load.print_media_type = !matches!(value, "false" | "0");
        }
        _ => return false,
    }
    true
}

fn pdf_object_get(obj: &PdfObject, name: &str) -> Option<String> {
    match name {
        "page" => Some(obj.page.clone().unwrap_or_default()),
        "useExternalLinks" => Some(bool_str(obj.use_external_links)),
        "useLocalLinks" => Some(bool_str(obj.use_local_links)),
        "produceForms" => Some(bool_str(obj.produce_forms)),
        "includeInOutline" => Some(bool_str(obj.include_in_outline)),
        "pagesCount" => Some(bool_str(obj.pages_count)),
        "toc.useDottedLines" => Some(bool_str(obj.toc.use_dotted_lines)),
        "toc.captionText" => Some(obj.toc.caption_text.clone()),
        "toc.forwardLinks" => Some(bool_str(obj.toc.forward_links)),
        "toc.backLinks" => Some(bool_str(obj.toc.back_links)),
        "toc.indentation" => Some(obj.toc.indentation.clone()),
        "toc.fontScale" => Some(obj.toc.font_scale.to_string()),
        "toc.depth" => Some(obj.toc.depth.to_string()),
        "header.fontSize" => Some(obj.header.font_size.to_string()),
        "header.fontName" => Some(obj.header.font_name.clone()),
        "header.left" => Some(obj.header.left.clone().unwrap_or_default()),
        "header.center" => Some(obj.header.center.clone().unwrap_or_default()),
        "header.right" => Some(obj.header.right.clone().unwrap_or_default()),
        "header.line" => Some(bool_str(obj.header.line)),
        "header.htmlUrl" => Some(obj.header.html_url.clone().unwrap_or_default()),
        "header.spacing" => Some(obj.header.spacing.to_string()),
        "footer.fontSize" => Some(obj.footer.font_size.to_string()),
        "footer.fontName" => Some(obj.footer.font_name.clone()),
        "footer.left" => Some(obj.footer.left.clone().unwrap_or_default()),
        "footer.center" => Some(obj.footer.center.clone().unwrap_or_default()),
        "footer.right" => Some(obj.footer.right.clone().unwrap_or_default()),
        "footer.line" => Some(bool_str(obj.footer.line)),
        "footer.htmlUrl" => Some(obj.footer.html_url.clone().unwrap_or_default()),
        "footer.spacing" => Some(obj.footer.spacing.to_string()),
        "web.background" => Some(bool_str(obj.web.background)),
        "web.loadImages" => Some(bool_str(obj.web.load_images)),
        "web.enableJavascript" | "web.enableJavaScript" => {
            Some(bool_str(obj.web.enable_javascript))
        }
        "web.enableIntelligentShrinking" => Some(bool_str(obj.web.enable_intelligent_shrinking)),
        "web.minimumFontSize" => Some(
            obj.web
                .minimum_font_size
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        "web.defaultEncoding" => Some(obj.web.default_encoding.clone().unwrap_or_default()),
        "web.userStyleSheet" => Some(obj.web.user_style_sheet.clone().unwrap_or_default()),
        "web.enablePlugins" => Some(bool_str(obj.web.enable_plugins)),
        "load.username" => Some(obj.load.username.clone().unwrap_or_default()),
        "load.password" => Some(obj.load.password.clone().unwrap_or_default()),
        "load.jsDelay" => Some(obj.load.js_delay.to_string()),
        "load.zoomFactor" => Some(obj.load.zoom.to_string()),
        "load.blockLocalFileAccess" => Some(bool_str(obj.load.block_local_file_access)),
        "load.stopSlowScripts" => Some(bool_str(obj.load.stop_slow_scripts)),
        "load.debugJavascript" | "load.debugJavaScript" => {
            Some(bool_str(obj.load.debug_javascript))
        }
        "load.loadErrorHandling" => Some(format!("{:?}", obj.load.load_error_handling)),
        "load.mediaLoadErrorHandling" => {
            Some(format!("{:?}", obj.load.media_load_error_handling))
        }
        "load.printMediaType" => Some(bool_str(obj.load.print_media_type)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Image global settings helpers
// ---------------------------------------------------------------------------

fn image_global_set(gs: &mut ImageGlobal, name: &str, value: &str) -> bool {
    match name {
        "out" => {
            gs.output = opt_str(value);
        }
        "in" => {
            gs.page = opt_str(value);
        }
        "fmt" => {
            gs.format = opt_str(value);
        }
        "screenWidth" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.screen_width = Some(v);
            } else if value.is_empty() {
                gs.screen_width = None;
            }
        }
        "screenHeight" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.screen_height = Some(v);
            } else if value.is_empty() {
                gs.screen_height = None;
            }
        }
        "smartWidth" => {
            gs.smart_width = !matches!(value, "false" | "0");
        }
        "quality" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.quality = v;
            }
        }
        "transparent" => {
            gs.transparent = !matches!(value, "false" | "0");
        }
        "dpi" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.dpi = Some(v);
            } else if value.is_empty() {
                gs.dpi = None;
            }
        }
        "crop.left" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.crop.left = v;
            }
        }
        "crop.top" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.crop.top = v;
            }
        }
        "crop.width" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.crop.width = v;
            }
        }
        "crop.height" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.crop.height = v;
            }
        }
        "web.background" => {
            gs.web.background = !matches!(value, "false" | "0");
        }
        "web.loadImages" => {
            gs.web.load_images = !matches!(value, "false" | "0");
        }
        "web.enableJavascript" | "web.enableJavaScript" => {
            gs.web.enable_javascript = !matches!(value, "false" | "0");
        }
        "web.enableIntelligentShrinking" => {
            gs.web.enable_intelligent_shrinking = !matches!(value, "false" | "0");
        }
        "web.minimumFontSize" => {
            if let Ok(v) = value.parse::<i32>() {
                gs.web.minimum_font_size = Some(v);
            } else if value.is_empty() {
                gs.web.minimum_font_size = None;
            }
        }
        "web.defaultEncoding" => {
            gs.web.default_encoding = opt_str(value);
        }
        "web.userStyleSheet" => {
            gs.web.user_style_sheet = opt_str(value);
        }
        "web.enablePlugins" => {
            gs.web.enable_plugins = !matches!(value, "false" | "0");
        }
        "load.username" => {
            gs.load_page.username = opt_str(value);
        }
        "load.password" => {
            gs.load_page.password = opt_str(value);
        }
        "load.jsDelay" => {
            if let Ok(v) = value.parse::<u32>() {
                gs.load_page.js_delay = v;
            }
        }
        "load.zoomFactor" => {
            if let Ok(v) = value.parse::<f64>() {
                gs.load_page.zoom = v;
            }
        }
        "load.blockLocalFileAccess" => {
            gs.load_page.block_local_file_access = !matches!(value, "false" | "0");
        }
        "load.stopSlowScripts" => {
            gs.load_page.stop_slow_scripts = !matches!(value, "false" | "0");
        }
        "load.debugJavascript" | "load.debugJavaScript" => {
            gs.load_page.debug_javascript = !matches!(value, "false" | "0");
        }
        "load.loadErrorHandling" => {
            gs.load_page.load_error_handling = parse_load_error_handling(value);
        }
        _ => return false,
    }
    true
}

fn image_global_get(gs: &ImageGlobal, name: &str) -> Option<String> {
    match name {
        "out" => Some(gs.output.clone().unwrap_or_default()),
        "in" => Some(gs.page.clone().unwrap_or_default()),
        "fmt" => Some(gs.format.clone().unwrap_or_default()),
        "screenWidth" => Some(
            gs.screen_width
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        "screenHeight" => Some(
            gs.screen_height
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        "smartWidth" => Some(bool_str(gs.smart_width)),
        "quality" => Some(gs.quality.to_string()),
        "transparent" => Some(bool_str(gs.transparent)),
        "dpi" => Some(gs.dpi.map(|v| v.to_string()).unwrap_or_default()),
        "crop.left" => Some(gs.crop.left.to_string()),
        "crop.top" => Some(gs.crop.top.to_string()),
        "crop.width" => Some(gs.crop.width.to_string()),
        "crop.height" => Some(gs.crop.height.to_string()),
        "web.background" => Some(bool_str(gs.web.background)),
        "web.loadImages" => Some(bool_str(gs.web.load_images)),
        "web.enableJavascript" | "web.enableJavaScript" => {
            Some(bool_str(gs.web.enable_javascript))
        }
        "web.enableIntelligentShrinking" => Some(bool_str(gs.web.enable_intelligent_shrinking)),
        "web.minimumFontSize" => Some(
            gs.web
                .minimum_font_size
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        "web.defaultEncoding" => Some(gs.web.default_encoding.clone().unwrap_or_default()),
        "web.userStyleSheet" => Some(gs.web.user_style_sheet.clone().unwrap_or_default()),
        "web.enablePlugins" => Some(bool_str(gs.web.enable_plugins)),
        "load.username" => Some(gs.load_page.username.clone().unwrap_or_default()),
        "load.password" => Some(gs.load_page.password.clone().unwrap_or_default()),
        "load.jsDelay" => Some(gs.load_page.js_delay.to_string()),
        "load.zoomFactor" => Some(gs.load_page.zoom.to_string()),
        "load.blockLocalFileAccess" => Some(bool_str(gs.load_page.block_local_file_access)),
        "load.stopSlowScripts" => Some(bool_str(gs.load_page.stop_slow_scripts)),
        "load.debugJavascript" | "load.debugJavaScript" => {
            Some(bool_str(gs.load_page.debug_javascript))
        }
        "load.loadErrorHandling" => Some(format!("{:?}", gs.load_page.load_error_handling)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Small utility functions
// ---------------------------------------------------------------------------

fn bool_str(b: bool) -> String {
    if b { "true" } else { "false" }.to_owned()
}

fn opt_str(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_owned()) }
}

fn parse_page_size(s: &str) -> PageSize {
    match s {
        "A0" => PageSize::A0,
        "A1" => PageSize::A1,
        "A2" => PageSize::A2,
        "A3" => PageSize::A3,
        "A4" => PageSize::A4,
        "A5" => PageSize::A5,
        "A6" => PageSize::A6,
        "Letter" => PageSize::Letter,
        "Legal" => PageSize::Legal,
        "Executive" => PageSize::Executive,
        "Tabloid" => PageSize::Tabloid,
        "Ledger" => PageSize::Ledger,
        _ => PageSize::A4,
    }
}

/// Parse a unit-real string such as `"10mm"`, `"1in"`, `"72pt"`, or a bare
/// number (treated as millimetres).
fn parse_unit_real(s: &str) -> Option<UnitReal> {
    if s.is_empty() {
        return None;
    }
    let (num, unit) = if s.ends_with("mm") {
        (&s[..s.len() - 2], Unit::Millimeter)
    } else if s.ends_with("cm") {
        (&s[..s.len() - 2], Unit::Centimeter)
    } else if s.ends_with("in") {
        (&s[..s.len() - 2], Unit::Inch)
    } else if s.ends_with("pt") {
        (&s[..s.len() - 2], Unit::Point)
    } else if s.ends_with("px") {
        (&s[..s.len() - 2], Unit::Pixel)
    } else {
        (s, Unit::Millimeter)
    };
    num.trim().parse::<f64>().ok().map(|value| UnitReal { value, unit })
}

fn unit_real_to_str(ur: &UnitReal) -> String {
    let suffix = match ur.unit {
        Unit::Millimeter => "mm",
        Unit::Centimeter => "cm",
        Unit::Inch => "in",
        Unit::Point => "pt",
        Unit::Pica => "pc",
        Unit::Pixel => "px",
    };
    format!("{}{}", ur.value, suffix)
}

fn parse_load_error_handling(s: &str) -> LoadErrorHandling {
    match s {
        "skip" | "Skip" => LoadErrorHandling::Skip,
        "ignore" | "Ignore" => LoadErrorHandling::Ignore,
        _ => LoadErrorHandling::Abort,
    }
}

// ---------------------------------------------------------------------------
// PDF C API
// ---------------------------------------------------------------------------

/// Initialise the wkhtmltopdf library.
///
/// `use_graphics` is ignored in this implementation (no Qt dependency) but is
/// accepted for API compatibility.  Returns 1 on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_init(_use_graphics: c_int) -> c_int {
    1
}

/// Deinitialise the wkhtmltopdf library.  Returns 1 on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_deinit() -> c_int {
    1
}

/// Returns 0 because this implementation does not require extended Qt support.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_extended_qt() -> c_int {
    0
}

/// Returns a pointer to the NUL-terminated version string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_version() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

/// Allocate a new `wkhtmltopdf_global_settings` struct with default values.
///
/// The caller must eventually pass the pointer to either
/// [`wkhtmltopdf_create_converter`] (which takes ownership) or
/// [`wkhtmltopdf_destroy_global_settings`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_create_global_settings(
) -> *mut wkhtmltopdf_global_settings {
    Box::into_raw(Box::new(wkhtmltopdf_global_settings {
        inner: PdfGlobal::default(),
    }))
}

/// Free a `wkhtmltopdf_global_settings` that was **not** transferred to a
/// converter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_destroy_global_settings(
    settings: *mut wkhtmltopdf_global_settings,
) {
    if !settings.is_null() {
        drop(Box::from_raw(settings));
    }
}

/// Allocate a new `wkhtmltopdf_object_settings` struct with default values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_create_object_settings(
) -> *mut wkhtmltopdf_object_settings {
    Box::into_raw(Box::new(wkhtmltopdf_object_settings {
        inner: PdfObject::default(),
    }))
}

/// Free a `wkhtmltopdf_object_settings` that was **not** added to a converter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_destroy_object_settings(
    settings: *mut wkhtmltopdf_object_settings,
) {
    if !settings.is_null() {
        drop(Box::from_raw(settings));
    }
}

/// Set a global PDF setting by name.
///
/// Returns 1 if the setting was recognised and applied, 0 otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_global_setting(
    settings: *mut wkhtmltopdf_global_settings,
    name: *const c_char,
    value: *const c_char,
) -> c_int {
    if settings.is_null() {
        return 0;
    }
    let name = cstr_to_str(name);
    let value = cstr_to_str(value);
    if pdf_global_set(&mut (*settings).inner, name, value) {
        1
    } else {
        0
    }
}

/// Get a global PDF setting by name, writing the NUL-terminated value into
/// `value` (a buffer of `vs` bytes).
///
/// Returns 1 if the setting was found and the value fit in the buffer, 0
/// otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_get_global_setting(
    settings: *mut wkhtmltopdf_global_settings,
    name: *const c_char,
    value: *mut c_char,
    vs: c_int,
) -> c_int {
    if settings.is_null() {
        return 0;
    }
    let name = cstr_to_str(name);
    match pdf_global_get(&(*settings).inner, name) {
        Some(v) => write_setting_str(&v, value, vs),
        None => 0,
    }
}

/// Set a per-object PDF setting by name.
///
/// Returns 1 if the setting was recognised and applied, 0 otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_object_setting(
    settings: *mut wkhtmltopdf_object_settings,
    name: *const c_char,
    value: *const c_char,
) -> c_int {
    if settings.is_null() {
        return 0;
    }
    let name = cstr_to_str(name);
    let value = cstr_to_str(value);
    if pdf_object_set(&mut (*settings).inner, name, value) {
        1
    } else {
        0
    }
}

/// Get a per-object PDF setting by name.
///
/// Returns 1 if the setting was found and the value fit in the buffer, 0
/// otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_get_object_setting(
    settings: *mut wkhtmltopdf_object_settings,
    name: *const c_char,
    value: *mut c_char,
    vs: c_int,
) -> c_int {
    if settings.is_null() {
        return 0;
    }
    let name = cstr_to_str(name);
    match pdf_object_get(&(*settings).inner, name) {
        Some(v) => write_setting_str(&v, value, vs),
        None => 0,
    }
}

/// Create a PDF converter that takes ownership of `settings`.
///
/// The global settings pointer must not be used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_create_converter(
    settings: *mut wkhtmltopdf_global_settings,
) -> *mut wkhtmltopdf_converter {
    if settings.is_null() {
        return ptr::null_mut();
    }
    // Take ownership of the global settings.
    let gs_box = Box::from_raw(settings);
    let phase_descriptions = vec![
        CString::new("Loading pages").unwrap_or_default(),
        CString::new("Counting pages").unwrap_or_default(),
        CString::new("Resolving links").unwrap_or_default(),
        CString::new("Loading headers and footers").unwrap_or_default(),
        CString::new("Printing pages").unwrap_or_default(),
        CString::new("Done").unwrap_or_default(),
    ];
    Box::into_raw(Box::new(wkhtmltopdf_converter {
        global: gs_box.inner,
        objects: Vec::new(),
        output: None,
        current_phase: 0,
        http_error_code: 0,
        progress_str: CString::new("").unwrap_or_default(),
        phase_descriptions,
        warning_cb: None,
        error_cb: None,
        phase_changed_cb: None,
        progress_changed_cb: None,
        finished_cb: None,
    }))
}

/// Destroy a PDF converter and free all associated resources.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_destroy_converter(
    converter: *mut wkhtmltopdf_converter,
) {
    if !converter.is_null() {
        drop(Box::from_raw(converter));
    }
}

/// Add an HTML object (page) to the converter.
///
/// `settings` is owned by the converter after this call.
/// `data` may be NULL (the page URL is taken from `settings.page`) or a
/// NUL-terminated string of inline HTML to render.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_add_object(
    converter: *mut wkhtmltopdf_converter,
    settings: *mut wkhtmltopdf_object_settings,
    data: *const c_char,
) {
    if converter.is_null() || settings.is_null() {
        return;
    }
    let obj_box = Box::from_raw(settings);
    let inline_data = if data.is_null() {
        None
    } else {
        let s = cstr_to_str(data);
        if s.is_empty() { None } else { Some(s.to_owned()) }
    };
    (*converter).objects.push((obj_box.inner, inline_data));
}

/// Register a warning-message callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_warning_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).warning_cb = cb;
    }
}

/// Register an error-message callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_error_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).error_cb = cb;
    }
}

/// Register a debug-message callback (alias for the warning callback in this
/// implementation).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_debug_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, *const c_char)>,
) {
    if !converter.is_null() {
        // Route debug messages through the warning callback slot.
        (*converter).warning_cb = cb;
    }
}

/// Register an info-message callback (alias for the warning callback in this
/// implementation).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_info_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).warning_cb = cb;
    }
}

/// Register a phase-changed callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_phase_changed_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter)>,
) {
    if !converter.is_null() {
        (*converter).phase_changed_cb = cb;
    }
}

/// Register a progress-changed callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_progress_changed_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, c_int)>,
) {
    if !converter.is_null() {
        (*converter).progress_changed_cb = cb;
    }
}

/// Register a finished callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_set_finished_callback(
    converter: *mut wkhtmltopdf_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltopdf_converter, c_int)>,
) {
    if !converter.is_null() {
        (*converter).finished_cb = cb;
    }
}

/// Run the PDF conversion.
///
/// Returns 1 on success, 0 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_convert(
    converter: *mut wkhtmltopdf_converter,
) -> c_int {
    if converter.is_null() {
        return 0;
    }
    let conv = &mut *converter;

    // Build the Rust PdfConverter.
    let mut pdf = PdfConverter::new(conv.global.clone());
    for (mut obj, inline) in conv.objects.iter().cloned() {
        // If inline HTML was supplied, override the page URL.
        if let Some(html) = inline {
            obj.page = Some(html);
        }
        pdf.add_object(obj);
    }

    conv.current_phase = 0;
    fire_phase_changed(conv);

    match pdf.convert() {
        Ok(bytes) => {
            conv.current_phase = 5;
            fire_phase_changed(conv);
            conv.progress_str = CString::new("Done").unwrap_or_default();
            conv.output = Some(bytes);
            fire_finished(conv, 1);
            1
        }
        Err(e) => {
            let msg = CString::new(e.to_string()).unwrap_or_default();
            if let Some(cb) = conv.error_cb {
                cb(converter, msg.as_ptr());
            }
            fire_finished(conv, 0);
            0
        }
    }
}

/// Return the current conversion phase (0-based index).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_current_phase(
    converter: *mut wkhtmltopdf_converter,
) -> c_int {
    if converter.is_null() {
        return -1;
    }
    (*converter).current_phase
}

/// Return the total number of conversion phases.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_phase_count(
    converter: *mut wkhtmltopdf_converter,
) -> c_int {
    if converter.is_null() {
        return 0;
    }
    (*converter).phase_descriptions.len() as c_int
}

/// Return a pointer to the NUL-terminated description of `phase`.
///
/// The pointer is valid for the lifetime of the converter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_phase_description(
    converter: *mut wkhtmltopdf_converter,
    phase: c_int,
) -> *const c_char {
    if converter.is_null() || phase < 0 {
        return ptr::null();
    }
    let idx = phase as usize;
    let descs = &(*converter).phase_descriptions;
    if idx >= descs.len() {
        return ptr::null();
    }
    descs[idx].as_ptr()
}

/// Return a pointer to the NUL-terminated progress string.
///
/// The pointer is valid until the next call to [`wkhtmltopdf_convert`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_progress_string(
    converter: *mut wkhtmltopdf_converter,
) -> *const c_char {
    if converter.is_null() {
        return ptr::null();
    }
    (*converter).progress_str.as_ptr()
}

/// Return the HTTP error code from the last conversion (0 = no error).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_http_error_code(
    converter: *mut wkhtmltopdf_converter,
) -> c_int {
    if converter.is_null() {
        return 0;
    }
    (*converter).http_error_code
}

/// Write a pointer to the raw PDF bytes into `*data` and return the byte
/// count.
///
/// Returns 0 if no output is available (conversion has not been run yet or
/// failed).  The pointer written into `*data` is valid for the lifetime of
/// the converter.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltopdf_get_output(
    converter: *mut wkhtmltopdf_converter,
    data: *mut *const c_uchar,
) -> c_long {
    if converter.is_null() || data.is_null() {
        return 0;
    }
    match &(*converter).output {
        Some(bytes) => {
            *data = bytes.as_ptr();
            bytes.len() as c_long
        }
        None => {
            *data = ptr::null();
            0
        }
    }
}

// ---------------------------------------------------------------------------
// PDF internal helpers
// ---------------------------------------------------------------------------

unsafe fn fire_phase_changed(conv: &mut wkhtmltopdf_converter) {
    let raw: *mut wkhtmltopdf_converter = conv;
    if let Some(cb) = conv.phase_changed_cb {
        cb(raw);
    }
}

unsafe fn fire_finished(conv: &mut wkhtmltopdf_converter, val: c_int) {
    let raw: *mut wkhtmltopdf_converter = conv;
    if let Some(cb) = conv.finished_cb {
        cb(raw, val);
    }
}

// ---------------------------------------------------------------------------
// Image C API
// ---------------------------------------------------------------------------

/// Initialise the wkhtmltoimage library.
///
/// Returns 1 on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_init(_use_graphics: c_int) -> c_int {
    1
}

/// Deinitialise the wkhtmltoimage library.  Returns 1 on success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_deinit() -> c_int {
    1
}

/// Returns 0 because this implementation does not require extended Qt support.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_extended_qt() -> c_int {
    0
}

/// Returns a pointer to the NUL-terminated version string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_version() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}

/// Allocate a new `wkhtmltoimage_global_settings` with default values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_create_global_settings(
) -> *mut wkhtmltoimage_global_settings {
    Box::into_raw(Box::new(wkhtmltoimage_global_settings {
        inner: ImageGlobal::default(),
    }))
}

/// Set an image global setting by name.
///
/// Returns 1 if recognised, 0 otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_global_setting(
    settings: *mut wkhtmltoimage_global_settings,
    name: *const c_char,
    value: *const c_char,
) -> c_int {
    if settings.is_null() {
        return 0;
    }
    let name = cstr_to_str(name);
    let value = cstr_to_str(value);
    if image_global_set(&mut (*settings).inner, name, value) {
        1
    } else {
        0
    }
}

/// Get an image global setting by name.
///
/// Returns 1 if the setting was found and the value fit in the buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_get_global_setting(
    settings: *mut wkhtmltoimage_global_settings,
    name: *const c_char,
    value: *mut c_char,
    vs: c_int,
) -> c_int {
    if settings.is_null() {
        return 0;
    }
    let name = cstr_to_str(name);
    match image_global_get(&(*settings).inner, name) {
        Some(v) => write_setting_str(&v, value, vs),
        None => 0,
    }
}

/// Create an image converter.
///
/// Takes ownership of `settings`.  `data` may be NULL (use the page set via
/// the `"in"` setting) or a NUL-terminated inline HTML string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_create_converter(
    settings: *mut wkhtmltoimage_global_settings,
    data: *const c_char,
) -> *mut wkhtmltoimage_converter {
    if settings.is_null() {
        return ptr::null_mut();
    }
    let gs_box = Box::from_raw(settings);
    let inline_data = if data.is_null() {
        None
    } else {
        let s = cstr_to_str(data);
        if s.is_empty() { None } else { Some(s.to_owned()) }
    };
    let phase_descriptions = vec![
        CString::new("Loading page").unwrap_or_default(),
        CString::new("Rendering").unwrap_or_default(),
        CString::new("Done").unwrap_or_default(),
    ];
    Box::into_raw(Box::new(wkhtmltoimage_converter {
        settings: gs_box.inner,
        inline_data,
        output: None,
        current_phase: 0,
        http_error_code: 0,
        progress_str: CString::new("").unwrap_or_default(),
        phase_descriptions,
        warning_cb: None,
        error_cb: None,
        phase_changed_cb: None,
        progress_changed_cb: None,
        finished_cb: None,
    }))
}

/// Destroy an image converter and free all associated resources.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_destroy_converter(
    converter: *mut wkhtmltoimage_converter,
) {
    if !converter.is_null() {
        drop(Box::from_raw(converter));
    }
}

/// Register a warning-message callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_warning_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).warning_cb = cb;
    }
}

/// Register an error-message callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_error_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).error_cb = cb;
    }
}

/// Register a debug-message callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_debug_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).warning_cb = cb;
    }
}

/// Register an info-message callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_info_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, *const c_char)>,
) {
    if !converter.is_null() {
        (*converter).warning_cb = cb;
    }
}

/// Register a phase-changed callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_phase_changed_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter)>,
) {
    if !converter.is_null() {
        (*converter).phase_changed_cb = cb;
    }
}

/// Register a progress-changed callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_progress_changed_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, c_int)>,
) {
    if !converter.is_null() {
        (*converter).progress_changed_cb = cb;
    }
}

/// Register a finished callback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_set_finished_callback(
    converter: *mut wkhtmltoimage_converter,
    cb: Option<unsafe extern "C" fn(*mut wkhtmltoimage_converter, c_int)>,
) {
    if !converter.is_null() {
        (*converter).finished_cb = cb;
    }
}

/// Run the image conversion.
///
/// Returns 1 on success, 0 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_convert(
    converter: *mut wkhtmltoimage_converter,
) -> c_int {
    if converter.is_null() {
        return 0;
    }
    let conv = &mut *converter;

    let mut settings = conv.settings.clone();
    // Inline HTML overrides the page URL.
    if let Some(ref html) = conv.inline_data {
        settings.page = Some(html.clone());
    }

    conv.current_phase = 0;
    fire_image_phase_changed(conv);

    let img_conv = ImageConverter::new(settings);
    match img_conv.convert() {
        Ok(bytes) => {
            conv.current_phase = 2;
            fire_image_phase_changed(conv);
            conv.progress_str = CString::new("Done").unwrap_or_default();
            conv.output = Some(bytes);
            fire_image_finished(conv, 1);
            1
        }
        Err(e) => {
            let msg = CString::new(e.to_string()).unwrap_or_default();
            if let Some(cb) = conv.error_cb {
                cb(converter, msg.as_ptr());
            }
            fire_image_finished(conv, 0);
            0
        }
    }
}

/// Return the current image conversion phase (0-based index).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_current_phase(
    converter: *mut wkhtmltoimage_converter,
) -> c_int {
    if converter.is_null() {
        return -1;
    }
    (*converter).current_phase
}

/// Return the total number of image conversion phases.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_phase_count(
    converter: *mut wkhtmltoimage_converter,
) -> c_int {
    if converter.is_null() {
        return 0;
    }
    (*converter).phase_descriptions.len() as c_int
}

/// Return a pointer to the NUL-terminated description of `phase`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_phase_description(
    converter: *mut wkhtmltoimage_converter,
    phase: c_int,
) -> *const c_char {
    if converter.is_null() || phase < 0 {
        return ptr::null();
    }
    let idx = phase as usize;
    let descs = &(*converter).phase_descriptions;
    if idx >= descs.len() {
        return ptr::null();
    }
    descs[idx].as_ptr()
}

/// Return a pointer to the NUL-terminated progress string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_progress_string(
    converter: *mut wkhtmltoimage_converter,
) -> *const c_char {
    if converter.is_null() {
        return ptr::null();
    }
    (*converter).progress_str.as_ptr()
}

/// Return the HTTP error code from the last image conversion (0 = no error).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_http_error_code(
    converter: *mut wkhtmltoimage_converter,
) -> c_int {
    if converter.is_null() {
        return 0;
    }
    (*converter).http_error_code
}

/// Write a pointer to the raw image bytes into `*data` and return the byte
/// count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wkhtmltoimage_get_output(
    converter: *mut wkhtmltoimage_converter,
    data: *mut *const c_uchar,
) -> c_long {
    if converter.is_null() || data.is_null() {
        return 0;
    }
    match &(*converter).output {
        Some(bytes) => {
            *data = bytes.as_ptr();
            bytes.len() as c_long
        }
        None => {
            *data = ptr::null();
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Image internal helpers
// ---------------------------------------------------------------------------

unsafe fn fire_image_phase_changed(conv: &mut wkhtmltoimage_converter) {
    let raw: *mut wkhtmltoimage_converter = conv;
    if let Some(cb) = conv.phase_changed_cb {
        cb(raw);
    }
}

unsafe fn fire_image_finished(conv: &mut wkhtmltoimage_converter, val: c_int) {
    let raw: *mut wkhtmltoimage_converter = conv;
    if let Some(cb) = conv.finished_cb {
        cb(raw, val);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    unsafe fn cstr(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    // --- lifecycle ---

    #[test]
    fn pdf_init_deinit() {
        unsafe {
            assert_eq!(wkhtmltopdf_init(0), 1);
            assert_eq!(wkhtmltopdf_deinit(), 1);
        }
    }

    #[test]
    fn image_init_deinit() {
        unsafe {
            assert_eq!(wkhtmltoimage_init(0), 1);
            assert_eq!(wkhtmltoimage_deinit(), 1);
        }
    }

    #[test]
    fn pdf_version_not_null() {
        unsafe {
            let v = wkhtmltopdf_version();
            assert!(!v.is_null());
            let s = CStr::from_ptr(v).to_str().unwrap();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn image_version_not_null() {
        unsafe {
            let v = wkhtmltoimage_version();
            assert!(!v.is_null());
        }
    }

    #[test]
    fn pdf_extended_qt_zero() {
        unsafe {
            assert_eq!(wkhtmltopdf_extended_qt(), 0);
        }
    }

    #[test]
    fn image_extended_qt_zero() {
        unsafe {
            assert_eq!(wkhtmltoimage_extended_qt(), 0);
        }
    }

    // --- PDF global settings ---

    #[test]
    fn pdf_global_settings_set_get() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            assert!(!gs.is_null());

            // Set "out"
            let name = cstr("out");
            let val = cstr("/tmp/test.pdf");
            assert_eq!(
                wkhtmltopdf_set_global_setting(gs, name.as_ptr(), val.as_ptr()),
                1
            );

            // Get "out"
            let mut buf = [0i8; 256];
            assert_eq!(
                wkhtmltopdf_get_global_setting(gs, name.as_ptr(), buf.as_mut_ptr(), 256),
                1
            );
            let got = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(got, "/tmp/test.pdf");

            wkhtmltopdf_destroy_global_settings(gs);
        }
    }

    #[test]
    fn pdf_global_unknown_setting_returns_zero() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let n = cstr("nonexistent.setting");
            let v = cstr("value");
            assert_eq!(
                wkhtmltopdf_set_global_setting(gs, n.as_ptr(), v.as_ptr()),
                0
            );
            wkhtmltopdf_destroy_global_settings(gs);
        }
    }

    #[test]
    fn pdf_global_buffer_too_small_returns_zero() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let name = cstr("title");
            let val = cstr("A fairly long document title");
            wkhtmltopdf_set_global_setting(gs, name.as_ptr(), val.as_ptr());

            let mut buf = [0i8; 4]; // too small
            assert_eq!(
                wkhtmltopdf_get_global_setting(gs, name.as_ptr(), buf.as_mut_ptr(), 4),
                0
            );
            wkhtmltopdf_destroy_global_settings(gs);
        }
    }

    // --- PDF object settings ---

    #[test]
    fn pdf_object_settings_set_get_page() {
        unsafe {
            let os = wkhtmltopdf_create_object_settings();
            assert!(!os.is_null());

            let name = cstr("page");
            let val = cstr("https://example.com");
            assert_eq!(
                wkhtmltopdf_set_object_setting(os, name.as_ptr(), val.as_ptr()),
                1
            );

            let mut buf = [0i8; 256];
            assert_eq!(
                wkhtmltopdf_get_object_setting(os, name.as_ptr(), buf.as_mut_ptr(), 256),
                1
            );
            let got = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(got, "https://example.com");

            wkhtmltopdf_destroy_object_settings(os);
        }
    }

    #[test]
    fn pdf_object_settings_header_footer() {
        unsafe {
            let os = wkhtmltopdf_create_object_settings();

            let n = cstr("header.left");
            let v = cstr("[page]");
            assert_eq!(
                wkhtmltopdf_set_object_setting(os, n.as_ptr(), v.as_ptr()),
                1
            );

            let mut buf = [0i8; 64];
            assert_eq!(
                wkhtmltopdf_get_object_setting(os, n.as_ptr(), buf.as_mut_ptr(), 64),
                1
            );
            let got = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(got, "[page]");

            wkhtmltopdf_destroy_object_settings(os);
        }
    }

    // --- PDF converter lifecycle ---

    #[test]
    fn pdf_converter_create_destroy() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let conv = wkhtmltopdf_create_converter(gs);
            assert!(!conv.is_null());

            assert_eq!(wkhtmltopdf_phase_count(conv), 6);
            assert_eq!(wkhtmltopdf_current_phase(conv), 0);
            assert_eq!(wkhtmltopdf_http_error_code(conv), 0);

            let phase_desc = wkhtmltopdf_phase_description(conv, 0);
            assert!(!phase_desc.is_null());
            let s = CStr::from_ptr(phase_desc).to_str().unwrap();
            assert!(!s.is_empty());

            wkhtmltopdf_destroy_converter(conv);
        }
    }

    #[test]
    fn pdf_get_output_before_convert_returns_zero() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let conv = wkhtmltopdf_create_converter(gs);
            let mut ptr: *const c_uchar = ptr::null();
            let len = wkhtmltopdf_get_output(conv, &mut ptr);
            assert_eq!(len, 0);
            assert!(ptr.is_null());
            wkhtmltopdf_destroy_converter(conv);
        }
    }

    #[test]
    fn pdf_convert_no_objects_returns_failure() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let conv = wkhtmltopdf_create_converter(gs);
            // No objects added – the underlying PdfConverter should return an error.
            let result = wkhtmltopdf_convert(conv);
            assert_eq!(result, 0);
            wkhtmltopdf_destroy_converter(conv);
        }
    }

    #[test]
    fn pdf_null_converter_safe() {
        unsafe {
            assert_eq!(wkhtmltopdf_convert(ptr::null_mut()), 0);
            assert_eq!(wkhtmltopdf_current_phase(ptr::null_mut()), -1);
            assert_eq!(wkhtmltopdf_phase_count(ptr::null_mut()), 0);
            assert!(wkhtmltopdf_phase_description(ptr::null_mut(), 0).is_null());
            assert!(wkhtmltopdf_progress_string(ptr::null_mut()).is_null());
            assert_eq!(wkhtmltopdf_http_error_code(ptr::null_mut()), 0);
            let mut p: *const c_uchar = ptr::null();
            assert_eq!(wkhtmltopdf_get_output(ptr::null_mut(), &mut p), 0);
        }
    }

    // --- PDF callbacks ---

    #[test]
    fn pdf_finished_callback_is_called() {
        use std::sync::atomic::{AtomicI32, Ordering};

        static RESULT: AtomicI32 = AtomicI32::new(-1);

        unsafe extern "C" fn finished_cb(
            _conv: *mut wkhtmltopdf_converter,
            val: c_int,
        ) {
            RESULT.store(val, Ordering::SeqCst);
        }

        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let conv = wkhtmltopdf_create_converter(gs);
            wkhtmltopdf_set_finished_callback(conv, Some(finished_cb));
            wkhtmltopdf_convert(conv); // will fail (no objects)
            assert_eq!(RESULT.load(Ordering::SeqCst), 0);
            wkhtmltopdf_destroy_converter(conv);
        }
    }

    // --- Image global settings ---

    #[test]
    fn image_global_settings_set_get() {
        unsafe {
            let gs = wkhtmltoimage_create_global_settings();
            assert!(!gs.is_null());

            let n = cstr("fmt");
            let v = cstr("png");
            assert_eq!(
                wkhtmltoimage_set_global_setting(gs, n.as_ptr(), v.as_ptr()),
                1
            );

            let mut buf = [0i8; 64];
            assert_eq!(
                wkhtmltoimage_get_global_setting(gs, n.as_ptr(), buf.as_mut_ptr(), 64),
                1
            );
            let got = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(got, "png");

            // settings ptr is still valid – no converter took ownership yet
            // (we never called wkhtmltoimage_create_converter)
            // Free manually.
            drop(Box::from_raw(gs));
        }
    }

    #[test]
    fn image_global_unknown_setting_returns_zero() {
        unsafe {
            let gs = wkhtmltoimage_create_global_settings();
            let n = cstr("nonexistent");
            let v = cstr("x");
            assert_eq!(
                wkhtmltoimage_set_global_setting(gs, n.as_ptr(), v.as_ptr()),
                0
            );
            drop(Box::from_raw(gs));
        }
    }

    // --- Image converter lifecycle ---

    #[test]
    fn image_converter_create_destroy() {
        unsafe {
            let gs = wkhtmltoimage_create_global_settings();
            let conv = wkhtmltoimage_create_converter(gs, ptr::null());
            assert!(!conv.is_null());

            assert_eq!(wkhtmltoimage_phase_count(conv), 3);
            assert_eq!(wkhtmltoimage_current_phase(conv), 0);
            assert_eq!(wkhtmltoimage_http_error_code(conv), 0);

            wkhtmltoimage_destroy_converter(conv);
        }
    }

    #[test]
    fn image_get_output_before_convert_returns_zero() {
        unsafe {
            let gs = wkhtmltoimage_create_global_settings();
            let conv = wkhtmltoimage_create_converter(gs, ptr::null());
            let mut ptr: *const c_uchar = std::ptr::null();
            let len = wkhtmltoimage_get_output(conv, &mut ptr);
            assert_eq!(len, 0);
            assert!(ptr.is_null());
            wkhtmltoimage_destroy_converter(conv);
        }
    }

    #[test]
    fn image_convert_no_page_returns_failure() {
        unsafe {
            let gs = wkhtmltoimage_create_global_settings();
            let conv = wkhtmltoimage_create_converter(gs, ptr::null());
            let result = wkhtmltoimage_convert(conv);
            assert_eq!(result, 0);
            wkhtmltoimage_destroy_converter(conv);
        }
    }

    #[test]
    fn image_null_converter_safe() {
        unsafe {
            assert_eq!(wkhtmltoimage_convert(ptr::null_mut()), 0);
            assert_eq!(wkhtmltoimage_current_phase(ptr::null_mut()), -1);
            assert_eq!(wkhtmltoimage_phase_count(ptr::null_mut()), 0);
            assert!(wkhtmltoimage_phase_description(ptr::null_mut(), 0).is_null());
            assert!(wkhtmltoimage_progress_string(ptr::null_mut()).is_null());
            assert_eq!(wkhtmltoimage_http_error_code(ptr::null_mut()), 0);
            let mut p: *const c_uchar = std::ptr::null();
            assert_eq!(wkhtmltoimage_get_output(ptr::null_mut(), &mut p), 0);
        }
    }

    // --- Setting helpers ---

    #[test]
    fn parse_unit_real_mm() {
        let ur = parse_unit_real("10mm").unwrap();
        assert_eq!(ur.value, 10.0);
        assert!(matches!(ur.unit, Unit::Millimeter));
    }

    #[test]
    fn parse_unit_real_in() {
        let ur = parse_unit_real("1.5in").unwrap();
        assert!((ur.value - 1.5).abs() < 1e-9);
        assert!(matches!(ur.unit, Unit::Inch));
    }

    #[test]
    fn parse_unit_real_bare_number() {
        let ur = parse_unit_real("25").unwrap();
        assert_eq!(ur.value, 25.0);
        assert!(matches!(ur.unit, Unit::Millimeter));
    }

    #[test]
    fn parse_unit_real_empty_returns_none() {
        assert!(parse_unit_real("").is_none());
    }

    #[test]
    fn unit_real_roundtrip() {
        let ur = UnitReal {
            value: 12.5,
            unit: Unit::Point,
        };
        let s = unit_real_to_str(&ur);
        assert_eq!(s, "12.5pt");
        let back = parse_unit_real(&s).unwrap();
        assert!((back.value - 12.5).abs() < 1e-9);
        assert!(matches!(back.unit, Unit::Point));
    }

    #[test]
    fn pdf_margin_settings_roundtrip() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let n = cstr("margin.top");
            let v = cstr("20mm");
            wkhtmltopdf_set_global_setting(gs, n.as_ptr(), v.as_ptr());

            let mut buf = [0i8; 64];
            wkhtmltopdf_get_global_setting(gs, n.as_ptr(), buf.as_mut_ptr(), 64);
            let got = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
            assert_eq!(got, "20mm");

            wkhtmltopdf_destroy_global_settings(gs);
        }
    }

    #[test]
    fn pdf_phase_description_out_of_range_returns_null() {
        unsafe {
            let gs = wkhtmltopdf_create_global_settings();
            let conv = wkhtmltopdf_create_converter(gs);
            assert!(wkhtmltopdf_phase_description(conv, 99).is_null());
            wkhtmltopdf_destroy_converter(conv);
        }
    }
}
