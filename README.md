# CraftMediaPreview

Reusable native Rust media previews for desktop authoring tools. Hosts retain project ownership, immutable storage, persistence, localization and navigation; this library decodes only explicitly supplied bytes and emits source-bound display data.

**Status:** alpha.3 foundation work for [WriterCraft](https://github.com/avacadorun-dev/writer-craft). The first implementation covers bounded image/PDF raster previews and stale-request-safe egui presentation. Hosted qualification is pending. Audio/video playback and source-segment acceptance remain required work; this is not a full media player or a completed host integration.

- Rust-only decoder/provider boundary; no WebView or FFmpeg runtime.
- Immutable hash-checked source bytes, bounded output and explicit cooperative cancellation.
- Native premultiplied PDF rasters and truthful backend diagnostics.
- Eight-language application labels belong to the host, not an embedded second UI catalog.

See [architecture](docs/architecture.md), [dependency decisions](docs/dependencies.md), [verification policy](docs/verification.md) and the [active specification](openspec/changes/native-media-foundation/specs/media-preview/spec.md).

MIT for this library. Dependencies retain their own licenses; the pinned Craft PDF/Hayro stack is MIT OR Apache-2.0 with separate embedded font/CMap notices. No upstream application is copied here.
