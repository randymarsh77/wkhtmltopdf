//! Rendering engine abstraction for wkhtmltopdf.
//!
//! This crate defines the `Converter` trait and shared conversion types that
//! both the `pdf` and `image` output crates implement.  It also provides the
//! [`renderer`] module, which contains the [`renderer::Renderer`] trait and
//! the [`renderer::HeadlessRenderer`] implementation.

pub mod renderer;
pub use renderer::{HeadlessRenderer, HtmlInput, RenderError, RenderedPage, Renderer};

#[cfg(feature = "qt-webkit")]
pub(crate) mod qt_webkit;

use thiserror::Error;

/// Errors that can occur during a conversion.
#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("rendering failed: {0}")]
    Render(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Progress information emitted during a conversion.
#[derive(Debug, Clone)]
pub struct Progress {
    pub phase: usize,
    pub phase_count: usize,
    pub description: String,
    pub percent: u8,
}

/// Trait implemented by all converters (PDF, image, …).
pub trait Converter {
    /// Run the conversion and return the raw output bytes.
    fn convert(&self) -> Result<Vec<u8>, ConvertError>;
}
