//! Configuration and settings structs for wkhtmltopdf.
//!
//! This crate mirrors the settings subsystem from the C++ codebase
//! (`src/lib/*settings.*`). Settings are plain structs that are populated
//! by the CLI parser or the public API and then passed into the converters.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------

/// Logging verbosity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LogLevel {
    #[default]
    Warn,
    Info,
    Error,
    None,
}

/// Page orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

/// Output color mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorMode {
    #[default]
    Color,
    Grayscale,
}

/// Printer resolution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PrintResolution {
    #[default]
    ScreenResolution,
    HighResolution,
}

/// Standard paper/page sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PageSize {
    A0,
    A1,
    A2,
    A3,
    #[default]
    A4,
    A5,
    A6,
    A7,
    A8,
    A9,
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    B9,
    B10,
    Letter,
    Legal,
    Executive,
    Tabloid,
    Ledger,
    Custom,
}

/// Unit for length measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Unit {
    #[default]
    Millimeter,
    Centimeter,
    Inch,
    Point,
    Pica,
    Pixel,
}

/// How to handle page load errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoadErrorHandling {
    #[default]
    Abort,
    Skip,
    Ignore,
}

/// Type of network proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProxyType {
    #[default]
    None,
    Socks5,
    Http,
}

// ---------------------------------------------------------------------------
// Shared helper types
// ---------------------------------------------------------------------------

/// A length value with an associated unit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnitReal {
    pub value: f64,
    pub unit: Unit,
}

// ---------------------------------------------------------------------------
// Proxy / network structs
// ---------------------------------------------------------------------------

/// HTTP/SOCKS proxy configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Proxy {
    pub proxy_type: ProxyType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// A single HTTP POST field (name/value pair or file upload).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostItem {
    pub name: String,
    pub value: String,
    /// When `true` the value is a file path whose contents are posted.
    pub file: bool,
}

// ---------------------------------------------------------------------------
// Web / browser settings
// ---------------------------------------------------------------------------

/// Browser rendering settings shared across page objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web {
    /// Render page background images.
    pub background: bool,
    /// Load images referenced by the page.
    pub load_images: bool,
    /// Enable JavaScript execution.
    pub enable_javascript: bool,
    /// Enable intelligent shrinking of wide content.
    pub enable_intelligent_shrinking: bool,
    /// Minimum font size in points.
    pub minimum_font_size: Option<i32>,
    /// Default text encoding used when the page does not declare one.
    pub default_encoding: Option<String>,
    /// URL or path to a user-supplied CSS stylesheet.
    pub user_style_sheet: Option<String>,
    /// Allow browser plugins.
    pub enable_plugins: bool,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            background: true,
            load_images: true,
            enable_javascript: true,
            enable_intelligent_shrinking: true,
            minimum_font_size: None,
            default_encoding: None,
            user_style_sheet: None,
            enable_plugins: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Load settings
// ---------------------------------------------------------------------------

/// Global network/load settings shared by PDF and image converters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadGlobal {
    /// Path to a cookie jar file.
    pub cookie_jar: Option<String>,
}

/// Per-page network/load settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadPage {
    /// Username for HTTP basic authentication.
    pub username: Option<String>,
    /// Password for HTTP basic authentication.
    pub password: Option<String>,
    /// Path to the SSL client certificate private key (OpenSSL PEM format).
    pub client_ssl_key_path: Option<String>,
    /// Password for the SSL client certificate private key.
    pub client_ssl_key_password: Option<String>,
    /// Path to the SSL client certificate public key (OpenSSL PEM format).
    pub client_ssl_crt_path: Option<String>,
    /// Additional milliseconds to wait after the page has loaded.
    pub js_delay: u32,
    /// Wait until `window.status` equals this string before rendering.
    pub window_status: Option<String>,
    /// Zoom factor applied to the page.
    pub zoom: f64,
    /// Custom HTTP headers added to every request.
    pub custom_headers: Vec<(String, String)>,
    /// Repeat custom headers for every resource request (not just the main page).
    pub repeat_custom_headers: bool,
    /// Cookies to include with requests.
    pub cookies: Vec<(String, String)>,
    /// HTTP POST data items.
    pub post: Vec<PostItem>,
    /// Block access to local files from the page.
    pub block_local_file_access: bool,
    /// Paths that are allowed even when `block_local_file_access` is `true`.
    pub allowed: Vec<String>,
    /// Abort JavaScript that runs too long.
    pub stop_slow_scripts: bool,
    /// Print JavaScript debug messages to the console.
    pub debug_javascript: bool,
    /// How to handle page load errors.
    pub load_error_handling: LoadErrorHandling,
    /// How to handle media (image/script/stylesheet) load errors.
    pub media_load_error_handling: LoadErrorHandling,
    /// Proxy configuration.
    pub proxy: Proxy,
    /// Additional JavaScript to execute after the page has loaded.
    pub run_script: Vec<String>,
    /// Hosts for which the configured proxy is bypassed.
    pub bypass_proxy_for_hosts: Vec<String>,
    /// Resolve host names through the proxy.
    pub proxy_hostname_lookup: bool,
    /// Use the `print` CSS media type instead of `screen`.
    pub print_media_type: bool,
    /// Local directory to use as a browser disk cache.
    pub cache_dir: Option<String>,
}

