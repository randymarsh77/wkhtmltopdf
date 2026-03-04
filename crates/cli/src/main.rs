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

    let args = &cli.inputs_and_output;
    let output = args.last().unwrap();
    let inputs = &args[..args.len() - 1];

    eprintln!(
        "PDF conversion: {:?} -> {}",
        inputs, output
    );
    eprintln!("PDF rendering not yet implemented.");
    std::process::exit(1);
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
}
