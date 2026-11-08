//! Native texture presentation for complete source/request-bound provider output. No decoder/I/O.
use craft_media_preview::{AlphaMode, PreviewError, PreviewRequest, RasterFrame, Result};
use egui::{ColorImage, Context, Response, TextureHandle, TextureOptions, Ui, Vec2};
/// One bounded current texture. Source/project/page transitions begin a fresh request.
#[derive(Default)]
pub struct RasterView {
    expected: Option<PreviewRequest>,
    texture: Option<TextureHandle>,
    dimensions: [u32; 2],
}
impl RasterView {
    /// Clear the old resource texture immediately and wait for this exact fresh request.
    pub fn request(&mut self, request: PreviewRequest) {
        self.expected = Some(request);
        self.texture = None;
        self.dimensions = [0, 0];
    }
    /// Discard all source-specific state when leaving a resource or project.
    pub fn clear(&mut self) {
        self.expected = None;
        self.texture = None;
        self.dimensions = [0, 0];
    }
    /// Accept a complete current result; an earlier request cannot replace the selected source.
    pub fn accept(&mut self, ctx: &Context, frame: &RasterFrame) -> Result<()> {
        if self.expected.as_ref() != Some(frame.request()) {
            return Err(PreviewError::StaleRequest);
        }
        let [width, height] = frame.dimensions();
        let dimensions = [
            usize::try_from(width).map_err(|_| PreviewError::Limit("texture width"))?,
            usize::try_from(height).map_err(|_| PreviewError::Limit("texture height"))?,
        ];
        let image = match frame.alpha_mode() {
            AlphaMode::Premultiplied => {
                ColorImage::from_rgba_premultiplied(dimensions, frame.rgba())
            }
            AlphaMode::Straight => ColorImage::from_rgba_unmultiplied(dimensions, frame.rgba()),
        };
        self.texture = Some(ctx.load_texture(
            format!("media-preview-{}", frame.request().sequence()),
            image,
            TextureOptions::LINEAR,
        ));
        self.dimensions = [width, height];
        Ok(())
    }
    /// Whether this exact request has a complete acknowledged native texture.
    pub fn is_ready(&self) -> bool {
        self.texture.is_some()
    }
    /// Paint inside the host's reserved media bounds; no media bytes are read or decoded here.
    pub fn show(&self, ui: &mut Ui, available: Vec2) -> Option<Response> {
        let texture = self.texture.as_ref()?;
        let size = Vec2::new(self.dimensions[0] as f32, self.dimensions[1] as f32);
        if available.x <= 0.0 || available.y <= 0.0 || !available.is_finite() {
            return None;
        }
        let scale = (available.x / size.x).min(available.y / size.y);
        Some(ui.image((texture.id(), size * scale)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_media_preview::{PreviewBudget, PreviewCancellation, PreviewSource, render_image};
    use sha2::{Digest, Sha256};
    use std::sync::Arc;
    #[test]
    fn late_matching_bytes_cannot_replace_a_new_resource_request()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([90, 30, 180, 128]),
        ));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png)?;
        let bytes = bytes.into_inner();
        let size = bytes.len();
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let budget = PreviewBudget::default();
        let source = PreviewSource::verified(Arc::new(bytes), size, &hash, "image/png", &budget)?;
        let first = PreviewRequest::new(&source)?;
        let current = PreviewRequest::new(&source)?;
        let old = render_image(
            &source,
            &first,
            [20, 20],
            &budget,
            &PreviewCancellation::default(),
        )?;
        let new = render_image(
            &source,
            &current,
            [20, 20],
            &budget,
            &PreviewCancellation::default(),
        )?;
        let context = Context::default();
        let mut view = RasterView::default();
        view.request(current.clone());
        view.accept(&context, &new)?;
        assert!(view.is_ready());
        assert_eq!(view.expected.as_ref(), Some(&current));
        assert!(matches!(
            view.accept(&context, &old),
            Err(PreviewError::StaleRequest)
        ));
        assert!(view.is_ready());
        assert_eq!(view.expected.as_ref(), Some(&current));
        view.clear();
        assert!(!view.is_ready());
        assert!(view.expected.is_none());
        Ok(())
    }
}
