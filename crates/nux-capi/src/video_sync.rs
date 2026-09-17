//! Thread-affine ownership and native-clock input for the shared controller.
use super::*;
use nuxie::video::{
    playback::Command,
    sync::{MediaClock, SyncError, SynchronizationGroup},
};

/// One member of a synchronization group. The first member is its leader.
/// Component IDs are local to the supplied live player occurrence.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NuxVideoSyncMember {
    pub player: *const NuxPlayer,
    pub component_id: usize,
}

/// Current native media clock, in the same order as group creation. Mark an
/// unavailable/seeking clock available=0. Boolean fields must be 0 or 1.
/// A stale generation is ignored; never supply a decoded frame PTS here.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NuxVideoClockSample {
    pub generation: u64,
    pub seconds: f64,
    pub rate: f64,
    pub playing: u32,
    pub available: u32,
}

/// Drift of one follower relative to the leader, valid during this callback.
#[repr(C)]
pub struct NuxVideoSyncMeasurement {
    pub member_index: usize,
    pub drift_seconds: f64,
    pub seek_prediction_seconds: f64,
    pub corrected: u32,
}
pub type NuxVideoSyncCallback =
    Option<unsafe extern "C" fn(*mut c_void, *const NuxVideoSyncMeasurement)>;

/// Retains scene occurrences, not decoder handles. Freeing the group stops
/// correction and releases those references; it does not dispose its videos.
pub struct NuxVideoSyncGroup {
    controller: RefCell<SynchronizationGroup>,
    members: Vec<nuxie::CoreHandle>,
    occurrences: Vec<Rc<ArtboardOccurrence>>,
}

impl NuxVideoSyncGroup {
    fn invalidate_members(&self) -> Result<(), NuxStatus> {
        for occurrence in &self.occurrences {
            occurrence.commit_runtime_change(true)?;
        }
        Ok(())
    }
}

pub(crate) fn sync_status(error: SyncError) -> NuxStatus {
    match error {
        SyncError::InvalidMember => NuxStatus::NotFound,
        SyncError::DuplicateMember | SyncError::InvalidPolicy | SyncError::InvalidTime => {
            NuxStatus::InvalidArgument
        }
        SyncError::Command(nuxie::video::playback::PlaybackError::QueueFull) => {
            NuxStatus::LimitExceeded
        }
        SyncError::Command(nuxie::video::playback::PlaybackError::InvalidValue) => {
            NuxStatus::InvalidArgument
        }
        SyncError::Command(_) => NuxStatus::RuntimeError,
    }
}

/// Create a group of 2..64 distinct videos. The first is the leader. Tolerance
/// and correction cooldown are finite positive seconds. All members must be
/// on the calling thread. The group retains occurrences after player handles
/// close. All later group calls remain creator-thread affine.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_sync_group_new(
    members: *const NuxVideoSyncMember,
    count: usize,
    tolerance_seconds: f64,
    cooldown_seconds: f64,
    out_group: *mut *mut NuxVideoSyncGroup,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        video::status((|| {
            if out_group.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            unsafe {
                *out_group = ptr::null_mut();
            }
            if !(2..=64).contains(&count) {
                return Err(NuxStatus::InvalidArgument);
            }
            if members.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            let definitions = unsafe { slice::from_raw_parts(members, count) };
            let mut cores = Vec::with_capacity(count);
            let mut occurrences = Vec::new();
            for member in definitions {
                video::with_video(member.player, member.component_id, |core, _| {
                    cores.push(core.clone());
                    // with_video validates this pointer and its creator thread.
                    let occurrence = unsafe { &*member.player }.artboard.clone();
                    if !occurrences.iter().any(|old| Rc::ptr_eq(old, &occurrence)) {
                        occurrences.push(occurrence);
                    }
                    Ok(())
                })?;
            }
            let controller = SynchronizationGroup::new(
                cores[0].clone(),
                cores[1..].to_vec(),
                tolerance_seconds,
                cooldown_seconds,
            )
            .map_err(sync_status)?;
            let handle = Box::into_raw(Box::new(NuxVideoSyncGroup {
                controller: RefCell::new(controller),
                members: cores,
                occurrences,
            }));
            register_handle(handle, HandleKind::VideoSyncGroup, thread::current().id());
            unsafe {
                *out_group = handle;
            }
            Ok(())
        })())
    })
}

fn with_group<R>(
    group: *const NuxVideoSyncGroup,
    body: impl FnOnce(&NuxVideoSyncGroup) -> Result<R, NuxStatus>,
) -> Result<R, NuxStatus> {
    let _handle = enter_handle(group, HandleKind::VideoSyncGroup)?;
    let group = unsafe { &*group };
    let _occurrences = group
        .occurrences
        .iter()
        .map(|occurrence| enter_occurrence(occurrence))
        .collect::<Result<Vec<_>, _>>()?;
    body(group)
}

