# Video qualification harness

This is a development proof, not a shipping SDK or a completion claim for the
video feature. It imports a real extended scene, binds a platform player to the
Video occurrence, uploads frames to the same persistent renderer factory, and
checks rendered pixels against the authored red/blue fixture. It exercises
seek/completion/disposal on Apple and Android and completion on browsers.
Default runs are muted; Apple also has an explicitly requested audible mode.

## Apple

`cargo run -p video-qualification` runs the Metal proof on macOS. Device apps
advance one tick per timer callback so AVFoundation's main-queue work is never
blocked by a synchronous test loop.

```sh
VIDEO_PROOF_DEVELOPMENT_TEAM=<team> bash tools/video-qualification/apple/build.sh
```

The generated app is under `tools/video-qualification/apple/DerivedData/`.
Install/launch it with `xcrun devicectl` against the explicitly selected device.
Its Documents/video-proof-result.txt contains the outcome. Pass `--audible`
as an application launch argument to enable the AAC tone at 20% volume. Source contains no
signing identity or device ID. Cross builds use rustup's installed targets and
an explicit Apple SDK for bindgen. Generated Xcode projects are ignored.

## Browser

```sh
bash tools/video-qualification/browser/build.sh
bash tools/video-qualification/browser/build-webgpu.sh
python3 -u tools/video-qualification/browser/serve.py --directory target/video-browser-proof
```

Open the printed localhost URL for WebGL2, or its `/webgpu/` child for WebGPU.
Append `?embedded=1` to exercise an MP4 embedded in the imported scene and
resolved through a Blob URL. The page displays a pass/failure and submitted frame count. These are owned
local fixture pages; no provider requests or credentials are involved. The
WebGPU proof uses the runtime's existing Dawn host and submits explicit
readback in the same browser task as rendering. The frame count is not a
performance qualification or an assertion of hardware decoding.

## Android

```sh
ANDROID_HOME=/path/to/android-sdk bash tools/video-qualification/android/build.sh
adb -s <selected-device> install -r target/video-android-proof/video-proof.apk
adb -s <selected-device> shell am start -n ai.nuxie.videoqualification/.MainActivity
adb -s <selected-device> shell run-as ai.nuxie.videoqualification cat files/video-proof-result.txt
```

The proof uses MediaPlayer with a private SurfaceTexture and bounded RGBA
readback, followed by the runtime's Vulkan upload and scene draw. It does not
use a VideoView overlay. The build generates a local debug signing key under
ignored `target/`; that key is only for this disposable qualification app.

## Evidence so far (2026-09-16)

- macOS AVFoundation -> `.nux` -> Metal: red/blue pixel checks, seek, completion,
  disposal passed with 22 submitted frames.
- Levi's Bat Phone, iPhone 17 Pro Max: same Metal proof passed with 22 frames.
  Repeated with AAC enabled at 20% volume and runtime-managed mix policy; Levi
  confirmed hearing the beep. This proves audible output, not measured A/V skew.
- Chrome WebGL2: red/blue scene pixel checks and completion passed; 12 frames
  in the run after deduplicating by decoded frame counter.
- Chrome WebGPU: red/blue scene pixel checks and completion passed; 11 frames.
- Android API 36 emulator: MediaPlayer -> `.nux` -> Vulkan pixel checks, seek,
  completion and disposal passed. Levi accepted emulator qualification for Task 1; this is not physical-device or hardware-decoder performance evidence.

The stronger scale/vector-overlay pixel oracle passed on macOS Metal, Bat Phone
Metal, Android emulator Vulkan, and Chrome WebGL2/WebGPU. WebGL readback is
normalized from bottom-row-first before comparing authored pixel coordinates.
Actual embedded MP4 playback passed on macOS Metal and Chrome WebGL2/WebGPU; source
lease tests verify bounded extraction, independent files, and cleanup.

These observations precede any subsequent source changes; rerun affected
proofs before release. No physical Android qualification, measured A/V skew,
hardware-decode measurement, saturation/long-run resource qualification, or
production-resolution performance claim has been made.

Managed browser qualification (2026-09-16): external and embedded clips passed
on Chrome WebGL2 and WebGPU after decoder reclamation, a denied seek, and
readmission. The fixture's serialized captions matched its playback clock;
“Red scene” and “Blue scene” appeared in a visible aria-live status region.
This verifies DOM caption delivery, not screen-reader announcements or native
accessibility. Local evidence is `target/video-managed-browser-evidence.log`.

