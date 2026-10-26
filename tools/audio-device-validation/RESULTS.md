# Device qualification — 2026-10-25

Production source: `99961425f09ab797cc7f950a7c5367d91fd38ffe` (audio port PR #1191).
This validation adds only standalone hosts, fixture code, and build/run instructions.
No production runtime fixes, SDK releases, dependency repins, or Luau changes were needed.

| Target | Configuration | Result |
| --- | --- | --- |
| MacBook Pro, built-in output / CPAL | 48 kHz stereo, requested and actual blocks 256 and 1024; separate open/close sessions | Both passed; live parameter change, detach, asset continuation, final silence, stopped clock after teardown |
| Physical iPhone 17 Pro Max, wireless install/launch, iOS 26.6.2 | 48 kHz stereo, 256-frame callbacks | Passed; 1,176 callbacks / 301,056 frames, zero invalid samples/layout errors/clock errors, renderer destroyed once |
| Android arm64 API 36 emulator | 48 kHz stereo, 256-frame Rust blocks, 512-frame application pulls | Two fresh launches passed; 192,000 frames written/played each; underruns 0 before final drain, 1 afterward |
| Shared fixture on Linux pool | `//tools/audio-device-validation:probe_test` | 1 test passed |

All four phases ran on every target: custom renderer plus decoded PCM WAV,
changed renderer parameters, renderer detached with asset continuing, stopped
asset with exact silence. See `results/` for the measured output records.

Builds ran through the pool, including when the scheduler selected the caller Mac
as a pool worker. Successful pool jobs:

- Linux fixture: `31e5541c-10ce-41a8-b24e-432625263414`.
- Mac executable: `3d01de5d-64a6-43ba-ae56-9956ec56ff42`.
- Final iOS executable: `5a8f5790-0c9e-465c-ac63-7459ed13f435` (installed Xcode 27.0 toolchain).
- Final Android APK: `0eef3a9a-f226-41e2-8a65-366e37eeec66` (build tools 37.0.0).
- Android shared library: `50e69b65-9904-4aa5-9128-4116b2d694cb`.

## Blockers resolved in the test hosts

- iOS needed CoreGraphics linked and noninterleaved stereo buffers for
  AVAudioSourceNode. A preallocated conversion buffer adapts the production
  interleaved mixer output; the adapter was independently reviewed.
- Android packaging now locates complete installed build tools and the installed
  Homebrew JDK instead of assuming SDK/JDK locations present on every worker.
- Android originally started with an empty track and reported one underrun. The
  final host writes an application buffer before starting and records underruns
  before and after its deliberate final drain. Those counters are observations,
  not an acoustic glitch guarantee.
- Registering the test host as a Cargo workspace member would require unrelated
  dependency-universe regeneration. Standalone Bazel targets use the shipping
  source and existing pins instead; generated repository metadata checks pass.

The common fixture/host integration, standalone build targets, planar iOS adapter,
Android packaging, and final Android startup/telemetry changes received independent
review. No assertion or tolerance was weakened to obtain a pass.

## Limits

These runs validate the custom-renderer/mixer output path and normal teardown in
standalone native hosts. They do not certify shipping SDK/view lifecycles, Bluetooth,
route changes, interruptions, backgrounding, acoustic quality, long-duration
stability, or physical Android hardware. Input decoding uses a generated PCM WAV;
this is not a full RIV-file integration run. Existing upstream/unit coverage remains
separate evidence. No Lua code executes in the audio callback.
