# Verification

Status: source foundation, no qualification or host-native acceptance yet. All fixtures must be synthetic.

Require exact hosted Rust1.95 committed-lock default/all-feature tests, rustfmt and strict Clippy on macOS/Windows/Linux. Decoder contracts include real coloured PDF pages, image bytes/dimensions, malformed/truncated input, cancellation, exact source preservation, finite pixel/byte bounds and stale/foreign request refusal. Future audio/video contracts require actual decoded data/timestamps/sample consumption, seek/end/segment/no-device behavior and cross-source cancellation.

A successful raster or decode does not establish complete PDF fidelity, native OS audio output, audiovisual synchronisation, IME/accessibility or host design acceptance. Source/project media remain private and read-only; no original archive, media or manuscript is a public fixture.


## First hosted lock resolution

Sourcef419133/run42501566440 resolved the initial dependency graph only on GitHub. Its lock gate deliberately refused qualification until the generated lock was reviewed and committed. The selected graph has one egui0.36.2 and no eframe/WebView/FFmpeg runtime; Craft PDF and all six required backend patches use exact0a9d1f48. The next source commits this actual hosted lock; compile/tests/native acceptance remain unqualified until that new exact run passes.


## First provider compilation

Exactcd3a0b1/run42501654259 compiled the PDF/image/native texture providers on macOS and Linux. All actual provider/texture and minimal-input tests passed; strict Clippy correctly blocked the checkpoint because the synthetic fixed PDF object list used an unnecessaryVec. The next focused increment uses an array and removes minimal-feature dead-code warnings without weakening checks. This source is not qualified for host adoption until the new exact all-platform run succeeds.
