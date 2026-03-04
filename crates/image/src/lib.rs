//! Image output for wkhtmltopdf.
//!
//! This crate implements the `Converter` trait from `wkhtmltopdf-core` to
//! produce raster images from HTML sources, mirroring the role of
//! `ImageConverter` in the C++ codebase.
//!
//! HTML content is fetched from the [`ImageGlobal`]'s `page` field and
//! rendered to an image via the headless browser backend.  The raw PNG output
//! is then optionally cropped, resized to `screen_width`, and re-encoded to
//! the requested format (PNG, JPG/JPEG, BMP, or SVG) with the configured
//! quality.

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, ImageReader};
use thiserror::Error;
use wkhtmltopdf_core::{ConvertError, Converter, HeadlessRenderer, HtmlInput, Renderer};
use wkhtmltopdf_settings::ImageGlobal;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// Errors specific to image conversion.
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("conversion failed: {0}")]
    Conversion(#[from] ConvertError),
}

// ---------------------------------------------------------------------------
// ImageConverter
// ---------------------------------------------------------------------------

/// Converts an HTML page to a raster image.
pub struct ImageConverter {
    settings: ImageGlobal,
}

impl ImageConverter {
    /// Create a new `ImageConverter` with the given settings.
    pub fn new(settings: ImageGlobal) -> Self {
        Self { settings }
    }

    /// Return the image settings.
    pub fn settings(&self) -> &ImageGlobal {
        &self.settings
    }
}

// ---------------------------------------------------------------------------
// Converter implementation
// ---------------------------------------------------------------------------

impl Converter for ImageConverter {
    /// Render the HTML page and return the raw image bytes.
    ///
    /// The pipeline is:
    /// 1. Render the page to a PNG via the headless browser backend.
    /// 2. Decode the PNG and apply crop settings (if any).
    /// 3. Resize to `screen_width` if specified.
    /// 4. Encode to the requested format (PNG, JPEG, BMP, or SVG) with quality.
    fn convert(&self) -> Result<Vec<u8>, ConvertError> {
        let page_src = self.settings.page.as_deref().ok_or_else(|| {
            ConvertError::Render(
                "no page specified: set ImageGlobal.page to a URL or file path".into(),
            )
        })?;

        // Render the HTML to a PNG screenshot via the headless browser.
        let renderer = HeadlessRenderer::new();
        let input = parse_input(page_src);
        let rendered = renderer
            .render(&input)
            .map_err(|e| ConvertError::Render(e.to_string()))?;

        // Decode the PNG bytes returned by the renderer.
        let img = decode_image(&rendered.bytes)?;

        // Apply crop settings when any crop dimension is non-zero.
        let img = apply_crop(img, &self.settings)?;

        // Resize to screen_width if requested, preserving aspect ratio.
        let img = apply_resize(img, &self.settings);

        // Encode to the target format.
        encode_image(img, &self.settings)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Parse a page source string into an [`HtmlInput`].
fn parse_input(source: &str) -> HtmlInput {
    if source.starts_with("http://") || source.starts_with("https://") {
        HtmlInput::Url(source.to_owned())
    } else {
        HtmlInput::File(source.into())
    }
}

/// Decode raw image bytes into a [`DynamicImage`].
fn decode_image(bytes: &[u8]) -> Result<DynamicImage, ConvertError> {
    let cursor = Cursor::new(bytes);
    let reader = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| ConvertError::Render(format!("failed to guess image format: {e}")))?;
    reader
        .decode()
        .map_err(|e| ConvertError::Render(format!("failed to decode rendered image: {e}")))
}

/// Crop the image according to `CropSettings` when any dimension is non-zero.
fn apply_crop(img: DynamicImage, settings: &ImageGlobal) -> Result<DynamicImage, ConvertError> {
    let c = &settings.crop;
    if c.left == 0 && c.top == 0 && c.width == 0 && c.height == 0 {
        return Ok(img);
    }

    let src_w = img.width();
    let src_h = img.height();

    let x = c.left.max(0) as u32;
    let y = c.top.max(0) as u32;
    // A crop width/height of 0 means "to the edge of the image".
    let w = if c.width > 0 {
        (c.width as u32).min(src_w.saturating_sub(x))
    } else {
        src_w.saturating_sub(x)
    };
    let h = if c.height > 0 {
        (c.height as u32).min(src_h.saturating_sub(y))
    } else {
        src_h.saturating_sub(y)
    };

    if w == 0 || h == 0 {
        return Err(ConvertError::Render(
            "crop rectangle results in a zero-size image".into(),
        ));
    }

    Ok(img.crop_imm(x, y, w, h))
}

/// Resize the image to `screen_width` (preserving aspect ratio) when set.
fn apply_resize(img: DynamicImage, settings: &ImageGlobal) -> DynamicImage {
    if let Some(sw) = settings.screen_width {
        if sw > 0 {
            let target_w = sw as u32;
            if img.width() != target_w {
                return img.resize(
                    target_w,
                    u32::MAX,
                    image::imageops::FilterType::Lanczos3,
                );
            }
        }
    }
    img
}

/// Encode the image to the target format and return the raw bytes.
fn encode_image(img: DynamicImage, settings: &ImageGlobal) -> Result<Vec<u8>, ConvertError> {
    let fmt_str = settings
        .format
        .as_deref()
        .unwrap_or("png")
        .to_ascii_lowercase();

    let mut buf = Vec::new();

    match fmt_str.as_str() {
        "jpg" | "jpeg" => {
            // Clamp quality to the valid JPEG range (1–100).
            let quality = settings.quality.clamp(1, 100) as u8;
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(Cursor::new(&mut buf), quality);
            encoder
                .encode_image(&img)
                .map_err(|e| ConvertError::Render(format!("JPEG encoding failed: {e}")))?;
        }
        "bmp" => {
            img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Bmp)
                .map_err(|e| ConvertError::Render(format!("BMP encoding failed: {e}")))?;
        }
        "svg" => {
            // Embed the raster image as a Base64-encoded PNG inside an SVG
            // wrapper so callers that expect an SVG container receive valid XML.
            let png_bytes = encode_png(&img)?;
            let b64 = base64_encode(&png_bytes);
            let (w, h) = (img.width(), img.height());
            let svg = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}"><image width="{w}" height="{h}" xlink:href="data:image/png;base64,{b64}"/></svg>"#
            );
            buf = svg.into_bytes();
        }
        // Default: PNG
        _ => {
            buf = encode_png(&img)?;
        }
    }

    Ok(buf)
}

