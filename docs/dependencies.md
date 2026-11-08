# Dependency review

Primary sources reviewed2026-11-08. Selected source pins are immutable; qualification and binary/graph measurements remain separate requirements.

| Dependency | Role and license | Boundary |
| --- | --- | --- |
| [craft-pdf0a9d1f48](https://github.com/storytold/craft-libs/blob/0a9d1f48da9e0e61a2c7b602ba96ef1c4672bcd9/crates/pdf/Cargo.toml) | Reader/raster only, MIT OR Apache-2.0, Rust1.95 | Enable raster; retain identical root patches for hayro/hayro-interpret/hayro-syntax/hayro-jbig2/hayro-cmap/hayro-postscript. Do not enable writing or upstream egui0.37. |
| [Craft PDF contract](https://github.com/storytold/craft-libs/blob/0a9d1f48da9e0e61a2c7b602ba96ef1c4672bcd9/crates/pdf/README.md) | Actual open/cancel/raster diagnostics | Input defaults512MiB; output backend8192side/64Mpx; wrapper applies its own lower requested output budget. Per-stream decoder limits do not bound all parser scratch memory or execution time. Retain embedded font/CMap licenses. |
| image0.25.10 | Existing Rust image decoder graph, MIT OR Apache-2.0 | PNG/JPEG/GIF/WebP only, no network or source mutation. Host actual animation workflow remains unfinished. |
| egui0.36.2 | Host-compatible native presentation, MIT OR Apache-2.0 | No eframe/window loop, file picker or second application. |
| SHA-20.10.9/thiserror2.0.21 | Exact input identity/typed errors, MIT OR Apache-2.0 | Reused mature narrow utilities. |

## Audio/video candidates, not yet dependencies

[craft-audio](https://github.com/storytold/craft-libs/blob/0a9d1f48da9e0e61a2c7b602ba96ef1c4672bcd9/crates/audio/Cargo.toml) with `decode` supplies native AAC/PCM and Symphonia0.6.1 MP3/other decoders. Symphonia is MPL-2.0; keep its source unmodified and distribute its license/source reference. [craft-media](https://github.com/storytold/craft-libs/blob/0a9d1f48da9e0e61a2c7b602ba96ef1c4672bcd9/crates/media/Cargo.toml) provides MP4/Matroska/Ogg container data, not playback by itself. Both use MIT OR Apache-2.0/Rust1.95 and must use the same source revision.

[FilmCraft H.2644a56be9c](https://github.com/storytold/filmcraft/blob/4a56be9cff419a5a9b16a25259c97dc823ca8bae/crates/h264/Cargo.toml), default features off, is a narrow pure Rust decoder with its bitstream crate; do not adopt the whole codec hub or app. Its documented progressive8bit4:2:0 Baseline/Main/High scope must be tested against actual reference-compatible synthetic MP4. Original WebM/Ogg and supported codecs remain required; an H.264 specimen cannot establish their acceptance.

[CPAL0.18.2](https://github.com/RustAudio/cpal/blob/v0.18.2/Cargo.toml) is Apache-2.0, Rust1.85, with CoreAudio/WASAPI/ALSA native backends. Its current documentation lists macOS14.2 for CoreAudio, conflicting with the host's14.0 baseline. Do not silently adopt/raise the platform minimum. Qualify an appropriate output backend first. Disable ASIO/JACK/realtime options; Linux ALSA development/runtime dependencies and actual installed-package behavior need qualification. A Rust wrapper over system audio APIs does not imply no system dependencies.

No local Cargo resolution, build or test is permitted in this task. Hosted qualification must retain source/compiler/lock/features, dependency notices and measured release-size deltas before host adoption.
