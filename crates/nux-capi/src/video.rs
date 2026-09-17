//! Host-neutral media bridge. Platform decoders own A/V timing; this surface
//! routes their commands and observations through the live scene occurrence.
use super::*;
use nuxie::video::{Video, VideoAsset, playback::*};

/// Explicit assertion that the host has initialized a compatible decoder and
/// renderer. A null capabilities pointer means video is unavailable. This does
/// not perform asset authentication or select/initialize a platform decoder.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxVideoPlaybackCapabilities {
    pub struct_size: u32,
    pub playback_available: u32,
    /// Per embedded video asset; zero permits only external/empty video assets.
    pub max_embedded_bytes: usize,
}
impl Default for NuxVideoPlaybackCapabilities {
    fn default() -> Self {
        Self {
            struct_size: size_of::<Self>() as u32,
            playback_available: 0,
            max_embedded_bytes: 64 * 1024 * 1024,
        }
    }
}
pub(crate) unsafe fn import_limits(
    capabilities: *const NuxVideoPlaybackCapabilities,
) -> Result<FileImportLimits, NuxStatus> {
    if capabilities.is_null() {
        return Ok(FileImportLimits::default());
    }
    let size = unsafe { capabilities.cast::<u32>().read() };
    if !struct_size_supports(size, size_of::<NuxVideoPlaybackCapabilities>()) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let capabilities = unsafe { capabilities.read() };
    if capabilities.playback_available > 1 {
        return Err(NuxStatus::InvalidArgument);
    }
    Ok(FileImportLimits::default()
        .with_video_playback(capabilities.playback_available == 1)
        .with_max_embedded_video_bytes(capabilities.max_embedded_bytes))
}

/// All views passed to this callback are valid only during the callback.
/// Copy source strings/embedded bytes before returning if retaining them.
#[repr(C)]
pub struct NuxVideoInfo {
    pub struct_size: u32,
    pub component_id: usize,
    pub asset_id: u32,
    pub generation: u64,
    /// Opening=0, ready=1, playing=2, paused=3, seeking=4, buffering=5,
    /// ended=6, failed=7, disposed=8.
    pub state: u32,
    pub position_seconds: f64,
    pub volume: f32,
    pub rate: f32,
    pub muted: u32,
    pub wants_play: u32,
    pub audio_policy: u32,
    pub source_key: NuxStringView,
    pub content_type: NuxStringView,
    pub embedded_bytes: NuxByteView,
}
pub type NuxVideoInfoCallback = Option<unsafe extern "C" fn(*mut c_void, *const NuxVideoInfo)>;

/// One decoder action. Play=0, pause=1, seek=2, rate=3, volume=4,
/// dispose=5. `value` is seconds/rate/volume; seek carries the new generation.
#[repr(C)]
pub struct NuxVideoAction {
    pub kind: u32,
    pub value: f64,
    pub generation: u64,
}
pub type NuxVideoActionCallback = Option<unsafe extern "C" fn(*mut c_void, *const NuxVideoAction)>;

