//! Worker-only bounded audio decode and exact half-open source segments; no device clock.
use crate::{PreviewCancellation, PreviewError, PreviewRequest, PreviewSource, Result};
use craft_audio::{Container, DecodeOptions, Decoder};

/// Exact source segment in milliseconds. The end is exclusive, never an instruction to rewrite audio.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioSegment {
    /// Inclusive source start; fractional frame boundaries round up to the first included frame.
    pub start_ms: u64,
    /// Exclusive end, or the actual source end when absent.
    pub end_ms: Option<u64>,
}
/// Independent finite audio capacities. Decoder packet allocations are checked after backend decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioLimits {
    /// Maximum accepted channel count, independently of source byte size.
    pub max_channels: u16,
    /// Maximum accepted source sample rate.
    pub max_sample_rate: u32,
    /// Maximum decoded frames per codec packet, checked before clipping/copying.
    pub max_packet_frames: usize,
    /// Maximum absolute source duration inspected through this decoder.
    pub max_duration_ms: u64,
}
impl Default for AudioLimits {
    fn default() -> Self {
        Self {
            max_channels: 32,
            max_sample_rate: 384_000,
            max_packet_frames: 65_536,
            max_duration_ms: 24 * 60 * 60 * 1000,
        }
    }
}
impl AudioLimits {
    fn validate(&self) -> Result<()> {
        if self.max_channels == 0
            || self.max_channels > 64
            || self.max_sample_rate == 0
            || self.max_sample_rate > 768_000
            || self.max_packet_frames == 0
            || self.max_packet_frames > 1_048_576
            || self.max_duration_ms == 0
        {
            return Err(PreviewError::Invalid(
                "nonzero finite audio limits are required",
            ));
        }
        Ok(())
    }
}
/// Actual header/segment facts. Unknown total length is not replaced by zero-duration success.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDescription {
    /// Native decoded sample rate, before any device resampling.
    pub sample_rate: u32,
    /// Exact planar channel count in source WAV speaker order.
    pub channels: u16,
    /// Complete source frames, only when headers establish them.
    pub source_frames: Option<u64>,
    /// First source frame included by the selected segment.
    pub segment_start: u64,
    /// Exclusive source frame end; absent until EOF for an unbounded unknown-length source.
    pub segment_end: Option<u64>,
}
/// One complete bounded planar decoded packet or packet slice. This is not audible-device progress.
#[derive(Debug)]
pub struct AudioBlock {
    request: PreviewRequest,
    first_frame: u64,
    channels: Vec<Vec<f32>>,
    damaged_packets: u64,
}
impl AudioBlock {
    /// Exact current immutable source/request.
    pub fn request(&self) -> &PreviewRequest {
        &self.request
    }
    /// Absolute source frame of the first included sample.
    pub fn first_frame(&self) -> u64 {
        self.first_frame
    }
    /// Frames per channel; all channel lengths are equal and nonempty.
    pub fn frames(&self) -> usize {
        self.channels.first().map_or(0, Vec::len)
    }
    /// Source-ordered planar samples; never mutable original encoded bytes.
    pub fn channels(&self) -> &[Vec<f32>] {
        &self.channels
    }
    /// Backend-reported damaged/concealed packets so far; hosts must retain a visible warning.
    pub fn damaged_packets(&self) -> u64 {
        self.damaged_packets
    }
}
/// Worker-owned streaming decoder. No GUI, device, implicit path or URL access.
pub struct AudioStream {
    decoder: Decoder<'static>,
    request: PreviewRequest,
    description: AudioDescription,
    limits: AudioLimits,
    decoded_frame: u64,
    floor: u64,
    segment_start_ms: u64,
    exhausted: bool,
    cancel: PreviewCancellation,
}
impl AudioStream {
    /// Open exact authorized bytes. Blocking codec operations remain on the caller's worker.
    pub fn open(
        source: &PreviewSource,
        request: &PreviewRequest,
        segment: AudioSegment,
        limits: AudioLimits,
        cancel: PreviewCancellation,
    ) -> Result<Self> {
        request.validate_source(source)?;
        cancel.check()?;
        limits.validate()?;
        if segment.start_ms > limits.max_duration_ms
            || segment
                .end_ms
                .is_some_and(|end| end <= segment.start_ms || end > limits.max_duration_ms)
        {
            return Err(PreviewError::Invalid(
                "audio segment must be nonempty and within duration limits",
            ));
        }
        let hint = match source.stamp().media_type.as_str() {
            "audio/mpeg" | "audio/mp3" => "mp3",
            "audio/wav" | "audio/x-wav" => "wav",
            "audio/mp4" | "video/mp4" => "mp4",
            "audio/ogg" => "ogg",
            "video/webm" | "audio/webm" => "webm",
            "audio/flac" => "flac",
            "audio/aac" => "aac",
            "audio/aiff" => "aiff",
            other => return Err(PreviewError::Unsupported(other.into())),
        };
        let decoder = Decoder::open(source.bytes.clone(), Some(hint), &DecodeOptions::default())
            .map_err(|error| PreviewError::Audio(error.to_string()))?;
        cancel.check()?;
        let info = decoder.info();
        let matches_mime = match hint {
            "mp3" => info.container == Container::MpegAudio,
            "wav" => info.container == Container::Wav,
            "mp4" => info.container == Container::Mp4,
            "ogg" => info.container == Container::Ogg,
            "webm" => info.container == Container::WebM,
            "flac" => info.container == Container::Flac,
            "aac" => info.container == Container::Adts,
            "aiff" => info.container == Container::Aiff,
            _ => false,
        };
        if !matches_mime {
            return Err(PreviewError::SourceMismatch);
        }
        if info.sample_rate == 0
            || info.sample_rate > limits.max_sample_rate
            || info.channels == 0
            || info.channels > limits.max_channels
        {
            return Err(PreviewError::Limit("audio channels/sample rate"));
        }
        let known = decoder.length_known().then_some(info.frames);
        let start = frame_at_or_after(segment.start_ms, info.sample_rate)?;
        let mut end = segment
            .end_ms
            .map(|ms| frame_at_or_after(ms, info.sample_rate))
            .transpose()?;
        if let Some(known) = known {
            if known > frame_at_or_after(limits.max_duration_ms, info.sample_rate)? {
                return Err(PreviewError::Limit("audio duration"));
            }
            end = Some(end.map_or(known, |end| end.min(known)));
        }
        if end.is_some_and(|end| start >= end) {
            return Err(PreviewError::Invalid(
                "audio segment begins beyond source end",
            ));
        }
        let description = AudioDescription {
            sample_rate: info.sample_rate,
            channels: info.channels,
            source_frames: known,
            segment_start: start,
            segment_end: end,
        };
        let mut stream = Self {
            decoder,
            request: request.clone(),
            description,
            limits,
            decoded_frame: 0,
            floor: start,
            segment_start_ms: segment.start_ms,
            exhausted: false,
            cancel,
        };
        if start > 0 {
            stream.seek_absolute(start)?;
        }
        Ok(stream)
    }
    /// Actual source format and selected half-open segment.
    pub fn description(&self) -> &AudioDescription {
        &self.description
    }
    /// Seek relative to the selected segment, after codec preroll and before the next exact block.
    /// The returned absolute frame is the first requested output frame, not a device timestamp.
    pub fn seek_ms(&mut self, relative_ms: u64) -> Result<u64> {
        self.cancel.check()?;
        let absolute_ms = self
            .segment_start_ms
            .checked_add(relative_ms)
            .ok_or(PreviewError::Limit("audio seek"))?;
        let target = frame_at_or_after(absolute_ms, self.description.sample_rate)?;
        if self
            .description
            .segment_end
            .is_some_and(|end| target >= end)
            || target
                > frame_at_or_after(self.limits.max_duration_ms, self.description.sample_rate)?
        {
            return Err(PreviewError::Invalid(
                "audio seek is outside the selected segment",
            ));
        }
        self.seek_absolute(target)?;
        Ok(target)
    }
    fn seek_absolute(&mut self, target: u64) -> Result<()> {
        self.cancel.check()?;
        let actual = self
            .decoder
            .seek(target)
            .map_err(|error| PreviewError::Audio(error.to_string()))?;
        self.cancel.check()?;
        if actual > target {
            return Err(PreviewError::Audio(
                "decoder resumed after the requested source sample".into(),
            ));
        }
        self.decoded_frame = actual;
        self.floor = target;
        self.exhausted = false;
        Ok(())
    }
    /// Decode a bounded packet and clip to the exact segment/seek target, or report actual EOF.
    /// A damaged source remains explicitly observable through the block's damaged packet count.
    pub fn next_block(&mut self) -> Result<Option<AudioBlock>> {
        for _ in 0..1024 {
            self.cancel.check()?;
            if self.exhausted
                || self
                    .description
                    .segment_end
                    .is_some_and(|end| self.decoded_frame >= end)
            {
                self.exhausted = true;
                return Ok(None);
            }
            let Some(block) = self
                .decoder
                .next_block()
                .map_err(|error| PreviewError::Audio(error.to_string()))?
            else {
                self.cancel.check()?;
                self.exhausted = true;
                return Ok(None);
            };
            self.cancel.check()?;
            if self.decoder.info().sample_rate != self.description.sample_rate {
                return Err(PreviewError::Audio(
                    "source sample rate changed during decoding".into(),
                ));
            }
            let count = block.frames();
            if count > self.limits.max_packet_frames
                || block.channels.len() != usize::from(self.description.channels)
                || block.channels.iter().any(|channel| {
                    channel.len() != count || channel.iter().any(|sample| !sample.is_finite())
                })
            {
                return Err(PreviewError::Limit("decoded audio packet/channel geometry"));
            }
            let first = self.decoded_frame;
            let after = first
                .checked_add(count as u64)
                .ok_or(PreviewError::Limit("audio frame capacity"))?;
            if after > frame_at_or_after(self.limits.max_duration_ms, self.description.sample_rate)?
            {
                return Err(PreviewError::Limit("audio duration"));
            }
            self.decoded_frame = after;
            let begin = first.max(self.floor);
            let end = after.min(self.description.segment_end.unwrap_or(after));
            if begin >= end {
                continue;
            }
            let offset = usize::try_from(begin - first)
                .map_err(|_| PreviewError::Limit("audio packet offset"))?;
            let length = usize::try_from(end - begin)
                .map_err(|_| PreviewError::Limit("audio packet length"))?;
            let channels = block
                .channels
                .into_iter()
                .map(|channel| channel[offset..offset + length].to_vec())
                .collect();
            return Ok(Some(AudioBlock {
                request: self.request.clone(),
                first_frame: begin,
                channels,
                damaged_packets: self.decoder.info().damaged,
            }));
        }
        Err(PreviewError::Limit("audio preroll/no-progress packets"))
    }
}
fn frame_at_or_after(ms: u64, rate: u32) -> Result<u64> {
    let frames = (u128::from(ms) * u128::from(rate)).div_ceil(1000);
    u64::try_from(frames).map_err(|_| PreviewError::Limit("audio timestamp capacity"))
}
