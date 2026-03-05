use wkhtmltopdf_core::Converter;
use wkhtmltopdf_diff::{diff_images, DiffError, DiffOptions};
use wkhtmltopdf_image::ImageConverter;
use wkhtmltopdf_pdf::{
    build_band_html, extract_headings, generate_toc_html, inject_header_footer,
    inject_heading_anchors, substitute_vars, PdfConverter,
};
use wkhtmltopdf_settings::{
    ColorMode, CropSettings, HeaderFooter, ImageGlobal, LoadErrorHandling, LoadGlobal, LoadPage,
    Margin, Orientation, PageSize, PdfAConformance, PdfGlobal, PdfObject, PrintResolution,
    ProxyType, Size, TableOfContent, Unit, UnitReal, Web,
};

#[test]
fn pdf_converter_returns_error_when_not_implemented() {
    let converter = PdfConverter::new(PdfGlobal::default());
    let result = converter.convert();
    assert!(result.is_err(), "expected an error from the stub converter");
}

#[test]
fn image_converter_returns_error_when_not_implemented() {
    let converter = ImageConverter::new(ImageGlobal::default());
    let result = converter.convert();
    assert!(result.is_err(), "expected an error from the stub converter");
}

#[test]
fn pdf_global_default_has_no_output() {
    let settings = PdfGlobal::default();
    assert!(settings.output.is_none());
}

#[test]
fn image_global_default_has_no_output() {
    let settings = ImageGlobal::default();
    assert!(settings.output.is_none());
}

// ---------------------------------------------------------------------------
// Settings struct defaults
// ---------------------------------------------------------------------------

#[test]
fn pdf_global_default_orientation_is_portrait() {
    let settings = PdfGlobal::default();
    assert!(matches!(settings.orientation, Orientation::Portrait));
}

#[test]
fn pdf_global_default_color_mode_is_color() {
    let settings = PdfGlobal::default();
    assert!(matches!(settings.color_mode, ColorMode::Color));
}

#[test]
fn pdf_global_default_outline_enabled() {
    let settings = PdfGlobal::default();
    assert!(settings.outline);
    assert_eq!(settings.outline_depth, 4);
}

#[test]
fn pdf_global_default_copies_is_one() {
    let settings = PdfGlobal::default();
    assert_eq!(settings.copies, 1);
}

#[test]
fn pdf_global_default_image_quality() {
    let settings = PdfGlobal::default();
    assert_eq!(settings.image_quality, 94);
    assert_eq!(settings.image_dpi, 600);
}

#[test]
fn pdf_object_default_uses_external_and_local_links() {
    let obj = PdfObject::default();
    assert!(obj.use_external_links);
    assert!(obj.use_local_links);
}

#[test]
fn pdf_object_default_include_in_outline() {
    let obj = PdfObject::default();
    assert!(obj.include_in_outline);
    assert!(obj.pages_count);
    assert!(!obj.is_table_of_content);
}

#[test]
fn header_footer_default_values() {
    let hf = HeaderFooter::default();
    assert_eq!(hf.font_size, 12);
    assert!(!hf.line);
    assert!(hf.html_url.is_none());
}

#[test]
fn table_of_content_default_values() {
    let toc = TableOfContent::default();
    assert!(toc.use_dotted_lines);
    assert!(toc.forward_links);
    assert!(toc.back_links);
    assert_eq!(toc.caption_text, "Table of Contents");
}

#[test]
fn web_default_enables_javascript_and_background() {
    let web = Web::default();
    assert!(web.background);
    assert!(web.load_images);
    assert!(web.enable_javascript);
    assert!(web.enable_intelligent_shrinking);
    assert!(!web.enable_plugins);
}

#[test]
fn load_page_default_js_delay_and_zoom() {
    let lp = LoadPage::default();
    assert_eq!(lp.js_delay, 200);
    assert!((lp.zoom - 1.0).abs() < f64::EPSILON);
    assert!(matches!(lp.load_error_handling, LoadErrorHandling::Abort));
    assert!(matches!(
        lp.media_load_error_handling,
        LoadErrorHandling::Ignore
    ));
}

#[test]
fn load_page_default_proxy_is_none() {
    let lp = LoadPage::default();
    assert!(matches!(lp.proxy.proxy_type, ProxyType::None));
    assert!(lp.proxy.host.is_none());
}

#[test]
fn crop_settings_default_is_all_zeros() {
    let crop = CropSettings::default();
    assert_eq!(crop.left, 0);
    assert_eq!(crop.top, 0);
    assert_eq!(crop.width, 0);
    assert_eq!(crop.height, 0);
}

#[test]
fn image_global_default_quality_and_smart_width() {
    let img = ImageGlobal::default();
    assert_eq!(img.quality, 94);
    assert!(img.smart_width);
    assert!(!img.transparent);
}

#[test]
fn size_default_is_a4() {
    let size = Size::default();
    assert!(matches!(size.page_size, PageSize::A4));
    assert!(size.height.is_none());
    assert!(size.width.is_none());
}

#[test]
fn unit_real_default_is_zero_millimeters() {
    let ur = UnitReal::default();
    assert!((ur.value - 0.0).abs() < f64::EPSILON);
    assert!(matches!(ur.unit, Unit::Millimeter));
}

