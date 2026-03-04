use wkhtmltopdf_core::Converter;
use wkhtmltopdf_image::ImageConverter;
use wkhtmltopdf_pdf::PdfConverter;
use wkhtmltopdf_settings::{
    ColorMode, CropSettings, HeaderFooter, ImageGlobal, LoadErrorHandling, LoadPage, Margin,
    Orientation, PageSize, PdfGlobal, PdfObject, ProxyType, Size, TableOfContent, Unit, UnitReal,
    Web,
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

