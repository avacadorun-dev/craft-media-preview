# Native audio source decoding

## Why

Original Canvas audio and manuscript source segments need actual decoded samples, precise clipping and seek before native device playback can be qualified.

## What changes

Add a separately gated worker-only streaming audio provider over exact immutable requests, using the reviewed CraftAudio native/Symphonia decoder. Bound channel/rate/packet/duration capacities, retain unknown length and damaged-packet diagnostics, and clip exact half-open source segments without changing original encoded bytes.

## Scope

This source decoder is not device output, audible progress, Opus-complete codec coverage or complete host playback. Those remain explicit follow-up requirements.
