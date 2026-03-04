use wkhtmltopdf_core::Converter;
use wkhtmltopdf_diff::{diff_images, DiffError, DiffOptions};
use wkhtmltopdf_image::ImageConverter;
use wkhtmltopdf_pdf::PdfConverter;
use wkhtmltopdf_settings::{
    ColorMode, CropSettings, HeaderFooter, ImageGlobal, LoadErrorHandling, LoadPage, Margin,
    Orientation, PageSize, PdfAConformance, PdfGlobal, PdfObject, ProxyType, Size, TableOfContent,
    Unit, UnitReal, Web,
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
