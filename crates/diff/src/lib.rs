//! Visual diffing tool for wkhtmltopdf.
//!
//! This crate provides perceptual image diffing to compare PDF-rendered-to-image
//! output between a reference implementation and a new one.  It performs a
//! pixel-level comparison, reports the diff percentage, and generates an
//! annotated diff image that highlights changed pixels.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use wkhtmltopdf_diff::{diff_images, DiffOptions};
//!
//! let reference_png: Vec<u8> = std::fs::read("reference.png").unwrap();
//! let actual_png: Vec<u8>    = std::fs::read("actual.png").unwrap();
//!
//! let result = diff_images(&reference_png, &actual_png, DiffOptions::default()).unwrap();
//!
//! println!("Diff: {:.2}%", result.diff_percentage());
//! println!("Changed pixels: {}/{}", result.different_pixels(), result.total_pixels());
//!
//! // Write the annotated diff image to disk.
//! std::fs::write("diff.png", result.diff_image()).unwrap();
//! ```

use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Rgba};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Public error type
// ---------------------------------------------------------------------------

/// Errors returned by [`diff_images`].
#[derive(Debug, Error)]
pub enum DiffError {
    /// Failed to decode one of the input images.
    #[error("failed to decode image: {0}")]
    Decode(String),

    /// Failed to encode the annotated diff image.
    #[error("failed to encode diff image: {0}")]
    Encode(String),

    /// The two images have different dimensions and `options.require_same_size`
    /// is `true`.
    #[error("image dimensions differ: reference is {ref_w}×{ref_h}, actual is {act_w}×{act_h}")]
    SizeMismatch {
        ref_w: u32,
        ref_h: u32,
        act_w: u32,
        act_h: u32,
    },
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Tuning knobs for the pixel diff algorithm.
#[derive(Debug, Clone)]
pub struct DiffOptions {
    /// Colour used to highlight changed pixels in the annotated diff image.
    /// Defaults to opaque red (`[255, 0, 0, 255]`).
    pub highlight_color: [u8; 4],

    /// Per-channel threshold (0–255).  A pixel is considered *different* when
    /// the Euclidean distance between the two RGBA values exceeds this value.
    /// Defaults to `8`.
    pub threshold: u8,

    /// When `true` (the default) [`diff_images`] returns [`DiffError::SizeMismatch`]
    /// if the two images have different dimensions.  When `false` only the
    /// overlapping region is compared.
    pub require_same_size: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            highlight_color: [255, 0, 0, 255],
            threshold: 8,
            require_same_size: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

/// The outcome of comparing two images.
#[derive(Debug)]
pub struct DiffResult {
    /// Number of pixels whose colour distance exceeded the threshold.
    different_pixels: u64,
    /// Total number of pixels in the comparison region.
    total_pixels: u64,
    /// PNG-encoded annotated diff image.
    diff_image: Vec<u8>,
}

impl DiffResult {
    /// Fraction of pixels that differ, expressed as a percentage (0.0–100.0).
    pub fn diff_percentage(&self) -> f64 {
        if self.total_pixels == 0 {
            return 0.0;
        }
        self.different_pixels as f64 / self.total_pixels as f64 * 100.0
    }

    /// Number of pixels whose colour distance exceeded the threshold.
    pub fn different_pixels(&self) -> u64 {
        self.different_pixels
    }

    /// Total number of pixels in the comparison region.
    pub fn total_pixels(&self) -> u64 {
        self.total_pixels
    }

