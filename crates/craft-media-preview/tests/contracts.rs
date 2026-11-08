//! Synthetic immutable-source/request/cancellation contracts, not host-native acceptance.
use craft_media_preview::*;
use sha2::{Digest, Sha256};
use std::sync::Arc;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn source(bytes: Vec<u8>, mime: &str, budget: &PreviewBudget) -> Result<PreviewSource> {
    let length = bytes.len();
    let hash = format!("{:x}", Sha256::digest(&bytes));
    PreviewSource::verified(Arc::new(bytes), length, &hash, mime, budget)
}
#[test]
fn source_descriptor_mismatch_refuses_before_any_source_is_exposed() -> TestResult {
    let bytes = Arc::new(b"synthetic immutable original".to_vec());
    let hash = format!("{:x}", Sha256::digest(bytes.as_slice()));
    let budget = PreviewBudget::default();
    assert!(matches!(
        PreviewSource::verified(
            bytes.clone(),
            bytes.len() + 1,
            &hash,
            "application/pdf",
            &budget
        ),
        Err(PreviewError::SourceMismatch)
    ));
    assert!(matches!(
        PreviewSource::verified(
            bytes.clone(),
            bytes.len(),
            &"0".repeat(64),
            "application/pdf",
            &budget
        ),
        Err(PreviewError::SourceMismatch)
    ));
    assert_eq!(bytes.as_slice(), b"synthetic immutable original");
    Ok(())
}
#[test]
fn identical_original_bytes_still_receive_distinct_process_request_identities() -> TestResult {
    let source = source(
        b"same bytes".to_vec(),
        "application/pdf",
        &PreviewBudget::default(),
    )?;
    let first = PreviewRequest::new(&source)?;
    let second = PreviewRequest::new(&source)?;
    assert_eq!(first.source(), second.source());
    assert_ne!(first, second);
    Ok(())
}
#[test]
fn source_and_raster_budgets_are_independent_and_checked_without_allocation() -> TestResult {
    let budget = PreviewBudget {
        max_source_bytes: 2,
        ..Default::default()
    };
    assert!(matches!(
        source(vec![1, 2, 3], "application/pdf", &budget),
        Err(PreviewError::Limit("source bytes"))
    ));
    let budget = PreviewBudget::default();
    assert!(budget.check_output(8192, 8192).is_err());
    assert!(budget.check_output(0, 200).is_err());
    assert_eq!(budget.check_output(100, 200)?, 80_000);
    Ok(())
}
#[test]
fn cancellation_is_shared_and_never_resets_itself() -> TestResult {
    let cancel = PreviewCancellation::default();
    let worker = cancel.clone();
    assert!(worker.check().is_ok());
    cancel.cancel();
    assert!(matches!(worker.check(), Err(PreviewError::Cancelled)));
    assert!(cancel.is_cancelled());
    Ok(())
}