#[test]
fn margin_default_is_all_zero() {
    let m = Margin::default();
    assert!((m.top.value - 0.0).abs() < f64::EPSILON);
    assert!((m.right.value - 0.0).abs() < f64::EPSILON);
    assert!((m.bottom.value - 0.0).abs() < f64::EPSILON);
    assert!((m.left.value - 0.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// Serde round-trip tests (JSON)
// ---------------------------------------------------------------------------

#[test]
fn pdf_global_serializes_and_deserializes() {
    let mut settings = PdfGlobal::default();
    settings.output = Some("output.pdf".into());
    settings.document_title = Some("Test Title".into());
    settings.dpi = Some(300);

    let json = serde_json::to_string(&settings).expect("serialize");
    let restored: PdfGlobal = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.output.as_deref(), Some("output.pdf"));
    assert_eq!(restored.document_title.as_deref(), Some("Test Title"));
    assert_eq!(restored.dpi, Some(300));
}

#[test]
fn pdf_object_serializes_and_deserializes() {
    let mut obj = PdfObject::default();
    obj.page = Some("https://example.com".into());
    obj.produce_forms = true;

    let json = serde_json::to_string(&obj).expect("serialize");
    let restored: PdfObject = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.page.as_deref(), Some("https://example.com"));
    assert!(restored.produce_forms);
}

#[test]
fn image_global_serializes_and_deserializes() {
    let mut settings = ImageGlobal::default();
    settings.output = Some("out.png".into());
    settings.format = Some("png".into());
    settings.quality = 80;

    let json = serde_json::to_string(&settings).expect("serialize");
    let restored: ImageGlobal = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.output.as_deref(), Some("out.png"));
    assert_eq!(restored.format.as_deref(), Some("png"));
    assert_eq!(restored.quality, 80);
}

#[test]
fn load_page_serializes_and_deserializes() {
    let mut lp = LoadPage::default();
    lp.username = Some("user".into());
    lp.cookies = vec![("session".into(), "abc123".into())];
    lp.print_media_type = true;

    let json = serde_json::to_string(&lp).expect("serialize");
    let restored: LoadPage = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.username.as_deref(), Some("user"));
    assert_eq!(restored.cookies.len(), 1);
    assert_eq!(restored.cookies[0].0, "session");
    assert!(restored.print_media_type);
}

#[test]
fn table_of_content_serializes_and_deserializes() {
    let mut toc = TableOfContent::default();
    toc.caption_text = "Contents".into();
    toc.use_dotted_lines = false;

    let json = serde_json::to_string(&toc).expect("serialize");
    let restored: TableOfContent = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.caption_text, "Contents");
    assert!(!restored.use_dotted_lines);
}

// ---------------------------------------------------------------------------
// New PdfGlobal metadata / PDF/A conformance fields
// ---------------------------------------------------------------------------

#[test]
fn pdf_global_default_author_and_subject_are_none() {
    let settings = PdfGlobal::default();
    assert!(settings.author.is_none());
    assert!(settings.subject.is_none());
}

#[test]
fn pdf_global_default_pdf_a_conformance_is_none() {
    let settings = PdfGlobal::default();
    assert!(matches!(settings.pdf_a_conformance, PdfAConformance::None));
}

#[test]
fn pdf_global_metadata_serializes_and_deserializes() {
    let mut settings = PdfGlobal::default();
    settings.document_title = Some("My Document".into());
    settings.author = Some("Jane Doe".into());
    settings.subject = Some("Testing".into());
    settings.pdf_a_conformance = PdfAConformance::A2b;

    let json = serde_json::to_string(&settings).expect("serialize");
    let restored: PdfGlobal = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.document_title.as_deref(), Some("My Document"));
    assert_eq!(restored.author.as_deref(), Some("Jane Doe"));
    assert_eq!(restored.subject.as_deref(), Some("Testing"));
    assert!(matches!(restored.pdf_a_conformance, PdfAConformance::A2b));
}

#[test]
fn pdf_converter_no_objects_returns_error() {
    let converter = PdfConverter::new(PdfGlobal::default());
    let result = converter.convert();
    assert!(result.is_err());
}

#[test]
fn pdf_converter_object_without_page_returns_error() {
    let mut converter = PdfConverter::new(PdfGlobal::default());
    converter.add_object(PdfObject::default()); // page is None
    let result = converter.convert();
    assert!(result.is_err());
}

#[test]
fn pdf_converter_produces_pdf_from_local_html() {
    use std::io::Write;

    // Write a minimal HTML file to a temp path.
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><p>Hello PDF</p></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let mut global = PdfGlobal::default();
    global.document_title = Some("Test PDF".into());
    global.author = Some("Test Author".into());
    global.subject = Some("Test Subject".into());

    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());

    let bytes = result.unwrap();
    // A valid PDF starts with "%PDF-".
    assert!(bytes.starts_with(b"%PDF-"), "output is not a valid PDF");
}

