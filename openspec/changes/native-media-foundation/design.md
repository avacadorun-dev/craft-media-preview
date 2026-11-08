# Design

Use the architecture and dependency decisions in docs/. PDF and image calls run on caller-owned workers, return complete finite rasters and never mutate original bytes. Explicit cancellation and request identities prevent late results replacing another source. Native textures have one bounded current result; no decoder or hidden I/O runs during paint. Rich application workflows stay outside this repository.