Managed Android qualification (2026-09-16): the approved API36 emulator passed
external and embedded `.nux` playback through Vulkan, including caption clock
assertions and visible TextView live-region updates, seek while denied,
reclamation/reopen, completion, disposal and extracted-source cleanup. Launch
with `--ez embedded true` to select the embedded fixture. This does not qualify
hardware decoder capacity or TalkBack announcements. Evidence is recorded in
`target/video-managed-android-{external,embedded}.log`.

Managed iOS qualification (2026-09-16): Bat Phone passed external and embedded
clips with 22 rendered frames each. The updated harness checks caption timing
and both strings in an accessible UILabel, decoder reclamation/reopen, seek,
completion and disposal. Pass `--embedded` to select embedded bytes. These runs
were muted; the earlier user-confirmed AAC beep is separate evidence. VoiceOver
announcements and measured A/V skew remain unqualified. Logs:
`target/video-managed-ios-{external,embedded}.log`.

Public native ABI qualification: run
`cargo run -p nux-capi --features apple-metal --example video_apple` on the macOS
main thread. It drives real AVFoundation observations/actions across public C
functions, submits decoded RGBA, captures presented Metal pixels, and checks
captions, seek, completion and disposal. The 2026-09-16 run passed 22 frames;
see `target/video-capi-live-metal.log`. This source-tree example does not replace
packaged C/Swift consumer or device ABI qualification.

Loop-range qualification (2026-09-16): all managed harnesses now perform two
[1.3,1.7) loops, disable looping, then require natural completion and disposal.
External and embedded cases passed on Chrome WebGL2/WebGPU, the approved Android
emulator/Vulkan, macOS/Metal and Bat Phone/Metal. Bat Phone and macOS each
presented 46 frames per run. Captions and pixel oracles remain active. These
runs verify frame-boundary looping, not sample-accurate audio seam timing.
Evidence is in the ignored `target/video-loop-*` logs.

Rectangular clip qualification: the shared fixture now clips the scaled video
inside a 24×12 rectangle. Four samples inside the video but outside that clip
must stay black, while center video and overlaid vector UI remain visible.
External and embedded runs passed on macOS/Metal, Bat Phone/Metal, Android
emulator/Vulkan and Chrome WebGL2/WebGPU (`target/video-clipping-*`).

Lifecycle host contract: call the managed scene player's `set_suspended` in
lifecycle callbacks before stopping rendering; resume clears only that reason.
This sends actions to an existing decoder without requiring another frame tick.
The iOS harness connects application active/inactive callbacks to this API.
Device background clock, audio-interruption and route-change qualification are
still required; ordinary playback and the dispatch regression do not prove them.

Two-player allocation proof: macOS runs now first exercise a one-slot priority
handoff followed by concurrent playback with two slots. All changed allocations
are reclaimed before new admission; each player retains its independent scene
and playback checks. Pass `--pool` to the iOS harness for the same proof (combine
with `--embedded` for embedded sources). This tests explicit slot budgets and
owned decoder handles, not maximum hardware capacity or forced software decode.

Native clock pause check: Apple runs now stop decoder/frame ticks for at least
250ms after requesting Background suspension and sample AVPlayer's media clock.
They require less than 30ms advancement, then resume existing playback checks.
macOS and Bat Phone reported zero advancement in the current runs. This is an
explicit lifecycle-call test while foreground, not an actual OS-background or
interruption test. `target/video-clock-*` contains builds and evidence.

Synchronization qualification: macOS runs first exercise two independent players,
delaying the follower until the leader reaches 400ms. The script-owned group
registration must correct that measured drift through automatic ScenePlayer
clock reports; this harness does not call the correction controller. An
independent oracle samples native clocks before each host tick and counts
follower generation changes caused by corrective seeks. It must converge and maintain <=60ms for >=300ms of
available clock observations. Both videos must render red and blue scene frames
with captions, then release their decoders. iOS uses `--sync`, optionally with
`--embedded`, for the same asynchronous proof. The controller predicts seek delay
from measured post-seek drift; a seek to an old leader time alone did not converge.
This short fixture does not establish long-run A/V skew or a universal tolerance.

Android clock pause check: the harness now requests Background suspension and
samples native MediaTimestamp-derived clocks for >=300ms without decoder/frame
ticks. The API36 emulator passed external and embedded runs with zero measured
advancement, then completed captions, seek, loop, Vulkan pixels and disposal.
The result file includes the measured drift and rendered frame count. These
foreground lifecycle calls do not establish OS background/audio focus behavior.

