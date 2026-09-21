//! Host-neutral media bridge. Platform decoders own A/V timing; this surface
//! routes their commands and observations through the live scene occurrence.
use super::*;
use nuxie::runtime::generated::core_registry::CoreCapabilities;
use nuxie::video::{
    Video, VideoAsset,
    playback::{
        Command, DecoderAction, PlaybackError, PlaybackEvent, PlaybackState, SuspensionReason,
    },
};

const MAX_VIDEO_SCENE_OBJECTS: usize = 65_536;
const FIRST_NESTED_VIDEO_ID: usize = 1 << 30;

pub(super) struct VideoOccurrences {
    ids: std::collections::BTreeMap<(usize, usize, u64), usize>,
    next_id: usize,
}

impl Default for VideoOccurrences {
    fn default() -> Self {
        Self {
            ids: Default::default(),
            next_id: FIRST_NESTED_VIDEO_ID,
        }
    }
}

struct LiveVideo {
    id: usize,
    source_artboard_index: usize,
    source_component_id: usize,
    handle: nuxie::CoreHandle,
}

impl VideoOccurrences {
    fn collect(&mut self, root: &ArtboardInstance) -> Result<Vec<LiveVideo>, NuxStatus> {
        use std::collections::BTreeSet;
        let file = root.native_file();
        let sources = file.with_file(|file| {
            (0..file.artboard_count())
                .map(|index| file.artboard_handle(index))
                .collect::<Vec<_>>()
        });
        let mut pending = vec![(root.native_handle(), true)];
        let mut visited = BTreeSet::new();
        let mut live = BTreeSet::new();
        let mut output = Vec::new();
        let mut new_ids = std::collections::BTreeMap::new();
        let mut object_count = 0usize;
        while let Some((artboard, is_root)) = pending.pop() {
            if !visited.insert(artboard.core_handle().identity_key()) {
                continue;
            }
            if visited.len() > MAX_VIDEO_SCENE_OBJECTS {
                return Err(NuxStatus::LimitExceeded);
            }
            let source = artboard.with_artboard(|artboard| artboard.base.artboard_source_handle());
            let source_artboard_index = sources
                .iter()
                .position(|candidate| candidate.as_ref() == source.as_ref())
                .filter(|_| source.is_some())
                .ok_or(NuxStatus::RuntimeError)?;
            let objects = artboard.with_artboard(|artboard| artboard.objects().to_vec());
            object_count = object_count
                .checked_add(objects.len())
                .ok_or(NuxStatus::LimitExceeded)?;
            if object_count > MAX_VIDEO_SCENE_OBJECTS {
                return Err(NuxStatus::LimitExceeded);
            }
            for (source_component_id, handle) in objects.into_iter().enumerate() {
                let Some(handle) = handle else {
                    continue;
                };
                if handle.is_type_of(Video::TYPE_KEY) {
                    let identity = handle.identity_key();
                    live.insert(identity);
                    let id = if is_root {
                        source_component_id
                    } else if let Some(id) =
                        self.ids.get(&identity).or_else(|| new_ids.get(&identity))
                    {
                        *id
                    } else {
                        let id = self.next_id;
                        self.next_id = id
                            .checked_add(1)
                            .filter(|next| *next <= i64::MAX as usize)
                            .ok_or(NuxStatus::LimitExceeded)?;
                        new_ids.insert(identity, id);
                        id
                    };
                    output.push(LiveVideo {
                        id,
                        source_artboard_index,
                        source_component_id,
                        handle: handle.clone(),
                    });
                }
                let children = handle
                    .with(|object| {
                        let Some(host) = object.as_artboard_host() else {
                            return Ok(Vec::new());
                        };
                        if host.artboard_count() > MAX_VIDEO_SCENE_OBJECTS {
                            return Err(NuxStatus::LimitExceeded);
                        }
                        Ok((0..host.artboard_count())
                            .filter_map(|index| host.artboard_instance(index as i32))
                            .collect::<Vec<_>>())
                    })
                    .ok_or(NuxStatus::RuntimeError)??;
                if pending.len().saturating_add(children.len()) > MAX_VIDEO_SCENE_OBJECTS {
                    return Err(NuxStatus::LimitExceeded);
                }
                pending.extend(children.into_iter().map(|child| (child, false)));
            }
        }
        self.ids.retain(|identity, _| live.contains(identity));
        self.ids.extend(new_ids);
        Ok(output)
    }
}

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
    /// Authored component name in this player's artboard. Names may be empty or
    /// repeated; hosts must reject ambiguous named targets rather than selecting
    /// the first match. The view is borrowed for the duration of the callback.
    pub component_name: NuxStringView,
    /// Higher priorities win when a host cannot admit every visible decoder.
    pub priority: u32,
    /// Show immediately with poster=0, wait for first frame=1.
    pub readiness: u32,
    /// Definition artboard index in the imported file, not a mounted-instance index.
    pub source_artboard_index: usize,
    /// Definition-local slot used by the signed authored-target inventory.
    pub source_component_id: usize,
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
    let videos = player
        .video_occurrences
        .try_borrow_mut()
        .map_err(|_| NuxStatus::ReentrantCall)?
        .collect(
            &*player
                .artboard
                .instance
                .try_borrow()
                .map_err(|_| NuxStatus::ReentrantCall)?,
        )?;
    let video = videos
        .into_iter()
        .find(|video| video.id == component_id)
        .ok_or(NuxStatus::NotFound)?;
    body(&video.handle, &player.artboard)
}
pub(crate) fn status(result: Result<(), NuxStatus>) -> NuxStatus {
    result.err().unwrap_or(NuxStatus::Ok)
}

