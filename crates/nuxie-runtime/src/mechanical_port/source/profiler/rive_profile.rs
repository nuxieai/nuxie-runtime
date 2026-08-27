use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::{Mutex, OnceLock};

pub const PROFILE_LOG_NONE: u32 = 0;
pub const PROFILE_LOG_TRANSITION_RECORDS: u32 = 1 << 0;
pub const PROFILE_LOG_LISTENER_PERFORM_CHANGES: u32 = 1 << 1;

const NESTED_ARTBOARD_TYPE_KEY: u16 = 92;
const ARTBOARD_COMPONENT_LIST_TYPE_KEY: u16 = 559;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtboardPathSegment {
    pub segment_type: u8,
    pub name_id: u32,
    pub index: i32,
}

impl Default for ArtboardPathSegment {
    fn default() -> Self {
        Self {
            segment_type: 0,
            name_id: 0,
            index: -1,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransitionRecord {
    pub artboard_id: u32,
    pub sm_id: u32,
    pub layer_id: u32,
    pub from_state_id: u32,
    pub to_state_id: u32,
    pub tick: u64,
    pub path: Vec<ArtboardPathSegment>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListenerPerformChangeRecord {
    pub artboard_id: u32,
    pub sm_id: u32,
    pub listener_name_id: u32,
    pub listener_type: u32,
    pub hit_event: u32,
    pub pointer_id: u32,
    pub tick: u64,
}

pub trait ProfileArtboard {
    fn name(&self) -> &str;
    fn host(&self) -> Option<NonNull<dyn ProfileArtboardHost>>;
    fn is_instance(&self) -> bool;
}

pub trait ProfileArtboardHost {
    fn core_type(&self) -> u16;
    fn name(&self) -> &str;
    fn index_of_artboard_instance(&self, artboard: NonNull<dyn ProfileArtboard>) -> i32;
    fn parent_artboard(&self) -> Option<NonNull<dyn ProfileArtboard>>;
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Debug, Default)]
pub struct MicroProfileTimerInfo {
    pub name: String,
    pub group_index: u32,
    pub color: u32,
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Debug, Default)]
pub struct MicroProfileGroupInfo {
    pub name: String,
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Copy, Debug, Default)]
pub enum MicroProfileLogType {
    Enter,
    Leave,
    #[default]
    Other,
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Copy, Debug, Default)]
pub struct MicroProfileLogEntry {
    pub log_type: MicroProfileLogType,
    pub timer_index: u32,
    pub tick: u64,
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Debug, Default)]
pub struct MicroProfileThreadLog {
    pub log: Vec<MicroProfileLogEntry>,
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Debug, Default)]
pub struct MicroProfileFrameState {
    pub frame_start_cpu: i64,
    pub log_start: Vec<u32>,
}

#[cfg(feature = "rive_microprofile")]
#[derive(Clone, Debug, Default)]
pub struct MicroProfileState {
    pub total_timers: u32,
    pub timer_info: Vec<MicroProfileTimerInfo>,
    pub group_count: u32,
    pub group_info: Vec<MicroProfileGroupInfo>,
    pub frames: Vec<MicroProfileFrameState>,
    pub pool: Vec<Option<MicroProfileThreadLog>>,
    pub frame_put_index: u64,
}

#[cfg(feature = "rive_microprofile")]
pub trait MicroProfileRuntime: Send {
    fn set_enable_all_groups(&mut self, enabled: bool);
    fn set_force_enable(&mut self, enabled: bool);
    fn on_thread_create(&mut self, name: &str);
    fn init(&mut self);
    fn flip(&mut self);
    fn ticks_per_second_cpu(&self) -> i64;
    fn state(&self) -> Option<&MicroProfileState>;
    fn tick(&self) -> u64;
    fn tick_difference(&self, frame_start_cpu: i64, entry: MicroProfileLogEntry) -> i64;
    fn max_threads(&self) -> u32;
    fn buffer_size(&self) -> u32;
    fn gpu_frame_delay(&self) -> u32;
    fn max_frame_history(&self) -> u32;
}

pub type FlushCallback = Box<dyn FnMut(&[TransitionRecord]) + Send>;
pub type ListenerPerformChangeFlushCallback = Box<dyn FnMut(&[ListenerPerformChangeRecord]) + Send>;

