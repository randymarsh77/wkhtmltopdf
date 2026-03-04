use clap::Parser;

/// Convert one or more HTML pages into a PDF document.
///
/// Usage: wkhtmltopdf [GLOBAL OPTION]... [PAGE_URL]... <output file>
///
/// Any option that is not explicitly provided keeps its built-in default.
/// Boolean flags with a --no-* variant respect the last flag given on the
/// command line.
#[derive(Parser, Debug)]
#[command(
    name = "wkhtmltopdf",
    version,
    about = "Convert HTML to PDF using Webkit",
    long_about = "Converts one or more HTML pages into a PDF document.\n\
                  \n\
                  Usage: wkhtmltopdf [GLOBAL OPTION]... [PAGE_URL]... <output file>"
)]
pub struct Cli {
    // -----------------------------------------------------------------------
    // Global options
    // -----------------------------------------------------------------------
    /// Collate when printing multiple copies (default: enabled).
    #[arg(long, overrides_with = "no_collate")]
    pub collate: bool,

    /// Do not collate when printing multiple copies.
    #[arg(long = "no-collate", overrides_with = "collate")]
    pub no_collate: bool,

    /// Read and write cookies from and to the supplied cookie jar file.
    #[arg(long, value_name = "path")]
    pub cookie_jar: Option<String>,

    /// Number of copies to print into the PDF file [default: 1].
    #[arg(long, value_name = "number")]
    pub copies: Option<u32>,

    /// Change the DPI explicitly (this has no effect on X11 based systems) [default: 96].
    #[arg(long, value_name = "dpi")]
    pub dpi: Option<i32>,

    /// PDF will be generated in grayscale.
    #[arg(long, short = 'g')]
    pub grayscale: bool,

    /// When embedding images scale them down to this DPI [default: 600].
    #[arg(long, value_name = "dpi")]
    pub image_dpi: Option<i32>,

    /// When jpeg compressing images use this quality [default: 94].
    #[arg(long, value_name = "integer")]
    pub image_quality: Option<i32>,

    /// Set log level to: none, error, warn, or info [default: warn].
    #[arg(long, value_name = "level", value_parser = ["none", "error", "warn", "info"])]
    pub log_level: Option<String>,

    /// Generates lower quality pdf/ps. Useful to shrink the result document space.
    #[arg(long, short = 'l')]
    pub lowquality: bool,

    /// Set the bottom page margin [default: 10mm].
    #[arg(long, short = 'B', value_name = "unitreal")]
    pub margin_bottom: Option<String>,

    /// Set the left page margin [default: 10mm].
    #[arg(long, short = 'L', value_name = "unitreal")]
    pub margin_left: Option<String>,

    /// Set the right page margin [default: 10mm].
    #[arg(long, short = 'R', value_name = "unitreal")]
    pub margin_right: Option<String>,

    /// Set the top page margin [default: 10mm].
    #[arg(long, short = 'T', value_name = "unitreal")]
    pub margin_top: Option<String>,

    /// Set orientation to Landscape or Portrait (default Portrait).
    #[arg(long, short = 'O', value_name = "orientation", value_parser = ["Landscape", "Portrait"])]
    pub orientation: Option<String>,

    /// Page height.
    #[arg(long, value_name = "unitreal")]
    pub page_height: Option<String>,

    /// Set paper size to: A4, Letter, etc. [default: A4].
    #[arg(long, short = 's', value_name = "size")]
    pub page_size: Option<String>,

    /// Page width.
    #[arg(long, value_name = "unitreal")]
    pub page_width: Option<String>,

    /// Do not use lossless compression on pdf objects.
    #[arg(long = "no-pdf-compression")]
    pub no_pdf_compression: bool,

    /// Be less verbose (sets log level to none).
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// The title of the generated pdf file (The title of the first document is used if not specified).
    #[arg(long, value_name = "text")]
    pub title: Option<String>,

    /// Use the X server (some plugins and other stuff might not work without X11).
    #[arg(long)]
    pub use_xserver: bool,

    /// Set viewport size if you are rendering a highly dynamic page e.g. 1280x1024.
    #[arg(long, value_name = "size")]
    pub viewport_size: Option<String>,

    /// Dump the default TOC xsl style sheet to stdout.
    #[arg(long)]
    pub dump_default_toc_xsl: bool,

    /// Dump the outline to a file.
    #[arg(long, value_name = "file")]
    pub dump_outline: Option<String>,

    /// Put an outline into the pdf [default: enabled].
    #[arg(long, overrides_with = "no_outline")]
    pub outline: bool,

    /// Do not put an outline into the pdf.
    #[arg(long = "no-outline", overrides_with = "outline")]
    pub no_outline: bool,

    /// Set the depth of the outline [default: 4].
    #[arg(long, value_name = "level")]
    pub outline_depth: Option<u32>,

    /// Set the starting page number [default: 0].
    #[arg(long, value_name = "offset")]
    pub page_offset: Option<i32>,

    /// Resolve relative external links into absolute links [default: enabled].
    #[arg(long, overrides_with = "no_resolve_relative_links")]
    pub resolve_relative_links: bool,

    /// Do not resolve relative external links into absolute links.
    #[arg(long = "no-resolve-relative-links", overrides_with = "resolve_relative_links")]
    pub no_resolve_relative_links: bool,

    // -----------------------------------------------------------------------
    // Page object options (applied to all page objects)
    // -----------------------------------------------------------------------
    /// Allow the file or files from the specified folder to be loaded (repeatable).
    #[arg(long, value_name = "path")]
    pub allow: Vec<String>,