fn state_id(state: PlaybackState) -> u32 {
    match state {
        PlaybackState::Opening => 0,
        PlaybackState::Ready => 1,
        PlaybackState::Playing => 2,
        PlaybackState::Paused => 3,
        PlaybackState::Seeking => 4,
        PlaybackState::Buffering => 5,
        PlaybackState::Ended => 6,
        PlaybackState::Failed => 7,
        PlaybackState::Disposed => 8,
    }
}
fn playback_status(error: PlaybackError) -> NuxStatus {
    match error {
        PlaybackError::InvalidValue => NuxStatus::InvalidArgument,
        PlaybackError::QueueFull => NuxStatus::LimitExceeded,
        PlaybackError::Disposed | PlaybackError::Failed => NuxStatus::RuntimeError,
    }
}
pub(crate) fn with_video<R>(
    player: *const NuxPlayer,
    component_id: usize,
    body: impl FnOnce(&nuxie::CoreHandle, &ArtboardOccurrence) -> Result<R, NuxStatus>,
) -> Result<R, NuxStatus> {
    let _handle = enter_handle(player, HandleKind::Player)?;
    let player = unsafe { &*player };
    let _occurrence = enter_occurrence(&player.artboard)?;
    let video = player
        .artboard
        .instance
        .try_borrow()
        .map_err(|_| NuxStatus::ReentrantCall)?
        .object_handle(component_id)
        .filter(|v| v.is_type_of(Video::TYPE_KEY))
        .ok_or(NuxStatus::NotFound)?;
    body(&video, &player.artboard)
}
pub(crate) fn status(result: Result<(), NuxStatus>) -> NuxStatus {
    result.err().unwrap_or(NuxStatus::Ok)
}
pub(crate) fn command(kind: u32, value: f64, reason: u32) -> Result<Command, NuxStatus> {
    Ok(match kind {
        0 => Command::Play,
        1 => Command::Pause,
        2 => Command::Seek(value),
        3 if value.is_finite() && value > 0.0 && value <= f64::from(f32::MAX) => {
            Command::Rate(value as f32)
        }
        4 if value.is_finite() && (0.0..=1.0).contains(&value) => Command::Volume(value as f32),
        5 if value == 0.0 || value == 1.0 => Command::Mute(value == 1.0),
        6 if value == 0.0 || value == 1.0 => Command::SuspendReason {
            reason: match reason {
                1 => SuspensionReason::Hidden,
                2 => SuspensionReason::Background,
                4 => SuspensionReason::Interruption,
                8 => SuspensionReason::Resources,
                _ => return Err(NuxStatus::InvalidArgument),
            },
            suspended: value == 1.0,
        },
        7 => Command::Reenter,
        8 => Command::Dispose,
        9 if value == 0.0 || value == 1.0 => Command::Loop(value == 1.0),
        _ => return Err(NuxStatus::InvalidArgument),
    })
}

/// Enumerate video occurrences in this player's artboard. Component IDs are
/// occurrence-local and remain valid until the player/occurrence is replaced.
/// Calls are creator-thread affine; callbacks cannot reenter any C API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_visit_videos(
    player: *const NuxPlayer,
    callback: NuxVideoInfoCallback,
    user_data: *mut c_void,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status((|| {
            let callback = callback.ok_or(NuxStatus::NullArgument)?;
            let _handle = enter_handle(player, HandleKind::Player)?;
            let player = unsafe { &*player };
            let _occurrence = enter_occurrence(&player.artboard)?;
            let native = player
                .artboard
                .instance
                .try_borrow()
                .map_err(|_| NuxStatus::ReentrantCall)?
                .native_handle();
            let objects = native.with_artboard(|a| a.objects().to_vec());
            for (id, object) in objects.into_iter().enumerate() {
                let Some(object) = object else {
                    continue;
                };
                object.with_downcast::<Video, _>(|v| {
                    let settings = v.playback.settings();
                    let source = v.asset().and_then(|asset| {
                        asset.with_downcast::<VideoAsset, _>(|a| {
                            (
                                a.source_key.clone(),
                                a.content_type.clone(),
                                a.encoded_bytes(),
                            )
                        })
                    });
                    let (source_key, content_type, bytes) = source.unwrap_or_default();
                    let info = NuxVideoInfo {
                        struct_size: std::mem::size_of::<NuxVideoInfo>() as u32,
                        component_id: id,
                        asset_id: v.asset_id(),
                        generation: v.playback.generation(),
                        state: state_id(v.playback.state()),
                        position_seconds: v.playback.position(),
                        volume: settings.volume,
                        rate: settings.rate,
                        muted: u32::from(settings.muted),
                        wants_play: u32::from(v.playback.wants_play()),
                        audio_policy: settings.audio_policy,
                        source_key: NuxStringView {
                            data: source_key.as_ptr().cast(),
                            len: source_key.len(),
                        },
                        content_type: NuxStringView {
                            data: content_type.as_ptr().cast(),
                            len: content_type.len(),
                        },
                        embedded_bytes: NuxByteView {
                            data: bytes.as_ref().map_or(ptr::null(), |b| b.as_ptr()),
                            len: bytes.as_ref().map_or(0, |b| b.len()),
                        },
                    };
                    with_platform_callback(|| unsafe { callback(user_data, &info) });
                });
            }
            Ok(())
        })())
    })
}

