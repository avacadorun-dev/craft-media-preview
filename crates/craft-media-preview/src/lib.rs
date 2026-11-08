//! Immutable, source-bound native media previews. Hosts own storage, authorization and workers.
#[cfg(feature = "images")]
mod images;
#[cfg(feature = "pdf")]
mod pdf;
#[cfg(feature = "images")]
pub use images::render_image;
#[cfg(feature = "pdf")]
pub use pdf::{PdfPageOptions, render_pdf_page};

use sha2::{Digest, Sha256};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

/// A recoverable input, resource, cancellation or decoder failure. Source bytes never change.
#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    /// A caller-specified finite source/output/decoder capacity was exceeded.
    #[error("Media preview limit exceeded: {0}")]
    Limit(&'static str),
    /// Input descriptor disagrees with the actual immutable bytes.
    #[error("Media preview source length or SHA-256 does not match")]
    SourceMismatch,
    /// Invalid MIME descriptor or request geometry.
    #[error("Invalid media preview request: {0}")]
    Invalid(&'static str),
    /// The caller cancelled this exact operation.
    #[error("Media preview cancelled")]
    Cancelled,
    /// A result belongs to an older or foreign in-process request.
    #[error("Media preview request changed; discard this stale result")]
    StaleRequest,
    /// The requested provider does not match the declared format.
    #[error("Unsupported media preview type: {0}")]
    Unsupported(String),
    /// PDF opening failed without changing the input.
    #[error("PDF preview could not be opened: {0}")]
    PdfOpen(String),
    /// The selected page is outside the actual PDF page list.
    #[error("PDF page {index} is unavailable; this document has {pages} pages")]
    PdfPage {
        /// Requested zero-based page index.
        index: usize,
        /// Actual complete page count.
        pages: usize,
    },
    /// A PDF backend operation failed; its partial cache/output must be discarded.
    #[error("PDF page preview failed: {0}")]
    PdfRender(String),
    /// An image codec refused the source.
    #[error("Image preview failed: {0}")]
    Image(String),
    /// A bounded fallible allocation was unavailable.
    #[error("Not enough memory for this bounded media preview")]
    Allocation,
    /// In-process request identity was exhausted.
    #[error("Media preview request identity exhausted")]
    IdentityExhausted,
}
/// Result of a source-preserving media operation.
pub type Result<T> = std::result::Result<T, PreviewError>;

/// Independent finite limits; these are not a universal backend scratch-memory/time guarantee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewBudget {
    /// Maximum supplied original source bytes.
    pub max_source_bytes: usize,
    /// Maximum requested raster width/height.
    pub max_output_side: u32,
    /// Maximum requested complete output pixels.
    pub max_output_pixels: u64,
    /// Maximum decoded input image width/height before thumbnailing.
    pub max_decoded_image_side: u32,
    /// Maximum decoded input image pixels before thumbnailing.
    pub max_decoded_image_pixels: u64,
    /// Maximum ordinary expanded PDF stream bytes, separate from streaming content/backend limits.
    pub max_pdf_stream_bytes: usize,
    /// Maximum accepted complete PDF page list.
    pub max_pdf_pages: usize,
}
impl Default for PreviewBudget {
    fn default() -> Self {
        Self {
            max_source_bytes: 512 * 1024 * 1024,
            max_output_side: 4096,
            max_output_pixels: 16_000_000,
            max_decoded_image_side: 32_768,
            max_decoded_image_pixels: 64_000_000,
            max_pdf_stream_bytes: 32 * 1024 * 1024,
            max_pdf_pages: 100_000,
        }
    }
}
impl PreviewBudget {
    /// Validate finite nonzero caller limits before parsing or output allocation.
    pub fn validate(&self) -> Result<()> {
        if self.max_source_bytes == 0
            || self.max_output_side == 0
            || self.max_output_side > 8192
            || self.max_output_pixels == 0
            || self.max_output_pixels > 64_000_000
            || self.max_decoded_image_side == 0
            || self.max_decoded_image_pixels == 0
            || self.max_decoded_image_pixels > u64::MAX / 4
            || self.max_pdf_stream_bytes == 0
            || self.max_pdf_pages == 0
        {
            return Err(PreviewError::Invalid("nonzero finite budgets are required"));
        }
        Ok(())
    }
    /// Check an exact complete raster before allocating/converting pixels.
    pub fn check_output(&self, width: u32, height: u32) -> Result<usize> {
        self.validate()?;
        if width == 0
            || height == 0
            || width > self.max_output_side
            || height > self.max_output_side
        {
            return Err(PreviewError::Limit("output side"));
        }
        let pixels = u64::from(width) * u64::from(height);
        if pixels > self.max_output_pixels {
            return Err(PreviewError::Limit("output pixels"));
        }
        usize::try_from(pixels * 4).map_err(|_| PreviewError::Limit("output byte capacity"))
    }
}