A separate Android OS lifecycle proof uses `--ez lifecycle true` with the
single-player fixture (optionally `--ez embedded true`). After decoded frames
and the direct pause check, the Activity calls moveTaskToBack, forwards onPause
to Background suspension, stops render ticks and polls native clocks. Bring the
existing Activity forward with `am start --activity-reorder-to-front` after at
least 500ms. onResume requires multiple paused samples and <30ms movement before
clearing suspension; the normal pixel/caption/loop/teardown proof must then
finish. The result includes `OS background/resume=true`. This proves Activity
background/resume, not audio-focus interruption or process recreation.

Android synchronization uses the same automatic script-owned group registration
and independent clock oracle as Metal/browser. Qualification uses `--ez sync true` (optionally
`--ez embedded true`) when starting `ai.nuxie.videoqualification/.MainActivity`.
It runs two independent decoders, delays the follower, and uses native media
clocks to require drift <=60ms for >=300ms after correction. Its ten-second
fixture leaves room for decoder startup. Every presented frame checks clipping,
UI composition, video color and timestamp-appropriate captions. A corrective
seek may skip cues; the separate single-player test requires both visible cues.
The result includes maximum initial drift, correction count, settled peak, frame
counts and owned decoder count after disposal.

Browser synchronization uses automatic script-owned group registration and an
independent native-clock drift oracle, without manual corrective seeks. It runs
at `/sync.html` and `/webgpu/sync.html`, with
`?embedded=1` selecting embedded bytes. Each test creates two scene players and
requires an injected >250ms offset, correction, <=60ms drift for >=300ms,
composition/caption checks and decoder disposal. WebGPU submits both surface
readback copies before awaiting either, because canvas textures can expire
across an asynchronous wait.

Use the qualification range server above for external media. Python's ordinary
`http.server` does not provide byte ranges: Chrome cold loads were observed to
reset attempted seeks to zero, while cached copies worked. Test with browser
cache disabled as well as embedded Blob media. The product asset resolver must
supply a seekable retained source as required by the runtime contract.

Browser frames are copied during `requestVideoFrameCallback`, with its
`mediaTime` timestamp. `currentTime` remains the playback clock; it must not be
used to label copied pixels. The host bounds pending frame storage to one frame,
cancels callbacks at seek/disposal, and tags captures with their source/seek
generation. Browsers without this callback API currently reject host creation;
verify required browser versions before release.

The macOS public C ABI example also runs two AVFoundation players through
`nux_video_sync_group_*`, with actual native clocks and public Metal frame
submission. It requires an injected offset, corrected drift <=60ms for >=300ms
and explicit decoder disposal. The preceding single-player phase retains the
Metal readback oracle. See `docs/video-runtime-host-contract.md` for SDK-facing
group ownership, clock and scheduling rules.

Android audio-focus regression: `--ez focus true` runs a native MediaPlayer
adapter proof with a competing real AudioManager transient focus request. It
waits for preparation, starts playback, confirms loss pauses the player, sends
the runtime-equivalent Pause, tries Play while interrupted and verifies it
cannot restart playback, abandons the challenger, consumes the retained
recovery edge, and confirms only explicit Play resumes. Both requests/player
are released before PASS. This checks native focus callbacks and ownership;
portable host tests separately prove runtime intent/lifecycle reconciliation,
including a whole loss/recovery between ticks and a full author-command queue.
It is not yet an end-to-end Journey or SDK interruption test.

Apple OS lifecycle qualification is automated by
`python3 tools/video-qualification/apple/qualify-lifecycle.py --device <id>`
against the installed harness. It opens the app's Settings, returns after a
bounded background interval, requires a stationary native media clock, and
finishes playback. Add `--embedded` for embedded bytes. Muted Bat Phone cases
pass, as do audible external and embedded cases using `--audible`. Three
consecutive audible external runs also pass the unchanged pause-clock oracle.
The app delegate forwards only real lifecycle transitions: its initial
DidBecomeActive callback must not clear the independent native pause probe. The script leaves the app's result
visible and saves its console under target/video-os-lifecycle-ios-*.log.

Android teardown retry qualification uses `--ez teardown true`. It blocks the
private decoder worker with a latch, requires the first close to time out, then
releases the latch during the second close and checks that the worker has
terminated before that call acknowledges disposal. A third close must succeed.
The blocker uses reflection in the fixture app, with no production test hook.
`target/video-disposal-android-live.log` records a passing emulator run. Portable
regressions also require failed reclamation, source replacement, poster
transition, and pool removal to retain decoder ownership until successful retry.

