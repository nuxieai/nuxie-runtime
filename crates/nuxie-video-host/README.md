# Video host adapters

These adapters bind platform audiovisual players to Nuxie's `video::Playback`
controller. They do not fetch or authenticate published assets. A caller must
retain its verified source/cache lease, marshal observations to the scene's
owning thread, and upload through the same persistent renderer factory used to
import the scene. Hardware decoding is **unknown** unless independently observed;
these initial implementations copy bounded RGBA frames and do not claim zero-copy.

- Apple: AVPlayer audio clock and AVPlayerItemVideoOutput. Methods and destruction
  run on the main thread. `AppleScenePlayer` owns one live video occurrence.
  `AudioSessionOwnership::HostManaged` preserves embedding-app ownership;
  `RuntimeManaged` explicitly delegates AVAudioSession policy while audible
  players run. Muted players never activate the session. Shared audible players
  combine mix/duck/exclusive requirements. Interruptions and headphone removal
  feed playback intent; the host still supplies application/visibility suspension.
- Browser: HTMLVideoElement plus bounded Canvas2D copies. CORS must permit pixel
  access. Rejected autoplay is observable and can be retried by a user action.
  Embedded bytes own a Blob URL that is revoked on disposal. Frame timestamps
  use requestVideoFrameCallback mediaTime captured with the decoded frame.
- Android: package `android/ai/nuxie/runtime/VideoPlayer.java` with the Rust JNI
  adapter. MediaPlayer renders to a private SurfaceTexture on its HandlerThread;
  bounded RGBA readback feeds Vulkan scene composition. Pass the application
  Context. Audio focus follows authored policy; muted motion does not request it.
  This path requires packaging the Java class with the native library.

`source::EmbeddedFile` materializes embedded data once into an app-owned private
cache directory. Keep the lease until the decoder closes; dropping it removes
only its own file. External acquisition/cache policy remains SDK-owned.

Qualification commands and measured evidence are in
[`tools/video-qualification/README.md`](../../tools/video-qualification/README.md).
The contract includes capability-gated C ABI import and frame submission,
caption tracks, readiness/poster handling, and synchronized groups. Qualification
records distinguish tested behavior from remaining device/resource measurements
and SDK adoption. Runtime artifact publication is a separate release gate.

`scene::ScenePlayer` owns a decoder only while admitted. Hosts feed allocations
from their measured budget, reclaim denied players before opening new ones,
and supply a mode-aware opener plus the scene factory upload callback. Queued
intent survives denial; a replacement restores position before play. Required
first-frame waits use a bounded gate; timeout drops the decoder and holds
fallback until a new generation/allocation. `AppleScenePlayer::tick_allocated`
uses this owner. AVPlayer chooses its implementation, so its slot is still
platform-managed/unknown; both forced hardware and forced software admission
are rejected before changing playback.
`BrowserScenePlayer` also owns external or embedded sources through the shared
scene owner, recreating Blob URLs after readmission and retaining the asset MIME
type. Browser managed playback is qualified on WebGL2/WebGPU. `AndroidScenePlayer` owns the application context and optional extracted source
lease; its tick must run from an app JNI entry point. External and embedded
managed playback passed on the approved Android emulator. Measured multi-player
budgets, native caption accessibility and physical-device pressure tests remain.

Android exposes `decoder_info()` as an optional immutable snapshot of the
MediaPlayer-selected codec. API 29+ classification uses an exact match in
MediaCodecList and the platform hardware/software flags; older or unmatched
identities remain unknown. `advertised_max_instances` is advisory, not measured
capacity. Optional diagnostic failures do not fail media playback, and no codec
name-prefix heuristic upgrades an unknown identity to hardware or software.

Platform adapters now accept `Allocation::PlatformManaged` or `Poster`. Their
MediaPlayer/AVPlayer/HTMLVideoElement APIs select the codec implementation.
A reported codec identity does not make a future open hardware-selectable.
The shared allocator separately limits managed player count and decoded pixel
rate; `max_players` bounds all admitted classes together. Default budgets admit
nothing. Hosts must supply measured limits and positive width×height×decode-rate
costs for managed/software requests, including the effect of playback rate.
Hardware/software modes remain available to hosts that can actually select them.
