Implemented P2F1 audio core and left the worktree uncommitted.

Key deliverables:

- Symphonia WAV/MP3/FLAC decoding with owned bytes, buffered sources, readers, resampling, and recognized-but-unwired Vorbis.
- Headless frame-clock mixer, scheduling/clipping, PCM pull, completion, levels, volume, lifecycle, and per-artboard stop.
- AudioAsset embedded/host loading, `has_audio`, and `Factory::decode_audio`.
- D18 approved adaptation and explicit ±2 resampled-frame tolerance; PCM is never byte-pinned.
- AudioEvent firing, Lua audio, and device output remain unwired.
- C++ headless oracle rebuilt and exercised successfully.

Gates:

- PASS `make runtime-frame-loop-port-check`
- PASS `make rust-attribution-check`
- PASS formatting, lockfile metadata, offline engine tests, and workspace type-check through the local API stub
- BLOCKED `cargo test -p nuxie-runtime` and `cargo test -p nuxie`: DNS could not resolve `index.crates.io` to download Symphonia
- `port-manifest-check` unit tests pass; inventory validation encounters unrelated existing `core_uint64_type.cpp` drift

Full details: [P2F1-report.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/P2F1-report.md) and [audio-core-parity.md](/Users/levi/dev/worktrees/nuxie-p2f-audio/docs/audio-core-parity.md).