    /// Do print background [default: enabled].
    #[arg(long, overrides_with = "no_background")]
    pub background: bool,

    /// Do not print background.
    #[arg(long = "no-background", overrides_with = "background")]
    pub no_background: bool,

    /// Bypass proxy for host (repeatable).
    #[arg(long, value_name = "value")]
    pub bypass_proxy_for: Vec<String>,

    /// Web cache directory.
    #[arg(long, value_name = "path")]
    pub cache_dir: Option<String>,

    /// Set an additional cookie (repeatable), value should be url encoded.
    #[arg(long, value_names = ["name", "value"], num_args = 2)]
    pub cookie: Vec<String>,

    /// Set an additional HTTP header (repeatable).
    #[arg(long, value_names = ["name", "value"], num_args = 2)]
    pub custom_header: Vec<String>,

    /// Add HTTP headers specified by --custom-header for each resource request.
    #[arg(long, overrides_with = "no_custom_header_propagation")]
    pub custom_header_propagation: bool,

    /// Do not add HTTP headers specified by --custom-header for each resource request.
    #[arg(
        long = "no-custom-header-propagation",
        overrides_with = "custom_header_propagation"
    )]
    pub no_custom_header_propagation: bool,

    /// Show javascript debugging output.
    #[arg(long, overrides_with = "no_debug_javascript")]
    pub debug_javascript: bool,

    /// Do not show javascript debugging output [default].
    #[arg(
        long = "no-debug-javascript",
        overrides_with = "debug_javascript"
    )]
    pub no_debug_javascript: bool,

    /// Set the default text encoding, for input.
    #[arg(long, value_name = "encoding")]
    pub encoding: Option<String>,

    /// Do not make links to remote web pages.
    #[arg(long = "disable-external-links", overrides_with = "enable_external_links")]
    pub disable_external_links: bool,

    /// Make links to remote web pages [default: enabled].
    #[arg(long = "enable-external-links", overrides_with = "disable_external_links")]
    pub enable_external_links: bool,

    /// Turn HTML form fields into pdf form fields.
    #[arg(long = "enable-forms", overrides_with = "disable_forms")]
    pub enable_forms: bool,

    /// Do not turn HTML form fields into pdf form fields [default].
    #[arg(long = "disable-forms", overrides_with = "enable_forms")]
    pub disable_forms: bool,

    /// Do load or print images [default: enabled].
    #[arg(long, overrides_with = "no_images")]
    pub images: bool,

    /// Do not load or print images.
    #[arg(long = "no-images", overrides_with = "images")]
    pub no_images: bool,

    /// Do not make local links [default: enabled].
    #[arg(long = "disable-internal-links", overrides_with = "enable_internal_links")]
    pub disable_internal_links: bool,

    /// Make local links [default: enabled].
    #[arg(long = "enable-internal-links", overrides_with = "disable_internal_links")]
    pub enable_internal_links: bool,

    /// Do not allow web pages to run javascript.
    #[arg(long = "disable-javascript", overrides_with = "enable_javascript")]
    pub disable_javascript: bool,

    /// Do allow web pages to run javascript [default: enabled].
    #[arg(long = "enable-javascript", overrides_with = "disable_javascript")]
    pub enable_javascript: bool,

    /// Wait some milliseconds for javascript finish [default: 200].
    #[arg(long, value_name = "msec")]
    pub javascript_delay: Option<u32>,

    /// Specify how to handle pages that fail to load: abort, ignore or skip [default: abort].
    #[arg(long, value_name = "handler", value_parser = ["abort", "ignore", "skip"])]
    pub load_error_handling: Option<String>,

    /// Specify how to handle media files that fail to load: abort, ignore or skip [default: ignore].
    #[arg(long, value_name = "handler", value_parser = ["abort", "ignore", "skip"])]
    pub load_media_error_handling: Option<String>,

    /// Do not allowed conversion of a local file to read in other local files, unless explicitly allowed with --allow.
    #[arg(long = "disable-local-file-access", overrides_with = "enable_local_file_access")]
    pub disable_local_file_access: bool,

    /// Allowed conversion of a local file to read in other local files [default: enabled].
    #[arg(long = "enable-local-file-access", overrides_with = "disable_local_file_access")]
    pub enable_local_file_access: bool,

    /// Minimum font size [default: none].
    #[arg(long, value_name = "int")]
    pub minimum_font_size: Option<i32>,

    /// Do not include the page in the table of contents and outlines.
    #[arg(long = "exclude-from-outline", overrides_with = "include_in_outline")]
    pub exclude_from_outline: bool,

    /// Include the page in the table of contents and outlines [default: enabled].
    #[arg(long = "include-in-outline", overrides_with = "exclude_from_outline")]
    pub include_in_outline: bool,

    /// HTTP Authentication password.
    #[arg(long, value_name = "password")]
    pub password: Option<String>,

    /// Enable installed plugins (plugins will likely not work).
    #[arg(long = "enable-plugins", overrides_with = "disable_plugins")]
    pub enable_plugins: bool,

    /// Disable installed plugins [default: disabled].
    #[arg(long = "disable-plugins", overrides_with = "enable_plugins")]
    pub disable_plugins: bool,

    /// Add an additional post field (repeatable).
    #[arg(long, value_names = ["name", "value"], num_args = 2)]
    pub post: Vec<String>,

    /// Post a file (repeatable).
    #[arg(long, value_names = ["name", "path"], num_args = 2)]
    pub post_file: Vec<String>,

    /// Use print media-type instead of screen.
    #[arg(long, overrides_with = "no_print_media_type")]
    pub print_media_type: bool,

    /// Do not use print media-type instead of screen [default: disabled].
    #[arg(long = "no-print-media-type", overrides_with = "print_media_type")]
    pub no_print_media_type: bool,

    /// Use a proxy.
    #[arg(long, value_name = "proxy")]
    pub proxy: Option<String>,

    /// Do lookups for hostnames through the proxy.
    #[arg(long, overrides_with = "no_proxy_hostname_lookup")]
    pub proxy_hostname_lookup: bool,

    /// Do not lookups for hostnames through the proxy [default].
    #[arg(long = "no-proxy-hostname-lookup", overrides_with = "proxy_hostname_lookup")]
    pub no_proxy_hostname_lookup: bool,

    /// Run this additional javascript after the page is done loading (repeatable).
    #[arg(long, value_name = "js")]
    pub run_script: Vec<String>,

    /// Disable the intelligent shrinking strategy used by WebKit that makes the pixel/dpi ratio non-constant.
    #[arg(long = "disable-smart-shrinking", overrides_with = "enable_smart_shrinking")]
    pub disable_smart_shrinking: bool,

    /// Enable the intelligent shrinking strategy used by WebKit [default: enabled].
    #[arg(long = "enable-smart-shrinking", overrides_with = "disable_smart_shrinking")]
    pub enable_smart_shrinking: bool,

    /// Path to the ssl client cert public key in OpenSSL PEM format.
    #[arg(long, value_name = "path")]
    pub ssl_crt_path: Option<String>,

    /// Password to ssl client cert private key.
    #[arg(long, value_name = "password")]
    pub ssl_key_password: Option<String>,

    /// Path to ssl client cert private key in OpenSSL PEM format.
    #[arg(long, value_name = "path")]
    pub ssl_key_path: Option<String>,

    /// Stop slow running javascripts [default: enabled].
    #[arg(long, overrides_with = "no_stop_slow_scripts")]
    pub stop_slow_scripts: bool,

    /// Do not stop slow running javascripts.
    #[arg(long = "no-stop-slow-scripts", overrides_with = "stop_slow_scripts")]
    pub no_stop_slow_scripts: bool,

    /// Disable the TOC back-link from each header.
    #[arg(long = "disable-toc-back-links", overrides_with = "enable_toc_back_links")]
    pub disable_toc_back_links: bool,

    /// Enable the TOC back-link from each header [default: enabled].
    #[arg(long = "enable-toc-back-links", overrides_with = "disable_toc_back_links")]
    pub enable_toc_back_links: bool,

    /// Specify a user style sheet, to load with every page.
    #[arg(long, value_name = "url")]
    pub user_style_sheet: Option<String>,

    /// HTTP Authentication username.
    #[arg(long, value_name = "username")]
    pub username: Option<String>,

    /// Wait until window.status is equal to this string before rendering page.
    #[arg(long, value_name = "window-status")]
    pub window_status: Option<String>,

    /// Use this zoom factor [default: 1].
    #[arg(long, value_name = "float")]
    pub zoom: Option<f64>,

    // -----------------------------------------------------------------------
    // Header options
    // -----------------------------------------------------------------------
    /// Centered header text.
    #[arg(long, value_name = "text")]
    pub header_center: Option<String>,

    /// Set header font name [default: Arial].
    #[arg(long, value_name = "name")]
    pub header_font_name: Option<String>,

    /// Set header font size [default: 12].
    #[arg(long, value_name = "size")]
    pub header_font_size: Option<i32>,

    /// Adds a html header; the URL/path of the document.
    #[arg(long, value_name = "url")]
    pub header_html: Option<String>,

    /// Left-aligned header text.
    #[arg(long, value_name = "text")]
    pub header_left: Option<String>,

    /// Display line below the header.
    #[arg(long, overrides_with = "no_header_line")]
    pub header_line: bool,

    /// Do not display line below the header [default: no line].
    #[arg(long = "no-header-line", overrides_with = "header_line")]
    pub no_header_line: bool,

    /// Right-aligned header text.
    #[arg(long, value_name = "text")]
    pub header_right: Option<String>,

    /// Spacing between header and content in mm [default: 0].
    #[arg(long, value_name = "real")]
    pub header_spacing: Option<f32>,

    // -----------------------------------------------------------------------
    // Footer options
    // -----------------------------------------------------------------------
    /// Centered footer text.
    #[arg(long, value_name = "text")]
    pub footer_center: Option<String>,

    /// Set footer font name [default: Arial].
    #[arg(long, value_name = "name")]
    pub footer_font_name: Option<String>,

    /// Set footer font size [default: 12].
    #[arg(long, value_name = "size")]
    pub footer_font_size: Option<i32>,

    /// Adds a html footer; the URL/path of the document.
    #[arg(long, value_name = "url")]
    pub footer_html: Option<String>,

    /// Left-aligned footer text.
    #[arg(long, value_name = "text")]
    pub footer_left: Option<String>,

    /// Display line above the footer.
    #[arg(long, overrides_with = "no_footer_line")]
    pub footer_line: bool,

    /// Do not display line above the footer [default: no line].
    #[arg(long = "no-footer-line", overrides_with = "footer_line")]
    pub no_footer_line: bool,

    /// Right-aligned footer text.
    #[arg(long, value_name = "text")]
    pub footer_right: Option<String>,

    /// Spacing between footer and content in mm [default: 0].
    #[arg(long, value_name = "real")]
    pub footer_spacing: Option<f32>,

    // -----------------------------------------------------------------------
    // TOC options
    // -----------------------------------------------------------------------
    /// Insert a Table of Contents before the first page.
    #[arg(long)]
    pub toc: bool,

    /// Maximum heading depth to include in the TOC [default: 3].
    #[arg(long, value_name = "level")]
    pub toc_depth: Option<u32>,

    /// Use dotted lines in the toc [default: enabled].
    #[arg(long, overrides_with = "disable_dotted_lines")]
    pub enable_dotted_lines: bool,

    /// Do not use dotted lines in the toc.
    #[arg(long, overrides_with = "enable_dotted_lines")]
    pub disable_dotted_lines: bool,

    /// The header text of the toc [default: "Table of Contents"].
    #[arg(long, value_name = "text")]
    pub toc_header_text: Option<String>,

    /// Do not link from the toc to sections.
    #[arg(long = "disable-toc-links", overrides_with = "enable_toc_links")]
    pub disable_toc_links: bool,

    /// Link from toc to sections [default: enabled].
    #[arg(long = "enable-toc-links", overrides_with = "disable_toc_links")]
    pub enable_toc_links: bool,

    /// For each level of headings in the toc indent by this amount [default: 1em].
    #[arg(long, value_name = "indentation")]
    pub toc_level_indentation: Option<String>,

    /// For each level of headings in the toc the font is scaled by this factor [default: 0.8].
    #[arg(long, value_name = "shrink")]
    pub toc_text_size_shrink: Option<f32>,

    /// Use the supplied xsl style sheet for printing the table of contents.
    #[arg(long, value_name = "file")]
    pub xsl_style_sheet: Option<String>,

    // -----------------------------------------------------------------------
    // Replace option
    // -----------------------------------------------------------------------
    /// Replace [name] with value in headers and footers (repeatable).
    #[arg(long, value_names = ["name", "value"], num_args = 2)]
    pub replace: Vec<String>,

    // -----------------------------------------------------------------------
    // Positional arguments: one or more input URLs/files, then output path
    // -----------------------------------------------------------------------
    /// One or more input HTML URLs or file paths followed by the output PDF path.
    #[arg(
        required = true,
        num_args = 2..,
        value_name = "URL|file",
        help = "Input URL(s)/file(s) followed by the output PDF path"
    )]
    pub inputs_and_output: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    // --dump-default-toc-xsl: print the default XSL stylesheet and exit.
    if cli.dump_default_toc_xsl {
        print!("{}", wkhtmltopdf_pdf::default_toc_xsl());
        return;
    }

    let args = &cli.inputs_and_output;
    let output = args.last().unwrap().clone();
    let inputs = &args[..args.len() - 1];

    // Build global settings from CLI flags.
    let global = build_global(&cli);

    // Build the converter and add one PdfObject per input.
    let mut converter = wkhtmltopdf_pdf::PdfConverter::new(global);

    // If --toc was specified, prepend a TOC page object before the first input.
    if cli.toc {
        let toc_obj = build_toc_object(&cli);
        converter.add_object(toc_obj);
    }

    for input in inputs {
        let object = build_object(&cli, input);
        converter.add_object(object);
    }

    // Run the conversion.
    use wkhtmltopdf_core::Converter;
    let pdf_bytes = match converter.convert() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    // Write output file.
    if let Err(e) = std::fs::write(&output, &pdf_bytes) {
        eprintln!("error writing output file '{output}': {e}");
        std::process::exit(1);
    }
}

