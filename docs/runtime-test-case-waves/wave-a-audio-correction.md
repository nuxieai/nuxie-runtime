# Wave A audio correction

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: only rejected cases 1, 2, 3, 4, 5, 7, 8, 9, and 11 in
`tests/unit_tests/runtime/audio_test.cpp`. Accepted cases 6, 10, 12, and 13
remain unchanged.

The independent Wave A review correctly rejected the original evidence. Case
1 pointed to decode rather than engine construction; case 2 omitted the three
reader configurations, level monitor, and playback pulls; cases 4 and 5 used
an artboard-clone proxy instead of sound handles outliving their engine; cases
7 through 9 omitted the direct `AudioEvent` counts; and case 11 did not call
the duration owner a second time.

The correction uses the approved native-audio adaptation without changing the
tested behavior. Nine case-specific Rust tests now execute the pinned fixture,
action order, lifetime boundary, and assertions through the live `AudioEngine`,
`AudioSource`, imported `File`, and artboard owners. In particular:

- case 1 constructs the two-channel, 44.1 kHz engine;
- case 2 opens `audio/what.wav`, constructs all three readers, reads initial
  levels, plays the source, performs both 512-frame pulls, reads both forms of
  post-playback levels, and finally asserts the three pinned frame counts;
- case 3 imports `sound.riv`, asserts one direct `AudioEvent`, resolves its
  authored `AudioAsset`, and verifies its decoded source;
- cases 4 and 5 retain respectively one and twenty live sound handles across
  engine destruction before stopping them;
- cases 7 through 9 import the named `sound2.riv` artboards and assert both the
  exact direct-event count and `hasAudio` result; and
- case 11 asserts channels, sample rate, duration, and the second cached
  duration call.

Case 2 is an executable expected-red port, not a metadata placeholder. Running
it reaches the final exact frame-count assertion after every translated reader,
monitor, playback, and pull action. The native Rust readers currently report
`[9688, 10545, 7030]`; pinned C++ reports `[9688, 10544, 7029]`. This test-port
phase records that concrete resampling seam without changing production code
or treating the mismatch as an approved backend result.

Focused verification:

```text
CARGO_INCREMENTAL=0 cargo test -p nuxie --test upstream_audio
running 9 tests
test result: ok. 8 passed; 0 failed; 1 ignored

CARGO_INCREMENTAL=0 cargo test -p nuxie --test upstream_audio \
  upstream_audio_case_02_source_reader_levels_and_playback -- --ignored --exact
running 1 test
left:  [9688, 10545, 7030]
right: [9688, 10544, 7029]
test result: FAILED. 0 passed; 1 failed
```

This correction resolves only the nine audio mappings named above. It does not
self-certify broad Wave A or alter any production behavior.