pub struct RiveProfile {
    flush_callback: Option<FlushCallback>,
    listener_perform_change_flush_callback: Option<ListenerPerformChangeFlushCallback>,
    transition_records: Vec<TransitionRecord>,
    listener_perform_change_records: Vec<ListenerPerformChangeRecord>,
    string_table: HashMap<String, u32>,
    string_list: Vec<String>,
    profiling_active: bool,
    log_flags: u32,
    header_written: bool,
    last_flushed_frame_index: u64,
    #[cfg(feature = "rive_microprofile")]
    micro_profile: Option<Box<dyn MicroProfileRuntime>>,
}

impl Default for RiveProfile {
    fn default() -> Self {
        Self {
            flush_callback: None,
            listener_perform_change_flush_callback: None,
            transition_records: Vec::new(),
            listener_perform_change_records: Vec::new(),
            string_table: HashMap::new(),
            string_list: Vec::new(),
            profiling_active: false,
            log_flags: 3,
            header_written: false,
            last_flushed_frame_index: 0,
            #[cfg(feature = "rive_microprofile")]
            micro_profile: None,
        }
    }
}

fn write_u8(buffer: &mut Vec<u8>, value: u8) {
    buffer.extend_from_slice(&value.to_ne_bytes());
}

fn write_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_ne_bytes());
}

fn write_i64(buffer: &mut Vec<u8>, value: i64) {
    buffer.extend_from_slice(&value.to_ne_bytes());
}