/// Build [`PdfGlobal`] from the parsed CLI arguments.
fn build_global(cli: &Cli) -> wkhtmltopdf_settings::PdfGlobal {
    use wkhtmltopdf_settings::{
        ColorMode, LogLevel, Margin, Orientation, PdfGlobal, Unit, UnitReal,
    };

    let mut g = PdfGlobal::default();

    // Paper size / dimensions
    if let Some(ref s) = cli.page_size {
        g.size.page_size = parse_page_size(s);
    }
    if let Some(ref w) = cli.page_width {
        g.size.width = Some(parse_unit_real(w));
    }
    if let Some(ref h) = cli.page_height {
        g.size.height = Some(parse_unit_real(h));
    }

    // Orientation
    if let Some(ref o) = cli.orientation {
        g.orientation = if o.eq_ignore_ascii_case("landscape") {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        };
    }

    // Color mode
    if cli.grayscale {
        g.color_mode = ColorMode::Grayscale;
    }

    // DPI
    if let Some(dpi) = cli.dpi {
        g.dpi = Some(dpi);
    }
    if let Some(q) = cli.image_quality {
        g.image_quality = q;
    }
    if let Some(d) = cli.image_dpi {
        g.image_dpi = d;
    }

    // Copies / collate
    if let Some(c) = cli.copies {
        g.copies = c;
    }
    if cli.no_collate {
        g.collate = false;
    } else if cli.collate {
        g.collate = true;
    }

    // Log level
    if cli.quiet {
        g.log_level = LogLevel::None;
    } else if let Some(ref l) = cli.log_level {
        g.log_level = match l.as_str() {
            "none" => LogLevel::None,
            "error" => LogLevel::Error,
            "info" => LogLevel::Info,
            _ => LogLevel::Warn,
        };
    }

    // Margins  (default 10 mm each side)
    let default_margin = || UnitReal { value: 10.0, unit: Unit::Millimeter };
    g.margin = Margin {
        top: cli.margin_top.as_deref().map(parse_unit_real).unwrap_or_else(default_margin),
        bottom: cli.margin_bottom.as_deref().map(parse_unit_real).unwrap_or_else(default_margin),
        left: cli.margin_left.as_deref().map(parse_unit_real).unwrap_or_else(default_margin),
        right: cli.margin_right.as_deref().map(parse_unit_real).unwrap_or_else(default_margin),
    };

    // Compression
    if cli.no_pdf_compression {
        g.use_compression = false;
    }

    // Outline
    if cli.no_outline {
        g.outline = false;
    } else if cli.outline {
        g.outline = true;
    }
    if let Some(d) = cli.outline_depth {
        g.outline_depth = d;
    }
    if let Some(ref f) = cli.dump_outline {
        g.dump_outline = Some(f.clone());
    }

    // Page offset
    if let Some(o) = cli.page_offset {
        g.page_offset = o;
    }

    // Metadata
    if let Some(ref t) = cli.title {
        g.document_title = Some(t.clone());
    }

    // Viewport
    if let Some(ref v) = cli.viewport_size {
        g.viewport_size = Some(v.clone());
    }

    // Load settings
    if let Some(ref c) = cli.cookie_jar {
        g.load.cookie_jar = Some(c.clone());
    }

    g
}

