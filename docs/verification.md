# Verification

Status: independently qualified raster and optional source-audio slices; host/native output acceptance remains pending. All fixtures are synthetic.

Require exact hosted Rust1.95 committed-lock default/all-feature tests, rustfmt and strict Clippy on macOS/Windows/Linux. Decoder contracts include real coloured PDF pages, image bytes/dimensions, malformed/truncated input, cancellation, exact source preservation, finite pixel/byte bounds and stale/foreign request refusal. Future audio/video contracts require actual decoded data/timestamps/sample consumption, seek/end/segment/no-device behavior and cross-source cancellation.

A successful raster or decode does not establish complete PDF fidelity, native OS audio output, audiovisual synchronisation, IME/accessibility or host design acceptance. Source/project media remain private and read-only; no original archive, media or manuscript is a public fixture.


## First hosted lock resolution

Sourcef419133/run42501566440 resolved the initial dependency graph only on GitHub. Its lock gate deliberately refused qualification until the generated lock was reviewed and committed. The selected graph has one egui0.36.2 and no eframe/WebView/FFmpeg runtime; Craft PDF and all six required backend patches use exact0a9d1f48. The next source commits this actual hosted lock; compile/tests/native acceptance remain unqualified until that new exact run passes.


## First provider compilation

Exactcd3a0b1/run42501654259 compiled the PDF/image/native texture providers on macOS and Linux. All actual provider/texture and minimal-input tests passed; strict Clippy correctly blocked the checkpoint because the synthetic fixed PDF object list used an unnecessaryVec. The next focused increment uses an array and removes minimal-feature dead-code warnings without weakening checks. This source is not qualified for host adoption until the new exact all-platform run succeeds.

## Exact qualified slices

| Exact source | Hosted evidence | Actual accepted engineering contracts | Remaining boundary |
| --- | --- | --- | --- |
| `f3705126177b694c4609f4e66799021000618bee` | [42501927770](https://github.com/avacadorun-dev/craft-media-preview/actions/runs/42501927770), all three operating systems, Rust 1.95 | Real PDF/image pixels, page list, immutable bytes/requests/cancellation, egui stale texture refusal, minimal boundary, fmt/strict Clippy/unchanged lock | Full PDF fidelity, native host input, animation and audio/video |
| `246350ebb72f052c31a01b40afd256000d64c1b4` | [42504549352](https://github.com/avacadorun-dev/craft-media-preview/actions/runs/42504549352), all three operating systems, Rust 1.95 | Prior raster contracts plus six real PCM source/segment/seek/fractional-boundary/cancellation/capacity/MIME contracts, minimal boundary, fmt/strict Clippy/unchanged lock | MP3/AAC/Vorbis/Opus corpus, native device output/clock, resampling and synchronized video/host UX |

The first audio run 42504161031 passed all actual decoder/raster tests on three platforms, then failed the minimal-feature test-crate documentation lint. Required-feature test manifest gating fixed that specific failure in 246350e; the exact subsequent run is green without weakening compiler/lint checks. The reviewed audio lock SHA-256 is `fb806cbb6d771ba8367dff1503102677fe2088b1a862c2a344d24f7c231bb1bc`, 143→165 packages with no prior package identity upgraded/removed. Source decoder cursors are never reported as audible playback.