### Sustained 720p Metal benchmark

Build `cargo build -p video-qualification --bin video-qualification --profile release-apple`,
then run `/usr/bin/time -l target/release-apple/video-qualification "$PWD/crates/nuxie-video-host/tests/fixtures/red-blue-720p.mp4"`.
The optional CLI argument selects this fixture-specific benchmark; no argument
continues to run the existing qualification suite. It loops the self-made
720p/30 fixture for 32 seconds, uploads every delivered frame, draws into a
1280×720 Metal target, reads back and checks clip/overlay/color pixels, then
requires decoder reclamation. It reports first-frame latency, presentation rate,
frame-work p50/p95 and maximum observed presentation gap. Minimum passing rate
is 24 fps with at least 300 observations of each color. The forced GPU readback
is part of this measured path; these timings are not an onscreen display or
A/V synchronization measurement. Acceleration remains unknown.

The debug build failed this threshold at 23.045 fps, p50 38.806 ms/p95 41.566 ms,
734 frames, 145.245 ms maximum gap (`target/video-720p-metal-live.log`). Do not
report this as a passing performance qualification. Release-profile results
must be recorded separately, and physical iOS/Android/browser measurements are
still required.

The optimized `release-apple` run passed the benchmark floor: 897 frames over
32 seconds, 28.157 fps, 176.986 ms first frame, frame-work p50 2.962 ms/p95
4.096 ms, and 143.242 ms maximum presentation gap. All pixel oracles passed
and final owned decoders were zero. `time -l` reported maximum RSS 132136960
bytes and peak footprint 245597096 bytes. Evidence:
`target/video-720p-metal-release-live.log`. This is the local Mac with repeated
two-second loops and synchronous readback; it does not qualify physical iOS,
Android, browsers, long-run memory stability or seamless loop timing.

Android decoder identity qualification passed on emulator-5556 with external
and embedded fixtures (`target/video-decoder-info-android-{external,embedded}.log`).
The Java owner queried MediaPlayer metrics on its private thread, matched the
actual codec, and Rust read the immutable snapshot over JNI. Both selected
`c2.android.avc.decoder`, reported as Software; the advertised instance hint was
32. This does not establish sustainable 32-player capacity. External/embedded
Vulkan proofs completed 34/37 frames, with zero native pause drift and successful
seek, two loops, reclamation/reopen, captions and disposal.

For physical iOS, build with
`CARGO_INCREMENTAL=0 VIDEO_PROOF_PROFILE=release-apple VIDEO_PROOF_DEVELOPMENT_TEAM=<team> bash tools/video-qualification/apple/build.sh`,
install the resulting fixture app, then launch `com.nuxie.video-proof --benchmark`.
The same benchmark now advances through timer ticks and shows sampled verified
frames in the app. It requires 32 seconds in the foreground; suspension fails
the run rather than silently measuring a shortened workload. The console emits
`NUX_VIDEO_BENCHMARK status=1` only after the throughput/color/cleanup checks pass.
The default build profile remains dev/debug for ordinary qualification; the
selected profile is passed explicitly to both Cargo and Xcode library paths.

Physical iOS run passed on the Bat Phone (iPhone 17 Pro Max, iOS 26.6.2):
861 verified frames, 26.976 fps, first frame 109.381 ms, frame-work p50 8.799 ms
and p95 10.970 ms, maximum presentation gap 201.143 ms, final owned decoders
zero. Evidence `target/video-720p-ios-release-live.log`; the optimized runtime
archive's link path is recorded in `target/video-ios-benchmark-release-build.log`.
The UIKit fixture shell remains Xcode Debug; Rust/native media code uses
release-apple. This includes repeated two-second loops and GPU readback, not
seamless-playback or A/V-skew qualification. The maximum gap remains visible in
the result. The app retained its visible PASS and preview after the driver exited.

Automate this installed-device check with
`python3 tools/video-qualification/apple/qualify-benchmark.py --device <id>`.

