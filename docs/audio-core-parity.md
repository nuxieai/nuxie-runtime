# Audio core parity boundary

P2F1 ports the pinned external/headless audio build. It has no device sink.

The exact contract covers the engine sample rate/channel count, absolute PCM
frame clock, start/end clipping windows, source cursor, pause/resume/seek,
completion and deferred disposal, sound volume, peak-positive levels, manual
PCM pull/sum, and artboard-tagged stop/unlink behavior.

D18 is deliberately narrower:

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

The live `--audio-oracle` probe uses three 512-frame pulls. It requires exact
silent/active/silent scheduling windows and completion/frame-clock state while
applying the two-frame tolerance only to resampled reader lengths.
