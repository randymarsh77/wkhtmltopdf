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

/// PDF/A archiving conformance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PdfAConformance {
    /// No PDF/A conformance (plain PDF).
    #[default]
    None,
    /// PDF/A-1b – ISO 19005-1:2005, basic level based on PDF 1.4.
    A1b,
    /// PDF/A-2b – ISO 19005-2:2011, based on PDF 1.7.
    A2b,
    /// PDF/A-3b – ISO 19005-3:2012, allows embedded files.
    A3b,
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
    /// Verify the server's SSL certificate chain.
    pub ssl_verify_peer: bool,
    /// Verify that the server's SSL certificate matches its hostname.
    pub ssl_verify_host: bool,
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
            ssl_verify_peer: true,
            ssl_verify_host: true,
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
    /// Maximum heading depth to include in the TOC (1–6).
    pub depth: u32,
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
            depth: 3,
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
    /// PDF document author metadata.
    pub author: Option<String>,
    /// PDF document subject metadata.
    pub subject: Option<String>,
    /// PDF/A archiving conformance level.
    pub pdf_a_conformance: PdfAConformance,
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
            author: None,
            subject: None,
            pdf_a_conformance: PdfAConformance::default(),
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
    /// Output image format (e.g. `"png"`, `"jpg"`, `"bmp"`, `"svg"`).
    pub format: Option<String>,
    /// Viewport width in pixels.
    pub screen_width: Option<i32>,
    /// Viewport height in pixels.
    pub screen_height: Option<i32>,
    /// JPEG image quality (1–100).
    pub quality: i32,
    /// Automatically adjust the viewport width to fit page content.
    pub smart_width: bool,
    /// Output image DPI (`None` uses the default).
    pub dpi: Option<i32>,
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
            dpi: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // -----------------------------------------------------------------------
    // Enum default values
    // -----------------------------------------------------------------------

    #[test]
    fn log_level_default_is_warn() {
        assert_eq!(LogLevel::default(), LogLevel::Warn);
    }

    #[test]
    fn orientation_default_is_portrait() {
        assert_eq!(Orientation::default(), Orientation::Portrait);
    }

    #[test]
    fn color_mode_default_is_color() {
        assert_eq!(ColorMode::default(), ColorMode::Color);
    }

    #[test]
    fn print_resolution_default_is_screen() {
        assert_eq!(
            PrintResolution::default(),
            PrintResolution::ScreenResolution
        );
    }

    #[test]
    fn pdf_a_conformance_default_is_none() {
        assert_eq!(PdfAConformance::default(), PdfAConformance::None);
    }

    #[test]
    fn page_size_default_is_a4() {
        assert_eq!(PageSize::default(), PageSize::A4);
    }

    #[test]
    fn unit_default_is_millimeter() {
        assert_eq!(Unit::default(), Unit::Millimeter);
    }

    #[test]
    fn load_error_handling_default_is_abort() {
        assert_eq!(LoadErrorHandling::default(), LoadErrorHandling::Abort);
    }

    #[test]
    fn proxy_type_default_is_none() {
        assert_eq!(ProxyType::default(), ProxyType::None);
    }

    // -----------------------------------------------------------------------
    // Serde round-trips for all enum variants
    // -----------------------------------------------------------------------

    #[test]
    fn log_level_all_variants_roundtrip() {
        for variant in [
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Error,
            LogLevel::None,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: LogLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    #[test]
    fn orientation_all_variants_roundtrip() {
        for variant in [Orientation::Portrait, Orientation::Landscape] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: Orientation = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    #[test]
    fn color_mode_all_variants_roundtrip() {
        for variant in [ColorMode::Color, ColorMode::Grayscale] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: ColorMode = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    #[test]
    fn pdf_a_conformance_all_variants_roundtrip() {
        for variant in [
            PdfAConformance::None,
            PdfAConformance::A1b,
            PdfAConformance::A2b,
            PdfAConformance::A3b,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: PdfAConformance = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    #[test]
    fn page_size_selected_variants_roundtrip() {
        for variant in [
            PageSize::A4,
            PageSize::A3,
            PageSize::Letter,
            PageSize::Legal,
            PageSize::Tabloid,
            PageSize::Custom,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: PageSize = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    #[test]
    fn load_error_handling_all_variants_roundtrip() {
        for variant in [
            LoadErrorHandling::Abort,
            LoadErrorHandling::Skip,
            LoadErrorHandling::Ignore,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: LoadErrorHandling = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    #[test]
    fn proxy_type_all_variants_roundtrip() {
        for variant in [ProxyType::None, ProxyType::Http, ProxyType::Socks5] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let restored: ProxyType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, restored);
        }
    }

    // -----------------------------------------------------------------------
    // UnitReal construction and default
    // -----------------------------------------------------------------------

    #[test]
    fn unit_real_default_is_zero_millimeters() {
        let ur = UnitReal::default();
        assert!((ur.value - 0.0).abs() < f64::EPSILON);
        assert_eq!(ur.unit, Unit::Millimeter);
    }

    #[test]
    fn unit_real_can_be_constructed_with_any_unit() {
        let ur = UnitReal {
            value: 2.54,
            unit: Unit::Centimeter,
        };
        assert!((ur.value - 2.54).abs() < f64::EPSILON);
        assert_eq!(ur.unit, Unit::Centimeter);

        let ur2 = UnitReal {
            value: 72.0,
            unit: Unit::Point,
        };
        assert!((ur2.value - 72.0).abs() < f64::EPSILON);
        assert_eq!(ur2.unit, Unit::Point);
    }

    #[test]
    fn unit_real_roundtrip() {
        let ur = UnitReal {
            value: 1.5,
            unit: Unit::Inch,
        };
        let json = serde_json::to_string(&ur).expect("serialize");
        let restored: UnitReal = serde_json::from_str(&json).expect("deserialize");
        assert!((restored.value - 1.5).abs() < f64::EPSILON);
        assert_eq!(restored.unit, Unit::Inch);
    }

    // -----------------------------------------------------------------------
    // Proxy struct
    // -----------------------------------------------------------------------

    #[test]
    fn proxy_default_has_no_host_or_port() {
        let proxy = Proxy::default();
        assert_eq!(proxy.proxy_type, ProxyType::None);
        assert!(proxy.host.is_none());
        assert!(proxy.port.is_none());
        assert!(proxy.username.is_none());
        assert!(proxy.password.is_none());
    }

    #[test]
    fn proxy_can_be_configured() {
        let proxy = Proxy {
            proxy_type: ProxyType::Http,
            host: Some("proxy.example.com".into()),
            port: Some(8080),
            username: Some("user".into()),
            password: Some("pass".into()),
        };
        assert_eq!(proxy.proxy_type, ProxyType::Http);
        assert_eq!(proxy.host.as_deref(), Some("proxy.example.com"));
        assert_eq!(proxy.port, Some(8080));
        assert_eq!(proxy.username.as_deref(), Some("user"));
        assert_eq!(proxy.password.as_deref(), Some("pass"));
    }

    #[test]
    fn proxy_socks5_roundtrip() {
        let proxy = Proxy {
            proxy_type: ProxyType::Socks5,
            host: Some("socks.example.com".into()),
            port: Some(1080),
            username: None,
            password: None,
        };
        let json = serde_json::to_string(&proxy).expect("serialize");
        let restored: Proxy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.proxy_type, ProxyType::Socks5);
        assert_eq!(restored.host.as_deref(), Some("socks.example.com"));
        assert_eq!(restored.port, Some(1080));
    }

    // -----------------------------------------------------------------------
    // PostItem
    // -----------------------------------------------------------------------

    #[test]
    fn post_item_default_is_empty() {
        let item = PostItem::default();
        assert!(item.name.is_empty());
        assert!(item.value.is_empty());
        assert!(!item.file);
    }

    #[test]
    fn post_item_file_flag() {
        let item = PostItem {
            name: "attachment".into(),
            value: "/path/to/file.pdf".into(),
            file: true,
        };
        assert!(item.file);
        assert_eq!(item.value, "/path/to/file.pdf");
    }

    // -----------------------------------------------------------------------
    // Web defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn web_default_values() {
        let web = Web::default();
        assert!(web.background);
        assert!(web.load_images);
        assert!(web.enable_javascript);
        assert!(web.enable_intelligent_shrinking);
        assert!(!web.enable_plugins);
        assert!(web.minimum_font_size.is_none());
        assert!(web.default_encoding.is_none());
        assert!(web.user_style_sheet.is_none());
    }

    #[test]
    fn web_fields_can_be_mutated() {
        let mut web = Web::default();
        web.background = false;
        web.load_images = false;
        web.enable_javascript = false;
        web.enable_intelligent_shrinking = false;
        web.enable_plugins = true;
        web.minimum_font_size = Some(10);
        web.default_encoding = Some("utf-8".into());
        web.user_style_sheet = Some("style.css".into());

        assert!(!web.background);
        assert!(!web.load_images);
        assert!(!web.enable_javascript);
        assert!(!web.enable_intelligent_shrinking);
        assert!(web.enable_plugins);
        assert_eq!(web.minimum_font_size, Some(10));
        assert_eq!(web.default_encoding.as_deref(), Some("utf-8"));
        assert_eq!(web.user_style_sheet.as_deref(), Some("style.css"));
    }

    // -----------------------------------------------------------------------
    // LoadGlobal
    // -----------------------------------------------------------------------

    #[test]
    fn load_global_default_has_no_cookie_jar() {
        let lg = LoadGlobal::default();
        assert!(lg.cookie_jar.is_none());
    }

    #[test]
    fn load_global_cookie_jar_roundtrip() {
        let mut lg = LoadGlobal::default();
        lg.cookie_jar = Some("/tmp/cookies.jar".into());
        let json = serde_json::to_string(&lg).expect("serialize");
        let restored: LoadGlobal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.cookie_jar.as_deref(), Some("/tmp/cookies.jar"));
    }

    // -----------------------------------------------------------------------
    // LoadPage defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn load_page_default_values() {
        let lp = LoadPage::default();
        assert!(lp.username.is_none());
        assert!(lp.password.is_none());
        assert_eq!(lp.js_delay, 200);
        assert!((lp.zoom - 1.0).abs() < f64::EPSILON);
        assert!(!lp.repeat_custom_headers);
        assert!(!lp.block_local_file_access);
        assert!(lp.stop_slow_scripts);
        assert!(!lp.debug_javascript);
        assert_eq!(lp.load_error_handling, LoadErrorHandling::Abort);
        assert_eq!(lp.media_load_error_handling, LoadErrorHandling::Ignore);
        assert!(!lp.proxy_hostname_lookup);
        assert!(!lp.print_media_type);
        assert!(lp.ssl_verify_peer);
        assert!(lp.ssl_verify_host);
    }

    #[test]
    fn load_page_fields_can_be_mutated() {
        let mut lp = LoadPage::default();
        lp.username = Some("admin".into());
        lp.password = Some("secret".into());
        lp.js_delay = 500;
        lp.zoom = 1.5;
        lp.block_local_file_access = true;
        lp.debug_javascript = true;
        lp.print_media_type = true;
        lp.ssl_verify_peer = false;
        lp.ssl_verify_host = false;
        lp.load_error_handling = LoadErrorHandling::Skip;
        lp.media_load_error_handling = LoadErrorHandling::Abort;
        lp.cookies = vec![("session".into(), "abc".into())];
        lp.custom_headers = vec![("X-Auth".into(), "token".into())];
        lp.run_script = vec!["console.log('hi');".into()];
        lp.allowed = vec!["/tmp/data".into()];
        lp.bypass_proxy_for_hosts = vec!["internal.host".into()];

        assert_eq!(lp.username.as_deref(), Some("admin"));
        assert_eq!(lp.js_delay, 500);
        assert!((lp.zoom - 1.5).abs() < f64::EPSILON);
        assert!(lp.block_local_file_access);
        assert!(!lp.ssl_verify_peer);
        assert_eq!(lp.load_error_handling, LoadErrorHandling::Skip);
        assert_eq!(lp.cookies.len(), 1);
        assert_eq!(lp.cookies[0].0, "session");
    }

    #[test]
    fn load_page_ssl_settings_roundtrip() {
        let mut lp = LoadPage::default();
        lp.ssl_verify_peer = false;
        lp.client_ssl_key_path = Some("/path/to/key.pem".into());
        lp.client_ssl_key_password = Some("keypass".into());
        lp.client_ssl_crt_path = Some("/path/to/cert.pem".into());

        let json = serde_json::to_string(&lp).expect("serialize");
        let restored: LoadPage = serde_json::from_str(&json).expect("deserialize");

        assert!(!restored.ssl_verify_peer);
        assert!(restored.ssl_verify_host);
        assert_eq!(
            restored.client_ssl_key_path.as_deref(),
            Some("/path/to/key.pem")
        );
        assert_eq!(restored.client_ssl_key_password.as_deref(), Some("keypass"));
        assert_eq!(
            restored.client_ssl_crt_path.as_deref(),
            Some("/path/to/cert.pem")
        );
    }

    // -----------------------------------------------------------------------
    // Margin and Size structs
    // -----------------------------------------------------------------------

    #[test]
    fn margin_default_is_zero() {
        let m = Margin::default();
        assert!((m.top.value - 0.0).abs() < f64::EPSILON);
        assert!((m.right.value - 0.0).abs() < f64::EPSILON);
        assert!((m.bottom.value - 0.0).abs() < f64::EPSILON);
        assert!((m.left.value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn margin_can_be_set_with_mixed_units() {
        let m = Margin {
            top: UnitReal {
                value: 20.0,
                unit: Unit::Millimeter,
            },
            bottom: UnitReal {
                value: 20.0,
                unit: Unit::Millimeter,
            },
            left: UnitReal {
                value: 1.0,
                unit: Unit::Inch,
            },
            right: UnitReal {
                value: 72.0,
                unit: Unit::Point,
            },
        };
        assert!((m.top.value - 20.0).abs() < f64::EPSILON);
        assert_eq!(m.left.unit, Unit::Inch);
        assert_eq!(m.right.unit, Unit::Point);
    }

    #[test]
    fn size_default_is_a4_no_explicit_dimensions() {
        let size = Size::default();
        assert_eq!(size.page_size, PageSize::A4);
        assert!(size.height.is_none());
        assert!(size.width.is_none());
    }

    #[test]
    fn size_can_specify_custom_dimensions() {
        let size = Size {
            page_size: PageSize::Custom,
            width: Some(UnitReal {
                value: 210.0,
                unit: Unit::Millimeter,
            }),
            height: Some(UnitReal {
                value: 297.0,
                unit: Unit::Millimeter,
            }),
        };
        assert_eq!(size.page_size, PageSize::Custom);
        assert!(size.width.is_some());
        assert!(size.height.is_some());
        assert!((size.width.as_ref().unwrap().value - 210.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // TableOfContent defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn table_of_content_default_values() {
        let toc = TableOfContent::default();
        assert!(toc.use_dotted_lines);
        assert_eq!(toc.caption_text, "Table of Contents");
        assert!(toc.forward_links);
        assert!(toc.back_links);
        assert_eq!(toc.indentation, "1em");
        assert!((toc.font_scale - 0.8).abs() < 1e-6);
        assert_eq!(toc.depth, 3);
    }

    #[test]
    fn table_of_content_fields_can_be_mutated() {
        let mut toc = TableOfContent::default();
        toc.use_dotted_lines = false;
        toc.caption_text = "Index".into();
        toc.forward_links = false;
        toc.back_links = false;
        toc.depth = 5;
        toc.indentation = "2em".into();
        toc.font_scale = 0.9;

        assert!(!toc.use_dotted_lines);
        assert_eq!(toc.caption_text, "Index");
        assert!(!toc.forward_links);
        assert!(!toc.back_links);
        assert_eq!(toc.depth, 5);
        assert_eq!(toc.indentation, "2em");
        assert!((toc.font_scale - 0.9).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // HeaderFooter defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn header_footer_default_values() {
        let hf = HeaderFooter::default();
        assert_eq!(hf.font_size, 12);
        assert_eq!(hf.font_name, "Arial");
        assert!(hf.left.is_none());
        assert!(hf.right.is_none());
        assert!(hf.center.is_none());
        assert!(!hf.line);
        assert!(hf.html_url.is_none());
        assert!((hf.spacing - 0.0).abs() < 1e-6);
    }

    #[test]
    fn header_footer_fields_can_be_mutated() {
        let mut hf = HeaderFooter::default();
        hf.left = Some("Left".into());
        hf.center = Some("Center".into());
        hf.right = Some("Right".into());
        hf.font_size = 14;
        hf.font_name = "Helvetica".into();
        hf.line = true;
        hf.html_url = Some("header.html".into());
        hf.spacing = 5.0;

        assert_eq!(hf.left.as_deref(), Some("Left"));
        assert_eq!(hf.center.as_deref(), Some("Center"));
        assert_eq!(hf.right.as_deref(), Some("Right"));
        assert_eq!(hf.font_size, 14);
        assert_eq!(hf.font_name, "Helvetica");
        assert!(hf.line);
        assert_eq!(hf.html_url.as_deref(), Some("header.html"));
        assert!((hf.spacing - 5.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // PdfGlobal defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn pdf_global_default_values() {
        let g = PdfGlobal::default();
        assert_eq!(g.log_level, LogLevel::Warn);
        assert!(!g.use_graphics);
        assert!(g.resolve_relative_links);
        assert_eq!(g.orientation, Orientation::Portrait);
        assert_eq!(g.color_mode, ColorMode::Color);
        assert!(g.dpi.is_none());
        assert_eq!(g.page_offset, 0);
        assert_eq!(g.copies, 1);
        assert!(g.collate);
        assert!(g.outline);
        assert_eq!(g.outline_depth, 4);
        assert!(g.dump_outline.is_none());
        assert!(g.output.is_none());
        assert!(g.document_title.is_none());
        assert!(g.author.is_none());
        assert!(g.subject.is_none());
        assert_eq!(g.pdf_a_conformance, PdfAConformance::None);
        assert!(g.use_compression);
        assert!(g.viewport_size.is_none());
        assert_eq!(g.image_dpi, 600);
        assert_eq!(g.image_quality, 94);
    }

    #[test]
    fn pdf_global_metadata_can_be_set() {
        let mut g = PdfGlobal::default();
        g.document_title = Some("My Title".into());
        g.author = Some("Jane Doe".into());
        g.subject = Some("Testing".into());
        g.pdf_a_conformance = PdfAConformance::A2b;
        g.dpi = Some(300);
        g.copies = 2;
        g.collate = false;
        g.outline = false;
        g.use_compression = false;
        g.page_offset = 5;

        assert_eq!(g.document_title.as_deref(), Some("My Title"));
        assert_eq!(g.author.as_deref(), Some("Jane Doe"));
        assert_eq!(g.subject.as_deref(), Some("Testing"));
        assert_eq!(g.pdf_a_conformance, PdfAConformance::A2b);
        assert_eq!(g.dpi, Some(300));
        assert_eq!(g.copies, 2);
        assert!(!g.collate);
        assert!(!g.outline);
        assert!(!g.use_compression);
        assert_eq!(g.page_offset, 5);
    }

    #[test]
    fn pdf_global_full_roundtrip() {
        let mut g = PdfGlobal::default();
        g.document_title = Some("Round-trip".into());
        g.author = Some("Author".into());
        g.pdf_a_conformance = PdfAConformance::A1b;
        g.orientation = Orientation::Landscape;
        g.color_mode = ColorMode::Grayscale;
        g.dpi = Some(150);

        let json = serde_json::to_string(&g).expect("serialize");
        let restored: PdfGlobal = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.document_title.as_deref(), Some("Round-trip"));
        assert_eq!(restored.author.as_deref(), Some("Author"));
        assert_eq!(restored.pdf_a_conformance, PdfAConformance::A1b);
        assert_eq!(restored.orientation, Orientation::Landscape);
        assert_eq!(restored.color_mode, ColorMode::Grayscale);
        assert_eq!(restored.dpi, Some(150));
    }

    // -----------------------------------------------------------------------
    // PdfObject defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn pdf_object_default_values() {
        let obj = PdfObject::default();
        assert!(obj.page.is_none());
        assert!(obj.use_external_links);
        assert!(obj.use_local_links);
        assert!(!obj.produce_forms);
        assert!(obj.include_in_outline);
        assert!(obj.pages_count);
        assert!(!obj.is_table_of_content);
        assert!(obj.replacements.is_empty());
        assert!(obj.toc_xsl.is_none());
    }

    #[test]
    fn pdf_object_fields_can_be_mutated() {
        let mut obj = PdfObject::default();
        obj.page = Some("https://example.com".into());
        obj.use_external_links = false;
        obj.use_local_links = false;
        obj.produce_forms = true;
        obj.include_in_outline = false;
        obj.is_table_of_content = true;
        obj.toc_xsl = Some("custom.xsl".into());
        obj.replacements = vec![("FOO".into(), "bar".into())];

        assert_eq!(obj.page.as_deref(), Some("https://example.com"));
        assert!(!obj.use_external_links);
        assert!(!obj.use_local_links);
        assert!(obj.produce_forms);
        assert!(!obj.include_in_outline);
        assert!(obj.is_table_of_content);
        assert_eq!(obj.toc_xsl.as_deref(), Some("custom.xsl"));
        assert_eq!(obj.replacements.len(), 1);
    }

    // -----------------------------------------------------------------------
    // CropSettings
    // -----------------------------------------------------------------------

    #[test]
    fn crop_settings_default_is_all_zeros() {
        let crop = CropSettings::default();
        assert_eq!(crop.left, 0);
        assert_eq!(crop.top, 0);
        assert_eq!(crop.width, 0);
        assert_eq!(crop.height, 0);
    }

    #[test]
    fn crop_settings_can_be_set() {
        let crop = CropSettings {
            left: 10,
            top: 20,
            width: 100,
            height: 200,
        };
        assert_eq!(crop.left, 10);
        assert_eq!(crop.top, 20);
        assert_eq!(crop.width, 100);
        assert_eq!(crop.height, 200);
    }

    #[test]
    fn crop_settings_roundtrip() {
        let crop = CropSettings {
            left: 5,
            top: 10,
            width: 640,
            height: 480,
        };
        let json = serde_json::to_string(&crop).expect("serialize");
        let restored: CropSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.left, 5);
        assert_eq!(restored.top, 10);
        assert_eq!(restored.width, 640);
        assert_eq!(restored.height, 480);
    }

    // -----------------------------------------------------------------------
    // ImageGlobal defaults and mutation
    // -----------------------------------------------------------------------

    #[test]
    fn image_global_default_values() {
        let img = ImageGlobal::default();
        assert_eq!(img.quality, 94);
        assert!(img.smart_width);
        assert!(!img.transparent);
        assert!(!img.use_graphics);
        assert!(img.page.is_none());
        assert!(img.output.is_none());
        assert!(img.format.is_none());
        assert!(img.screen_width.is_none());
        assert!(img.screen_height.is_none());
        assert!(img.dpi.is_none());
        assert_eq!(img.log_level, LogLevel::Warn);
    }

    #[test]
    fn image_global_fields_can_be_mutated() {
        let mut img = ImageGlobal::default();
        img.format = Some("jpg".into());
        img.screen_width = Some(800);
        img.screen_height = Some(600);
        img.quality = 80;
        img.transparent = true;
        img.smart_width = false;
        img.dpi = Some(150);
        img.page = Some("https://example.com".into());
        img.output = Some("out.jpg".into());

        assert_eq!(img.format.as_deref(), Some("jpg"));
        assert_eq!(img.screen_width, Some(800));
        assert_eq!(img.screen_height, Some(600));
        assert_eq!(img.quality, 80);
        assert!(img.transparent);
        assert!(!img.smart_width);
        assert_eq!(img.dpi, Some(150));
    }

    #[test]
    fn image_global_roundtrip() {
        let mut img = ImageGlobal::default();
        img.format = Some("png".into());
        img.quality = 75;
        img.dpi = Some(96);

        let json = serde_json::to_string(&img).expect("serialize");
        let restored: ImageGlobal = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.format.as_deref(), Some("png"));
        assert_eq!(restored.quality, 75);
        assert_eq!(restored.dpi, Some(96));
    }
}