impl Default for LoadPage {
    fn default() -> Self {
        Self {
            username: None,
            password: None,
            client_ssl_key_path: None,
            client_ssl_key_password: None,
            client_ssl_crt_path: None,
            js_delay: 200,
            window_status: None,
            zoom: 1.0,
            custom_headers: Vec::new(),
            repeat_custom_headers: false,
            cookies: Vec::new(),
            post: Vec::new(),
            block_local_file_access: false,
            allowed: Vec::new(),
            stop_slow_scripts: true,
            debug_javascript: false,
            load_error_handling: LoadErrorHandling::Abort,
            media_load_error_handling: LoadErrorHandling::Ignore,
            proxy: Proxy::default(),
            run_script: Vec::new(),
            bypass_proxy_for_hosts: Vec::new(),
            proxy_hostname_lookup: false,
            print_media_type: false,
            cache_dir: None,
        }
    }
}

// ---------------------------------------------------------------------------
// PDF-specific settings
// ---------------------------------------------------------------------------

/// Page margin values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: UnitReal,
    pub right: UnitReal,
    pub bottom: UnitReal,
    pub left: UnitReal,
}

/// Page size settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Size {
    /// Named paper size.
    pub page_size: PageSize,
    /// Explicit page height (overrides `page_size` when set).
    pub height: Option<UnitReal>,
    /// Explicit page width (overrides `page_size` when set).
    pub width: Option<UnitReal>,
}

/// Table-of-contents generation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContent {
    /// Print dots between the entry name and the page number.
    pub use_dotted_lines: bool,
    /// Title text for the TOC section.
    pub caption_text: String,
    /// Add links from TOC entries to the corresponding section headings.
    pub forward_links: bool,
    /// Add links from section headings back to the TOC.
    pub back_links: bool,
    /// Indentation applied per TOC nesting level.
    pub indentation: String,
    /// Font scale factor applied per TOC nesting level.
    pub font_scale: f32,
}

impl Default for TableOfContent {
    fn default() -> Self {
        Self {
            use_dotted_lines: true,
            caption_text: "Table of Contents".into(),
            forward_links: true,
            back_links: true,
            indentation: "1em".into(),
            font_scale: 0.8,
        }
    }
}

/// Header or footer band settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooter {
    /// Font size in points.
    pub font_size: i32,
    /// Font family name.
    pub font_name: String,
    /// Text rendered on the left side of the band.
    pub left: Option<String>,
    /// Text rendered on the right side of the band.
    pub right: Option<String>,
    /// Text rendered in the centre of the band.
    pub center: Option<String>,
    /// Draw a separator line between the band and the page body.
    pub line: bool,
    /// URL of an HTML document to use as the header/footer.
    pub html_url: Option<String>,
    /// Spacing in millimetres between the band and the page body.
    pub spacing: f32,
}

impl Default for HeaderFooter {
    fn default() -> Self {
        Self {
            font_size: 12,
            font_name: "Arial".into(),
            left: None,
            right: None,
            center: None,
            line: false,
            html_url: None,
            spacing: 0.0,
        }
    }
}

/// Global PDF conversion settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfGlobal {
    /// Page-size settings.
    pub size: Size,
    /// Logging verbosity.
    pub log_level: LogLevel,
    /// Render using the graphics subsystem.
    pub use_graphics: bool,
    /// Resolve relative links before embedding them.
    pub resolve_relative_links: bool,
    /// Page orientation.
    pub orientation: Orientation,
    /// Color or grayscale output.
    pub color_mode: ColorMode,
    /// Printer resolution mode.
    pub resolution: PrintResolution,
    /// Rendering DPI (`None` uses the default).
    pub dpi: Option<i32>,
    /// Offset added to every printed page number.
    pub page_offset: i32,
    /// Number of copies to produce.
    pub copies: u32,
    /// Print all pages of one copy before starting the next.
    pub collate: bool,
    /// Generate a PDF outline (bookmarks panel).
    pub outline: bool,
    /// Maximum nesting depth of the generated outline.
    pub outline_depth: u32,
    /// Write the outline to this file path in XML format.
    pub dump_outline: Option<String>,
    /// Path to the output PDF file.
    pub output: Option<String>,
    /// PDF document title metadata.
    pub document_title: Option<String>,
    /// Apply zlib compression to the PDF output stream.
    pub use_compression: bool,
    /// Page margin settings.
    pub margin: Margin,
    /// Viewport size string (e.g. `"1280x1024"`).
    pub viewport_size: Option<String>,
    /// DPI used when embedding raster images.
    pub image_dpi: i32,
    /// Quality used when embedding JPEG images (1–100).
    pub image_quality: i32,
    /// Global network/load settings.
    pub load: LoadGlobal,
}

