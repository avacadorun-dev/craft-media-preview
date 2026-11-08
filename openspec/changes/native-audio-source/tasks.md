- [x] Source-audit original audio MIME/source-segment workflow and exact decoder APIs.
- [x] Implement immutable request-bound audio decoding, clipping and seek.
- [x] Add actual synthetic PCM sample/segment/seek, capacity and cancellation contracts.
- [x] Resolve/review the actual hosted lock and qualify all three operating systems (`246350e`, run 42504549352); six actual PCM source contracts and prior raster/minimal/fmt/strict Clippy checks passed.
- [ ] Qualify actual MP3/AAC/Vorbis/Opus corpus and matching license/source notices.
- [ ] Implement bounded native output ring, actual consumed-device progress, resampling and no-device/error states.
- [ ] Implement original MP4/WebM video sync and host source-segment controls, then independent native acceptance.

Hosted resolver run 42504034454 at 23da296 retained the actual lock with SHA-256 `fb806cbb6d771ba8367dff1503102677fe2088b1a862c2a344d24f7c231bb1bc`. The graph adds 22 packages (143→165), removing/upgrading no prior package identities. Exact 0a9d1f48 CraftAudio/CraftMedia and unmodified Symphonia 0.6.1 decoder/container packages are the additions. Device/output/FFmpeg/egui-app dependencies are absent. The resolver's intentional stale-lock failure is not test qualification.