/// Queue the same authored command on every member atomically. Command numbers
/// and arguments match nux_player_video_command. Lifecycle vetoes still apply.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_sync_group_command(
    group: *const NuxVideoSyncGroup,
    kind: u32,
    value: f64,
    reason: u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        video::status(with_group(group, |group| {
            let command = video::command(kind, value, reason)?;
            group
                .controller
                .try_borrow_mut()
                .map_err(|_| NuxStatus::ReentrantCall)?
                .command(command)
                .map_err(sync_status)?;
            group.invalidate_members()
        }))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_sync_group_set_loop_range(
    group: *const NuxVideoSyncGroup,
    start_seconds: f64,
    end_seconds: f64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        video::status(with_group(group, |group| {
            group
                .controller
                .try_borrow_mut()
                .map_err(|_| NuxStatus::ReentrantCall)?
                .command(Command::LoopRange {
                    start: start_seconds,
                    end: end_seconds,
                })
                .map_err(sync_status)?;
            group.invalidate_members()
        }))
    })
}

/// Call after processing member commands/observations, with current native
/// clocks and monotonic host seconds. Samples are borrowed only for this call;
/// count must equal the group member count. Corrections enqueue ordinary seeks
/// for the host to drain. Rate-mismatched, stale, suspended or unavailable
/// members are skipped. Optional callbacks cannot reenter any C API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_sync_group_update(
    group: *const NuxVideoSyncGroup,
    monotonic_seconds: f64,
    samples: *const NuxVideoClockSample,
    count: usize,
    callback: NuxVideoSyncCallback,
    user_data: *mut c_void,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        video::status(with_group(group, |group| {
            if count != group.members.len() {
                return Err(NuxStatus::InvalidArgument);
            }
            if samples.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            let samples = unsafe { slice::from_raw_parts(samples, count) };
            // Validate the entire input before the controller can queue a correction.
            for sample in samples {
                if sample.available > 1
                    || sample.playing > 1
                    || (sample.available == 1
                        && (!sample.seconds.is_finite()
                            || sample.seconds < 0.0
                            || !sample.rate.is_finite()
                            || sample.rate < 0.0))
                {
                    return Err(NuxStatus::InvalidArgument);
                }
            }
            let measurements = group
                .controller
                .try_borrow_mut()
                .map_err(|_| NuxStatus::ReentrantCall)?
                .correct(monotonic_seconds, |member| {
                    let index = group
                        .members
                        .iter()
                        .position(|core| core.identity_key() == member.identity_key())?;
                    let sample = samples[index];
                    (sample.available == 1).then_some(MediaClock {
                        generation: sample.generation,
                        seconds: sample.seconds,
                        rate: sample.rate,
                        playing: sample.playing == 1,
                    })
                })
                .map_err(sync_status)?;
            if measurements.iter().any(|measurement| measurement.corrected) {
                group.invalidate_members()?;
            }
            if let Some(callback) = callback {
                for measurement in measurements {
                    let view = NuxVideoSyncMeasurement {
                        member_index: measurement.follower + 1,
                        drift_seconds: measurement.seconds,
                        seek_prediction_seconds: measurement.seek_prediction_seconds,
                        corrected: u32::from(measurement.corrected),
                    };
                    with_platform_callback(|| unsafe { callback(user_data, &view) });
                }
            }
            Ok(())
        }))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_sync_group_free(group: *mut NuxVideoSyncGroup) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if group.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(group, HandleKind::VideoSyncGroup) {
            return status;
        }
        unsafe {
            drop(Box::from_raw(group));
        }
        NuxStatus::Ok
    })
}

/// Feed the current native clock to groups authored by Luau in this occurrence.
/// Use one monotonic seconds domain for all players; a null sample clears clock
/// availability. Call after commands/observations, even when no frame is uploaded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_report_clock(
    player: *const NuxPlayer,
    component_id: usize,
    monotonic_seconds: f64,
    sample: *const NuxVideoClockSample,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        video::status(video::with_video(
            player,
            component_id,
            |video, occurrence| {
                let sample = if sample.is_null() {
                    None
                } else {
                    let sample = unsafe { sample.read() };
                    if sample.playing > 1 || sample.available > 1 {
                        return Err(NuxStatus::InvalidArgument);
                    }
                    (sample.available == 1).then_some(MediaClock {
                        generation: sample.generation,
                        seconds: sample.seconds,
                        rate: sample.rate,
                        playing: sample.playing == 1,
                    })
                };
                let result =
                    nuxie::video::sync::report_media_clock(video, monotonic_seconds, sample);
                // Independent groups may correct successfully before another
                // group reports an error. Preserve scheduling for those commands.
                if !matches!(result, Ok(false)) {
                    occurrence.commit_runtime_change(true)?;
                }
                result.map(|_| ()).map_err(sync_status)
            },
        ))
    })
}
