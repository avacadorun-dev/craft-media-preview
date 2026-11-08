#![cfg(feature = "audio")]
//! Actual synthetic PCM packets, not a fake playback clock or native device qualification.
use craft_media_preview::*;
use sha2::{Digest, Sha256};
use std::sync::Arc;
type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;
fn wave(rate: u32, frames: u32) -> Vec<u8> {
    let mut b = b"RIFF".to_vec();
    b.extend((36 + frames * 2).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(rate.to_le_bytes());
    b.extend((rate * 2).to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend((frames * 2).to_le_bytes());
    for frame in 0..frames {
        b.extend((frame as i16).to_le_bytes());
    }
    b
}
fn source(bytes: Vec<u8>, mime: &str) -> Result<PreviewSource> {
    let len = bytes.len();
    let sha = format!("{:x}", Sha256::digest(&bytes));
    PreviewSource::verified(Arc::new(bytes), len, &sha, mime, &PreviewBudget::default())
}
#[test]
fn real_pcm_segment_should_clip_exact_source_samples_seek_and_preserve_original() -> TestResult {
    let bytes = wave(8000, 8000);
    let original = bytes.clone();
    let source = source(bytes, "audio/wav")?;
    let request = PreviewRequest::new(&source)?;
    let mut stream = AudioStream::open(
        &source,
        &request,
        AudioSegment {
            start_ms: 100,
            end_ms: Some(125),
        },
        AudioLimits::default(),
        PreviewCancellation::default(),
    )?;
    assert_eq!(stream.description().source_frames, Some(8000));
    assert_eq!(
        (
            stream.description().segment_start,
            stream.description().segment_end
        ),
        (800, Some(1000))
    );
    let mut blocks = Vec::new();
    while let Some(block) = stream.next_block()? {
        blocks.push(block);
    }
    assert_eq!(blocks.iter().map(AudioBlock::frames).sum::<usize>(), 200);
    assert_eq!(blocks[0].first_frame(), 800);
    assert_eq!(blocks[0].request(), &request);
    assert!((blocks[0].channels()[0][0] - 800.0 / 32768.0).abs() < 0.00001);
    assert_eq!(stream.seek_ms(20)?, 960);
    let block = stream.next_block()?.ok_or("missing post-seek PCM")?;
    assert_eq!(block.first_frame(), 960);
    assert_eq!(block.frames(), 40);
    assert!(stream.next_block()?.is_none());
    assert!(stream.seek_ms(25).is_err());
    assert_eq!(source.bytes(), original);
    Ok(())
}
#[test]
fn cancelled_foreign_request_invalid_segment_and_header_capacity_should_refuse() -> TestResult {
    let source = source(wave(8000, 8000), "audio/wav")?;
    let request = PreviewRequest::new(&source)?;
    let cancel = PreviewCancellation::default();
    cancel.cancel();
    assert!(matches!(
        AudioStream::open(
            &source,
            &request,
            AudioSegment::default(),
            AudioLimits::default(),
            cancel
        ),
        Err(PreviewError::Cancelled)
    ));
    let foreign = super_source()?;
    assert!(matches!(
        AudioStream::open(
            &source,
            &PreviewRequest::new(&foreign)?,
            AudioSegment::default(),
            AudioLimits::default(),
            PreviewCancellation::default()
        ),
        Err(PreviewError::StaleRequest)
    ));
    assert!(
        AudioStream::open(
            &source,
            &request,
            AudioSegment {
                start_ms: 200,
                end_ms: Some(100)
            },
            AudioLimits::default(),
            PreviewCancellation::default()
        )
        .is_err()
    );
    assert!(
        AudioStream::open(
            &source,
            &request,
            AudioSegment {
                start_ms: 2000,
                end_ms: None
            },
            AudioLimits::default(),
            PreviewCancellation::default()
        )
        .is_err()
    );
    let limits = AudioLimits {
        max_sample_rate: 4000,
        ..Default::default()
    };
    assert!(matches!(
        AudioStream::open(
            &source,
            &request,
            AudioSegment::default(),
            limits,
            PreviewCancellation::default()
        ),
        Err(PreviewError::Limit("audio channels/sample rate"))
    ));
    Ok(())
}
fn super_source() -> Result<PreviewSource> {
    source(wave(8000, 4000), "audio/wav")
}
#[test]
fn cancellation_after_open_should_stop_before_decoding_more_source() -> TestResult {
    let source = source(wave(8000, 8000), "audio/wav")?;
    let cancel = PreviewCancellation::default();
    let mut stream = AudioStream::open(
        &source,
        &PreviewRequest::new(&source)?,
        AudioSegment::default(),
        AudioLimits::default(),
        cancel.clone(),
    )?;
    cancel.cancel();
    assert!(matches!(stream.next_block(), Err(PreviewError::Cancelled)));
    Ok(())
}
#[test]
fn packet_budget_should_refuse_instead_of_silently_truncating_decoded_content() -> TestResult {
    let source = source(wave(8000, 8000), "audio/wav")?;
    let mut stream = AudioStream::open(
        &source,
        &PreviewRequest::new(&source)?,
        AudioSegment::default(),
        AudioLimits {
            max_packet_frames: 1,
            ..Default::default()
        },
        PreviewCancellation::default(),
    )?;
    assert!(matches!(
        stream.next_block(),
        Err(PreviewError::Limit("decoded audio packet/channel geometry"))
    ));
    Ok(())
}

#[test]
fn mismatched_declared_audio_container_should_refuse_without_changing_bytes() -> TestResult {
    let source = source(wave(8000, 8000), "audio/mpeg")?;
    assert!(matches!(
        AudioStream::open(
            &source,
            &PreviewRequest::new(&source)?,
            AudioSegment::default(),
            AudioLimits::default(),
            PreviewCancellation::default()
        ),
        Err(PreviewError::SourceMismatch)
    ));
    Ok(())
}
#[test]
fn fractional_source_boundaries_should_use_one_absolute_rational_seek_position() -> TestResult {
    let source = source(wave(11025, 11025), "audio/wav")?;
    let mut stream = AudioStream::open(
        &source,
        &PreviewRequest::new(&source)?,
        AudioSegment {
            start_ms: 1,
            end_ms: Some(3),
        },
        AudioLimits::default(),
        PreviewCancellation::default(),
    )?;
    assert_eq!(
        (
            stream.description().segment_start,
            stream.description().segment_end
        ),
        (12, Some(34))
    );
    assert_eq!(stream.seek_ms(1)?, 23);
    let block = stream
        .next_block()?
        .ok_or("missing fractional PCM segment")?;
    assert_eq!((block.first_frame(), block.frames()), (23, 11));
    Ok(())
}