/// Query decoder demand after advancing the scene. Viewport coordinates are in
/// root-artboard space (undo the host's fit transform first). Includes authored
/// visibility, transforms, ancestor clipping and viewport intersection, even
/// before decoding a frame. Returns 0 or 1; output is unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_is_visible(
    player: *const NuxPlayer,
    component_id: usize,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    out_visible: *mut u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, _| {
            if out_visible.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            if ![min_x, min_y, max_x, max_y]
                .iter()
                .all(|value| value.is_finite())
                || min_x > max_x
                || min_y > max_y
            {
                return Err(NuxStatus::InvalidArgument);
            }
            let viewport = nuxie::runtime::semantic::semantic_snapshot::Bounds {
                min_x,
                min_y,
                max_x,
                max_y,
            };
            let visible = nuxie::video::visibility::is_visible_in(video, viewport)
                .map_err(|_| NuxStatus::RuntimeError)?;
            unsafe {
                *out_visible = u32::from(visible);
            }
            Ok(())
        }))
    })
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

/// Enumerate root, nested, and materialized list video occurrences. Root IDs
/// retain their component slots; nested IDs are opaque and never reused within
/// this player. Removed occurrences reject further commands. Definition fields
/// identify authored targets; component_id identifies the mounted player only.
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
            let videos = player
                .video_occurrences
                .try_borrow_mut()
                .map_err(|_| NuxStatus::ReentrantCall)?
                .collect(
                    &*player
                        .artboard
                        .instance
                        .try_borrow()
                        .map_err(|_| NuxStatus::ReentrantCall)?,
                )?;
            for video in videos {
                let id = video.id;
                let object = video.handle;
                object.with_downcast::<Video, _>(|v| {
                    let component_name = v.as_component().map_or("", |c| c.name());
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
                        component_name: NuxStringView {
                            data: component_name.as_ptr().cast(),
                            len: component_name.len(),
                        },
                        priority: settings.priority,
                        readiness: settings.readiness,
                        source_artboard_index: video.source_artboard_index,
                        source_component_id: video.source_component_id,
                    };
                    with_platform_callback(|| unsafe { callback(user_data, &info) });
                });
            }
            Ok(())
        })())
    })
}

