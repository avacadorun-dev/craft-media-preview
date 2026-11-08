//! One complete bounded PDF page render; parser/cache stay on the caller's worker.
use crate::{
    AlphaMode, PreviewBudget, PreviewCancellation, PreviewError, PreviewRequest, PreviewSource,
    PreviewWarning, RasterFrame, Result,
};
use craft_pdf::{
    interpret::InterpreterSettings,
    raster::{self, RenderCache},
};

/// Explicit selected PDF page, bounded target and optional caller-supplied password.
#[derive(Clone)]
pub struct PdfPageOptions<'a> {
    /// Zero-based page in the complete parsed page list.
    pub page_index: usize,
    /// Complete output contain-fit rectangle in device pixels.
    pub target: [u32; 2],
    /// Straight-alpha page background; output pixels remain premultiplied.
    pub background: [u8; 4],
    /// Optional user-supplied password, never retained in output or diagnostics.
    pub password: Option<&'a str>,
}
impl Default for PdfPageOptions<'_> {
    fn default() -> Self {
        Self {
            page_index: 0,
            target: [1024, 1024],
            background: [255; 4],
            password: None,
        }
    }
}

/// Render an exact PDF page into a bounded contain-fit raster on a caller-owned worker.
///
/// The host owns project/asset scope and worker scheduling. This API does no I/O. Backend
/// diagnostics and explicit limits are retained; total parser scratch memory/time and complete
/// PDF fidelity are not implied by a successful result. Cancellation is cooperative.
pub fn render_pdf_page(
    source: &PreviewSource,
    request: &PreviewRequest,
    page_options: &PdfPageOptions<'_>,
    budget: &PreviewBudget,
    cancel: &PreviewCancellation,
) -> Result<RasterFrame> {
    let PdfPageOptions {
        page_index,
        target,
        background,
        password,
    } = *page_options;
    request.validate_source(source)?;
    cancel.check()?;
    budget.check_output(target[0], target[1])?;
    if source.stamp().media_type != "application/pdf" {
        return Err(PreviewError::Unsupported(source.stamp().media_type.clone()));
    }
    if source.bytes().len() > budget.max_source_bytes {
        return Err(PreviewError::Limit("source bytes"));
    }
    let token = cancel.clone();
    let backend_cancel = craft_pdf::syntax::Cancellation::new(move || token.is_cancelled());
    let options = craft_pdf::OpenOptions {
        max_input_bytes: budget.max_source_bytes,
        decode_limits: craft_pdf::syntax::DecodeLimits::default()
            .with_max_decoded_stream(budget.max_pdf_stream_bytes),
    };
    let pdf = craft_pdf::open_with_options_and_cancel(
        source.bytes.clone(),
        password,
        &options,
        &backend_cancel,
    )
    .map_err(|error| {
        if cancel.is_cancelled() {
            PreviewError::Cancelled
        } else {
            PreviewError::PdfOpen(error.to_string())
        }
    })?;
    cancel.check()?;
    let pages = pdf.pages();
    let count = pages.len();
    if count > budget.max_pdf_pages {
        return Err(PreviewError::Limit("PDF pages"));
    }
    let page = pages.get(page_index).ok_or(PreviewError::PdfPage {
        index: page_index,
        pages: count,
    })?;
    let (width, height) = page.render_dimensions();
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(PreviewError::Invalid("PDF page dimensions"));
    }
    let scale = (target[0] as f32 / width).min(target[1] as f32 / height);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(PreviewError::Invalid("PDF raster scale"));
    }
    let dimensions = [
        (width * scale).ceil().max(1.0).min(target[0] as f32) as u32,
        (height * scale).ceil().max(1.0).min(target[1] as f32) as u32,
    ];
    budget.check_output(dimensions[0], dimensions[1])?;
    let settings = InterpreterSettings {
        cancelled: Some(cancel.0.clone()),
        ..Default::default()
    };
    let raster = raster::render(
        page,
        &RenderCache::new(),
        &settings,
        &raster::Options {
            scale: [scale, scale],
            size: Some(dimensions),
            background,
            ..Default::default()
        },
    )
    .map_err(|error| {
        if cancel.is_cancelled() {
            PreviewError::Cancelled
        } else {
            PreviewError::PdfRender(error.to_string())
        }
    })?;
    cancel.check()?;
    let mut warnings = vec![];
    if raster.diagnostics.image_decode_failed {
        warnings.push(PreviewWarning::ImageDecodeFailed);
    }
    if raster.diagnostics.unsupported_font {
        warnings.push(PreviewWarning::UnsupportedFont);
    }
    if raster.diagnostics.content_truncated {
        warnings.push(PreviewWarning::ContentTruncated);
    }
    let count = budget.check_output(raster.width, raster.height)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|_| PreviewError::Allocation)?;
    pixels.extend_from_slice(raster.bytes());
    RasterFrame::new(
        request.clone(),
        [raster.width, raster.height],
        pixels,
        AlphaMode::Premultiplied,
        warnings,
        Some((page_index, pages.len())),
        budget,
    )
}