    /// PNG bytes of the annotated diff image.
    ///
    /// This image is a copy of the *reference* with changed pixels replaced by
    /// the [`DiffOptions::highlight_color`].
    pub fn diff_image(&self) -> &[u8] {
        &self.diff_image
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Compare two PNG/JPEG/BMP images and return a [`DiffResult`].
///
/// Both `reference` and `actual` must be raw image bytes in any format
/// supported by the `image` crate (PNG, JPEG, BMP, …).
pub fn diff_images(
    reference: &[u8],
    actual: &[u8],
    options: DiffOptions,
) -> Result<DiffResult, DiffError> {
    let ref_img = decode(reference)?;
    let act_img = decode(actual)?;

    let (ref_w, ref_h) = ref_img.dimensions();
    let (act_w, act_h) = act_img.dimensions();

    if options.require_same_size && (ref_w != act_w || ref_h != act_h) {
        return Err(DiffError::SizeMismatch {
            ref_w,
            ref_h,
            act_w,
            act_h,
        });
    }

    // Compare only the overlapping region when sizes differ.
    let cmp_w = ref_w.min(act_w);
    let cmp_h = ref_h.min(act_h);

    let total_pixels = cmp_w as u64 * cmp_h as u64;
    let mut different_pixels: u64 = 0;

    // Build a mutable RGBA copy of the reference for annotation.
    let mut diff_canvas = ref_img.to_rgba8();

    let highlight = Rgba(options.highlight_color);
    let threshold_sq = options.threshold as u32 * options.threshold as u32;

    for y in 0..cmp_h {
        for x in 0..cmp_w {
            let rp = ref_img.get_pixel(x, y);
            let ap = act_img.get_pixel(x, y);

            if pixel_distance_sq(&rp, &ap) > threshold_sq {
                different_pixels += 1;
                diff_canvas.put_pixel(x, y, highlight);
            }
        }
    }

    let diff_image = encode_png(&DynamicImage::ImageRgba8(diff_canvas))?;

    Ok(DiffResult {
        different_pixels,
        total_pixels,
        diff_image,
    })
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Decode raw image bytes into a [`DynamicImage`].
fn decode(bytes: &[u8]) -> Result<DynamicImage, DiffError> {
    let cursor = Cursor::new(bytes);
    let reader = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| DiffError::Decode(format!("could not guess format: {e}")))?;
    reader
        .decode()
        .map_err(|e| DiffError::Decode(format!("decode error: {e}")))
}

/// Encode a [`DynamicImage`] to PNG bytes.
fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, DiffError> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| DiffError::Encode(format!("PNG encoding failed: {e}")))?;
    Ok(buf)
}

/// Squared Euclidean distance between two RGBA pixels.
///
/// Using the squared form avoids a `sqrt` call while preserving the ordering
/// needed for threshold comparisons.
fn pixel_distance_sq(a: &Rgba<u8>, b: &Rgba<u8>) -> u32 {
    let dr = a.0[0] as i32 - b.0[0] as i32;
    let dg = a.0[1] as i32 - b.0[1] as i32;
    let db = a.0[2] as i32 - b.0[2] as i32;
    let da = a.0[3] as i32 - b.0[3] as i32;
    (dr * dr + dg * dg + db * db + da * da) as u32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgba, RgbaImage};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Encode a `DynamicImage` as PNG bytes for use in tests.
    fn to_png(img: DynamicImage) -> Vec<u8> {
        encode_png(&img).expect("encode_png")
    }

    /// Build a solid-colour 10×10 RGBA image.
    fn solid(r: u8, g: u8, b: u8) -> DynamicImage {
        let mut img = RgbaImage::new(10, 10);
        for px in img.pixels_mut() {
            *px = Rgba([r, g, b, 255]);
        }
        DynamicImage::ImageRgba8(img)
    }

    // -----------------------------------------------------------------------
    // pixel_distance_sq
    // -----------------------------------------------------------------------

    #[test]
    fn distance_identical_pixels_is_zero() {
        let p = Rgba([128u8, 64, 32, 255]);
        assert_eq!(pixel_distance_sq(&p, &p), 0);
    }

    #[test]
    fn distance_black_white() {
        let black = Rgba([0u8, 0, 0, 255]);
        let white = Rgba([255u8, 255, 255, 255]);
        // dr=255, dg=255, db=255, da=0  → 3 * 255^2 = 195_075
        assert_eq!(pixel_distance_sq(&black, &white), 3 * 255 * 255);
    }

    // -----------------------------------------------------------------------
    // diff_images – identical images
    // -----------------------------------------------------------------------