/// Queue one authored command: play=0, pause=1, seek=2, rate=3, volume=4,
/// mute=5, suspend-reason=6, reenter=7, dispose=8, looping=9. Boolean values must be 0/1.
/// Suspension reasons are hidden=1, background=2, interruption=4, resources=8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_command(
    player: *const NuxPlayer,
    component_id: usize,
    kind: u32,
    value: f64,
    reason: u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, occurrence| {
            let command = command(kind, value, reason)?;
            video
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(command))
                .ok_or(NuxStatus::NotFound)?
                .map_err(playback_status)?;
            occurrence.commit_runtime_change(true)
        }))
    })
}

/// Drain queued commands and apply one decoder observation. Observation kinds:
/// none=0, ready=1 (`value`=duration), playing=2, ended=3, buffering=4,
/// play-blocked=5, failed=6. Stale generations are ignored. Deliver every action
/// synchronously in order; decode callbacks marshal back to the creator thread.
/// A callback is required even when this step happens to emit no actions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_step(
    player: *const NuxPlayer,
    component_id: usize,
    observation: u32,
    generation: u64,
    value: f64,
    callback: NuxVideoActionCallback,
    user_data: *mut c_void,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, occurrence| {
            let callback = callback.ok_or(NuxStatus::NullArgument)?;
            if observation > 6 || (observation == 1 && (!value.is_finite() || value < 0.0)) {
                return Err(NuxStatus::InvalidArgument);
            }
            let before = video
                .with_downcast::<Video, _>(|v| {
                    (
                        v.playback.state(),
                        v.playback.generation(),
                        v.playback.position(),
                    )
                })
                .ok_or(NuxStatus::NotFound)?;
            let actions = video
                .with_downcast_mut::<Video, _>(|v| {
                    if observation == 6 && generation == v.playback.generation() {
                        if matches!(
                            v.playback.state(),
                            PlaybackState::Failed | PlaybackState::Disposed
                        ) {
                            return Vec::new();
                        }
                        v.playback.failed(generation);
                        return vec![DecoderAction::Dispose];
                    }
                    let mut actions = v.playback.drain_actions();
                    match observation {
                        1 => actions.extend(v.playback.opened(generation, value)),
                        2 => v.playback.observed_playing(generation),
                        3 => actions.extend(v.playback.ended(generation)),
                        4 => v.playback.observed_buffering(generation),
                        5 => v.playback.observed_play_blocked(generation),
                        _ => {}
                    }
                    if v.playback.state() == PlaybackState::Disposed {
                        v.clear_frame();
                    }
                    actions
                })
                .ok_or(NuxStatus::NotFound)?;
            let generation = video
                .with_downcast::<Video, _>(|v| v.playback.generation())
                .ok_or(NuxStatus::NotFound)?;
            for action in actions {
                let action = match action {
                    DecoderAction::Play => NuxVideoAction {
                        kind: 0,
                        value: 0.0,
                        generation,
                    },
                    DecoderAction::Pause => NuxVideoAction {
                        kind: 1,
                        value: 0.0,
                        generation,
                    },
                    DecoderAction::Seek {
                        seconds,
                        generation,
                    } => NuxVideoAction {
                        kind: 2,
                        value: seconds,
                        generation,
                    },
                    DecoderAction::Rate(rate) => NuxVideoAction {
                        kind: 3,
                        value: f64::from(rate),
                        generation,
                    },
                    DecoderAction::Volume(volume) => NuxVideoAction {
                        kind: 4,
                        value: f64::from(volume),
                        generation,
                    },
                    DecoderAction::Dispose => NuxVideoAction {
                        kind: 5,
                        value: 0.0,
                        generation,
                    },
                };
                with_platform_callback(|| unsafe { callback(user_data, &action) });
            }
            let after = video
                .with_downcast::<Video, _>(|v| {
                    (
                        v.playback.state(),
                        v.playback.generation(),
                        v.playback.position(),
                    )
                })
                .ok_or(NuxStatus::NotFound)?;
            occurrence.commit_runtime_change(before != after)
        }))
    })
}