/// Build a [`PdfObject`] for one input URL/file, applying all per-page CLI
/// options (including header/footer).
fn build_object(cli: &Cli, input: &str) -> wkhtmltopdf_settings::PdfObject {
    use wkhtmltopdf_settings::{
        LoadErrorHandling, PdfObject,
    };

    let mut obj = PdfObject::default();
    obj.page = Some(input.to_string());

    // Header settings
    obj.header = build_header_footer(
        cli.header_left.as_deref(),
        cli.header_center.as_deref(),
        cli.header_right.as_deref(),
        cli.header_html.as_deref(),
        cli.header_font_name.as_deref(),
        cli.header_font_size,
        cli.header_line,
        cli.no_header_line,
        cli.header_spacing,
    );

    // Footer settings
    obj.footer = build_header_footer(
        cli.footer_left.as_deref(),
        cli.footer_center.as_deref(),
        cli.footer_right.as_deref(),
        cli.footer_html.as_deref(),
        cli.footer_font_name.as_deref(),
        cli.footer_font_size,
        cli.footer_line,
        cli.no_footer_line,
        cli.footer_spacing,
    );

    // Web settings
    if cli.no_background {
        obj.web.background = false;
    }
    if cli.no_images {
        obj.web.load_images = false;
    }
    if cli.disable_javascript {
        obj.web.enable_javascript = false;
    }
    if cli.disable_smart_shrinking {
        obj.web.enable_intelligent_shrinking = false;
    }
    if let Some(s) = cli.minimum_font_size {
        obj.web.minimum_font_size = Some(s);
    }
    if let Some(ref enc) = cli.encoding {
        obj.web.default_encoding = Some(enc.clone());
    }
    if let Some(ref css) = cli.user_style_sheet {
        obj.web.user_style_sheet = Some(css.clone());
    }
    if cli.enable_plugins {
        obj.web.enable_plugins = true;
    }

    // Load settings
    if let Some(ref u) = cli.username {
        obj.load.username = Some(u.clone());
    }
    if let Some(ref p) = cli.password {
        obj.load.password = Some(p.clone());
    }
    if let Some(ms) = cli.javascript_delay {
        obj.load.js_delay = ms;
    }
    if let Some(z) = cli.zoom {
        obj.load.zoom = z;
    }
    if let Some(ref ws) = cli.window_status {
        obj.load.window_status = Some(ws.clone());
    }
    if let Some(ref h) = cli.load_error_handling {
        obj.load.load_error_handling = match h.as_str() {
            "skip" => LoadErrorHandling::Skip,
            "ignore" => LoadErrorHandling::Ignore,
            _ => LoadErrorHandling::Abort,
        };
    }
    if let Some(ref h) = cli.load_media_error_handling {
        obj.load.media_load_error_handling = match h.as_str() {
            "abort" => LoadErrorHandling::Abort,
            "skip" => LoadErrorHandling::Skip,
            _ => LoadErrorHandling::Ignore,
        };
    }
    if cli.print_media_type {
        obj.load.print_media_type = true;
    }
    if cli.debug_javascript {
        obj.load.debug_javascript = true;
    }
    if cli.proxy_hostname_lookup {
        obj.load.proxy_hostname_lookup = true;
    }
    if cli.disable_local_file_access {
        obj.load.block_local_file_access = true;
    }
    if cli.stop_slow_scripts {
        obj.load.stop_slow_scripts = true;
    }
    if let Some(ref c) = cli.cache_dir {
        obj.load.cache_dir = Some(c.clone());
    }
    if let Some(ref key) = cli.ssl_key_path {
        obj.load.client_ssl_key_path = Some(key.clone());
    }
    if let Some(ref pw) = cli.ssl_key_password {
        obj.load.client_ssl_key_password = Some(pw.clone());
    }
    if let Some(ref crt) = cli.ssl_crt_path {
        obj.load.client_ssl_crt_path = Some(crt.clone());
    }
    if cli.custom_header_propagation {
        obj.load.repeat_custom_headers = true;
    }
    // Custom headers (pairs)
    obj.load.custom_headers.extend(collect_pairs(&cli.custom_header, "--custom-header"));
    // Cookies (pairs)
    obj.load.cookies.extend(collect_pairs(&cli.cookie, "--cookie"));
    // Run scripts
    obj.load.run_script.extend(cli.run_script.iter().cloned());
    // Allow paths
    obj.load.allowed.extend(cli.allow.iter().cloned());
    // Bypass proxy hosts
    obj.load.bypass_proxy_for_hosts.extend(cli.bypass_proxy_for.iter().cloned());

    // Links
    if cli.disable_external_links {
        obj.use_external_links = false;
    }
    if cli.disable_internal_links {
        obj.use_local_links = false;
    }
    if cli.enable_forms {
        obj.produce_forms = true;
    }

    // Outline inclusion
    if cli.exclude_from_outline {
        obj.include_in_outline = false;
    }

    // Text replacements (pairs)
    obj.replacements.extend(collect_pairs(&cli.replace, "--replace"));

    // TOC settings
    if cli.disable_dotted_lines {
        obj.toc.use_dotted_lines = false;
    }
    if let Some(ref t) = cli.toc_header_text {
        obj.toc.caption_text = t.clone();
    }
    if cli.disable_toc_links {
        obj.toc.forward_links = false;
    }
    if cli.disable_toc_back_links {
        obj.toc.back_links = false;
    }
    if let Some(ref i) = cli.toc_level_indentation {
        obj.toc.indentation = i.clone();
    }
    if let Some(s) = cli.toc_text_size_shrink {
        obj.toc.font_scale = s;
    }
    if let Some(d) = cli.toc_depth {
        obj.toc.depth = d;
    }
    if let Some(ref x) = cli.xsl_style_sheet {
        obj.toc_xsl = Some(x.clone());
    }

    obj
}