/// Encode a `DynamicImage` as PNG bytes.
fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, ConvertError> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| ConvertError::Render(format!("PNG encoding failed: {e}")))?;
    Ok(buf)
}

/// Minimal Base64 encoder (avoids adding an extra crate dependency).
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wkhtmltopdf_settings::{CropSettings, ImageGlobal};

    #[test]
    fn image_converter_new_stores_settings() {
        let s = ImageGlobal::default();
        let conv = ImageConverter::new(s.clone());
        assert_eq!(conv.settings().quality, s.quality);
    }

    #[test]
    fn convert_no_page_returns_error() {
        let conv = ImageConverter::new(ImageGlobal::default());
        assert!(conv.convert().is_err());
    }

    #[test]
    fn parse_input_url() {
        let input = parse_input("https://example.com");
        assert!(matches!(input, HtmlInput::Url(_)));
    }

    #[test]
    fn parse_input_file() {
        let input = parse_input("/tmp/test.html");
        assert!(matches!(input, HtmlInput::File(_)));
    }

    #[test]
    fn apply_crop_noop_when_all_zero() {
        let img = DynamicImage::new_rgb8(100, 200);
        let settings = ImageGlobal::default(); // crop all zeros
        let result = apply_crop(img, &settings).unwrap();
        assert_eq!(result.width(), 100);
        assert_eq!(result.height(), 200);
    }

    #[test]
    fn apply_crop_with_dimensions() {
        let img = DynamicImage::new_rgb8(100, 200);
        let mut settings = ImageGlobal::default();
        settings.crop = CropSettings {
            left: 10,
            top: 20,
            width: 50,
            height: 80,
        };
        let result = apply_crop(img, &settings).unwrap();
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 80);
    }

    #[test]
    fn apply_crop_zero_width_means_to_edge() {
        let img = DynamicImage::new_rgb8(100, 200);
        let mut settings = ImageGlobal::default();
        settings.crop = CropSettings {
            left: 10,
            top: 0,
            width: 0,
            height: 100,
        };
        let result = apply_crop(img, &settings).unwrap();
        assert_eq!(result.width(), 90); // 100 - 10
        assert_eq!(result.height(), 100);
    }

    #[test]
    fn apply_resize_no_screen_width() {
        let img = DynamicImage::new_rgb8(100, 200);
        let settings = ImageGlobal::default();
        let result = apply_resize(img, &settings);
        assert_eq!(result.width(), 100);
    }

    #[test]
    fn apply_resize_to_screen_width() {
        let img = DynamicImage::new_rgb8(200, 100);
        let mut settings = ImageGlobal::default();
        settings.screen_width = Some(100);
        let result = apply_resize(img, &settings);
        assert_eq!(result.width(), 100);
    }

    #[test]
    fn encode_image_png_default() {
        let img = DynamicImage::new_rgb8(4, 4);
        let settings = ImageGlobal::default(); // format = None → png
        let bytes = encode_image(img, &settings).unwrap();
        // PNG files start with the PNG magic bytes.
        assert!(bytes.starts_with(b"\x89PNG"));
    }

    #[test]
    fn encode_image_jpeg() {
        let img = DynamicImage::new_rgb8(4, 4);
        let mut settings = ImageGlobal::default();
        settings.format = Some("jpg".into());
        let bytes = encode_image(img, &settings).unwrap();
        // JPEG files start with 0xFF 0xD8.
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn encode_image_bmp() {
        let img = DynamicImage::new_rgb8(4, 4);
        let mut settings = ImageGlobal::default();
        settings.format = Some("bmp".into());
        let bytes = encode_image(img, &settings).unwrap();
        // BMP files start with 'BM'.
        assert_eq!(&bytes[..2], b"BM");
    }

    #[test]
    fn encode_image_svg_contains_xml() {
        let img = DynamicImage::new_rgb8(4, 4);
        let mut settings = ImageGlobal::default();
        settings.format = Some("svg".into());
        let bytes = encode_image(img, &settings).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("<svg"));
        assert!(s.contains("data:image/png;base64,"));
    }

    #[test]
    fn base64_encode_hello() {
        // "Hello" → "SGVsbG8="
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn image_global_default_dpi_is_none() {
        let s = ImageGlobal::default();
        assert!(s.dpi.is_none());
    }
}