/// Top-row-first opaque SDR RGBA8, or premultiplied RGBA8 sRGB. Caller storage
/// is borrowed for this call only; the renderer owns the uploaded frame.
#[repr(C)]
pub struct NuxVideoFrame {
    pub struct_size: u32,
    pub generation: u64,
    pub presentation_seconds: f64,
    pub width: u32,
    pub height: u32,
    pub row_bytes: u32,
    pub pixels: NuxByteView,
}

#[cfg(any(
    all(feature = "apple-metal", any(target_os = "ios", target_os = "macos")),
    feature = "android-vulkan"
))]
pub(crate) unsafe fn present_frame(
    video: &nuxie::CoreHandle,
    occurrence: &ArtboardOccurrence,
    frame: *const NuxVideoFrame,
    upload: impl FnOnce(u32, u32, u32, &[u8]) -> Result<Box<dyn nuxie::RenderImage>, NuxStatus>,
) -> Result<(), NuxStatus> {
    let frame = unsafe { frame.as_ref() }.ok_or(NuxStatus::NullArgument)?;
    if frame.struct_size as usize != std::mem::size_of::<NuxVideoFrame>() {
        return Err(NuxStatus::InvalidStructSize);
    }
    let required = (frame.row_bytes as usize)
        .checked_mul(frame.height as usize)
        .filter(|size| *size > 0 && *size <= 64 * 1024 * 1024)
        .ok_or(NuxStatus::LimitExceeded)?;
    if frame.width == 0
        || frame
            .width
            .checked_mul(4)
            .is_none_or(|row| row > frame.row_bytes)
        || required != frame.pixels.len
        || !frame.presentation_seconds.is_finite()
        || frame.presentation_seconds < 0.0
    {
        return Err(NuxStatus::InvalidArgument);
    }
    if frame.pixels.data.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let accepts = video
        .with_downcast::<Video, _>(|v| {
            v.playback.generation() == frame.generation
                && !matches!(
                    v.playback.state(),
                    PlaybackState::Disposed | PlaybackState::Failed
                )
        })
        .unwrap_or(false);
    if !accepts {
        return Ok(());
    }
    let pixels = unsafe { std::slice::from_raw_parts(frame.pixels.data, frame.pixels.len) };
    let image = upload(frame.width, frame.height, frame.row_bytes, pixels)?;
    let changed = video
        .with_downcast_mut::<Video, _>(|v| {
            v.present(
                frame.generation,
                Rc::from(image),
                frame.presentation_seconds,
            )
        })
        .ok_or(NuxStatus::NotFound)?;
    occurrence.commit_runtime_change(changed)
}

/// One event consumed from the occurrence's shared event queue. State=0
/// (`state` uses NuxVideoInfo's values), first-frame=1, looped=2,
/// play-blocked=3, error=4, resource-limited=5 (`state` is 1 when limited, 0
/// when restored). Luau nextEvent and this API consume the same queue;
/// hosts should select one event owner and fan out notifications themselves.
#[repr(C)]
pub struct NuxVideoEvent {
    pub kind: u32,
    pub state: u32,
}
/// Return NotFound when no event is pending. Output is written only on Ok.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_next_event(
    player: *const NuxPlayer,
    component_id: usize,
    out_event: *mut NuxVideoEvent,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, _| {
            if out_event.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            let event = video
                .with_downcast_mut::<Video, _>(|v| v.playback.pop_event())
                .flatten()
                .ok_or(NuxStatus::NotFound)?;
            let event = match event {
                PlaybackEvent::State(state) => NuxVideoEvent {
                    kind: 0,
                    state: state_id(state),
                },
                PlaybackEvent::FirstFrame => NuxVideoEvent { kind: 1, state: 0 },
                PlaybackEvent::Looped => NuxVideoEvent { kind: 2, state: 0 },
                PlaybackEvent::PlayBlocked => NuxVideoEvent { kind: 3, state: 0 },
                PlaybackEvent::Error => NuxVideoEvent { kind: 4, state: 0 },
                PlaybackEvent::ResourceLimited(limited) => NuxVideoEvent {
                    kind: 5,
                    state: u32::from(limited),
                },
            };
            unsafe {
                *out_event = event;
            }
            Ok(())
        }))
    })
}

