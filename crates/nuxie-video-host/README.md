# Video host adapters

These adapters bind platform audiovisual players to Nuxie's `video::Playback`
controller. They do not fetch or authenticate published assets. A caller must
retain its verified source/cache lease, marshal observations to the scene's
owning thread, and upload through the same persistent renderer factory used to
import the scene. Hardware decoding is **unknown** unless independently observed.
All three adapters hand the renderer each decoded frame on the GPU; no pixels
pass through the CPU.

- Apple: AVPlayer audio clock and AVPlayerItemVideoOutput. Methods and destruction
  run on the main thread. `AppleScenePlayer` owns one live video occurrence.
  `AudioSessionOwnership::HostManaged` preserves embedding-app ownership;
  `RuntimeManaged` explicitly delegates AVAudioSession policy while audible
  players run. Muted players never activate the session. Shared audible players
  combine mix/duck/exclusive requirements. Interruptions and headphone removal
  feed playback intent; the host still supplies application/visibility suspension.
  Frames arrive as `FramePixels::PixelBuffer`: AVFoundation's IOSurface-backed
  32BGRA buffer, which `NativeMetalFactory::import_pixel_buffer` (or the C ABI's
  `nux_player_video_present_metal_pixel_buffer`) wraps as a BGRA8Unorm texture
  with no CPU copy. It samples the same bytes the RGBA path uploaded, so colors
  are unchanged. The texture holds the buffer until it is released, so
  AVFoundation cannot reuse the surface while a frame is shown. The byte budget
  bounds each frame's RGBA size. `PixelBuffer::read_rgba` copies a frame to the
  CPU for checks.
- Browser: HTMLVideoElement. Inside requestVideoFrameCallback the adapter draws
  the frame the element shows into the player's own 2D canvas, so pixels and the
  callback's mediaTime describe the same decoded frame, and hands the canvas over
  as `FramePixels::Canvas` without reading it back. The canvas stays GPU-backed
  because nothing reads it: WebGPU renderers copy it into a reused texture with
  `NativeWebGpuFactory::copy_external_image` (`GPUQueue.copyExternalImageToTexture`),
  and others read it back through a scratch canvas with `CanvasFrame::read_rgba`.
  Going through Canvas2D keeps the colors the earlier byte path had. Chrome's
  WebGPU import of a video element or WebCodecs VideoFrame instead converts the
  BT.709 transfer curve, as Chrome's own `<video>` display does, which renders
  mid-tones about 11 levels brighter (gray 125 becomes 136). Each draw replaces the
  canvas, so translucent frames never blend over the previous one. The canvas
  holds only the latest capture, so a frame is uploaded in the tick that
  returns it. The byte budget bounds each frame's RGBA size, which is also the
  size of the texture a GPU copy writes. CORS must permit pixel access.
  Rejected autoplay is observable and can be retried by a user action.
  Embedded bytes own a Blob URL that is revoked on disposal.
- Android: package `android/ai/nuxie/runtime/VideoPlayer.java` with the Rust JNI
  adapter; Android 10 (API 29) or later. MediaPlayer decodes into an
  ImageReader with GPU sampled-image usage on its HandlerThread, and each frame
  arrives as `FramePixels::HardwareBuffer` with its crop, rotation and color.
  `NativeVulkanFactory::import_hardware_buffer` imports the buffer with no copy
  and converts it from YUV to RGBA in one draw into textures reused per video.
  The color is the stream's own description, or Android's decoder defaults
  when it has none (limited range; BT.2020 from 4K, BT.601 up to 720x576,
  BT.709 between), because drivers' suggestions are not reliable: the
  emulator reports full range for limited-range video. Dropping the frame
  returns the buffer to the decoder. Pass the application Context. Audio focus
  follows authored policy; muted motion does not request it. This path
  requires packaging the Java class with the native library.

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