/// Evaluate initial presentation against the live frame and decoded poster.
/// elapsed_seconds is monotonic time since this occurrence began waiting;
/// timeout_seconds must be within 0..=60 and optional must be 0 or 1.
/// Result: waiting=0, video frame=1, poster/optional blank=2, unavailable=3.
/// This query does not mutate playback. For authored wait-mode occurrences,
/// hosts latch the first non-waiting decision; seeks do not restart admission.
/// After choosing wait-mode fallback, stop decoding before drawing the poster.
/// Immediate-mode hosts present without this gate and continue decoding.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_readiness(
    player: *const NuxPlayer,
    component_id: usize,
    elapsed_seconds: f64,
    timeout_seconds: f64,
    optional: u32,
    out_readiness: *mut u32,
) -> NuxStatus {
    use nuxie::video::readiness::{FirstFrameGate, Readiness};
    ffi_guard(NuxStatus::RuntimeError, || {
        status(with_video(player, component_id, |video, _| {
            if out_readiness.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            if !elapsed_seconds.is_finite() || elapsed_seconds < 0.0 || optional > 1 {
                return Err(NuxStatus::InvalidArgument);
            }
            let result = video
                .with_downcast::<Video, _>(|video| {
                    let generation = video.playback.generation();
                    let mut gate = FirstFrameGate::new(
                        generation,
                        0.0,
                        timeout_seconds,
                        video.playback.settings().readiness == 1,
                        optional == 1,
                    )
                    .ok_or(NuxStatus::InvalidArgument)?;
                    let failed = matches!(
                        video.playback.state(),
                        PlaybackState::Failed | PlaybackState::Disposed
                    );
                    Ok(
                        match gate.evaluate(
                            generation,
                            elapsed_seconds,
                            !failed && video.has_video_frame(),
                            failed,
                            video.has_poster(),
                        ) {
                            Readiness::Waiting => 0,
                            Readiness::Frame => 1,
                            Readiness::Poster => 2,
                            Readiness::Unavailable => 3,
                        },
                    )
                })
                .ok_or(NuxStatus::NotFound)??;
            unsafe {
                *out_readiness = result;
            }
            Ok(())
        }))
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
/// play-blocked=5, failed=6, selected-seek-frame=7 (`value`=actual decoded PTS).
/// Kind 7 certifies pixels selected after successful current-generation seek completion;
/// deliver it immediately before presenting those pixels, never with a media-clock value.
/// Stale generations are ignored. Deliver every action
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
            if observation > 7
                || (matches!(observation, 1 | 7) && (!value.is_finite() || value < 0.0))
            {
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
                        7 => {
                            v.playback.observe_selected_seek_frame(generation, value);
                        }
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

#[cfg(test)]
mod occurrence_tests {
    use super::*;
    use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
    use nuxie_runtime::source::{
        core::CoreArena, nested_artboard::NestedArtboard,
        viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
    };

    #[test]
    fn list_videos_keep_surviving_ids_and_reject_removed_rows() {
        use nuxie_runtime::source::{
            artboard_component_list::ArtboardComponentList,
            viewmodel::{
                viewmodel_instance::ViewModelInstance,
                viewmodel_instance_list_item::ViewModelInstanceListItem,
            },
        };
        let records = vec![
            R {
                type_key: 23,
                properties: vec![],
            },
            R {
                type_key: 60000,
                properties: vec![P {
                    key: 60000,
                    value: V::String("asset:clip".into()),
                }],
            },
            R {
                type_key: 1,
                properties: vec![],
            },
            R {
                type_key: 559,
                properties: vec![P {
                    key: 5,
                    value: V::Uint(0),
                }],
            },
            R {
                type_key: 1,
                properties: vec![P {
                    key: 583,
                    value: V::Uint(0),
                }],
            },
            R {
                type_key: 60001,
                properties: vec![
                    P {
                        key: 5,
                        value: V::Uint(0),
                    },
                    P {
                        key: 206,
                        value: V::Uint(0),
                    },
                ],
            },
        ];
        let bytes = nuxie_binary::encode_runtime_file(
            &nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap(),
        )
        .unwrap();
        unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
            let info = unsafe { &*info };
            assert_eq!(info.source_artboard_index, 1);
            assert_eq!(info.source_component_id, 1);
            unsafe { &mut *data.cast::<Vec<usize>>() }.push(info.component_id);
        }
        unsafe fn ids(player: *const NuxPlayer) -> Vec<usize> {
            let mut ids = Vec::new();
            assert_eq!(
                unsafe {
                    nux_player_visit_videos(player, Some(collect), ptr::from_mut(&mut ids).cast())
                },
                NuxStatus::Ok
            );
            ids.sort();
            ids
        }
        let (mut file, mut artboard, mut player, mut result) = (
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        let capabilities = NuxVideoPlaybackCapabilities {
            playback_available: 1,
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_file_import_with_video_capabilities(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &capabilities,
                    &mut file,
                    &mut result
                ),
                NuxStatus::Ok
            );
            nux_capi_result_free(result);
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
            let host = (&*player)
                .artboard
                .instance
                .borrow()
                .object_handle(1)
                .unwrap();
            let arena = CoreArena::default();
            let make_item = || {
                let instance = arena.insert(ViewModelInstance::default());
                let mut item = ViewModelInstanceListItem::default();
                item.set_view_model_instance(Some(instance));
                arena.insert(item)
            };
            let a = make_item();
            let b = make_item();
            assert!(ids(player).is_empty());
            ArtboardComponentList::update_list_occurrence(&host, &[a.clone(), b.clone()]);
            let first = ids(player);
            assert_eq!(first.len(), 2);
            assert_ne!(first[0], first[1]);
            ArtboardComponentList::update_list_occurrence(&host, &[b.clone(), a.clone()]);
            assert_eq!(ids(player), first, "reordering preserves playback owners");
            ArtboardComponentList::update_list_occurrence(&host, &[b]);
            let surviving = ids(player);
            assert_eq!(surviving.len(), 1);
            assert!(first.contains(&surviving[0]));
            let removed = *first.iter().find(|id| **id != surviving[0]).unwrap();
            assert_eq!(
                nux_player_video_command(player, removed, 0, 0.0, 0),
                NuxStatus::NotFound
            );
            assert_eq!(
                nux_player_video_command(player, surviving[0], 0, 0.0, 0),
                NuxStatus::Ok
            );
            ArtboardComponentList::update_list_occurrence(&host, &[]);
            assert!(ids(player).is_empty());
            ArtboardComponentList::update_list_occurrence(&host, &[make_item()]);
            let remounted = ids(player);
            assert_eq!(remounted.len(), 1);
            assert!(!first.contains(&remounted[0]));
            nux_player_free(player);
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
        }
    }

    #[test]
    fn detached_nested_video_rejects_stale_commands_and_remount_gets_a_new_id() {
        let records = vec![
            R {
                type_key: 23,
                properties: vec![],
            },
            R {
                type_key: 60000,
                properties: vec![P {
                    key: 60000,
                    value: V::String("asset:clip".into()),
                }],
            },
            R {
                type_key: 1,
                properties: vec![],
            },
            R {
                type_key: 92,
                properties: vec![
                    P {
                        key: 5,
                        value: V::Uint(0),
                    },
                    P {
                        key: 197,
                        value: V::Uint(1),
                    },
                ],
            },
            R {
                type_key: 1,
                properties: vec![],
            },
            R {
                type_key: 60001,
                properties: vec![
                    P {
                        key: 5,
                        value: V::Uint(0),
                    },
                    P {
                        key: 206,
                        value: V::Uint(0),
                    },
                ],
            },
        ];
        let bytes = nuxie_binary::encode_runtime_file(
            &nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap(),
        )
        .unwrap();
        unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
            unsafe { &mut *data.cast::<Vec<usize>>() }.push(unsafe { (*info).component_id });
        }
        unsafe fn ids(player: *const NuxPlayer) -> Vec<usize> {
            let mut ids = Vec::new();
            assert_eq!(
                unsafe {
                    nux_player_visit_videos(player, Some(collect), ptr::from_mut(&mut ids).cast())
                },
                NuxStatus::Ok
            );
            ids
        }
        let (mut file, mut artboard, mut player, mut result) = (
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        let capabilities = NuxVideoPlaybackCapabilities {
            playback_available: 1,
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_file_import_with_video_capabilities(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &capabilities,
                    &mut file,
                    &mut result
                ),
                NuxStatus::Ok
            );
            nux_capi_result_free(result);
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
            let first = ids(player);
            assert_eq!(first.len(), 1);
            let host = (&*player)
                .artboard
                .instance
                .borrow()
                .object_handle(1)
                .unwrap();
            let arena = CoreArena::default();
            let mut property = ViewModelInstanceArtboard::default();
            property.set_property_value(u32::MAX);
            let property = arena.insert(property);
            NestedArtboard::update_artboard_occurrence(&host, Some(property.clone()));
            assert!(ids(player).is_empty());
            assert_eq!(
                nux_player_video_command(player, first[0], 0, 0.0, 0),
                NuxStatus::NotFound
            );
            property.with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
                property.set_property_value(1)
            });
            NestedArtboard::update_artboard_occurrence(&host, Some(property));
            let second = ids(player);
            assert_eq!(second.len(), 1);
            assert_ne!(first[0], second[0]);
            assert_eq!(
                nux_player_video_command(player, first[0], 0, 0.0, 0),
                NuxStatus::NotFound
            );
            assert_eq!(
                nux_player_video_command(player, second[0], 0, 0.0, 0),
                NuxStatus::Ok
            );
            nux_player_free(player);
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
        }
    }
}
