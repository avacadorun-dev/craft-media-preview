## ADDED Requirements

### Requirement: Immutable authorized input

The provider SHALL decode only caller-supplied immutable sources after exact byte-length, SHA-256 and finite source-budget checks. File/URL strings SHALL remain inert. Every result SHALL bind an exact source identity and fresh current request; no deserialization SHALL grant project or I/O authority.

#### Scenario: A result arrives after another project opens
- WHEN a completed media result carries an earlier request even if its bytes match the new source
- THEN the native receiver rejects it and keeps the new source/view unchanged

### Requirement: Actual bounded raster output

PDF and image previews SHALL return actual decoded pixels within independently checked side/pixel/byte bounds, correct alpha metadata and explicit page/error diagnostics. Cancellation SHALL preserve original bytes and never publish a partial successful raster. Empty backend diagnostics SHALL NOT be advertised as complete fidelity.

#### Scenario: A PDF page is requested
- WHEN an author requests an existing page at a valid bounded output size
- THEN real page pixels and page metadata are returned for that source/request without source mutation or browser rendering

### Requirement: Native source playback

Supported original audio/video formats SHALL use real decoder/device data, bounded queues and a consumed-sample playback clock. Play/pause/seek/volume/end and bounded source segments SHALL preserve source bytes and stop stale/cancelled output. No-device or unsupported-codec states SHALL be explicit instead of simulated playback.

#### Scenario: A source segment ends
- WHEN real playback reaches the exact requested source-segment boundary
- THEN audible/video output stops at that boundary and the UI reports the actual completed position rather than advancing a timer beyond the segment
