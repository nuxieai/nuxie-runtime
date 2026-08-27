use std::collections::HashMap;
pub const PROFILE_LOG_NONE: u32 = 0;
pub const PROFILE_LOG_TRANSITION_RECORDS: u32 = 1;
pub const PROFILE_LOG_LISTENER_PERFORM_CHANGES: u32 = 2;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArtboardPathSegment {
    pub segment_type: u8,
    pub name_id: u32,
    pub index: i32,
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
pub type FlushCallback = Box<dyn FnMut(&[TransitionRecord])>;
pub type ListenerFlushCallback = Box<dyn FnMut(&[ListenerPerformChangeRecord])>;
pub struct RiveProfile {
    flush_callback: Option<FlushCallback>,
    listener_callback: Option<ListenerFlushCallback>,
    transition_records: Vec<TransitionRecord>,
    listener_records: Vec<ListenerPerformChangeRecord>,
    string_table: HashMap<String, u32>,
    string_list: Vec<String>,
    profiling_active: bool,
    log_flags: u32,
    header_written: bool,
    last_flushed_frame_index: u64,
    tick: u64,
}
impl Default for RiveProfile {
    fn default() -> Self {
        Self {
            flush_callback: None,
            listener_callback: None,
            transition_records: Vec::new(),
            listener_records: Vec::new(),
            string_table: HashMap::new(),
            string_list: Vec::new(),
            profiling_active: false,
            log_flags: 3,
            header_written: false,
            last_flushed_frame_index: 0,
            tick: 0,
        }
    }
}
impl RiveProfile {
    pub fn init(&mut self) {}
    pub fn frame(&mut self) {}
    pub fn end_frame(&mut self) {
        if self.profiling_active {
            self.tick += 1
        }
    }
    pub fn start(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        {
            self.profiling_active = true;
            self.header_written = false;
            self.last_flushed_frame_index = 1;
            self.string_table.clear();
            self.string_list.clear();
        }
    }
    pub fn stop(&mut self) {
        #[cfg(feature = "rive_microprofile")]
        {
            self.profiling_active = false
        }
    }
    pub fn is_active(&self) -> bool {
        cfg!(feature = "rive_microprofile") && self.profiling_active
    }
    pub fn set_log_flags(&mut self, v: u32) {
        self.log_flags = v
    }
    pub fn log_flags(&self) -> u32 {
        self.log_flags
    }
    pub fn resolve_string_id(&mut self, s: &str) -> u32 {
        #[cfg(feature = "rive_microprofile")]
        {
            if let Some(v) = self.string_table.get(s) {
                return *v;
            }
            let id = self.string_list.len() as u32;
            self.string_table.insert(s.into(), id);
            self.string_list.push(s.into());
            id
        }
        #[cfg(not(feature = "rive_microprofile"))]
        {
            let _ = s;
            0
        }
    }
    pub fn get_string_table(&self) -> &[String] {
        &self.string_list
    }
    pub fn set_flush_callback(&mut self, c: Option<FlushCallback>) {
        self.flush_callback = c
    }
    pub fn set_listener_perform_change_flush_callback(&mut self, c: Option<ListenerFlushCallback>) {
        self.listener_callback = c
    }
    pub fn record_transition(
        &mut self,
        artboard: &str,
        sm: &str,
        layer: &str,
        from: &str,
        to: &str,
        path: Vec<ArtboardPathSegment>,
    ) {
        if !self.is_active()
            || self.flush_callback.is_none()
            || self.log_flags & PROFILE_LOG_TRANSITION_RECORDS == 0
        {
            return;
        }
        let rec = TransitionRecord {
            artboard_id: self.resolve_string_id(artboard),
            sm_id: self.resolve_string_id(sm),
            layer_id: self.resolve_string_id(layer),
            from_state_id: self.resolve_string_id(from),
            to_state_id: self.resolve_string_id(to),
            tick: self.tick,
            path,
        };
        self.transition_records.push(rec)
    }
    pub fn flush_transition_records(&mut self) {
        if let Some(c) = &mut self.flush_callback {
            if !self.transition_records.is_empty() {
                c(&self.transition_records)
            }
        }
        self.transition_records.clear()
    }
    pub fn record_listener_perform_change(
        &mut self,
        a: &str,
        sm: &str,
        name: &str,
        listener_type: u32,
        hit_event: u32,
        pointer_id: u32,
    ) {
        if !self.is_active()
            || self.listener_callback.is_none()
            || self.log_flags & PROFILE_LOG_LISTENER_PERFORM_CHANGES == 0
        {
            return;
        }
        let rec = ListenerPerformChangeRecord {
            artboard_id: self.resolve_string_id(a),
            sm_id: self.resolve_string_id(sm),
            listener_name_id: self.resolve_string_id(name),
            listener_type,
            hit_event,
            pointer_id,
            tick: self.tick,
        };
        self.listener_records.push(rec)
    }
    pub fn flush_listener_perform_change_records(&mut self) {
        if let Some(c) = &mut self.listener_callback {
            if !self.listener_records.is_empty() {
                c(&self.listener_records)
            }
        }
        self.listener_records.clear()
    }
    pub fn flush_frame_data_to(&mut self, buffer: &mut Vec<u8>) {
        #[cfg(feature = "rive_microprofile")]
        {
            if !self.header_written {
                buffer.extend_from_slice(&0x5250_5246u32.to_ne_bytes());
                buffer.extend_from_slice(&2u32.to_ne_bytes());
                self.header_written = true;
            }
            self.last_flushed_frame_index = self.tick;
        }
    }
}
