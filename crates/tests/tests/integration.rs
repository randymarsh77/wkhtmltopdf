use wkhtmltopdf_core::Converter;
use wkhtmltopdf_image::ImageConverter;
use wkhtmltopdf_pdf::PdfConverter;
use wkhtmltopdf_settings::{ImageGlobal, PdfGlobal};

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
