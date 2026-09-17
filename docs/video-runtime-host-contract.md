# Video host integration

The video extension uses the ordinary RIV stream with Nuxie type/property keys.
A Video is a scene occurrence; a VideoAsset carries an external source key or
embedded bytes. The product resolver owns acquisition and authentication. The
host owns a seekable source lease, decoder, audio coordination and media clock.
Frames enter the runtime renderer domain instead of a platform-player overlay.

This document describes implemented integration seams. The feature remains
unreleased; `tools/video-qualification/PROGRESS.md` records qualification gaps.

## Capability admission

Native presentation imports default to video unavailable. Rust hosts must opt
in with `FileImportLimits::with_video_playback(true)` only after initializing a
compatible decoder and renderer. `FileImportLimits::unbounded()` relaxes
allocation budgets; it does not grant video capability. A bounded per-video
embedded-byte ceiling remains independently configurable.

C hosts supply `NuxVideoPlaybackCapabilities` through the appended
`NuxFileImportConfig.video_playback` child for native configured imports, or
`nux_file_import_with_video_capabilities` for callback-renderer imports. The
capability record declares playback availability and a per-asset embedded byte
limit. Null, disabled and legacy import configurations reject video scenes.
Ordinary RIV scenes remain accepted by the existing entry points.

The full stream is structurally checked for video before asset hooks, native
loaders, scripts or decoders run. Capability declarations are assertions by the
host, not automatic codec/hardware discovery or authorization to fetch assets.
The runtime still checks embedded bytes at the asset admission boundary. Old
C import-config prefixes cannot accidentally copy a partial capability pointer;
all prefixes smaller than the appended field keep video unavailable.

Policy-free `File::import` remains available to scene tooling. Presentation
hosts must use the admitted native import API; successful tooling import is not
proof that the device can play the scene.

## Independent playback

Enumerate `nux_player_visit_videos` and copy any source strings/embedded bytes
needed beyond the callback. Each occurrence gets an independent decoder unless
it is denied by the host's resource allocator. Shared encoded bytes do not
share playback state. Apply commands through `nux_player_video_command` and
`nux_player_video_set_loop_range`.

On the creator thread, call `nux_player_video_step` to drain actions and deliver
ready/playing/ended/buffering/blocked/error observations. Execute all returned
actions in order. Do not reenter a C API from a callback. Marshal native decoder
callbacks onto this thread first. Seek actions carry the new generation; tag
frames and observations accordingly. Stale generations cannot replace the
current scene frame. Submit canonical frame bytes with the renderer-specific
video presentation API, using the renderer which owns the occurrence.

Lifecycle suspension must reach the actual decoder before the app stops frame
updates. Rust managed players provide `set_suspended` for this purpose. C hosts
must queue the suspension reason, immediately drain its actions, and execute
pause before suspending their render loop. Resuming one reason does not clear
other reasons. Requested play intent survives temporary suspension.

## Explicit synchronization

Use `nux_video_sync_group_new` with 2–64 distinct video occurrences. Member zero
is the leader, normally the audible foreground video; each follower retains its
own decoder. The group owns references to those scene occurrences, so closing
an original player handle does not invalidate the group. The group is thread
affine and does not own platform decoders.

Group commands and loop ranges use the same runtime queues as individual
commands. All members are preflighted before any command is queued. Successful
commands invalidate occurrence scheduling so idle SDK hosts notice the work.

After processing member commands and decoder observations, sample each native
media clock and call `nux_video_sync_group_update` with monotonic host seconds.
Clock samples must be in creation order. Each sample carries its generation,
media seconds, rate, playing flag and availability flag. Mark seeking or
unavailable clocks unavailable. Do not substitute decoded-frame presentation
timestamps for current media clocks. Android may extrapolate a fresh native
MediaTimestamp over a bounded window; unbounded cached positions are invalid.

The controller skips suspended, stale, unavailable or rate-mismatched members.
It corrects follower drift with bounded seek-latency prediction and a cooldown.
All proposed corrections are queue-validated before any are committed. Optional
measurement callbacks report the follower member index, signed drift,
prediction and whether a seek was queued; they cannot reenter the C API.