Android 720p qualification uses
`ANDROID_HOME=<sdk> VIDEO_PROOF_PROFILE=release bash tools/video-qualification/android/build.sh`.
After installing its APK on the explicitly chosen emulator/device, run
`ANDROID_HOME=<sdk> python3 tools/video-qualification/android/qualify-benchmark.py --serial <serial>`.
The app's `--ez benchmark true` mode uses the same 32-second/24-fps/color floor,
uploads 720p frames to a 720p Vulkan target, checks scaled clip/overlay/color
pixels, presents a sampled preview, records selected-decoder metadata and
acknowledges final cleanup. Backgrounding fails the benchmark. This extends
fixture qualification; it does not claim native SDK packaging or physical
Android hardware acceleration. Ordinary APK builds still default to dev.

Android emulator 5556 cleared the throughput/pixel/cleanup floor in two optimized
720p runs, but startup/frame pacing is not yet qualified as acceptable. First run:
743 frames, 24.747fps, first frame2020.296ms, work-p95=21.894ms, max-gap2058.962ms
(`target/video-720p-android-first-live.log`). Repeat: 844 frames, 27.575fps, first
frame1403.231ms, work-p50=8.702ms/p95=13.853ms, max-gap362.149ms from media PTS
0.09 to0.672 (`target/video-720p-android-release-live.log`). Thus the repeat's
largest gap was not a loop rewind. Both reclaimed every decoder. At this size,
MediaPlayer selected c2.goldfish.h264.decoder, reported Hardware with instance
hint4; this is emulator metadata, not physical Android hardware qualification.

Android phase reporting now separates decoded arrival, upload and rendering.
Baseline first decoded arrival632.643ms versus first presentation1243.962ms
coincided with a606.790ms maximum render/readback, versus10.014ms render p95.
A private staging-buffer reuse experiment regressed throughput twice and was
reverted; the control recovered27.694fps with12.812ms total-work p95. Its first
frame still took1436.519ms, including a573.153ms maximum render/readback.
See `target/video-720p-android-control-driver.log`; failed-run metrics are now
preserved in app results. These observations narrow startup diagnosis without
claiming physical Android performance or a resolved frame-pacing problem.

Browser 720p benchmark builds use `VIDEO_PROOF_PROFILE=release` with
`browser/build.sh` and then `browser/build-webgpu.sh`. Each stages
`benchmark.html` beside its own wasm bindings and 720p fixture. Run them
sequentially in a foreground tab via the range-capable loopback server.
A latched visibility change invalidates the run even if the tab later returns.
Both paths render 1280x720, verify clip/overlay/decoded colors, run 32 seconds and
require 24 fps plus 300 frames of each color and acknowledged cleanup. WebGL2
normalizes its bottom-up readback; WebGPU awaits an explicit capture before
verification. Timings include readback and verification; this is not an audio
or hardware-decoder selection measurement.

Optimized browser benchmarks passed in Chrome152 on the local Mac, cache
disabled and foreground visibility retained throughout. Evidence:
`target/video-720p-webgl-live.log` and `target/video-720p-webgpu-live.log`.

| Renderer | Verified frames | FPS | First frame | Frame-work p95 | Largest gap | Final decoders |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| WebGL2 | 935 | 29.289 | 104.8 ms | 5.8 ms | 76.2 ms | 0 |
| WebGPU | 936 | 29.268 | 34.8 ms | 5.4 ms | 77.3 ms | 0 |

Both results include720p rendering/readback and the independent composition
oracles. They qualify this browser/host, not every browser implementation,
hardware acceleration, audio synchronization or long-run memory stability.

Managed decoder admission regression (2026-09-16): supplied AVFoundation,
MediaPlayer and HTMLVideoElement adapters reject forced hardware/software
allocation without acquiring a decoder, then pass real playback with
PlatformManaged. Metal priority handoff, Android embedded Vulkan, and both
browser embedded renderer fixtures passed. Bat Phone audible/external and
muted/embedded Settings background/resume each passed47 frames, two loops,
source replacement, captions and teardown with zero measured native clock
drift during1.712s and1.658s pauses. These are explicit application budgets;
they do not measure maximum device decoder capacity. Evidence is under
`target/video-managed-budget-*`.

Playback-rate regression (2026-09-16): the two-loop fixtures now use0.5x and
1.5x, require platform-clock observations at both rates, then return to1x for
completion. This exposed and fixed an Android resume-after-seek path that
failed to apply rates received while preparing/seeking. The same failing
embedded Vulkan fixture passed after the fix; real Android audio-focus
recovery also passed. Metal/macOS, Bat Phone audible/external and muted/embedded,
and Chrome WebGL2/WebGPU passed. These checks establish rate propagation and
continued composed playback, not sample-accurate audible-output timing.
Evidence: `target/video-rate-*`.