/// Build a [`PdfObject`] that represents a generated Table of Contents page.
///
/// This object has no source URL; the HTML is auto-generated by
/// [`PdfConverter`] from the headings found in the other page objects.
fn build_toc_object(cli: &Cli) -> wkhtmltopdf_settings::PdfObject {
    use wkhtmltopdf_settings::PdfObject;
    let mut obj = PdfObject::default();
    obj.is_table_of_content = true;
    obj.page = None;

    // Apply the same TOC settings that build_object() applies.
    if cli.disable_dotted_lines {
        obj.toc.use_dotted_lines = false;
    }
    if let Some(ref t) = cli.toc_header_text {
        obj.toc.caption_text = t.clone();
    }
    if cli.disable_toc_links {
        obj.toc.forward_links = false;
    }
    if cli.disable_toc_back_links {
        obj.toc.back_links = false;
    }
    if let Some(ref i) = cli.toc_level_indentation {
        obj.toc.indentation = i.clone();
    }
    if let Some(s) = cli.toc_text_size_shrink {
        obj.toc.font_scale = s;
    }
    if let Some(d) = cli.toc_depth {
        obj.toc.depth = d;
    }
    if let Some(ref x) = cli.xsl_style_sheet {
        obj.toc_xsl = Some(x.clone());
    }

    obj
}

