# Audio core parity boundary

P2F1 and P2F2 port the pinned external/headless audio build, AudioEvent
activation, and Artboard volume/lifecycle integration. It has no device sink.

The exact contract covers the engine sample rate/channel count, absolute PCM
frame clock, start/end clipping windows, source cursor, pause/resume/seek,
completion and deferred disposal, sound volume, peak-positive levels, manual
PCM pull/sum, and artboard-tagged stop/unlink behavior. `AudioEvent.assetId`
resolves as a dense file-asset ordinal; playback multiplies AudioAsset and
Artboard volume, schedules at the engine's current PCM frame during event
recursion unwind, falls back to the retained 2-channel/48 kHz runtime engine
when no external Artboard engine is configured, and ignores the reported event
delay exactly like C++.

D17 is deliberately narrower:

- Symphonia replaces miniaudio only for encoded decode, channel conversion,
  and resampling.
- WAV, MP3, and FLAC are accepted by this pinned feature set. An `OggS`
  stream is recognized as Vorbis but rejected as unsupported because the
  pinned Rive build does not wire its optional Vorbis decoder.
- Native WAV metadata and native frame count are exact. A resampled frame
  count may differ from the pinned miniaudio oracle by at most two frames.
- PCM bytes and individual sample arrays are never equality-pinned. Offline
  differentials compare silence/activity windows, channel peak presence, and
  higher-level energy/envelope behavior instead.

The live `--audio-oracle` probe takes the pinned WAV and MP3 fixtures. It uses
three 512-frame pulls and requires exact silent/active/silent scheduling
windows, completion/frame-clock and lifecycle state, buffered duration, MP3
duration/cache behavior, and twenty-handle engine-outliving behavior while
applying the two-frame tolerance only to resampled reader lengths. The pinned
level-monitor API is compiled only under `WITH_RIVE_AUDIO_TOOLS`, so the
ordinary provenance-bound external-audio probe cannot call it; the matching
peak-positive/reset behavior remains a direct Rust test.

The live `--audio-riv-oracle` probe loads `sound.riv` and `sound2.riv` in both
runtimes. It compares dense and semantic asset identities, decoded-source
availability, playback count, multiplied volume, zero-volume suppression,
runtime-engine fallback, Artboard-scoped cleanup, nested engine/volume
propagation, and direct/nested/no-audio queries. Lua audio and CPAL device output
remain later packages.
