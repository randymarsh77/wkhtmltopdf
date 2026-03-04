use clap::Parser;

/// Convert an HTML page into an image.
///
/// Usage: wkhtmltoimage [OPTIONS] <URL|file> <output>
#[derive(Parser, Debug)]
#[command(
    name = "wkhtmltoimage",
    version,
    about = "Convert HTML to an image using Webkit",
    long_about = "Converts an HTML page into a PNG, JPEG, BMP, or SVG image."
)]
pub struct Cli {
    // -----------------------------------------------------------------------
    // Output format
    // -----------------------------------------------------------------------
    /// Output format: png, jpg, jpeg, bmp, svg [default: png].
    #[arg(long, short = 'f', value_name = "format",
          value_parser = ["png", "jpg", "jpeg", "bmp", "svg"])]
    pub format: Option<String>,

    // -----------------------------------------------------------------------
    // Viewport / size
    // -----------------------------------------------------------------------
    /// Set viewport width [default: 1024].
    #[arg(long, value_name = "width")]
    pub width: Option<i32>,

    /// Set viewport height (0 means auto-detect) [default: 0].
    #[arg(long, value_name = "height")]
    pub height: Option<i32>,

    // -----------------------------------------------------------------------
    // Quality / DPI
    // -----------------------------------------------------------------------
    /// Output image quality (JPEG only, 1–100) [default: 94].
    #[arg(long, value_name = "integer")]
    pub quality: Option<i32>,

    /// Change DPI explicitly [default: 96].
    #[arg(long, value_name = "dpi")]
    pub dpi: Option<i32>,

    // -----------------------------------------------------------------------
    // Background
    // -----------------------------------------------------------------------
    /// Render a transparent background (PNG only).
    #[arg(long, overrides_with = "no_transparent")]
    pub transparent: bool,

    /// Do not render a transparent background [default].
    #[arg(long = "no-transparent", overrides_with = "transparent")]
    pub no_transparent: bool,

    // -----------------------------------------------------------------------
    // Crop settings
    // -----------------------------------------------------------------------
    /// Left edge of the crop rectangle.
    #[arg(long, value_name = "x")]
    pub crop_x: Option<i32>,

    /// Top edge of the crop rectangle.
    #[arg(long, value_name = "y")]
    pub crop_y: Option<i32>,

    /// Width of the crop rectangle (0 = full width).
    #[arg(long, value_name = "width")]
    pub crop_w: Option<i32>,

    /// Height of the crop rectangle (0 = full height).
    #[arg(long, value_name = "height")]
    pub crop_h: Option<i32>,

    // -----------------------------------------------------------------------
    // Smart width
    // -----------------------------------------------------------------------
    /// Enable intelligent viewport width adjustment [default: enabled].
    #[arg(long = "enable-smart-width", overrides_with = "disable_smart_width")]
    pub enable_smart_width: bool,

    /// Disable intelligent viewport width adjustment.
    #[arg(long = "disable-smart-width", overrides_with = "enable_smart_width")]
    pub disable_smart_width: bool,

    // -----------------------------------------------------------------------
    // Load options (subset)
    // -----------------------------------------------------------------------
    /// HTTP Authentication username.
    #[arg(long, value_name = "username")]
    pub username: Option<String>,

    /// HTTP Authentication password.
    #[arg(long, value_name = "password")]
    pub password: Option<String>,

    /// Set an additional cookie (repeatable).
    #[arg(long, value_names = ["name", "value"], num_args = 2)]
    pub cookie: Vec<String>,

    /// Set an additional HTTP header (repeatable).
    #[arg(long, value_names = ["name", "value"], num_args = 2)]
    pub custom_header: Vec<String>,

    /// Use a proxy.
    #[arg(long, value_name = "proxy")]
    pub proxy: Option<String>,

    /// Wait some milliseconds for JavaScript to finish [default: 200].
    #[arg(long, value_name = "msec")]
    pub javascript_delay: Option<u32>,

    /// Do not allow web pages to run JavaScript.
    #[arg(long = "disable-javascript", overrides_with = "enable_javascript")]
    pub disable_javascript: bool,

    /// Allow web pages to run JavaScript [default: enabled].
    #[arg(long = "enable-javascript", overrides_with = "disable_javascript")]
    pub enable_javascript: bool,

    /// Set log level to: none, error, warn, or info [default: warn].
    #[arg(long, value_name = "level",
          value_parser = ["none", "error", "warn", "info"])]
    pub log_level: Option<String>,

    /// Be less verbose (sets log level to none).
    #[arg(long, short = 'q')]
    pub quiet: bool,

    // -----------------------------------------------------------------------
    // Positional: input URL/file and output path
    // -----------------------------------------------------------------------
    /// Input HTML URL or file path.
    #[arg(required = true, value_name = "URL|file")]
    pub input: String,

    /// Output image file path.
    #[arg(required = true, value_name = "output")]
    pub output: String,
}