#[test]
fn pdf_converter_multi_page_produces_pdf() {
    use std::io::Write;

    let mut tmp1 = tempfile::NamedTempFile::new().expect("temp file 1");
    write!(tmp1, "<html><body><p>Page 1</p></body></html>").expect("write");
    let path1 = tmp1.path().to_string_lossy().to_string();

    let mut tmp2 = tempfile::NamedTempFile::new().expect("temp file 2");
    write!(tmp2, "<html><body><p>Page 2</p></body></html>").expect("write");
    let path2 = tmp2.path().to_string_lossy().to_string();

    let mut converter = PdfConverter::new(PdfGlobal::default());
    let mut obj1 = PdfObject::default();
    obj1.page = Some(path1);
    converter.add_object(obj1);
    let mut obj2 = PdfObject::default();
    obj2.page = Some(path2);
    converter.add_object(obj2);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_landscape_orientation() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body>Landscape</body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let mut global = PdfGlobal::default();
    global.orientation = Orientation::Landscape;

    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_pdf_a_conformance() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body>PDF/A</body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let mut global = PdfGlobal::default();
    global.pdf_a_conformance = PdfAConformance::A2b;
    global.document_title = Some("Archived Doc".into());

    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

// ---------------------------------------------------------------------------
// Network / proxy settings
// ---------------------------------------------------------------------------

#[test]
fn load_page_default_ssl_verify_enabled() {
    let lp = LoadPage::default();
    assert!(lp.ssl_verify_peer, "ssl_verify_peer should default to true");
    assert!(lp.ssl_verify_host, "ssl_verify_host should default to true");
}

#[test]
fn load_page_ssl_verify_can_be_disabled() {
    let mut lp = LoadPage::default();
    lp.ssl_verify_peer = false;
    lp.ssl_verify_host = false;
    assert!(!lp.ssl_verify_peer);
    assert!(!lp.ssl_verify_host);
}

#[test]
fn load_page_ssl_verify_roundtrips_via_json() {
    let mut lp = LoadPage::default();
    lp.ssl_verify_peer = false;

    let json = serde_json::to_string(&lp).expect("serialize");
    let restored: LoadPage = serde_json::from_str(&json).expect("deserialize");
    assert!(!restored.ssl_verify_peer);
    assert!(restored.ssl_verify_host);
}

// ---------------------------------------------------------------------------
// Additional settings struct tests
// ---------------------------------------------------------------------------

#[test]
fn print_resolution_default_is_screen() {
    let settings = PdfGlobal::default();
    assert!(matches!(
        settings.resolution,
        PrintResolution::ScreenResolution
    ));
}

#[test]
fn pdf_global_default_collate_is_true() {
    let settings = PdfGlobal::default();
    assert!(settings.collate);
}

#[test]
fn pdf_global_default_use_compression_is_true() {
    let settings = PdfGlobal::default();
    assert!(settings.use_compression);
}

#[test]
fn pdf_global_default_page_offset_is_zero() {
    let settings = PdfGlobal::default();
    assert_eq!(settings.page_offset, 0);
}

#[test]
fn pdf_global_default_resolve_relative_links() {
    let settings = PdfGlobal::default();
    assert!(settings.resolve_relative_links);
}

#[test]
fn load_global_default_cookie_jar_is_none() {
    let lg = LoadGlobal::default();
    assert!(lg.cookie_jar.is_none());
}

#[test]
fn image_global_default_screen_height_is_none() {
    let img = ImageGlobal::default();
    assert!(img.screen_height.is_none());
    assert!(img.screen_width.is_none());
}

#[test]
fn image_global_default_format_is_none() {
    let img = ImageGlobal::default();
    assert!(img.format.is_none());
}

#[test]
fn image_global_default_page_is_none() {
    let img = ImageGlobal::default();
    assert!(img.page.is_none());
}

#[test]
fn pdf_object_default_is_not_toc() {
    let obj = PdfObject::default();
    assert!(!obj.is_table_of_content);
    assert!(obj.toc_xsl.is_none());
}

#[test]
fn pdf_object_default_replacements_empty() {
    let obj = PdfObject::default();
    assert!(obj.replacements.is_empty());
}

#[test]
fn header_footer_serializes_and_deserializes() {
    let mut hf = HeaderFooter::default();
    hf.left = Some("Left Text".into());
    hf.center = Some("Center".into());
    hf.right = Some("[page]".into());
    hf.line = true;
    hf.spacing = 2.5;
    hf.font_size = 10;
    hf.font_name = "Times New Roman".into();

    let json = serde_json::to_string(&hf).expect("serialize");
    let restored: HeaderFooter = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.left.as_deref(), Some("Left Text"));
    assert_eq!(restored.center.as_deref(), Some("Center"));
    assert_eq!(restored.right.as_deref(), Some("[page]"));
    assert!(restored.line);
    assert!((restored.spacing - 2.5).abs() < f32::EPSILON);
    assert_eq!(restored.font_size, 10);
    assert_eq!(restored.font_name, "Times New Roman");
}

#[test]
fn size_serializes_and_deserializes_with_custom_page() {
    let mut size = Size::default();
    size.page_size = PageSize::Letter;
    size.height = Some(UnitReal {
        value: 279.4,
        unit: Unit::Millimeter,
    });
    size.width = Some(UnitReal {
        value: 215.9,
        unit: Unit::Millimeter,
    });

    let json = serde_json::to_string(&size).expect("serialize");
    let restored: Size = serde_json::from_str(&json).expect("deserialize");

    assert!(matches!(restored.page_size, PageSize::Letter));
    assert!((restored.height.unwrap().value - 279.4).abs() < f64::EPSILON);
    assert!((restored.width.unwrap().value - 215.9).abs() < f64::EPSILON);
}

#[test]
fn margin_with_mixed_units_serializes_and_deserializes() {
    let margin = Margin {
        top: UnitReal {
            value: 10.0,
            unit: Unit::Millimeter,
        },
        right: UnitReal {
            value: 1.0,
            unit: Unit::Inch,
        },
        bottom: UnitReal {
            value: 1.5,
            unit: Unit::Centimeter,
        },
        left: UnitReal {
            value: 72.0,
            unit: Unit::Point,
        },
    };

    let json = serde_json::to_string(&margin).expect("serialize");
    let restored: Margin = serde_json::from_str(&json).expect("deserialize");

    assert!((restored.top.value - 10.0).abs() < f64::EPSILON);
    assert!(matches!(restored.top.unit, Unit::Millimeter));
    assert!((restored.right.value - 1.0).abs() < f64::EPSILON);
    assert!(matches!(restored.right.unit, Unit::Inch));
    assert!((restored.bottom.value - 1.5).abs() < f64::EPSILON);
    assert!(matches!(restored.bottom.unit, Unit::Centimeter));
    assert!((restored.left.value - 72.0).abs() < f64::EPSILON);
    assert!(matches!(restored.left.unit, Unit::Point));
}

