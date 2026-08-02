//! Faithful Rive profile records and profile-stream serialization.
//!
//! Record ownership and the `RPRF` byte format directly correspond to pinned
//! C++ `src/profiler/rive_profile.cpp`. MicroProfile itself is intentionally
//! adapted behind [`ProfileCapture`], so embedders can supply a Rust capture
//! backend without linking C++.

use nuxie_binary::BinaryWriter;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

pub const PROFILE_LOG_NONE: u32 = 0;
pub const PROFILE_LOG_TRANSITION_RECORDS: u32 = 1 << 0;
pub const PROFILE_LOG_LISTENER_PERFORM_CHANGES: u32 = 1 << 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtboardPathSegment {
    pub segment_type: u8,
    pub name_id: u32,
    pub index: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionRecord {
    pub artboard_id: u32,
    pub sm_id: u32,
    pub layer_id: u32,
    pub from_state_id: u32,
    pub to_state_id: u32,
    pub tick: u64,
    pub path: Vec<ArtboardPathSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListenerPerformChangeRecord {
    pub artboard_id: u32,
    pub sm_id: u32,
    pub listener_name_id: u32,
    pub listener_type: u32,
    pub hit_event: u32,
    pub pointer_id: u32,
    pub tick: u64,
}

/// Unresolved, owner-safe input used to build a C++ `ArtboardPathSegment`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfilePathSegment {
    pub segment_type: u8,
    pub name: String,
    pub index: i32,
}

impl ProfilePathSegment {
    pub fn nested_artboard(name: impl Into<String>) -> Self {
        Self {
            segment_type: 0,
            name: name.into(),
            index: -1,
        }
    }

    pub fn component_list(name: impl Into<String>, index: i32) -> Self {
        Self {
            segment_type: 1,
            name: name.into(),
            index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileCaptureTimer {
    pub name: String,
    pub group_index: u32,
    pub color: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileCaptureMetadata {
    pub ticks_per_second_cpu: i64,
    pub timers: Vec<ProfileCaptureTimer>,
    pub groups: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProfileCaptureEvent {
    pub event_type: u8,
    pub timer_index: u32,
    pub tick: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileCaptureFrame {
    pub frame_start_cpu: i64,
    pub next_frame_start_cpu: i64,
    pub events: Vec<ProfileCaptureEvent>,
}

/// Pluggable Rust seam replacing the MicroProfile dependency.
///
/// Frame indices are monotonically increasing. `captured_frame(index)` must
/// return the frame beginning at `index`, including the next frame's start
/// tick, for indices still retained by `max_frame_history`.
/// `gpu_frame_delay()` is the complete flush delay: the capture backend's GPU
/// delay plus the extra frame retained by the pinned `RiveProfile` algorithm.
pub trait ProfileCapture: Send {
    fn init(&mut self) {}
    fn frame(&mut self) {}
    fn end_frame(&mut self) {}
    fn set_enabled(&mut self, _enabled: bool) {}
    fn tick(&mut self) -> u64;
    fn metadata(&self) -> ProfileCaptureMetadata;
    fn current_frame_index(&self) -> u64;
    fn gpu_frame_delay(&self) -> u64;
    fn max_frame_history(&self) -> u64;
    fn captured_frame(&self, frame_index: u64) -> Option<ProfileCaptureFrame>;
}

pub type TransitionFlushCallback = Box<dyn FnMut(&[TransitionRecord]) + Send>;
pub type ListenerFlushCallback = Box<dyn FnMut(&[ListenerPerformChangeRecord]) + Send>;

pub struct RiveProfile {
    capture: Box<dyn ProfileCapture>,
    transition_flush_callback: Option<TransitionFlushCallback>,
    listener_flush_callback: Option<ListenerFlushCallback>,
    transition_records: Vec<TransitionRecord>,
    listener_records: Vec<ListenerPerformChangeRecord>,
    string_ids: HashMap<String, u32>,
    string_list: Vec<String>,
    profiling_active: bool,
    log_flags: u32,
    header_written: bool,
    last_flushed_frame_index: u64,
}

impl RiveProfile {
    pub fn new(capture: Box<dyn ProfileCapture>) -> Self {
        Self {
            capture,
            transition_flush_callback: None,
            listener_flush_callback: None,
            transition_records: Vec::new(),
            listener_records: Vec::new(),
            string_ids: HashMap::new(),
            string_list: Vec::new(),
            profiling_active: false,
            log_flags: PROFILE_LOG_TRANSITION_RECORDS | PROFILE_LOG_LISTENER_PERFORM_CHANGES,
            header_written: false,
            last_flushed_frame_index: 0,
        }
    }

    pub fn init(&mut self) {
        self.capture.init();
    }

    pub fn frame(&mut self) {
        self.capture.frame();
    }

    pub fn end_frame(&mut self) {
        if self.profiling_active {
            self.capture.end_frame();
        }
    }

    pub fn start(&mut self) {
        self.profiling_active = true;
        self.header_written = false;
        self.last_flushed_frame_index = 0;
        self.string_ids.clear();
        self.string_list.clear();
        self.capture.set_enabled(true);
        self.last_flushed_frame_index = self.capture.current_frame_index();
    }

    pub fn stop(&mut self) {
        self.profiling_active = false;
        self.capture.set_enabled(false);
    }

    pub fn is_active(&self) -> bool {
        self.profiling_active
    }

    pub fn set_capture(&mut self, capture: Box<dyn ProfileCapture>) {
        self.capture.set_enabled(false);
        self.capture = capture;
        self.profiling_active = false;
        self.header_written = false;
        self.last_flushed_frame_index = 0;
        self.transition_records.clear();
        self.listener_records.clear();
        self.string_ids.clear();
        self.string_list.clear();
    }

    pub fn set_log_flags(&mut self, flags: u32) {
        self.log_flags = flags;
    }

    pub fn log_flags(&self) -> u32 {
        self.log_flags
    }

    pub fn set_transition_flush_callback(&mut self, callback: Option<TransitionFlushCallback>) {
        self.transition_flush_callback = callback;
    }

    pub fn set_listener_perform_change_flush_callback(
        &mut self,
        callback: Option<ListenerFlushCallback>,
    ) {
        self.listener_flush_callback = callback;
    }

    pub fn record_transition(
        &mut self,
        artboard_name: &str,
        state_machine_name: &str,
        layer_name: &str,
        from_state_name: &str,
        to_state_name: &str,
        path: &[ProfilePathSegment],
    ) {
        if self.log_flags & PROFILE_LOG_TRANSITION_RECORDS == 0
            || !self.profiling_active
            || self.transition_flush_callback.is_none()
        {
            return;
        }
        let artboard_id = self.resolve_string_id(artboard_name);
        let sm_id = self.resolve_string_id(state_machine_name);
        let layer_id = self.resolve_string_id(layer_name);
        let from_state_id = self.resolve_string_id(from_state_name);
        let to_state_id = self.resolve_string_id(to_state_name);
        let path = path
            .iter()
            .map(|segment| ArtboardPathSegment {
                segment_type: segment.segment_type,
                name_id: self.resolve_string_id(&segment.name),
                index: segment.index,
            })
            .collect();
        self.transition_records.push(TransitionRecord {
            artboard_id,
            sm_id,
            layer_id,
            from_state_id,
            to_state_id,
            tick: self.capture.tick(),
            path,
        });
    }

    pub fn flush_transition_records(&mut self) {
        if let Some(callback) = self.transition_flush_callback.as_mut()
            && !self.transition_records.is_empty()
        {
            callback(&self.transition_records);
        }
        self.transition_records.clear();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_listener_perform_change(
        &mut self,
        artboard_name: &str,
        state_machine_name: &str,
        listener_name: &str,
        listener_type: u32,
        hit_event: u32,
        pointer_id: u32,
    ) {
        if self.log_flags & PROFILE_LOG_LISTENER_PERFORM_CHANGES == 0
            || !self.profiling_active
            || self.listener_flush_callback.is_none()
        {
            return;
        }
        let artboard_id = self.resolve_string_id(artboard_name);
        let sm_id = self.resolve_string_id(state_machine_name);
        let listener_name_id = self.resolve_string_id(listener_name);
        self.listener_records.push(ListenerPerformChangeRecord {
            artboard_id,
            sm_id,
            listener_name_id,
            listener_type,
            hit_event,
            pointer_id,
            tick: self.capture.tick(),
        });
    }

    pub fn flush_listener_perform_change_records(&mut self) {
        if let Some(callback) = self.listener_flush_callback.as_mut()
            && !self.listener_records.is_empty()
        {
            callback(&self.listener_records);
        }
        self.listener_records.clear();
    }

    pub fn resolve_string_id(&mut self, value: &str) -> u32 {
        if let Some(id) = self.string_ids.get(value) {
            return *id;
        }
        let id = self.string_list.len() as u32;
        self.string_ids.insert(value.to_owned(), id);
        self.string_list.push(value.to_owned());
        id
    }

    pub fn string_table(&self) -> &[String] {
        &self.string_list
    }

    pub fn flush_frame_data_to(&mut self, buffer: &mut Vec<u8>) {
        let mut writer = BinaryWriter::new(buffer);
        if !self.header_written {
            write_profile_header(&mut writer, &self.capture.metadata());
            self.header_written = true;
        }
        let current_frame_index = self.capture.current_frame_index();
        let gpu_delay = self.capture.gpu_frame_delay();
        if current_frame_index <= self.last_flushed_frame_index.saturating_add(gpu_delay) {
            return;
        }
        let end_frame_index = current_frame_index.saturating_sub(gpu_delay);
        let max_history = self
            .capture
            .max_frame_history()
            .saturating_sub(gpu_delay)
            .saturating_sub(2);
        if end_frame_index.saturating_sub(self.last_flushed_frame_index) > max_history {
            self.last_flushed_frame_index = end_frame_index.saturating_sub(max_history);
        }
        for frame_index in self.last_flushed_frame_index..end_frame_index {
            if let Some(frame) = self.capture.captured_frame(frame_index) {
                write_frame_events(&mut writer, &frame);
            }
        }
        self.last_flushed_frame_index = end_frame_index;
    }
}

fn write_profile_header(writer: &mut BinaryWriter<'_>, metadata: &ProfileCaptureMetadata) {
    writer.write_u32(0x5250_5246);
    writer.write_u32(2);
    writer.write_bytes(&metadata.ticks_per_second_cpu.to_le_bytes());
    writer.write_var_uint32(metadata.timers.len() as u32);
    for timer in &metadata.timers {
        writer.write_string(timer.name.as_bytes());
        writer.write_u32(timer.group_index);
        writer.write_u32(timer.color);
    }
    writer.write_var_uint32(metadata.groups.len() as u32);
    for group in &metadata.groups {
        writer.write_string(group.as_bytes());
    }
}

fn write_frame_events(writer: &mut BinaryWriter<'_>, frame: &ProfileCaptureFrame) {
    let mut payload = Vec::new();
    {
        let mut payload_writer = BinaryWriter::new(&mut payload);
        payload_writer.write_bytes(&frame.frame_start_cpu.to_le_bytes());
        payload_writer.write_bytes(&frame.next_frame_start_cpu.to_le_bytes());
        payload_writer.write_var_uint32(frame.events.len() as u32);
        for event in &frame.events {
            payload_writer.write_u8(event.event_type);
            payload_writer.write_var_uint32(event.timer_index);
            payload_writer.write_bytes(&event.tick.to_le_bytes());
        }
    }
    writer.write_u8(0x01);
    writer.write_var_uint32(payload.len() as u32);
    writer.write_bytes(&payload);
}

struct SystemProfileCapture {
    epoch: Instant,
    enabled: bool,
    current_frame_index: u64,
    frame_starts: VecDeque<(u64, i64)>,
}

impl Default for SystemProfileCapture {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            enabled: false,
            current_frame_index: 0,
            frame_starts: VecDeque::from([(0, 0)]),
        }
    }
}

impl SystemProfileCapture {
    fn elapsed_ticks(&self) -> i64 {
        i64::try_from(self.epoch.elapsed().as_nanos()).unwrap_or(i64::MAX)
    }
}

impl ProfileCapture for SystemProfileCapture {
    fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }
        self.current_frame_index = self.current_frame_index.saturating_add(1);
        self.frame_starts
            .push_back((self.current_frame_index, self.elapsed_ticks()));
        while self.frame_starts.len() > 512 {
            self.frame_starts.pop_front();
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn tick(&mut self) -> u64 {
        self.elapsed_ticks() as u64
    }

    fn metadata(&self) -> ProfileCaptureMetadata {
        ProfileCaptureMetadata {
            ticks_per_second_cpu: 1_000_000_000,
            ..ProfileCaptureMetadata::default()
        }
    }

    fn current_frame_index(&self) -> u64 {
        self.current_frame_index
    }

    fn gpu_frame_delay(&self) -> u64 {
        1
    }

    fn max_frame_history(&self) -> u64 {
        512
    }

    fn captured_frame(&self, frame_index: u64) -> Option<ProfileCaptureFrame> {
        let position = self
            .frame_starts
            .iter()
            .position(|(index, _)| *index == frame_index)?;
        let frame_start_cpu = self.frame_starts.get(position)?.1;
        let next_frame_start_cpu = self.frame_starts.get(position.saturating_add(1))?.1;
        Some(ProfileCaptureFrame {
            frame_start_cpu,
            next_frame_start_cpu,
            events: Vec::new(),
        })
    }
}

fn global_profile() -> &'static Mutex<RiveProfile> {
    static PROFILE: OnceLock<Mutex<RiveProfile>> = OnceLock::new();
    PROFILE.get_or_init(|| Mutex::new(RiveProfile::new(Box::new(SystemProfileCapture::default()))))
}

static GLOBAL_RECORD_FLAGS: AtomicU32 = AtomicU32::new(PROFILE_LOG_NONE);

fn enabled_record_flags(profile: &RiveProfile) -> u32 {
    if !profile.profiling_active {
        return PROFILE_LOG_NONE;
    }
    let mut flags = PROFILE_LOG_NONE;
    if profile.transition_flush_callback.is_some() {
        flags |= profile.log_flags & PROFILE_LOG_TRANSITION_RECORDS;
    }
    if profile.listener_flush_callback.is_some() {
        flags |= profile.log_flags & PROFILE_LOG_LISTENER_PERFORM_CHANGES;
    }
    flags
}

fn lock_profile() -> MutexGuard<'static, RiveProfile> {
    global_profile()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn with_rive_profile<R>(operation: impl FnOnce(&mut RiveProfile) -> R) -> R {
    let mut profile = lock_profile();
    let result = operation(&mut profile);
    GLOBAL_RECORD_FLAGS.store(enabled_record_flags(&profile), Ordering::Release);
    result
}

pub(crate) fn record_global_transition(
    artboard_name: &str,
    state_machine_name: &str,
    layer_name: &str,
    from_state_name: &str,
    to_state_name: &str,
    path: &[ProfilePathSegment],
) {
    if GLOBAL_RECORD_FLAGS.load(Ordering::Acquire) & PROFILE_LOG_TRANSITION_RECORDS == 0 {
        return;
    }
    lock_profile().record_transition(
        artboard_name,
        state_machine_name,
        layer_name,
        from_state_name,
        to_state_name,
        path,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn record_global_listener_perform_change(
    artboard_name: &str,
    state_machine_name: &str,
    listener_name: &str,
    listener_type: u32,
    hit_event: u32,
    pointer_id: u32,
) {
    if GLOBAL_RECORD_FLAGS.load(Ordering::Acquire) & PROFILE_LOG_LISTENER_PERFORM_CHANGES == 0 {
        return;
    }
    lock_profile().record_listener_perform_change(
        artboard_name,
        state_machine_name,
        listener_name,
        listener_type,
        hit_event,
        pointer_id,
    );
}