    #[test]
    fn identical_images_produce_zero_diff() {
        let png = to_png(solid(100, 150, 200));
        let result = diff_images(&png, &png, DiffOptions::default()).unwrap();
        assert_eq!(result.different_pixels(), 0);
        assert_eq!(result.total_pixels(), 100);
        assert!((result.diff_percentage() - 0.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // diff_images – completely different images
    // -----------------------------------------------------------------------

    #[test]
    fn completely_different_images_report_100_percent() {
        // Use a large colour difference to ensure it exceeds any threshold.
        let ref_png = to_png(solid(0, 0, 0));
        let act_png = to_png(solid(255, 255, 255));

        // threshold=0 forces every pixel to count as different.
        let opts = DiffOptions {
            threshold: 0,
            ..Default::default()
        };
        let result = diff_images(&ref_png, &act_png, opts).unwrap();
        assert_eq!(result.different_pixels(), 100);
        assert!((result.diff_percentage() - 100.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // diff_images – partial change
    // -----------------------------------------------------------------------

    #[test]
    fn partial_change_reports_correct_percentage() {
        // Reference: all black.
        let ref_img = solid(0, 0, 0);
        // Actual: top half white, bottom half black.
        let mut act_raw = RgbaImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                let colour = if y < 5 { 255 } else { 0 };
                act_raw.put_pixel(x, y, Rgba([colour, colour, colour, 255]));
            }
        }
        let act_img = DynamicImage::ImageRgba8(act_raw);

        let opts = DiffOptions {
            threshold: 0,
            ..Default::default()
        };
        let result =
            diff_images(&to_png(ref_img), &to_png(act_img), opts).unwrap();
        assert_eq!(result.different_pixels(), 50);
        assert_eq!(result.total_pixels(), 100);
        assert!((result.diff_percentage() - 50.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // diff_images – threshold suppresses small differences
    // -----------------------------------------------------------------------

    #[test]
    fn threshold_suppresses_small_differences() {
        // Reference: (100, 100, 100), Actual: (105, 100, 100)
        // Per-channel diff is 5.  Squared distance = 25.
        // threshold=8  → threshold_sq=64 → pixel is the SAME.
        // threshold=4  → threshold_sq=16 → pixel is DIFFERENT.
        let ref_png = to_png(solid(100, 100, 100));
        let act_png = to_png(solid(105, 100, 100));

        let opts_loose = DiffOptions {
            threshold: 8,
            ..Default::default()
        };
        let result_loose = diff_images(&ref_png, &act_png, opts_loose).unwrap();
        assert_eq!(
            result_loose.different_pixels(),
            0,
            "threshold=8 should absorb a 5-unit colour shift"
        );

        let opts_strict = DiffOptions {
            threshold: 4,
            ..Default::default()
        };
        let result_strict = diff_images(&ref_png, &act_png, opts_strict).unwrap();
        assert_eq!(
            result_strict.different_pixels(),
            100,
            "threshold=4 should flag a 5-unit colour shift"
        );
    }

    // -----------------------------------------------------------------------
    // diff_images – diff image annotation
    // -----------------------------------------------------------------------

    #[test]
    fn diff_image_highlights_changed_pixels() {
        let ref_png = to_png(solid(0, 0, 0));
        let act_png = to_png(solid(255, 255, 255));

        let highlight = [255u8, 0, 0, 255];
        let opts = DiffOptions {
            threshold: 0,
            highlight_color: highlight,
            ..Default::default()
        };

        let result = diff_images(&ref_png, &act_png, opts).unwrap();

        // The diff image must be valid PNG.
        let diff_png = result.diff_image();
        assert!(diff_png.starts_with(b"\x89PNG"), "diff image must be PNG");

        // Decode the diff image and verify that every pixel has the highlight colour.
        let decoded = decode(diff_png).expect("decode diff image");
        let rgba = decoded.to_rgba8();
        for px in rgba.pixels() {
            assert_eq!(
                px.0, highlight,
                "every pixel in the diff image should be the highlight colour"
            );
        }
    }

    #[test]
    fn diff_image_leaves_unchanged_pixels_as_reference() {
        // Reference and actual are identical → no pixel should be highlighted.
        let png = to_png(solid(42, 84, 168));
        let result = diff_images(&png, &png, DiffOptions::default()).unwrap();

        let decoded = decode(result.diff_image()).expect("decode diff image");
        let rgba = decoded.to_rgba8();
        // Every pixel should match the original colour (42, 84, 168, 255).
        for px in rgba.pixels() {
            assert_eq!(px.0, [42u8, 84, 168, 255]);
        }
    }

    // -----------------------------------------------------------------------
    // diff_images – size mismatch handling
    // -----------------------------------------------------------------------

    #[test]
    fn size_mismatch_returns_error_when_required() {
        let ref_png = to_png(solid(0, 0, 0)); // 10×10
        let act_img = DynamicImage::new_rgba8(8, 8);
        let act_png = to_png(act_img);

        let opts = DiffOptions {
            require_same_size: true,
            ..Default::default()
        };
        let err = diff_images(&ref_png, &act_png, opts).unwrap_err();
        assert!(matches!(err, DiffError::SizeMismatch { .. }));
    }

    #[test]
    fn size_mismatch_compares_overlap_when_not_required() {
        let ref_png = to_png(solid(0, 0, 0)); // 10×10 black opaque
        let act_png = to_png(solid(0, 0, 0).crop_imm(0, 0, 5, 5)); // 5×5 black opaque

        let opts = DiffOptions {
            require_same_size: false,
            threshold: 0,
            ..Default::default()
        };
        let result = diff_images(&ref_png, &act_png, opts).unwrap();
        // Overlap is 5×5 = 25 pixels, all identical.
        assert_eq!(result.total_pixels(), 25);
        assert_eq!(result.different_pixels(), 0);
    }

    // -----------------------------------------------------------------------
    // DiffResult accessors
    // -----------------------------------------------------------------------

    #[test]
    fn diff_percentage_zero_total_returns_zero() {
        let r = DiffResult {
            different_pixels: 0,
            total_pixels: 0,
            diff_image: vec![],
        };
        assert!((r.diff_percentage() - 0.0).abs() < f64::EPSILON);
    }
}
