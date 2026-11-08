# Decoder authority and timeline

Use CraftAudio at exact `0a9d1f48da9e0e61a2c7b602ba96ef1c4672bcd9`, `decode` only. Native AAC/PCM plus unmodified MPL-2.0 Symphonia 0.6.1 handle the admitted corpus; retain upstream source/license provenance. No upstream UI, file lookup, network, decoder callback work or new app runtime. Opus and original WebM video remain explicit separate codec requirements; a PCM contract cannot establish those formats.

The immutable source and fresh PreviewRequest validate before decoder construction. A caller-owned cancellation token is checked before/after every backend operation. Source frames are absolute rational millisecond boundaries rounded up; segments are half-open. Seek uses the decoder's actual resume frame, rejects late resume and discards earlier preroll before returning exact requested samples. Unknown duration remains None; actual EOF is distinct from a zero header. Damaged/concealed packets are observable in every returned block.

One bounded backend packet is decoded on a worker then checked for channel geometry, finite samples and capacity before clipping/copying. These post-decode bounds do not falsely promise universal allocator or codec-call preemption. No device sample clock or playback success is reported by the source cursor.
