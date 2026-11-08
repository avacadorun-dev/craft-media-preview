# CraftMediaPreview

Reusable native Rust media previews for desktop authoring tools. Hosts retain project ownership, immutable storage, persistence, localization and navigation; this library decodes only explicitly supplied bytes and emits source-bound display data.

**Status:** alpha.3 foundation work for [WriterCraft](https://github.com/avacadorun-dev/writer-craft). The first implementation covers bounded image/PDF raster previews and stale-request-safe egui presentation. Exact raster source `f370512` and optional audio-source decoder `246350e` passed hosted Rust 1.95 checks on macOS, Windows and Linux. The `audio` feature adds worker-only exact source-segment decode/seek; native audio/video output and host source-segment acceptance remain required work; this is not a full media player or a completed host integration.

- Rust-only decoder/provider boundary; no WebView or FFmpeg runtime.
- Immutable hash-checked source bytes, bounded output and explicit cooperative cancellation.
- Native premultiplied PDF rasters and truthful backend diagnostics.
- Optional native AAC/PCM and Symphonia-backed audio source decoding, exact half-open segments and seek; no invented device playback clock.
- Eight-language application labels belong to the host, not an embedded second UI catalog.

See [architecture](docs/architecture.md), [dependency decisions](docs/dependencies.md), [verification policy](docs/verification.md) and the [active specification](openspec/changes/native-media-foundation/specs/media-preview/spec.md).

MIT for this library. Dependencies retain their own licenses; the pinned Craft PDF/Hayro stack is MIT OR Apache-2.0 with separate embedded font/CMap notices. The optional audio decoder includes unmodified MPL-2.0 Symphonia dependencies; retain their source/license notices. No upstream application is copied here.