#[test]
fn toc_depth_setting_serializes_correctly() {
    let mut toc = TableOfContent::default();
    toc.depth = 6;
    toc.font_scale = 0.7;
    toc.indentation = "2em".into();

    let json = serde_json::to_string(&toc).expect("serialize");
    let restored: TableOfContent = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.depth, 6);
    assert!((restored.font_scale - 0.7).abs() < f32::EPSILON);
    assert_eq!(restored.indentation, "2em");
}

#[test]
fn web_settings_serializes_and_deserializes() {
    let mut web = Web::default();
    web.enable_javascript = false;
    web.background = false;
    web.load_images = false;
    web.minimum_font_size = Some(14);
    web.user_style_sheet = Some("/path/to/custom.css".into());
    web.default_encoding = Some("UTF-8".into());

    let json = serde_json::to_string(&web).expect("serialize");
    let restored: Web = serde_json::from_str(&json).expect("deserialize");

    assert!(!restored.enable_javascript);
    assert!(!restored.background);
    assert!(!restored.load_images);
    assert_eq!(restored.minimum_font_size, Some(14));
    assert_eq!(
        restored.user_style_sheet.as_deref(),
        Some("/path/to/custom.css")
    );
    assert_eq!(restored.default_encoding.as_deref(), Some("UTF-8"));
}

#[test]
fn image_global_with_jpeg_format_and_quality_serializes() {
    let mut img = ImageGlobal::default();
    img.format = Some("jpg".into());
    img.quality = 75;
    img.dpi = Some(150);
    img.transparent = false;
    img.smart_width = false;
    img.screen_width = Some(1280);
    img.screen_height = Some(1024);

    let json = serde_json::to_string(&img).expect("serialize");
    let restored: ImageGlobal = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.format.as_deref(), Some("jpg"));
    assert_eq!(restored.quality, 75);
    assert_eq!(restored.dpi, Some(150));
    assert!(!restored.transparent);
    assert!(!restored.smart_width);
    assert_eq!(restored.screen_width, Some(1280));
    assert_eq!(restored.screen_height, Some(1024));
}

#[test]
fn image_global_with_crop_settings_serializes() {
    let mut img = ImageGlobal::default();
    img.crop = CropSettings {
        left: 10,
        top: 20,
        width: 640,
        height: 480,
    };

    let json = serde_json::to_string(&img).expect("serialize");
    let restored: ImageGlobal = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.crop.left, 10);
    assert_eq!(restored.crop.top, 20);
    assert_eq!(restored.crop.width, 640);
    assert_eq!(restored.crop.height, 480);
}

#[test]
fn image_global_with_web_settings_serializes() {
    let mut img = ImageGlobal::default();
    img.web.enable_javascript = false;
    img.web.background = false;
    img.web.minimum_font_size = Some(12);

    let json = serde_json::to_string(&img).expect("serialize");
    let restored: ImageGlobal = serde_json::from_str(&json).expect("deserialize");

    assert!(!restored.web.enable_javascript);
    assert!(!restored.web.background);
    assert_eq!(restored.web.minimum_font_size, Some(12));
}

#[test]
fn image_global_with_load_page_settings_serializes() {
    let mut img = ImageGlobal::default();
    img.load_page.print_media_type = true;
    img.load_page.zoom = 1.5;
    img.load_page.js_delay = 500;

    let json = serde_json::to_string(&img).expect("serialize");
    let restored: ImageGlobal = serde_json::from_str(&json).expect("deserialize");

    assert!(restored.load_page.print_media_type);
    assert!((restored.load_page.zoom - 1.5).abs() < f64::EPSILON);
    assert_eq!(restored.load_page.js_delay, 500);
}

#[test]
fn image_converter_no_page_with_format_returns_error() {
    let mut settings = ImageGlobal::default();
    settings.format = Some("jpg".into());
    let conv = ImageConverter::new(settings);
    assert!(conv.convert().is_err());
}

#[test]
fn image_converter_no_page_with_crop_returns_error() {
    let mut settings = ImageGlobal::default();
    settings.crop = CropSettings {
        left: 0,
        top: 0,
        width: 100,
        height: 100,
    };
    let conv = ImageConverter::new(settings);
    assert!(conv.convert().is_err());
}

#[test]
fn image_converter_no_page_with_transparent_returns_error() {
    let mut settings = ImageGlobal::default();
    settings.transparent = true;
    let conv = ImageConverter::new(settings);
    assert!(conv.convert().is_err());
}

// ---------------------------------------------------------------------------
// PDF pipeline – feature flag integration tests
// ---------------------------------------------------------------------------

/// Helper: build a minimal PDF from an inline HTML string, with custom global settings.
fn make_pdf_with_global(html: &str, global: PdfGlobal) -> Vec<u8> {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "{}", html).expect("write html");
    let path = tmp.path().to_string_lossy().to_string();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    converter.add_object(obj);
    converter.convert().expect("PDF conversion failed")
}