Freeing a group stops future corrections and releases scene references; it does
not dispose the videos. To dispose the whole group, send the dispose command,
drain and execute every member's actions, then free the group. The host must
still close its decoder/source leases and respect pending GPU work.

The same implementation is available to Rust as
`nuxie_runtime::video::sync::SynchronizationGroup`; `nuxie_video_host::sync`
reexports it. Luau can retain `context:videoGroup({"leader", "follower"})`
for the lifetime of its script context. The first named video leads; group
play/pause, seek, rate, mute, volume and loop commands share the same bounded
queues as individual commands. `takeError()` consumes an asynchronous correction
error. Retain the returned group: weak registrations stop when it is collected
or its script context expires.

Managed ScenePlayer ticks report native clocks automatically. C ABI hosts call
`nux_player_video_report_clock` for each occurrence after commands/observations,
even without a new frame. Use one monotonic seconds domain across all members;
null clears availability. Cached clocks older than 100ms, stale generations and
suspended players cannot drive correction. Fresh playing clocks are projected
to the report time to avoid update-order drift. Authored-group unit and bridge
checks do not replace live cross-platform qualification of this automatic path.

## Browser frame timestamps

Browser frame copies use `requestVideoFrameCallback` and its `mediaTime`.
`HTMLVideoElement.currentTime` is a playback clock and can be ahead of the frame
currently available for copying. The host bounds pending captured frames to
one, cancels captures on seek/disposal and generation-tags every capture.
Required browser minimum versions still need release qualification.

External URLs must be seekable. For local qualification use the range-capable
server in `tools/video-qualification/browser/serve.py`; cold-cache Chrome seeks
against ordinary Python `http.server` were observed to reset to zero. Product
acquisition must supply a verified retained seekable source, as specified in
the parent video implementation plan.

Decoder teardown is acknowledged, not fire-and-forget. `Decoder::close` must
return success only after releasing the native decoder. `ScenePlayer` retains
a decoder after failed close, and refuses source replacement or allocation
changes until a retry succeeds. `PlayerPool::remove` is fallible and retains
the member on reclamation failure. Owners must retry cleanup before dropping
their pool; destructor cleanup alone cannot acknowledge capacity reuse. Android
close retries join the original worker even after shutdown was requested, and
propagate timeout or cleanup failure. JNI clears the translated exception so
a subsequent cleanup attempt can execute.

Terminal failed/disposed occurrences do not reserve future decoder slots. The
pool checks `ManagedPlayer::can_decode` before allocation, reclaims any existing
owner first, and admits healthy occurrences only after acknowledged cleanup.
An explicit source replacement restores eligibility; ticking does not silently
retry a failed source forever. Cleanup failure still aborts new admission.

Android `decoder_info()` reports the actual MediaPlayer codec identity when
available, with API-29 hardware/software classification from an exact codec-list
match. API-23–25 has no identity metric; API-26–28 may expose a name while
classification remains unknown. Missing data and diagnostic failures remain
unknown. The advertised maximum instance count is a capability hint, not a
measured budget, and must not directly drive admission. Sources:
[MediaPlayer metrics](https://developer.android.com/reference/android/media/MediaPlayer.MetricsConstants),
[codec classification](https://developer.android.com/media/optimize/performance/codec),
[instance capability](https://developer.android.com/reference/android/media/MediaCodecInfo.CodecCapabilities#getMaxSupportedInstances()).

Resource admission distinguishes guaranteed hardware, selectable software and
platform-managed decoding. `DecoderBudget::max_players` bounds total live
admissions, while managed count/pixel-rate and software pixel-rate limits bound
those workloads separately. Zero/unknown managed workload is denied; default
budgets admit no players. Hardware availability, platform instance hints and
measured concurrent capacity are different inputs. The host must reserve a
conservative total budget for the combined workload rather than adding vendor
limits together.

The supplied AVPlayer, MediaPlayer and HTMLVideoElement adapters accept only
`PlatformManaged`/`Poster`; unsupported forced modes fail before decoder
creation or playback mutation. Qualification fixtures now use the managed
class. The allocator can fall back to separately declared software support
within its budget; these platform APIs do not promise selectable software.
