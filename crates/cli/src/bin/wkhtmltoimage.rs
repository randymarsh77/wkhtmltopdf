use clap::Parser;

/// Convert an HTML page into an image.
///
/// Usage: wkhtmltoimage \[OPTIONS\] \<URL|file\> \<output\>
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

    /// Do not render a transparent background \[default\].
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

    /// Verify the SSL peer certificate [default: enabled].
    #[arg(long = "ssl-verify-peer", overrides_with = "no_ssl_verify_peer")]
    pub ssl_verify_peer: bool,

    /// Do not verify the SSL peer certificate (insecure).
    #[arg(long = "no-ssl-verify-peer", overrides_with = "ssl_verify_peer")]
    pub no_ssl_verify_peer: bool,

    /// Verify the SSL certificate hostname [default: enabled].
    #[arg(long = "ssl-verify-host", overrides_with = "no_ssl_verify_host")]
    pub ssl_verify_host: bool,

    /// Do not verify the SSL certificate hostname (insecure).
    #[arg(long = "no-ssl-verify-host", overrides_with = "ssl_verify_host")]
    pub no_ssl_verify_host: bool,

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

    let mut settings = ImageGlobal {
        page: Some(cli.input.clone()),
        output: Some(cli.output.clone()),
        ..Default::default()
    };

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
    if let Some(x) = cli.crop_x {
        crop.left = x;
    }
    if let Some(y) = cli.crop_y {
        crop.top = y;
    }
    if let Some(w) = cli.crop_w {
        crop.width = w;
    }
    if let Some(h) = cli.crop_h {
        crop.height = h;
    }
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
    if let Some(u) = cli.username {
        settings.load_page.username = Some(u);
    }
    if let Some(p) = cli.password {
        settings.load_page.password = Some(p);
    }
    if let Some(ms) = cli.javascript_delay {
        settings.load_page.js_delay = ms;
    }
    if cli.disable_javascript {
        settings.web.enable_javascript = false;
    }
    for pair in cli.cookie.chunks(2) {
        if pair.len() == 2 {
            settings
                .load_page
                .cookies
                .push((pair[0].clone(), pair[1].clone()));
        }
    }
    for pair in cli.custom_header.chunks(2) {
        if pair.len() == 2 {
            settings
                .load_page
                .custom_headers
                .push((pair[0].clone(), pair[1].clone()));
        }
    }
    if let Some(ref proxy_str) = cli.proxy {
        settings.load_page.proxy = parse_proxy_url(proxy_str);
    }
    if cli.no_ssl_verify_peer {
        settings.load_page.ssl_verify_peer = false;
    }
    if cli.no_ssl_verify_host {
        settings.load_page.ssl_verify_host = false;
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

/// Parse a proxy URL string (e.g., `"http://user:pass@host:8080"` or
/// `"socks5://host:1080"`) into a [`wkhtmltopdf_settings::Proxy`] settings struct.
fn parse_proxy_url(url: &str) -> wkhtmltopdf_settings::Proxy {
    use wkhtmltopdf_settings::{Proxy, ProxyType};
    let (proxy_type, rest) = if let Some(r) = url.strip_prefix("socks5://") {
        (ProxyType::Socks5, r)
    } else if let Some(r) = url.strip_prefix("https://") {
        (ProxyType::Http, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (ProxyType::Http, r)
    } else {
        return Proxy::default();
    };

    let (auth_str, host_str) = if let Some(at_pos) = rest.rfind('@') {
        (&rest[..at_pos], &rest[at_pos + 1..])
    } else {
        ("", rest)
    };

    let (username, password) = if !auth_str.is_empty() {
        if let Some(colon) = auth_str.find(':') {
            (
                Some(auth_str[..colon].to_string()),
                Some(auth_str[colon + 1..].to_string()),
            )
        } else {
            (Some(auth_str.to_string()), None)
        }
    } else {
        (None, None)
    };

    // Split host and port.  IPv6 addresses are enclosed in brackets
    // (e.g., "[::1]:8080"), so we strip the brackets first.
    let (host, port) = if host_str.starts_with('[') {
        if let Some(close) = host_str.find(']') {
            let ipv6_host = &host_str[1..close];
            let port = host_str[close + 1..]
                .strip_prefix(':')
                .and_then(|p| p.parse::<u16>().ok());
            (Some(ipv6_host.to_string()), port)
        } else {
            (Some(host_str.to_string()), None)
        }
    } else if let Some(colon) = host_str.rfind(':') {
        let port = host_str[colon + 1..].parse::<u16>().ok();
        (Some(host_str[..colon].to_string()), port)
    } else {
        (Some(host_str.to_string()), None)
    };

    Proxy {
        proxy_type,
        host,
        port,
        username,
        password,
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
        let cli = parse(&[
            "wkhtmltoimage",
            "--width",
            "800",
            "--height",
            "600",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.width, Some(800));
        assert_eq!(cli.height, Some(600));
    }

    #[test]
    fn quality_dpi() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--quality",
            "80",
            "--dpi",
            "150",
            "in.html",
            "out.jpg",
        ]);
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
            "--crop-x",
            "10",
            "--crop-y",
            "20",
            "--crop-w",
            "100",
            "--crop-h",
            "200",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.crop_x, Some(10));
        assert_eq!(cli.crop_y, Some(20));
        assert_eq!(cli.crop_w, Some(100));
        assert_eq!(cli.crop_h, Some(200));
    }

    #[test]
    fn disable_smart_width() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--disable-smart-width",
            "in.html",
            "out.png",
        ]);
        assert!(cli.disable_smart_width);
    }

    #[test]
    fn invalid_format_fails() {
        assert!(
            Cli::try_parse_from(["wkhtmltoimage", "--format", "gif", "in.html", "out.gif"])
                .is_err()
        );
    }

    #[test]
    fn missing_output_fails() {
        assert!(Cli::try_parse_from(["wkhtmltoimage", "input.html"]).is_err());
    }

    #[test]
    fn no_ssl_verify_peer_flag() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--no-ssl-verify-peer",
            "in.html",
            "out.png",
        ]);
        assert!(cli.no_ssl_verify_peer);
    }

    #[test]
    fn no_ssl_verify_host_flag() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--no-ssl-verify-host",
            "in.html",
            "out.png",
        ]);
        assert!(cli.no_ssl_verify_host);
    }

    #[test]
    fn proxy_flag() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--proxy",
            "http://proxy:3128",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.proxy.as_deref(), Some("http://proxy:3128"));
        let proxy = parse_proxy_url(cli.proxy.as_deref().unwrap());
        use wkhtmltopdf_settings::ProxyType;
        assert!(matches!(proxy.proxy_type, ProxyType::Http));
        assert_eq!(proxy.host.as_deref(), Some("proxy"));
        assert_eq!(proxy.port, Some(3128));
    }

    // -----------------------------------------------------------------------
    // Log level flags
    // -----------------------------------------------------------------------

    #[test]
    fn quiet_flag() {
        let cli = parse(&["wkhtmltoimage", "-q", "in.html", "out.png"]);
        assert!(cli.quiet);
    }

    #[test]
    fn log_level_option() {
        let cli = parse(&["wkhtmltoimage", "--log-level", "info", "in.html", "out.png"]);
        assert_eq!(cli.log_level.as_deref(), Some("info"));
    }

    #[test]
    fn invalid_log_level_fails() {
        assert!(Cli::try_parse_from([
            "wkhtmltoimage",
            "--log-level",
            "verbose",
            "in.html",
            "out.png"
        ])
        .is_err());
    }

    // -----------------------------------------------------------------------
    // JavaScript flags
    // -----------------------------------------------------------------------

    #[test]
    fn disable_javascript_flag() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--disable-javascript",
            "in.html",
            "out.png",
        ]);
        assert!(cli.disable_javascript);
    }

    #[test]
    fn javascript_delay_option() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--javascript-delay",
            "500",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.javascript_delay, Some(500));
    }

    // -----------------------------------------------------------------------
    // Authentication options
    // -----------------------------------------------------------------------

    #[test]
    fn username_and_password() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--username",
            "admin",
            "--password",
            "secret",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.username.as_deref(), Some("admin"));
        assert_eq!(cli.password.as_deref(), Some("secret"));
    }

    // -----------------------------------------------------------------------
    // Cookie and custom header options
    // -----------------------------------------------------------------------

    #[test]
    fn cookie_option() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--cookie",
            "session",
            "abc123",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.cookie, vec!["session", "abc123"]);
    }

    #[test]
    fn custom_header_option() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--custom-header",
            "X-Auth",
            "token",
            "in.html",
            "out.png",
        ]);
        assert_eq!(cli.custom_header, vec!["X-Auth", "token"]);
    }

    // -----------------------------------------------------------------------
    // All supported format values
    // -----------------------------------------------------------------------

    #[test]
    fn all_valid_formats() {
        for fmt in &["png", "jpg", "jpeg", "bmp", "svg"] {
            let cli = parse(&["wkhtmltoimage", "--format", fmt, "in.html", "out"]);
            assert_eq!(cli.format.as_deref(), Some(*fmt));
        }
    }

    // -----------------------------------------------------------------------
    // Smart width toggle
    // -----------------------------------------------------------------------

    #[test]
    fn enable_smart_width_flag() {
        let cli = parse(&[
            "wkhtmltoimage",
            "--enable-smart-width",
            "in.html",
            "out.png",
        ]);
        assert!(cli.enable_smart_width);
        assert!(!cli.disable_smart_width);
    }

    #[test]
    fn smart_width_last_flag_wins() {
        // --enable-smart-width followed by --disable-smart-width: disable wins
        let cli = parse(&[
            "wkhtmltoimage",
            "--enable-smart-width",
            "--disable-smart-width",
            "in.html",
            "out.png",
        ]);
        assert!(cli.disable_smart_width);
        assert!(!cli.enable_smart_width);
    }

    // -----------------------------------------------------------------------
    // Proxy URL parsing helper (shared copy in wkhtmltoimage)
    // -----------------------------------------------------------------------

    #[test]
    fn parse_proxy_url_socks5() {
        use wkhtmltopdf_settings::ProxyType;
        let proxy = parse_proxy_url("socks5://myproxy:1080");
        assert!(matches!(proxy.proxy_type, ProxyType::Socks5));
        assert_eq!(proxy.host.as_deref(), Some("myproxy"));
        assert_eq!(proxy.port, Some(1080));
        assert!(proxy.username.is_none());
    }

    #[test]
    fn parse_proxy_url_with_auth() {
        use wkhtmltopdf_settings::ProxyType;
        let proxy = parse_proxy_url("http://user:pass@proxy.example.com:8080");
        assert!(matches!(proxy.proxy_type, ProxyType::Http));
        assert_eq!(proxy.host.as_deref(), Some("proxy.example.com"));
        assert_eq!(proxy.port, Some(8080));
        assert_eq!(proxy.username.as_deref(), Some("user"));
        assert_eq!(proxy.password.as_deref(), Some("pass"));
    }

    #[test]
    fn parse_proxy_url_no_port() {
        use wkhtmltopdf_settings::ProxyType;
        let proxy = parse_proxy_url("http://proxy.example.com");
        assert!(matches!(proxy.proxy_type, ProxyType::Http));
        assert_eq!(proxy.host.as_deref(), Some("proxy.example.com"));
        assert!(proxy.port.is_none());
    }

    #[test]
    fn parse_proxy_url_no_scheme_returns_default() {
        use wkhtmltopdf_settings::ProxyType;
        let proxy = parse_proxy_url("notaproxy");
        assert!(matches!(proxy.proxy_type, ProxyType::None));
    }
}