impl Default for PdfGlobal {
    fn default() -> Self {
        Self {
            size: Size::default(),
            log_level: LogLevel::default(),
            use_graphics: false,
            resolve_relative_links: true,
            orientation: Orientation::default(),
            color_mode: ColorMode::default(),
            resolution: PrintResolution::default(),
            dpi: None,
            page_offset: 0,
            copies: 1,
            collate: true,
            outline: true,
            outline_depth: 4,
            dump_outline: None,
            output: None,
            document_title: None,
            use_compression: true,
            margin: Margin::default(),
            viewport_size: None,
            image_dpi: 600,
            image_quality: 94,
            load: LoadGlobal::default(),
        }
    }
}

/// Per-page PDF settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfObject {
    /// Table-of-contents settings for this object.
    pub toc: TableOfContent,
    /// URL or path of the HTML page to render.
    pub page: Option<String>,
    /// Header band settings.
    pub header: HeaderFooter,
    /// Footer band settings.
    pub footer: HeaderFooter,
    /// Render external hyperlinks as active PDF links.
    pub use_external_links: bool,
    /// Render internal (anchor) links as active PDF links.
    pub use_local_links: bool,
    /// Text replacement pairs applied before rendering.
    pub replacements: Vec<(String, String)>,
    /// Convert HTML form elements into PDF form fields.
    pub produce_forms: bool,
    /// Per-page load settings.
    pub load: LoadPage,
    /// Browser rendering settings.
    pub web: Web,
    /// Include this object's headings in the PDF outline.
    pub include_in_outline: bool,
    /// Include this object's page count in the document total.
    pub pages_count: bool,
    /// Treat this object as a table of contents.
    pub is_table_of_content: bool,
    /// Path to an XSL stylesheet used to render the TOC.
    pub toc_xsl: Option<String>,
}

impl Default for PdfObject {
    fn default() -> Self {
        Self {
            toc: TableOfContent::default(),
            page: None,
            header: HeaderFooter::default(),
            footer: HeaderFooter::default(),
            use_external_links: true,
            use_local_links: true,
            replacements: Vec::new(),
            produce_forms: false,
            load: LoadPage::default(),
            web: Web::default(),
            include_in_outline: true,
            pages_count: true,
            is_table_of_content: false,
            toc_xsl: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Image-specific settings
// ---------------------------------------------------------------------------

/// Cropping rectangle for image output (all values in pixels).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CropSettings {
    /// Left edge of the crop rectangle.
    pub left: i32,
    /// Top edge of the crop rectangle.
    pub top: i32,
    /// Width of the crop rectangle (0 = full width).
    pub width: i32,
    /// Height of the crop rectangle (0 = full height).
    pub height: i32,
}

/// Global image conversion settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGlobal {
    /// Crop settings.
    pub crop: CropSettings,
    /// Global network/load settings.
    pub load_global: LoadGlobal,
    /// Per-page load settings.
    pub load_page: LoadPage,
    /// Browser rendering settings.
    pub web: Web,
    /// Logging verbosity.
    pub log_level: LogLevel,
    /// Render with a transparent background.
    pub transparent: bool,
    /// Render using the graphics subsystem.
    pub use_graphics: bool,
    /// URL or path of the HTML page to render.
    pub page: Option<String>,
    /// Path to the output image file.
    pub output: Option<String>,
    /// Output image format (e.g. `"png"`, `"jpg"`).
    pub format: Option<String>,
    /// Viewport width in pixels.
    pub screen_width: Option<i32>,
    /// Viewport height in pixels.
    pub screen_height: Option<i32>,
    /// JPEG image quality (1–100).
    pub quality: i32,
    /// Automatically adjust the viewport width to fit page content.
    pub smart_width: bool,
}

impl Default for ImageGlobal {
    fn default() -> Self {
        Self {
            crop: CropSettings::default(),
            load_global: LoadGlobal::default(),
            load_page: LoadPage::default(),
            web: Web::default(),
            log_level: LogLevel::default(),
            transparent: false,
            use_graphics: false,
            page: None,
            output: None,
            format: None,
            screen_width: None,
            screen_height: None,
            quality: 94,
            smart_width: true,
        }
    }
}