#[cfg(feature = "pdf")]
mod pdf {
    use super::*;
    fn document() -> Vec<u8> {
        let red = "1 0 0 rg 0 0 100 50 re f\n";
        let blue = "0 0 1 rg 0 0 100 50 re f\n";
        let objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R 5 0 R] /Count 2 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 50] /Contents 4 0 R >>".to_owned(),
            format!("<< /Length {} >>\nstream\n{}endstream", red.len(), red),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 50] /Contents 6 0 R >>".to_owned(),
            format!("<< /Length {} >>\nstream\n{}endstream", blue.len(), blue),
        ];
        let mut bytes = b"%PDF-1.7\n".to_vec();
        let mut offsets = vec![0];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(bytes.len());
            bytes
                .extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
        }
        let xref = bytes.len();
        bytes.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes(),
        );
        for offset in &offsets[1..] {
            bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        bytes.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                offsets.len()
            )
            .as_bytes(),
        );
        bytes
    }
    #[test]
    fn real_pdf_pages_render_distinct_pixels_without_mutating_the_original() -> TestResult {
        let original = document();
        let budget = PreviewBudget::default();
        let source = source(original.clone(), "application/pdf", &budget)?;
        let first = PreviewRequest::new(&source)?;
        let second = PreviewRequest::new(&source)?;
        let red = render_pdf_page(
            &source,
            &first,
            &PdfPageOptions {
                target: [200, 200],
                ..Default::default()
            },
            &budget,
            &PreviewCancellation::default(),
        )?;
        let blue = render_pdf_page(
            &source,
            &second,
            &PdfPageOptions {
                page_index: 1,
                target: [200, 200],
                ..Default::default()
            },
            &budget,
            &PreviewCancellation::default(),
        )?;
        assert_eq!(red.dimensions(), [200, 100]);
        assert_eq!(red.page(), Some((0, 2)));
        assert_eq!(blue.page(), Some((1, 2)));
        assert_eq!(red.alpha_mode(), AlphaMode::Premultiplied);
        let center = (50 * 200 + 100) * 4;
        assert_eq!(&red.rgba()[center..center + 4], &[255, 0, 0, 255]);
        assert_eq!(&blue.rgba()[center..center + 4], &[0, 0, 255, 255]);
        assert!(red.warnings().is_empty());
        assert!(blue.warnings().is_empty());
        assert_eq!(source.bytes(), original);
        Ok(())
    }
    #[test]
    fn malformed_or_unavailable_pdf_pages_never_publish_fake_success() -> TestResult {
        let budget = PreviewBudget::default();
        let broken = source(b"not a PDF".to_vec(), "application/pdf", &budget)?;
        assert!(
            render_pdf_page(
                &broken,
                &PreviewRequest::new(&broken)?,
                &PdfPageOptions::default(),
                &budget,
                &PreviewCancellation::default()
            )
            .is_err()
        );
        let source = source(document(), "application/pdf", &budget)?;
        assert!(matches!(
            render_pdf_page(
                &source,
                &PreviewRequest::new(&source)?,
                &PdfPageOptions {
                    page_index: 2,
                    ..Default::default()
                },
                &budget,
                &PreviewCancellation::default()
            ),
            Err(PreviewError::PdfPage { index: 2, pages: 2 })
        ));
        Ok(())
    }
    #[test]
    fn cancelled_pdf_or_foreign_request_refuses_without_output() -> TestResult {
        let budget = PreviewBudget::default();
        let source = source(document(), "application/pdf", &budget)?;
        let cancel = PreviewCancellation::default();
        cancel.cancel();
        assert!(matches!(
            render_pdf_page(
                &source,
                &PreviewRequest::new(&source)?,
                &PdfPageOptions::default(),
                &budget,
                &cancel
            ),
            Err(PreviewError::Cancelled)
        ));
        let other = super::source(b"different".to_vec(), "application/pdf", &budget)?;
        assert!(matches!(
            render_pdf_page(
                &source,
                &PreviewRequest::new(&other)?,
                &PdfPageOptions::default(),
                &budget,
                &PreviewCancellation::default()
            ),
            Err(PreviewError::StaleRequest)
        ));
        Ok(())
    }
}
#[cfg(feature = "images")]
mod images {
    use super::*;
    #[test]
    fn supported_first_frame_decoders_keep_actual_source_and_contain_fit_pixels() -> TestResult {
        for (format, mime) in [
            (image::ImageFormat::Png, "image/png"),
            (image::ImageFormat::Jpeg, "image/jpeg"),
            (image::ImageFormat::Gif, "image/gif"),
            (image::ImageFormat::WebP, "image/webp"),
        ] {
            let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
                8,
                4,
                image::Rgb([25, 80, 160]),
            ));
            let mut bytes = std::io::Cursor::new(Vec::new());
            image.write_to(&mut bytes, format)?;
            let original = bytes.into_inner();
            let budget = PreviewBudget::default();
            let source = source(original.clone(), mime, &budget)?;
            let frame = render_image(
                &source,
                &PreviewRequest::new(&source)?,
                [4, 4],
                &budget,
                &PreviewCancellation::default(),
            )?;
            assert_eq!(frame.dimensions(), [4, 2]);
            assert_eq!(frame.rgba().len(), 32);
            assert_eq!(frame.alpha_mode(), AlphaMode::Straight);
            assert_eq!(source.bytes(), original);
        }
        Ok(())
    }
    #[test]
    fn decoded_image_bounds_refuse_before_pixel_allocation_without_touching_original() -> TestResult
    {
        let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            4,
            image::Rgb([25, 80, 160]),
        ));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png)?;
        let original = bytes.into_inner();
        let budget = PreviewBudget {
            max_decoded_image_pixels: 1,
            ..Default::default()
        };
        let source = source(original.clone(), "image/png", &budget)?;
        assert!(matches!(
            render_image(
                &source,
                &PreviewRequest::new(&source)?,
                [4, 4],
                &budget,
                &PreviewCancellation::default()
            ),
            Err(PreviewError::Limit("decoded image dimensions"))
        ));
        assert_eq!(source.bytes(), original);
        Ok(())
    }
}