/// Exact descriptive input identity. This is data, never project/I/O authorization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceStamp {
    /// Exact lowercase encoded-byte digest.
    pub sha256: String,
    /// Exact original encoded length.
    pub byte_size: usize,
    /// Caller-declared media type; provider APIs additionally check their type.
    pub media_type: String,
}
/// Shared immutable authorized input. Debug output omits its potentially private media bytes.
#[derive(Clone)]
pub struct PreviewSource {
    bytes: Arc<Vec<u8>>,
    stamp: SourceStamp,
}
impl std::fmt::Debug for PreviewSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreviewSource")
            .field("stamp", &self.stamp)
            .finish()
    }
}
impl PreviewSource {
    /// Verify a caller-owned original on its worker. This never reads a path or URL.
    pub fn verified(
        bytes: Arc<Vec<u8>>,
        declared_size: usize,
        expected_sha256: &str,
        media_type: &str,
        budget: &PreviewBudget,
    ) -> Result<Self> {
        budget.validate()?;
        if bytes.len() > budget.max_source_bytes {
            return Err(PreviewError::Limit("source bytes"));
        }
        if media_type.is_empty()
            || media_type.len() > 255
            || media_type.chars().any(char::is_control)
        {
            return Err(PreviewError::Invalid("media type"));
        }
        let sha256 = format!("{:x}", Sha256::digest(bytes.as_slice()));
        if bytes.len() != declared_size || sha256 != expected_sha256.to_ascii_lowercase() {
            return Err(PreviewError::SourceMismatch);
        }
        Ok(Self {
            bytes,
            stamp: SourceStamp {
                sha256,
                byte_size: declared_size,
                media_type: media_type.into(),
            },
        })
    }
    /// Exact original bytes; no mutable or implicit I/O interface is exposed.
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
    /// Exact identity verified from the original bytes.
    pub fn stamp(&self) -> &SourceStamp {
        &self.stamp
    }
}
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
/// Fresh non-deserializable native request identity, independent of matching source hashes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewRequest {
    id: u64,
    source: SourceStamp,
}
impl PreviewRequest {
    /// Begin a fresh request when the selected project/resource/page/geometry changes.
    pub fn new(source: &PreviewSource) -> Result<Self> {
        let id = NEXT_REQUEST
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| PreviewError::IdentityExhausted)?;
        Ok(Self {
            id,
            source: source.stamp.clone(),
        })
    }
    /// Opaque request sequence for native widget/debug identity, not a persistence authority.
    pub fn sequence(&self) -> u64 {
        self.id
    }
    /// Exact descriptive source requested by this process-owned token.
    pub fn source(&self) -> &SourceStamp {
        &self.source
    }
    pub(crate) fn validate_source(&self, source: &PreviewSource) -> Result<()> {
        if self.source != source.stamp {
            return Err(PreviewError::StaleRequest);
        }
        Ok(())
    }
}
/// Caller-owned cooperative cancellation. A token belongs to one bounded worker request.
#[derive(Clone, Default)]
pub struct PreviewCancellation(Arc<AtomicBool>);
impl PreviewCancellation {
    /// Signal cancellation without waiting for GUI/audio callbacks or a decoder lock.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    /// Current explicit cancellation state.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    /// Refuse further work once cancelled.
    pub fn check(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(PreviewError::Cancelled)
        } else {
            Ok(())
        }
    }
}
/// Pixel channel interpretation; PDF raster output is premultiplied, image output is straight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaMode {
    /// RGB bytes already contain alpha multiplication.
    Premultiplied,
    /// Original decoded straight RGBA channels.
    Straight,
}
/// Actual backend diagnostic; absence does not certify complete format fidelity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewWarning {
    /// A PDF image could not be decoded.
    ImageDecodeFailed,
    /// A backend reported an unsupported font.
    UnsupportedFont,
    /// Painting content was omitted, failed or hit a backend content budget.
    ContentTruncated,
}
/// Complete immutable decoded output. Partial failed rasters are never constructible externally.
pub struct RasterFrame {
    request: PreviewRequest,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    alpha: AlphaMode,
    warnings: Vec<PreviewWarning>,
    page: Option<(usize, usize)>,
}
impl RasterFrame {
    pub(crate) fn new(
        request: PreviewRequest,
        dimensions: [u32; 2],
        rgba: Vec<u8>,
        alpha: AlphaMode,
        warnings: Vec<PreviewWarning>,
        page: Option<(usize, usize)>,
        budget: &PreviewBudget,
    ) -> Result<Self> {
        let [width, height] = dimensions;
        if budget.check_output(width, height)? != rgba.len() {
            return Err(PreviewError::Invalid("incomplete raster byte length"));
        }
        Ok(Self {
            request,
            width,
            height,
            rgba,
            alpha,
            warnings,
            page,
        })
    }
    /// Exact request, including source identity, which produced these pixels.
    pub fn request(&self) -> &PreviewRequest {
        &self.request
    }
    /// Complete pixel width and height.
    pub fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }
    /// Actual complete RGBA8 pixels.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
    /// Correct alpha interpretation for the native texture upload.
    pub fn alpha_mode(&self) -> AlphaMode {
        self.alpha
    }
    /// Actual backend diagnostics, distinct from a universal fidelity claim.
    pub fn warnings(&self) -> &[PreviewWarning] {
        &self.warnings
    }
    /// Requested zero-based PDF page and actual page count, or no PDF metadata for images.
    pub fn page(&self) -> Option<(usize, usize)> {
        self.page
    }
}