fn write_var_uint(buffer: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buffer.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn write_string(buffer: &mut Vec<u8>, value: &str) {
    write_var_uint(buffer, value.len() as u64);
    buffer.extend_from_slice(value.as_bytes());
}

#[cfg(feature = "rive_microprofile")]
fn write_profile_header(
    buffer: &mut Vec<u8>,
    state: &MicroProfileState,
    ticks_per_second_cpu: i64,
) {
    write_u32(buffer, 0x5250_5246); // "RPRF"
    write_u32(buffer, 2);
    write_i64(buffer, ticks_per_second_cpu);
    write_var_uint(buffer, state.total_timers as u64);
    for timer_index in 0..state.total_timers as usize {
        let info = &state.timer_info[timer_index];
        write_string(buffer, &info.name);
        write_u32(buffer, info.group_index);
        write_u32(buffer, info.color);
    }
    write_var_uint(buffer, state.group_count as u64);
    for group_index in 0..state.group_count as usize {
        write_string(buffer, &state.group_info[group_index].name);
    }
}

#[cfg(feature = "rive_microprofile")]
fn write_frame_events(
    buffer: &mut Vec<u8>,
    runtime: &dyn MicroProfileRuntime,
    state: &MicroProfileState,
    frame_index: u32,
    next_frame_index: u32,
) {
    let frame = &state.frames[frame_index as usize];
    let next_frame = &state.frames[next_frame_index as usize];
    let mut payload = Vec::new();
    write_i64(&mut payload, frame.frame_start_cpu);
    write_i64(&mut payload, next_frame.frame_start_cpu);

    let mut total_events = 0u32;
    for thread_index in 0..runtime.max_threads() {
        let Some(_log) = state
            .pool
            .get(thread_index as usize)
            .and_then(Option::as_ref)
        else {
            continue;
        };
        let log_start = frame.log_start[thread_index as usize];
        let log_end = next_frame.log_start[thread_index as usize];
        if log_end >= log_start {
            total_events += log_end - log_start;
        } else {
            total_events += (runtime.buffer_size() - log_start) + log_end;
        }
    }
    write_var_uint(&mut payload, total_events as u64);

    for thread_index in 0..runtime.max_threads() {
        let Some(log) = state
            .pool
            .get(thread_index as usize)
            .and_then(Option::as_ref)
        else {
            continue;
        };
        let log_start = frame.log_start[thread_index as usize];
        let log_end = next_frame.log_start[thread_index as usize];
        let mut event_index = log_start;
        while event_index != log_end {
            let entry = log.log[event_index as usize];
            let event_type = match entry.log_type {
                MicroProfileLogType::Enter => 0,
                MicroProfileLogType::Leave => 1,
                MicroProfileLogType::Other => 2,
            };
            let tick = runtime.tick_difference(frame.frame_start_cpu, entry);
            write_u8(&mut payload, event_type);
            write_var_uint(&mut payload, entry.timer_index as u64);
            write_i64(&mut payload, tick);
            event_index = (event_index + 1) % runtime.buffer_size();
        }
    }

    write_u8(buffer, 0x01);
    write_var_uint(buffer, payload.len() as u64);
    buffer.extend_from_slice(&payload);
}

impl RiveProfile {
    pub fn instance() -> &'static Mutex<RiveProfile> {
        static INSTANCE: OnceLock<Mutex<RiveProfile>> = OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(RiveProfile::default()))
    }

    #[cfg(feature = "rive_microprofile")]
    pub fn set_micro_profile_runtime(&mut self, runtime: Box<dyn MicroProfileRuntime>) {
        self.micro_profile = Some(runtime);
    }

    pub fn init(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        if let Some(runtime) = self.micro_profile.as_mut() {
            runtime.set_enable_all_groups(true);
            runtime.set_force_enable(true);
            runtime.on_thread_create("MainThread");
            runtime.init();
        }
    }

    pub fn frame(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        {
            // MicroProfile tracks the frame start internally.
        }
    }

    pub fn end_frame(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        if self.profiling_active {
            if let Some(runtime) = self.micro_profile.as_mut() {
                runtime.flip();
            }
        }
    }

    pub fn start(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        {
            self.profiling_active = true;
            self.header_written = false;
            self.last_flushed_frame_index = 0;
            self.string_table.clear();
            self.string_list.clear();
            if let Some(runtime) = self.micro_profile.as_mut() {
                runtime.set_enable_all_groups(true);
                runtime.set_force_enable(true);
                if let Some(state) = runtime.state() {
                    // Skip the stale boundary frame from before this session.
                    self.last_flushed_frame_index = state.frame_put_index + 1;
                }
            }
        }
    }

    pub fn stop(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        {
            self.profiling_active = false;
            if let Some(runtime) = self.micro_profile.as_mut() {
                runtime.set_force_enable(false);
            }
        }
    }

    pub fn is_active(&self) -> bool {
        #[cfg(feature = "rive_microprofile")]
        {
            self.profiling_active
        }
        #[cfg(not(feature = "rive_microprofile"))]
        {
            false
        }
    }

    pub fn flush_frame_data_to(&mut self, buffer: &mut Vec<u8>) {
        #[cfg(feature = "rive_microprofile")]
        {
            let Some(runtime) = self.micro_profile.as_ref() else {
                return;
            };
            let Some(state) = runtime.state() else {
                return;
            };
            if !self.header_written {
                write_profile_header(buffer, state, runtime.ticks_per_second_cpu());
                self.header_written = true;
            }
            let current_frame_index = state.frame_put_index;
            let gpu_delay = runtime.gpu_frame_delay() + 1;
            if current_frame_index <= self.last_flushed_frame_index + gpu_delay as u64 {
                return;
            }
            let end_frame_index = current_frame_index - gpu_delay as u64;
            let max_history = runtime.max_frame_history() as u64 - gpu_delay as u64 - 2;
            if end_frame_index - self.last_flushed_frame_index > max_history {
                self.last_flushed_frame_index = end_frame_index - max_history;
            }

            for frame_index in self.last_flushed_frame_index..end_frame_index {
                let ring_index = (frame_index % runtime.max_frame_history() as u64) as u32;
                let next_ring_index =
                    ((frame_index + 1) % runtime.max_frame_history() as u64) as u32;
                let duration_ticks = state.frames[next_ring_index as usize].frame_start_cpu
                    - state.frames[ring_index as usize].frame_start_cpu;
                if duration_ticks < 0 {
                    continue;
                }
                write_frame_events(buffer, runtime.as_ref(), state, ring_index, next_ring_index);
            }
            self.last_flushed_frame_index = end_frame_index;
        }
        #[cfg(not(feature = "rive_microprofile"))]
        let _ = buffer;
    }

    fn build_artboard_path(
        &mut self,
        artboard: Option<NonNull<dyn ProfileArtboard>>,
    ) -> Vec<ArtboardPathSegment> {
        let Some(mut current) = artboard else {
            return Vec::new();
        };
        let mut path = Vec::new();
        while let Some(host_ptr) = unsafe { current.as_ref() }.host() {
            let host = unsafe { host_ptr.as_ref() };
            let segment = if host.core_type() == NESTED_ARTBOARD_TYPE_KEY {
                ArtboardPathSegment {
                    segment_type: 0,
                    name_id: self.resolve_string_id(host.name()),
                    index: -1,
                }
            } else if host.core_type() == ARTBOARD_COMPONENT_LIST_TYPE_KEY {
                ArtboardPathSegment {
                    segment_type: 1,
                    name_id: self.resolve_string_id(host.name()),
                    index: if unsafe { current.as_ref() }.is_instance() {
                        host.index_of_artboard_instance(current)
                    } else {
                        -1
                    },
                }
            } else {
                break;
            };
            path.push(segment);
            path.push(ArtboardPathSegment {
                segment_type: 0,
                name_id: self.resolve_string_id(unsafe { current.as_ref() }.name()),
                index: -1,
            });
            let Some(parent) = host.parent_artboard() else {
                break;
            };
            current = parent;
        }
        path.reverse();
        path
    }

    pub fn record_transition(
        &mut self,
        artboard_name: &str,
        state_machine_name: &str,
        layer_name: &str,
        from_state_name: &str,
        to_state_name: &str,
        artboard_for_path: Option<NonNull<dyn ProfileArtboard>>,
    ) {
        #[cfg(feature = "rive_microprofile")]
        {
            if self.log_flags & PROFILE_LOG_TRANSITION_RECORDS == 0 {
                return;
            }
            if !self.is_active() || self.flush_callback.is_none() {
                return;
            }
            let artboard_id = self.resolve_string_id(artboard_name);
            let sm_id = self.resolve_string_id(state_machine_name);
            let layer_id = self.resolve_string_id(layer_name);
            let from_state_id = self.resolve_string_id(from_state_name);
            let to_state_id = self.resolve_string_id(to_state_name);
            let tick = self
                .micro_profile
                .as_ref()
                .map_or(0, |runtime| runtime.tick());
            let path = if artboard_for_path.is_some() {
                self.build_artboard_path(artboard_for_path)
            } else {
                Vec::new()
            };
            self.transition_records.push(TransitionRecord {
                artboard_id,
                sm_id,
                layer_id,
                from_state_id,
                to_state_id,
                tick,
                path,
            });
        }
        #[cfg(not(feature = "rive_microprofile"))]
        let _ = (
            artboard_name,
            state_machine_name,
            layer_name,
            from_state_name,
            to_state_name,
            artboard_for_path,
        );
    }

    pub fn set_flush_callback(&mut self, callback: Option<FlushCallback>) {
        self.flush_callback = callback;
    }

    pub fn flush_transition_records(&mut self) {
        if let Some(callback) = self.flush_callback.as_mut() {
            if !self.transition_records.is_empty() {
                callback(&self.transition_records);
            }
        }
        self.transition_records.clear();
    }

    pub fn record_listener_perform_change(
        &mut self,
        artboard_name: &str,
        state_machine_name: &str,
        listener_name: &str,
        listener_type: u32,
        hit_event: u32,
        pointer_id: u32,
    ) {
        #[cfg(feature = "rive_microprofile")]
        {
            if self.log_flags & PROFILE_LOG_LISTENER_PERFORM_CHANGES == 0 {
                return;
            }
            if !self.is_active() || self.listener_perform_change_flush_callback.is_none() {
                return;
            }
            let artboard_id = self.resolve_string_id(artboard_name);
            let sm_id = self.resolve_string_id(state_machine_name);
            let listener_name_id = self.resolve_string_id(listener_name);
            let tick = self
                .micro_profile
                .as_ref()
                .map_or(0, |runtime| runtime.tick());
            self.listener_perform_change_records
                .push(ListenerPerformChangeRecord {
                    artboard_id,
                    sm_id,
                    listener_name_id,
                    listener_type,
                    hit_event,
                    pointer_id,
                    tick,
                });
        }
        #[cfg(not(feature = "rive_microprofile"))]
        let _ = (
            artboard_name,
            state_machine_name,
            listener_name,
            listener_type,
            hit_event,
            pointer_id,
        );
    }

    pub fn set_listener_perform_change_flush_callback(
        &mut self,
        callback: Option<ListenerPerformChangeFlushCallback>,
    ) {
        self.listener_perform_change_flush_callback = callback;
    }

    pub fn flush_listener_perform_change_records(&mut self) {
        if let Some(callback) = self.listener_perform_change_flush_callback.as_mut() {
            if !self.listener_perform_change_records.is_empty() {
                callback(&self.listener_perform_change_records);
            }
        }
        self.listener_perform_change_records.clear();
    }

    pub fn set_log_flags(&mut self, flags: u32) {
        self.log_flags = flags;
    }

    pub fn log_flags(&self) -> u32 {
        self.log_flags
    }

    pub fn resolve_string_id(&mut self, value: &str) -> u32 {
        #[cfg(feature = "rive_microprofile")]
        {
            if let Some(id) = self.string_table.get(value) {
                return *id;
            }
            let id = self.string_list.len() as u32;
            self.string_table.insert(value.to_owned(), id);
            self.string_list.push(value.to_owned());
            id
        }
        #[cfg(not(feature = "rive_microprofile"))]
        {
            let _ = value;
            0
        }
    }

    pub fn get_string_table(&self) -> &[String] {
        &self.string_list
    }
}