/// Plain text cue; [start,end) is in the video's media timeline. Text is copied
/// during installation. Markup interpretation belongs to the caption importer.
#[repr(C)]
pub struct NuxVideoCaptionCue {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub text: NuxStringView,
}
pub type NuxVideoCaptionCallback =
    Option<unsafe extern "C" fn(*mut c_void, NuxStringView, NuxStringView)>;

/// Atomically replace captions with bounded owned cues. Empty input clears the
/// track. Validation failure preserves the old track. Supply a language tag;
/// callers render the projected plain text and expose it through accessibility.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_set_captions(
    player: *const NuxPlayer,
    component_id: usize,
    language: NuxStringView,
    cues: *const NuxVideoCaptionCue,
    count: usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, occurrence| {
            use nuxie::video::captions::{CaptionTrack, Cue};
            if count > 100_000 || language.len > 256 {
                return Err(NuxStatus::LimitExceeded);
            }
            if count != 0 && cues.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            let language = with_utf8_view(language, str::to_owned)?;
            let input = if count == 0 {
                &[]
            } else {
                unsafe { slice::from_raw_parts(cues, count) }
            };
            let mut total = 0usize;
            let mut owned = Vec::with_capacity(count);
            for cue in input {
                total = total
                    .checked_add(cue.text.len)
                    .filter(|n| *n <= 8 * 1024 * 1024)
                    .ok_or(NuxStatus::LimitExceeded)?;
                owned.push(Cue {
                    start: cue.start_seconds,
                    end: cue.end_seconds,
                    text: with_utf8_view(cue.text, str::to_owned)?,
                });
            }
            let track = if owned.is_empty() {
                None
            } else {
                Some(CaptionTrack::new(language, owned).map_err(|_| NuxStatus::InvalidArgument)?)
            };
            video
                .with_downcast_mut::<Video, _>(|v| v.set_captions(track))
                .ok_or(NuxStatus::NotFound)?;
            occurrence.commit_runtime_change(true)
        }))
    })
}

/// Synchronously visit the current caption language and text. Views live only
/// during the callback; copy them before returning. This never consumes cues.
/// Seek, pause and independent occurrences use the same playback clock as Luau.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_caption(
    player: *const NuxPlayer,
    component_id: usize,
    callback: NuxVideoCaptionCallback,
    user_data: *mut c_void,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, _| {
            let callback = callback.ok_or(NuxStatus::NullArgument)?;
            let (language, text) = video
                .with_downcast::<Video, _>(|v| (v.caption_language().to_owned(), v.caption_text()))
                .ok_or(NuxStatus::NotFound)?;
            with_platform_callback(|| unsafe {
                callback(
                    user_data,
                    NuxStringView {
                        data: language.as_ptr().cast(),
                        len: language.len(),
                    },
                    NuxStringView {
                        data: text.as_ptr().cast(),
                        len: text.len(),
                    },
                )
            });
            Ok(())
        }))
    })
}

/// Set the loop interval in seconds. Start is inclusive, end exclusive; zero
/// end means source duration. Enable looping separately with command kind 9.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_set_loop_range(
    player: *const NuxPlayer,
    component_id: usize,
    start_seconds: f64,
    end_seconds: f64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, occurrence| {
            video
                .with_downcast_mut::<Video, _>(|v| {
                    v.playback.enqueue(Command::LoopRange {
                        start: start_seconds,
                        end: end_seconds,
                    })
                })
                .ok_or(NuxStatus::NotFound)?
                .map_err(playback_status)?;
            occurrence.commit_runtime_change(true)
        }))
    })
}