/// Construct a [`HeaderFooter`] value from the individual CLI header/footer
/// option values.
fn build_header_footer(
    left: Option<&str>,
    center: Option<&str>,
    right: Option<&str>,
    html_url: Option<&str>,
    font_name: Option<&str>,
    font_size: Option<i32>,
    line_on: bool,
    line_off: bool,
    spacing: Option<f32>,
) -> wkhtmltopdf_settings::HeaderFooter {
    use wkhtmltopdf_settings::HeaderFooter;
    let mut hf = HeaderFooter::default();
    hf.left = left.map(str::to_string);
    hf.center = center.map(str::to_string);
    hf.right = right.map(str::to_string);
    hf.html_url = html_url.map(str::to_string);
    if let Some(n) = font_name {
        hf.font_name = n.to_string();
    }
    if let Some(s) = font_size {
        hf.font_size = s;
    }
    if line_on {
        hf.line = true;
    } else if line_off {
        hf.line = false;
    }
    if let Some(s) = spacing {
        hf.spacing = s;
    }
    hf
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Collect `(name, value)` string pairs from a flat list.
///
/// Clap guarantees that multi-value args with `num_args = 2` always produce
/// an even-length list; this helper handles the general case and emits a
/// warning to stderr for any trailing unpaired element.
fn collect_pairs(items: &[String], flag: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::with_capacity(items.len() / 2);
    let mut iter = items.iter();
    loop {
        match (iter.next(), iter.next()) {
            (Some(k), Some(v)) => pairs.push((k.clone(), v.clone())),
            (Some(k), None) => {
                eprintln!("warning: {flag} '{k}' has no matching value and will be ignored");
                break;
            }
            _ => break,
        }
    }
    pairs
}

/// Parse a `unitreal` string such as `"10mm"`, `"1.5in"`, `"72pt"` into a
/// [`UnitReal`].  Falls back to millimetres when no unit suffix is recognised.
fn parse_unit_real(s: &str) -> wkhtmltopdf_settings::UnitReal {
    use wkhtmltopdf_settings::{Unit, UnitReal};
    let s = s.trim();
    let suffixes: &[(&str, Unit)] = &[
        ("mm", Unit::Millimeter),
        ("cm", Unit::Centimeter),
        ("in", Unit::Inch),
        ("pt", Unit::Point),
        ("pc", Unit::Pica),
        ("px", Unit::Pixel),
    ];
    for (suffix, unit) in suffixes {
        if let Some(num) = s.strip_suffix(suffix) {
            if let Ok(v) = num.trim().parse::<f64>() {
                return UnitReal { value: v, unit: *unit };
            }
        }
    }
    // No unit suffix — try plain number (assume mm).
    let v = s.parse::<f64>().unwrap_or(0.0);
    UnitReal { value: v, unit: Unit::Millimeter }
}

/// Map a page-size string (case-insensitive) to [`PageSize`].
fn parse_page_size(s: &str) -> wkhtmltopdf_settings::PageSize {
    use wkhtmltopdf_settings::PageSize;
    match s.to_ascii_uppercase().as_str() {
        "A0" => PageSize::A0,
        "A1" => PageSize::A1,
        "A2" => PageSize::A2,
        "A3" => PageSize::A3,
        "A4" => PageSize::A4,
        "A5" => PageSize::A5,
        "A6" => PageSize::A6,
        "A7" => PageSize::A7,
        "A8" => PageSize::A8,
        "A9" => PageSize::A9,
        "B0" => PageSize::B0,
        "B1" => PageSize::B1,
        "B2" => PageSize::B2,
        "B3" => PageSize::B3,
        "B4" => PageSize::B4,
        "B5" => PageSize::B5,
        "B6" => PageSize::B6,
        "B7" => PageSize::B7,
        "B8" => PageSize::B8,
        "B9" => PageSize::B9,
        "B10" => PageSize::B10,
        "LETTER" => PageSize::Letter,
        "LEGAL" => PageSize::Legal,
        "EXECUTIVE" => PageSize::Executive,
        "TABLOID" => PageSize::Tabloid,
        "LEDGER" => PageSize::Ledger,
        _ => PageSize::Custom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("argument parsing failed")
    }

    #[test]
    fn minimal_invocation_single_input() {
        let cli = parse(&["wkhtmltopdf", "input.html", "output.pdf"]);
        assert_eq!(cli.inputs_and_output, vec!["input.html", "output.pdf"]);
    }

    #[test]
    fn multiple_inputs() {
        let cli = parse(&["wkhtmltopdf", "a.html", "b.html", "out.pdf"]);
        assert_eq!(cli.inputs_and_output.len(), 3);
        assert_eq!(cli.inputs_and_output.last().unwrap(), "out.pdf");
    }

    #[test]
    fn grayscale_flag() {
        let cli = parse(&["wkhtmltopdf", "--grayscale", "in.html", "out.pdf"]);
        assert!(cli.grayscale);
    }

    #[test]
    fn quiet_flag() {
        let cli = parse(&["wkhtmltopdf", "-q", "in.html", "out.pdf"]);
        assert!(cli.quiet);
    }

    #[test]
    fn orientation_landscape() {
        let cli = parse(&["wkhtmltopdf", "--orientation", "Landscape", "in.html", "out.pdf"]);
        assert_eq!(cli.orientation.as_deref(), Some("Landscape"));
    }

    #[test]
    fn page_size_letter() {
        let cli = parse(&["wkhtmltopdf", "-s", "Letter", "in.html", "out.pdf"]);
        assert_eq!(cli.page_size.as_deref(), Some("Letter"));
    }

    #[test]
    fn log_level_none() {
        let cli = parse(&["wkhtmltopdf", "--log-level", "none", "in.html", "out.pdf"]);
        assert_eq!(cli.log_level.as_deref(), Some("none"));
    }

    #[test]
    fn copies_option() {
        let cli = parse(&["wkhtmltopdf", "--copies", "3", "in.html", "out.pdf"]);
        assert_eq!(cli.copies, Some(3));
    }

    #[test]
    fn dpi_option() {
        let cli = parse(&["wkhtmltopdf", "--dpi", "150", "in.html", "out.pdf"]);
        assert_eq!(cli.dpi, Some(150));
    }

    #[test]
    fn margins_short_flags() {
        let cli = parse(&[
            "wkhtmltopdf",
            "-T", "20mm",
            "-B", "20mm",
            "-L", "15mm",
            "-R", "15mm",
            "in.html",
            "out.pdf",
        ]);
        assert_eq!(cli.margin_top.as_deref(), Some("20mm"));
        assert_eq!(cli.margin_bottom.as_deref(), Some("20mm"));
        assert_eq!(cli.margin_left.as_deref(), Some("15mm"));
        assert_eq!(cli.margin_right.as_deref(), Some("15mm"));
    }

    #[test]
    fn outline_flags() {
        let cli = parse(&["wkhtmltopdf", "--outline", "in.html", "out.pdf"]);
        assert!(cli.outline);

        let cli2 = parse(&["wkhtmltopdf", "--no-outline", "in.html", "out.pdf"]);
        assert!(cli2.no_outline);
    }

    #[test]
    fn outline_depth() {
        let cli = parse(&["wkhtmltopdf", "--outline-depth", "6", "in.html", "out.pdf"]);
        assert_eq!(cli.outline_depth, Some(6));
    }

    #[test]
    fn title_option() {
        let cli = parse(&["wkhtmltopdf", "--title", "My Document", "in.html", "out.pdf"]);
        assert_eq!(cli.title.as_deref(), Some("My Document"));
    }

    #[test]
    fn dump_outline() {
        let cli = parse(&["wkhtmltopdf", "--dump-outline", "toc.xml", "in.html", "out.pdf"]);
        assert_eq!(cli.dump_outline.as_deref(), Some("toc.xml"));
    }

    #[test]
    fn cookie_flag() {
        let cli = parse(&["wkhtmltopdf", "--cookie", "name", "val", "in.html", "out.pdf"]);
        assert_eq!(cli.cookie, vec!["name", "val"]);
    }

    #[test]
    fn custom_header_flag() {
        let cli = parse(&[
            "wkhtmltopdf",
            "--custom-header", "X-Auth", "token123",
            "in.html",
            "out.pdf",
        ]);
        assert_eq!(cli.custom_header, vec!["X-Auth", "token123"]);
    }

    #[test]
    fn load_error_handling() {
        let cli = parse(&[
            "wkhtmltopdf",
            "--load-error-handling", "ignore",
            "in.html",
            "out.pdf",
        ]);
        assert_eq!(cli.load_error_handling.as_deref(), Some("ignore"));
    }

    #[test]
    fn zoom_option() {
        let cli = parse(&["wkhtmltopdf", "--zoom", "1.5", "in.html", "out.pdf"]);
        assert!((cli.zoom.unwrap() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn header_and_footer_options() {
        let cli = parse(&[
            "wkhtmltopdf",
            "--header-left", "[title]",
            "--header-right", "[page]/[topage]",
            "--header-line",
            "--footer-center", "Page [page]",
            "--footer-font-size", "9",
            "in.html",
            "out.pdf",
        ]);
        assert_eq!(cli.header_left.as_deref(), Some("[title]"));
        assert_eq!(cli.header_right.as_deref(), Some("[page]/[topage]"));
        assert!(cli.header_line);
        assert_eq!(cli.footer_center.as_deref(), Some("Page [page]"));
        assert_eq!(cli.footer_font_size, Some(9));
    }

    #[test]
    fn replace_flag() {
        let cli = parse(&[
            "wkhtmltopdf",
            "--replace", "TITLE", "My Title",
            "in.html",
            "out.pdf",
        ]);
        assert_eq!(cli.replace, vec!["TITLE", "My Title"]);
    }

    #[test]
    fn javascript_flags() {
        let cli = parse(&[
            "wkhtmltopdf",
            "--disable-javascript",
            "--javascript-delay", "500",
            "in.html",
            "out.pdf",
        ]);
        assert!(cli.disable_javascript);
        assert_eq!(cli.javascript_delay, Some(500));
    }

    #[test]
    fn no_collate_flag() {
        let cli = parse(&["wkhtmltopdf", "--no-collate", "in.html", "out.pdf"]);
        assert!(cli.no_collate);
    }

    #[test]
    fn image_quality_and_dpi() {
        let cli = parse(&[
            "wkhtmltopdf",
            "--image-quality", "80",
            "--image-dpi", "300",
            "in.html",
            "out.pdf",
        ]);
        assert_eq!(cli.image_quality, Some(80));
        assert_eq!(cli.image_dpi, Some(300));
    }

    #[test]
    fn missing_output_fails() {
        assert!(Cli::try_parse_from(["wkhtmltopdf", "input.html"]).is_err());
    }

    #[test]
    fn invalid_orientation_fails() {
        assert!(Cli::try_parse_from([
            "wkhtmltopdf",
            "--orientation", "Invalid",
            "in.html", "out.pdf"
        ])
        .is_err());
    }

    #[test]
    fn invalid_log_level_fails() {
        assert!(Cli::try_parse_from([
            "wkhtmltopdf",
            "--log-level", "verbose",
            "in.html", "out.pdf"
        ])
        .is_err());
    }

    #[test]
    fn toc_flag() {
        let cli = parse(&["wkhtmltopdf", "--toc", "in.html", "out.pdf"]);
        assert!(cli.toc);
    }

    #[test]
    fn toc_depth_option() {
        let cli = parse(&["wkhtmltopdf", "--toc-depth", "4", "in.html", "out.pdf"]);
        assert_eq!(cli.toc_depth, Some(4));
    }

    #[test]
    fn toc_header_text_option() {
        let cli = parse(&["wkhtmltopdf", "--toc-header-text", "Contents", "in.html", "out.pdf"]);
        assert_eq!(cli.toc_header_text.as_deref(), Some("Contents"));
    }

    #[test]
    fn disable_dotted_lines_flag() {
        let cli = parse(&["wkhtmltopdf", "--disable-dotted-lines", "in.html", "out.pdf"]);
        assert!(cli.disable_dotted_lines);
    }

    #[test]
    fn xsl_style_sheet_option() {
        let cli = parse(&["wkhtmltopdf", "--xsl-style-sheet", "toc.xsl", "in.html", "out.pdf"]);
        assert_eq!(cli.xsl_style_sheet.as_deref(), Some("toc.xsl"));
    }

    #[test]
    fn dump_default_toc_xsl_flag() {
        let cli = parse(&["wkhtmltopdf", "--dump-default-toc-xsl", "in.html", "out.pdf"]);
        assert!(cli.dump_default_toc_xsl);
    }

    #[test]
    fn build_toc_object_sets_is_toc() {
        let cli = parse(&["wkhtmltopdf", "--toc", "--toc-depth", "5", "in.html", "out.pdf"]);
        let obj = build_toc_object(&cli);
        assert!(obj.is_table_of_content);
        assert!(obj.page.is_none());
        assert_eq!(obj.toc.depth, 5);
    }
}