#[test]
fn pdf_converter_a3_page_size() {
    let mut global = PdfGlobal::default();
    global.size.page_size = PageSize::A3;
    let bytes = make_pdf_with_global("<html><body><p>A3 page</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_a5_page_size() {
    let mut global = PdfGlobal::default();
    global.size.page_size = PageSize::A5;
    let bytes = make_pdf_with_global("<html><body><p>A5 page</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_letter_page_size() {
    let mut global = PdfGlobal::default();
    global.size.page_size = PageSize::Letter;
    let bytes = make_pdf_with_global("<html><body><p>Letter page</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_legal_page_size() {
    let mut global = PdfGlobal::default();
    global.size.page_size = PageSize::Legal;
    let bytes = make_pdf_with_global("<html><body><p>Legal page</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_grayscale_color_mode() {
    let mut global = PdfGlobal::default();
    global.color_mode = ColorMode::Grayscale;
    let bytes = make_pdf_with_global("<html><body><p>Grayscale</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_custom_margins() {
    let mut global = PdfGlobal::default();
    global.margin = Margin {
        top: UnitReal {
            value: 20.0,
            unit: Unit::Millimeter,
        },
        right: UnitReal {
            value: 15.0,
            unit: Unit::Millimeter,
        },
        bottom: UnitReal {
            value: 20.0,
            unit: Unit::Millimeter,
        },
        left: UnitReal {
            value: 15.0,
            unit: Unit::Millimeter,
        },
    };
    let bytes = make_pdf_with_global("<html><body><p>Custom margins</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_custom_dpi() {
    let mut global = PdfGlobal::default();
    global.dpi = Some(150);
    let bytes = make_pdf_with_global("<html><body><p>DPI 150</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_no_compression() {
    let mut global = PdfGlobal::default();
    global.use_compression = false;
    let bytes = make_pdf_with_global("<html><body><p>No compression</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_outline_disabled() {
    let mut global = PdfGlobal::default();
    global.outline = false;
    let bytes = make_pdf_with_global("<html><body><h1>No outline</h1></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_outline_custom_depth() {
    let mut global = PdfGlobal::default();
    global.outline = true;
    global.outline_depth = 2;
    let bytes = make_pdf_with_global(
        "<html><body><h1>H1</h1><h2>H2</h2><h3>H3 hidden</h3></body></html>",
        global,
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_pdf_a_a1b_conformance() {
    let mut global = PdfGlobal::default();
    global.pdf_a_conformance = PdfAConformance::A1b;
    global.document_title = Some("PDF/A-1b Doc".into());
    let bytes = make_pdf_with_global("<html><body><p>PDF/A-1b</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_pdf_a_a3b_conformance() {
    let mut global = PdfGlobal::default();
    global.pdf_a_conformance = PdfAConformance::A3b;
    global.document_title = Some("PDF/A-3b Doc".into());
    let bytes = make_pdf_with_global("<html><body><p>PDF/A-3b</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_with_page_offset() {
    let mut global = PdfGlobal::default();
    global.page_offset = 5;
    let bytes = make_pdf_with_global("<html><body><p>Page offset 5</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_with_viewport_size() {
    let mut global = PdfGlobal::default();
    global.viewport_size = Some("1280x1024".into());
    let bytes = make_pdf_with_global("<html><body><p>Viewport</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_with_low_image_quality() {
    let mut global = PdfGlobal::default();
    global.image_quality = 50;
    global.image_dpi = 150;
    let bytes = make_pdf_with_global("<html><body><p>Low image quality</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_with_header_text() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><p>With header</p></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.header.left = Some("Left Header".into());
    obj.header.center = Some("Center Header".into());
    obj.header.right = Some("[page]".into());
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_with_footer_text_and_line() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><p>With footer</p></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.footer.center = Some("Page [page] of [toPage]".into());
    obj.footer.line = true;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_with_header_and_footer() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><p>Header and footer</p></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let mut global = PdfGlobal::default();
    global.document_title = Some("My Doc".into());

    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.header.left = Some("[title]".into());
    obj.header.right = Some("[date]".into());
    obj.footer.center = Some("[page] / [toPage]".into());
    obj.footer.line = true;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_toc_object_produces_pdf() {
    use std::io::Write;

    let mut content_tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(
        content_tmp,
        "<html><body><h1>Chapter 1</h1><p>Content 1</p><h2>Section 1.1</h2></body></html>"
    )
    .expect("write");
    let content_path = content_tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);

    // First: TOC object
    let mut toc_obj = PdfObject::default();
    toc_obj.is_table_of_content = true;
    toc_obj.page = Some("<toc>".into());
    converter.add_object(toc_obj);

    // Second: content
    let mut content_obj = PdfObject::default();
    content_obj.page = Some(content_path);
    converter.add_object(content_obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_toc_with_custom_depth() {
    use std::io::Write;

    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(
        tmp,
        "<html><body><h1>H1</h1><h2>H2</h2><h3>H3</h3></body></html>"
    )
    .expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);

    let mut toc_obj = PdfObject::default();
    toc_obj.is_table_of_content = true;
    toc_obj.toc.depth = 2;
    toc_obj.toc.caption_text = "Contents".into();
    toc_obj.toc.use_dotted_lines = false;
    toc_obj.page = Some("<toc>".into());
    converter.add_object(toc_obj);

    let mut content_obj = PdfObject::default();
    content_obj.page = Some(path);
    converter.add_object(content_obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_with_replacements() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><p>Hello PLACEHOLDER</p></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.replacements = vec![("PLACEHOLDER".into(), "World".into())];
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_include_in_outline_false() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><h1>Not in outline</h1></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.include_in_outline = false;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_produce_forms() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(
        tmp,
        "<html><body><form><input type='text' name='f'/></form></body></html>"
    )
    .expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.produce_forms = true;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_with_print_media_type() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(
        tmp,
        "<html><head><style>@media print {{ p {{ color: red; }} }}</style></head>\
         <body><p>Print media</p></body></html>"
    )
    .expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.load.print_media_type = true;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_with_zoom() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(tmp, "<html><body><p>Zoomed content</p></body></html>").expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.load.zoom = 1.5;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_disable_external_links() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(
        tmp,
        "<html><body><a href='https://example.com'>Link</a></body></html>"
    )
    .expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.use_external_links = false;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_object_disable_local_links() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().expect("temp file");
    write!(
        tmp,
        "<html><body><a href='#anchor'>Local link</a></body></html>"
    )
    .expect("write");
    let path = tmp.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);
    let mut obj = PdfObject::default();
    obj.page = Some(path);
    obj.use_local_links = false;
    converter.add_object(obj);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_multiple_copies_setting() {
    let mut global = PdfGlobal::default();
    global.copies = 3;
    global.collate = true;
    let bytes = make_pdf_with_global("<html><body><p>Multiple copies</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_full_metadata_set() {
    let mut global = PdfGlobal::default();
    global.document_title = Some("Full Metadata Test".into());
    global.author = Some("Test Author".into());
    global.subject = Some("Integration Testing".into());
    global.pdf_a_conformance = PdfAConformance::A2b;
    let bytes = make_pdf_with_global("<html><body><p>Full metadata</p></body></html>", global);
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_two_objects_one_without_outline() {
    use std::io::Write;

    let mut tmp1 = tempfile::NamedTempFile::new().expect("temp 1");
    write!(tmp1, "<html><body><h1>In outline</h1></body></html>").expect("write");
    let path1 = tmp1.path().to_string_lossy().to_string();

    let mut tmp2 = tempfile::NamedTempFile::new().expect("temp 2");
    write!(tmp2, "<html><body><h1>Not in outline</h1></body></html>").expect("write");
    let path2 = tmp2.path().to_string_lossy().to_string();

    let global = PdfGlobal::default();
    let mut converter = PdfConverter::new(global);

    let mut obj1 = PdfObject::default();
    obj1.page = Some(path1);
    obj1.include_in_outline = true;
    converter.add_object(obj1);

    let mut obj2 = PdfObject::default();
    obj2.page = Some(path2);
    obj2.include_in_outline = false;
    converter.add_object(obj2);

    let result = converter.convert();
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    assert!(result.unwrap().starts_with(b"%PDF-"));
}

#[test]
fn pdf_converter_landscape_with_margins_and_metadata() {
    let mut global = PdfGlobal::default();
    global.orientation = Orientation::Landscape;
    global.margin = Margin {
        top: UnitReal {
            value: 10.0,
            unit: Unit::Millimeter,
        },
        right: UnitReal {
            value: 10.0,
            unit: Unit::Millimeter,
        },
        bottom: UnitReal {
            value: 10.0,
            unit: Unit::Millimeter,
        },
        left: UnitReal {
            value: 10.0,
            unit: Unit::Millimeter,
        },
    };
    global.document_title = Some("Landscape with margins".into());
    let bytes = make_pdf_with_global(
        "<html><body><p>Landscape with margins and title</p></body></html>",
        global,
    );
    assert!(bytes.starts_with(b"%PDF-"));
}

// ---------------------------------------------------------------------------
// PDF utility function tests (substitute_vars, build_band_html, inject_header_footer,
// extract_headings, inject_heading_anchors, generate_toc_html)
// ---------------------------------------------------------------------------

#[test]
fn substitute_vars_replaces_page_with_css_counter_span() {
    let result = substitute_vars("[page]", "2024-01-01", "My Title", "http://example.com");
    assert!(
        result.contains("_wk_page"),
        "should contain CSS counter span for [page]"
    );
}

#[test]
fn substitute_vars_replaces_topage_with_css_counter_span() {
    let result = substitute_vars("[toPage]", "2024-01-01", "Title", "url");
    assert!(
        result.contains("_wk_topage"),
        "should contain CSS counter span for [toPage]"
    );
}

#[test]
fn substitute_vars_replaces_date() {
    let result = substitute_vars("[date]", "2024-06-15", "Title", "url");
    assert_eq!(result, "2024-06-15");
}

#[test]
fn substitute_vars_replaces_title() {
    let result = substitute_vars("[title]", "2024-01-01", "My PDF Title", "url");
    assert_eq!(result, "My PDF Title");
}

#[test]
fn substitute_vars_replaces_url() {
    let result = substitute_vars("[url]", "2024-01-01", "title", "https://example.com/page");
    assert_eq!(result, "https://example.com/page");
}

#[test]
fn substitute_vars_multiple_replacements_in_one_string() {
    let result = substitute_vars(
        "Title: [title] | Date: [date] | Page [page]",
        "2025-01-01",
        "Doc",
        "url",
    );
    assert!(result.contains("Doc"));
    assert!(result.contains("2025-01-01"));
    assert!(result.contains("_wk_page"));
}

#[test]
fn build_band_html_empty_when_no_content() {
    let hf = HeaderFooter::default();
    let result = build_band_html(
        &hf,
        true,
        "2024-01-01",
        "Title",
        "url",
        &LoadPage::default(),
    );
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty(), "no content → empty string");
}

#[test]
fn build_band_html_non_empty_with_left_text() {
    let mut hf = HeaderFooter::default();
    hf.left = Some("Header Left".into());
    let result = build_band_html(
        &hf,
        true,
        "2024-01-01",
        "Title",
        "url",
        &LoadPage::default(),
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(!html.is_empty());
    assert!(html.contains("Header Left"));
}

#[test]
fn build_band_html_contains_font_settings() {
    let mut hf = HeaderFooter::default();
    hf.center = Some("Centered".into());
    hf.font_name = "Courier".into();
    hf.font_size = 9;
    let result = build_band_html(
        &hf,
        false,
        "2024-01-01",
        "Title",
        "url",
        &LoadPage::default(),
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Courier"));
    assert!(html.contains("9pt"));
}

#[test]
fn build_band_html_contains_line_separator_for_header() {
    let mut hf = HeaderFooter::default();
    hf.line = true;
    let result = build_band_html(
        &hf,
        true,
        "2024-01-01",
        "Title",
        "url",
        &LoadPage::default(),
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("border-bottom"));
}

#[test]
fn build_band_html_contains_line_separator_for_footer() {
    let mut hf = HeaderFooter::default();
    hf.line = true;
    let result = build_band_html(
        &hf,
        false,
        "2024-01-01",
        "Title",
        "url",
        &LoadPage::default(),
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("border-top"));
}

#[test]
fn build_band_html_replaces_title_and_url() {
    let mut hf = HeaderFooter::default();
    hf.left = Some("[title]".into());
    hf.right = Some("[url]".into());
    let result = build_band_html(
        &hf,
        true,
        "2024-01-01",
        "My Document",
        "https://example.com",
        &LoadPage::default(),
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("My Document"));
    assert!(html.contains("https://example.com"));
}

#[test]
fn inject_header_footer_noop_when_both_empty() {
    let html = "<html><body><p>Content</p></body></html>";
    let result = inject_header_footer(html, "", "", 0.0, 0.0);
    assert_eq!(
        result, html,
        "should return the input unchanged when both bands are empty"
    );
}

#[test]
fn inject_header_footer_injects_header_after_body_tag() {
    let html = "<html><body><p>Content</p></body></html>";
    let result = inject_header_footer(html, "<div>HEADER</div>", "", 0.0, 0.0);
    // The header should appear inside the <body>.
    let body_pos = result.find("<body>").expect("<body> tag");
    let header_pos = result.find("HEADER").expect("HEADER in output");
    assert!(
        header_pos > body_pos,
        "header must appear after the opening <body> tag"
    );
}

#[test]
fn inject_header_footer_injects_footer() {
    let html = "<html><body><p>Content</p></body></html>";
    let result = inject_header_footer(html, "", "<div>FOOTER</div>", 0.0, 0.0);
    assert!(
        result.contains("FOOTER"),
        "footer should appear in the output"
    );
}

#[test]
fn inject_header_footer_adds_body_margin_style() {
    let html = "<html><body><p>Content</p></body></html>";
    let result = inject_header_footer(html, "<div>HEADER</div>", "<div>FOOTER</div>", 5.0, 5.0);
    assert!(
        result.contains("margin-top"),
        "body top margin should be injected"
    );
    assert!(
        result.contains("margin-bottom"),
        "body bottom margin should be injected"
    );
}

#[test]
fn extract_headings_returns_empty_for_no_headings() {
    let html = "<html><body><p>No headings here</p></body></html>";
    let headings = extract_headings(html, 6);
    assert!(headings.is_empty());
}

#[test]
fn extract_headings_finds_h1_through_h3() {
    let html = "<html><body><h1>Title</h1><h2>Section</h2><h3>Sub</h3></body></html>";
    let headings = extract_headings(html, 3);
    assert_eq!(headings.len(), 3);
    assert_eq!(headings[0].level, 1);
    assert_eq!(headings[0].text, "Title");
    assert_eq!(headings[1].level, 2);
    assert_eq!(headings[2].level, 3);
}

#[test]
fn extract_headings_respects_max_depth() {
    let html = "<html><body><h1>H1</h1><h2>H2</h2><h3>H3</h3></body></html>";
    // With max_depth=2, h3 should not be included.
    let headings = extract_headings(html, 2);
    assert_eq!(headings.len(), 2);
    assert!(headings.iter().all(|h| h.level <= 2));
}

#[test]
fn extract_headings_preserves_existing_id_as_anchor() {
    let html = "<html><body><h1 id=\"my-anchor\">Title</h1></body></html>";
    let headings = extract_headings(html, 1);
    assert_eq!(headings.len(), 1);
    assert_eq!(headings[0].anchor, "my-anchor");
}

#[test]
fn extract_headings_generates_anchor_from_text() {
    let html = "<html><body><h1>Hello World</h1></body></html>";
    let headings = extract_headings(html, 1);
    assert_eq!(headings.len(), 1);
    // The auto-generated anchor should be a non-empty slug.
    assert!(!headings[0].anchor.is_empty());
}

#[test]
fn inject_heading_anchors_adds_id_to_headings() {
    let html = "<html><body><h1>Title</h1></body></html>";
    let headings = extract_headings(html, 1);
    let result = inject_heading_anchors(html, &headings, 1);
    assert!(
        result.contains("id="),
        "anchors should be injected into heading tags"
    );
}

#[test]
fn generate_toc_html_contains_caption_text() {
    let html = "<html><body><h1>Chapter 1</h1><h2>Section 1.1</h2></body></html>";
    let headings = extract_headings(html, 3);
    let mut toc = TableOfContent::default();
    toc.caption_text = "My Contents".into();
    let toc_html = generate_toc_html(&headings, &toc);
    assert!(
        toc_html.contains("My Contents"),
        "TOC should contain the caption text"
    );
}

#[test]
fn generate_toc_html_contains_heading_entries() {
    let html = "<html><body><h1>First Chapter</h1><h2>A Section</h2></body></html>";
    let headings = extract_headings(html, 3);
    let toc = TableOfContent::default();
    let toc_html = generate_toc_html(&headings, &toc);
    assert!(toc_html.contains("First Chapter"));
    assert!(toc_html.contains("A Section"));
}

#[test]
fn generate_toc_html_with_forward_links_contains_anchors() {
    let html = "<html><body><h1>Chapter</h1></body></html>";
    let headings = extract_headings(html, 1);
    let mut toc = TableOfContent::default();
    toc.forward_links = true;
    let toc_html = generate_toc_html(&headings, &toc);
    assert!(
        toc_html.contains("href"),
        "forward links should produce anchor href"
    );
}

#[test]
fn generate_toc_html_without_dotted_lines() {
    let html = "<html><body><h1>Chapter</h1></body></html>";
    let headings = extract_headings(html, 1);
    let mut toc = TableOfContent::default();
    toc.use_dotted_lines = false;
    let toc_html = generate_toc_html(&headings, &toc);
    // Should be valid HTML regardless.
    assert!(!toc_html.is_empty());
}

#[test]
fn generate_toc_html_empty_headings_returns_caption_only() {
    let toc = TableOfContent::default();
    let toc_html = generate_toc_html(&[], &toc);
    // With no headings the output should still contain the caption.
    assert!(toc_html.contains("Table of Contents"));
}

// ---------------------------------------------------------------------------
// Visual diffing tool
// ---------------------------------------------------------------------------

/// Encode a solid-colour 8×8 RGBA image as PNG bytes.
fn make_solid_png(r: u8, g: u8, b: u8) -> Vec<u8> {
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
    let mut img = RgbaImage::new(8, 8);
    for px in img.pixels_mut() {
        *px = Rgba([r, g, b, 255]);
    }
    let dyn_img = DynamicImage::ImageRgba8(img);
    let mut buf = Vec::new();
    dyn_img
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .expect("encode PNG");
    buf
}

#[test]
fn visual_diff_identical_images_reports_zero_percent() {
    let png = make_solid_png(128, 64, 32);
    let result = diff_images(&png, &png, DiffOptions::default()).unwrap();
    assert_eq!(result.different_pixels(), 0);
    assert!((result.diff_percentage() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn visual_diff_completely_different_images_reports_nonzero() {
    let ref_png = make_solid_png(0, 0, 0);
    let act_png = make_solid_png(255, 255, 255);
    let opts = DiffOptions {
        threshold: 0,
        ..Default::default()
    };
    let result = diff_images(&ref_png, &act_png, opts).unwrap();
    assert!(result.different_pixels() > 0);
    assert!(result.diff_percentage() > 0.0);
}

#[test]
fn visual_diff_total_pixels_equals_image_area() {
    let png = make_solid_png(10, 20, 30);
    let result = diff_images(&png, &png, DiffOptions::default()).unwrap();
    assert_eq!(result.total_pixels(), 8 * 8);
}

#[test]
fn visual_diff_produces_valid_png_diff_image() {
    let ref_png = make_solid_png(0, 0, 0);
    let act_png = make_solid_png(200, 200, 200);
    let opts = DiffOptions {
        threshold: 0,
        ..Default::default()
    };
    let result = diff_images(&ref_png, &act_png, opts).unwrap();
    // The diff image must start with the PNG magic bytes.
    assert!(result.diff_image().starts_with(b"\x89PNG"));
}

#[test]
fn visual_diff_size_mismatch_returns_error_by_default() {
    let ref_png = make_solid_png(0, 0, 0); // 8×8
                                           // Build a 4×4 PNG manually.
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
    let small = DynamicImage::ImageRgba8({
        let mut img = RgbaImage::new(4, 4);
        for px in img.pixels_mut() {
            *px = Rgba([0, 0, 0, 255]);
        }
        img
    });
    let mut act_png = Vec::new();
    small
        .write_to(&mut Cursor::new(&mut act_png), ImageFormat::Png)
        .unwrap();

    let err = diff_images(&ref_png, &act_png, DiffOptions::default()).unwrap_err();
    assert!(matches!(err, DiffError::SizeMismatch { .. }));
}

#[test]
fn visual_diff_allow_size_mismatch_compares_overlap() {
    let ref_png = make_solid_png(0, 0, 0); // 8×8 black
                                           // 4×4 black image
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
    let small = DynamicImage::ImageRgba8({
        let mut img = RgbaImage::new(4, 4);
        for px in img.pixels_mut() {
            *px = Rgba([0, 0, 0, 255]);
        }
        img
    });
    let mut act_png = Vec::new();
    small
        .write_to(&mut Cursor::new(&mut act_png), ImageFormat::Png)
        .unwrap();

    let opts = DiffOptions {
        require_same_size: false,
        threshold: 0,
        ..Default::default()
    };
    let result = diff_images(&ref_png, &act_png, opts).unwrap();
    // Overlap is 4×4 = 16 pixels, all identical (black vs black).
    assert_eq!(result.total_pixels(), 16);
    assert_eq!(result.different_pixels(), 0);
}
