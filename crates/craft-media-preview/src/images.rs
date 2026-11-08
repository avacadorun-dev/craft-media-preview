//! Actual bounded first-frame image decoding; immutable input remains separate from display pixels.
use crate::{
    AlphaMode, PreviewBudget, PreviewCancellation, PreviewError, PreviewRequest, PreviewSource,
    RasterFrame, Result,
};
use std::io::Cursor;
fn reader(source: &PreviewSource) -> Result<image::ImageReader<Cursor<&[u8]>>> {
    image::ImageReader::new(Cursor::new(source.bytes()))
        .with_guessed_format()
        .map_err(|error| PreviewError::Image(error.to_string()))
}
/// Decode a real PNG/JPEG/GIF/WebP first frame, then contain-fit it within a finite target.
///
/// Animation playback is a separate contract. Codec work may finish before cooperative
/// cancellation is noticed; no partially decoded result is published on cancellation/error.
pub fn render_image(
    source: &PreviewSource,
    request: &PreviewRequest,
    target: [u32; 2],
    budget: &PreviewBudget,
    cancel: &PreviewCancellation,
) -> Result<RasterFrame> {
    request.validate_source(source)?;
    cancel.check()?;
    budget.check_output(target[0], target[1])?;
    if !matches!(
        source.stamp().media_type.as_str(),
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    ) {
        return Err(PreviewError::Unsupported(source.stamp().media_type.clone()));
    }
    if source.bytes().len() > budget.max_source_bytes {
        return Err(PreviewError::Limit("source bytes"));
    }
    let header = reader(source)?;
    let expected = match source.stamp().media_type.as_str() {
        "image/png" => image::ImageFormat::Png,
        "image/jpeg" => image::ImageFormat::Jpeg,
        "image/gif" => image::ImageFormat::Gif,
        "image/webp" => image::ImageFormat::WebP,
        _ => return Err(PreviewError::Unsupported(source.stamp().media_type.clone())),
    };
    if header.format() != Some(expected) {
        return Err(PreviewError::Invalid(
            "declared image type differs from encoded bytes",
        ));
    }
    let (width, height) = header
        .into_dimensions()
        .map_err(|error| PreviewError::Image(error.to_string()))?;
    if width == 0
        || height == 0
        || width > budget.max_decoded_image_side
        || height > budget.max_decoded_image_side
        || u64::from(width) * u64::from(height) > budget.max_decoded_image_pixels
    {
        return Err(PreviewError::Limit("decoded image dimensions"));
    }
    let mut decoder = reader(source)?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(budget.max_decoded_image_side);
    limits.max_image_height = Some(budget.max_decoded_image_side);
    limits.max_alloc = Some(budget.max_decoded_image_pixels * 4);
    decoder.limits(limits);
    let decoded = decoder
        .decode()
        .map_err(|error| PreviewError::Image(error.to_string()))?;
    cancel.check()?;
    let fitted = decoded.thumbnail(target[0], target[1]).to_rgba8();
    budget.check_output(fitted.width(), fitted.height())?;
    cancel.check()?;
    RasterFrame::new(
        request.clone(),
        [fitted.width(), fitted.height()],
        fitted.into_raw(),
        AlphaMode::Straight,
        vec![],
        None,
        budget,
    )
}
