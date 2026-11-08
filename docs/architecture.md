# Architecture

## ADR001: host-authorized immutable input

Accepted2026-11-08. `PreviewSource` verifies exact original length, SHA-256 and descriptive MIME against a caller-supplied source budget. Its bytes are immutable and shared; constructing it grants no filesystem/network/project capability. The host must acquire bytes through its own fixed-project authority. Requests use fresh non-deserializable process identities. A host or widget accepts a result only for its exact current request and source identity; identical bytes in another project do not make an old request current.

## ADR002: bounded workers and disposable presentation

Accepted2026-11-08. Provider calls are synchronous building blocks for host-owned bounded workers, never operations performed inside an audio callback or native paint loop. Source, raster dimensions/byte count and decoder stream expansion have independent finite bounds. Cooperative cancellation binds parser/interpreter operations, but external codec calls and allocator/process failure are not falsely described as universally preemptible. Cache/parser/device state is disposable after failure. Original sources remain exportable.

## ADR003: PDF first native slice

Accepted2026-11-08. Reuse exact Craft PDF/Hayro reader/raster crates rather than an upstream app or C++ PDF runtime. The wrapper exposes pixel buffers, page metadata, source/request identity and actual backend diagnostics. It preserves ordinary PDF colours and alpha; host theme does not rewrite source content. Empty diagnostics are not a fidelity certificate. Image decoding uses the existing pure Rust image graph and explicit allocation/dimension limits. Initial image preview is the first frame, with animation remaining a separately stated playback requirement.

## ADR004: audio/video implementation boundary

Planned, not implemented. Native MP3/AAC and other original formats require a bounded decoder worker, bounded audio ring, device-consumed sample clock, resampling/channel policy, exact seek/end/segment boundaries and latest-frame video queue. Device absence is an explicit state, never simulated playback. Craft audio/media plus independently gated FilmCraft codecs are the current narrow candidates; no codec hub, upstream UI or browser runtime is selected. Device backend adoption must preserve WriterCraft's existing macOS14.0 baseline rather than silently raising it.