fn main() {
    let cli = Cli::parse();

    use wkhtmltopdf_core::Converter;
    use wkhtmltopdf_image::ImageConverter;
    use wkhtmltopdf_settings::{CropSettings, ImageGlobal, LogLevel};

    let mut settings = ImageGlobal::default();

    settings.page = Some(cli.input.clone());
    settings.output = Some(cli.output.clone());

    if let Some(f) = cli.format {
        settings.format = Some(f);
    }
    if let Some(w) = cli.width {
        settings.screen_width = Some(w);
    }
    if let Some(h) = cli.height {
        settings.screen_height = Some(h);
    }
    if let Some(q) = cli.quality {
        settings.quality = q;
    }
    if let Some(d) = cli.dpi {
        settings.dpi = Some(d);
    }
    if cli.transparent {
        settings.transparent = true;
    }
    if cli.disable_smart_width {
        settings.smart_width = false;
    }

    // Crop
    let mut crop = CropSettings::default();
    if let Some(x) = cli.crop_x { crop.left = x; }
    if let Some(y) = cli.crop_y { crop.top = y; }
    if let Some(w) = cli.crop_w { crop.width = w; }
    if let Some(h) = cli.crop_h { crop.height = h; }
    settings.crop = crop;

    // Log level
    if cli.quiet {
        settings.log_level = LogLevel::None;
    } else if let Some(ref l) = cli.log_level {
        settings.log_level = match l.as_str() {
            "none" => LogLevel::None,
            "error" => LogLevel::Error,
            "info" => LogLevel::Info,
            _ => LogLevel::Warn,
        };
    }

    // Load settings
    if let Some(u) = cli.username { settings.load_page.username = Some(u); }
    if let Some(p) = cli.password { settings.load_page.password = Some(p); }
    if let Some(ms) = cli.javascript_delay { settings.load_page.js_delay = ms; }
    if cli.disable_javascript { settings.web.enable_javascript = false; }
    for pair in cli.cookie.chunks(2) {
        if pair.len() == 2 {
            settings.load_page.cookies.push((pair[0].clone(), pair[1].clone()));
        }
    }
    for pair in cli.custom_header.chunks(2) {
        if pair.len() == 2 {
            settings.load_page.custom_headers.push((pair[0].clone(), pair[1].clone()));
        }
    }

    let converter = ImageConverter::new(settings);
    let bytes = match converter.convert() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = std::fs::write(&cli.output, &bytes) {
        eprintln!("error writing output file '{}': {e}", cli.output);
        std::process::exit(1);
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
    fn minimal_invocation() {
        let cli = parse(&["wkhtmltoimage", "input.html", "output.png"]);
        assert_eq!(cli.input, "input.html");
        assert_eq!(cli.output, "output.png");
    }

    #[test]
    fn format_option() {
        let cli = parse(&["wkhtmltoimage", "--format", "jpg", "in.html", "out.jpg"]);
        assert_eq!(cli.format.as_deref(), Some("jpg"));
    }

    #[test]
    fn width_height() {
        let cli = parse(&["wkhtmltoimage", "--width", "800", "--height", "600", "in.html", "out.png"]);
        assert_eq!(cli.width, Some(800));
        assert_eq!(cli.height, Some(600));
    }

    #[test]
    fn quality_dpi() {
        let cli = parse(&["wkhtmltoimage", "--quality", "80", "--dpi", "150", "in.html", "out.jpg"]);
        assert_eq!(cli.quality, Some(80));
        assert_eq!(cli.dpi, Some(150));
    }

    #[test]
    fn transparent_flag() {
        let cli = parse(&["wkhtmltoimage", "--transparent", "in.html", "out.png"]);
        assert!(cli.transparent);
    }

    #[test]
    fn crop_options() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--crop-x", "10", "--crop-y", "20",
            "--crop-w", "100", "--crop-h", "200",
            "in.html", "out.png",
        ]);
        assert_eq!(cli.crop_x, Some(10));
        assert_eq!(cli.crop_y, Some(20));
        assert_eq!(cli.crop_w, Some(100));
        assert_eq!(cli.crop_h, Some(200));
    }

    #[test]
    fn disable_smart_width() {
        let cli = parse(&["wkhtmltoimage", "--disable-smart-width", "in.html", "out.png"]);
        assert!(cli.disable_smart_width);
    }

    #[test]
    fn invalid_format_fails() {
        assert!(Cli::try_parse_from(["wkhtmltoimage", "--format", "gif", "in.html", "out.gif"]).is_err());
    }

    #[test]
    fn missing_output_fails() {
        assert!(Cli::try_parse_from(["wkhtmltoimage", "input.html"]).is_err());
    }
}
