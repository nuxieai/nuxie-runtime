use nuxie_runtime::{
    PROFILE_LOG_LISTENER_PERFORM_CHANGES, PROFILE_LOG_TRANSITION_RECORDS, ProfileCapture,
    ProfileCaptureEvent, ProfileCaptureFrame, ProfileCaptureMetadata, ProfileCaptureTimer,
    ProfilePathSegment, RiveProfile,
};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct TestCapture {
    calls: Arc<Mutex<Vec<&'static str>>>,
    ticks: Vec<u64>,
    next_tick: usize,
    current_frame: u64,
    frames: BTreeMap<u64, ProfileCaptureFrame>,
}

impl ProfileCapture for TestCapture {
    fn init(&mut self) {
        self.calls.lock().unwrap().push("init");
    }

    fn frame(&mut self) {
        self.calls.lock().unwrap().push("frame");
    }

    fn end_frame(&mut self) {
        self.calls.lock().unwrap().push("end_frame");
        self.current_frame += 1;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.calls
            .lock()
            .unwrap()
            .push(if enabled { "enable" } else { "disable" });
    }

    fn tick(&mut self) -> u64 {
        let tick = self.ticks.get(self.next_tick).copied().unwrap_or_default();
        self.next_tick += 1;
        tick
    }

    fn metadata(&self) -> ProfileCaptureMetadata {
        ProfileCaptureMetadata {
            ticks_per_second_cpu: 1_000_000,
            timers: vec![ProfileCaptureTimer {
                name: "advance".into(),
                group_index: 2,
                color: 0x11_22_33,
            }],
            groups: vec!["runtime".into()],
        }
    }

    fn current_frame_index(&self) -> u64 {
        self.current_frame
    }

    fn gpu_frame_delay(&self) -> u64 {
        1
    }

    fn max_frame_history(&self) -> u64 {
        8
    }

    fn captured_frame(&self, frame_index: u64) -> Option<ProfileCaptureFrame> {
        self.frames.get(&frame_index).cloned()
    }
}

#[test]
fn records_use_stable_string_ids_and_flush_in_capture_order() {
    let capture = TestCapture {
        ticks: vec![41, 42],
        ..TestCapture::default()
    };
    let transitions = Arc::new(Mutex::new(Vec::new()));
    let listeners = Arc::new(Mutex::new(Vec::new()));
    let mut profile = RiveProfile::new(Box::new(capture));
    profile.set_log_flags(PROFILE_LOG_TRANSITION_RECORDS | PROFILE_LOG_LISTENER_PERFORM_CHANGES);
    profile.set_transition_flush_callback(Some(Box::new({
        let transitions = Arc::clone(&transitions);
        move |records| transitions.lock().unwrap().extend_from_slice(records)
    })));
    profile.set_listener_perform_change_flush_callback(Some(Box::new({
        let listeners = Arc::clone(&listeners);
        move |records| listeners.lock().unwrap().extend_from_slice(records)
    })));
    profile.start();

    profile.record_transition(
        "Board",
        "Machine",
        "Layer",
        "Idle",
        "Run",
        &[
            ProfilePathSegment::nested_artboard("Child"),
            ProfilePathSegment::component_list("Rows", 7),
        ],
    );
    profile.record_listener_perform_change("Board", "Machine", "Tap", 6, 2, 99);
    profile.flush_transition_records();
    profile.flush_listener_perform_change_records();

    assert_eq!(
        profile.string_table(),
        [
            "Board", "Machine", "Layer", "Idle", "Run", "Child", "Rows", "Tap"
        ]
    );
    let transitions = transitions.lock().unwrap();
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].artboard_id, 0);
    assert_eq!(transitions[0].sm_id, 1);
    assert_eq!(transitions[0].layer_id, 2);
    assert_eq!(transitions[0].from_state_id, 3);
    assert_eq!(transitions[0].to_state_id, 4);
    assert_eq!(transitions[0].tick, 41);
    assert_eq!(transitions[0].path[0].segment_type, 0);
    assert_eq!(transitions[0].path[0].name_id, 5);
    assert_eq!(transitions[0].path[0].index, -1);
    assert_eq!(transitions[0].path[1].segment_type, 1);
    assert_eq!(transitions[0].path[1].name_id, 6);
    assert_eq!(transitions[0].path[1].index, 7);
    let listeners = listeners.lock().unwrap();
    assert_eq!(listeners.len(), 1);
    assert_eq!(listeners[0].listener_name_id, 7);
    assert_eq!(listeners[0].listener_type, 6);
    assert_eq!(listeners[0].hit_event, 2);
    assert_eq!(listeners[0].pointer_id, 99);
    assert_eq!(listeners[0].tick, 42);
}

#[test]
fn records_are_gated_by_session_flags_and_callback_presence() {
    let transitions = Arc::new(Mutex::new(Vec::new()));
    let listeners = Arc::new(Mutex::new(Vec::new()));
    let mut profile = RiveProfile::new(Box::new(TestCapture {
        ticks: vec![11, 12],
        ..TestCapture::default()
    }));
    profile.set_transition_flush_callback(Some(Box::new({
        let transitions = Arc::clone(&transitions);
        move |records| transitions.lock().unwrap().extend_from_slice(records)
    })));
    profile.set_listener_perform_change_flush_callback(Some(Box::new({
        let listeners = Arc::clone(&listeners);
        move |records| listeners.lock().unwrap().extend_from_slice(records)
    })));

    profile.record_transition("inactive", "sm", "layer", "from", "to", &[]);
    profile.record_listener_perform_change("inactive", "sm", "listener", 6, 3, 1);
    assert!(profile.string_table().is_empty());

    profile.start();
    profile.set_log_flags(0);
    profile.record_transition("disabled", "sm", "layer", "from", "to", &[]);
    profile.record_listener_perform_change("disabled", "sm", "listener", 6, 3, 1);
    assert!(profile.string_table().is_empty());

    profile.set_log_flags(PROFILE_LOG_TRANSITION_RECORDS);
    profile.record_transition("transition", "sm", "layer", "from", "to", &[]);
    profile.record_listener_perform_change("listener disabled", "sm", "listener", 6, 3, 1);
    profile.flush_transition_records();
    profile.flush_listener_perform_change_records();
    assert_eq!(transitions.lock().unwrap().len(), 1);
    assert!(listeners.lock().unwrap().is_empty());

    profile.set_transition_flush_callback(None);
    profile.record_transition("no callback", "sm", "layer", "from", "to", &[]);
    assert!(
        !profile
            .string_table()
            .iter()
            .any(|value| value == "no callback")
    );

    profile.set_log_flags(PROFILE_LOG_LISTENER_PERFORM_CHANGES);
    profile.record_listener_perform_change("listener", "sm", "tap", 6, 3, 1);
    profile.flush_listener_perform_change_records();
    assert_eq!(listeners.lock().unwrap().len(), 1);

    profile.stop();
    profile.record_listener_perform_change("stopped", "sm", "tap", 6, 3, 1);
    assert!(
        !profile
            .string_table()
            .iter()
            .any(|value| value == "stopped")
    );
}

#[test]
fn capture_lifecycle_resets_session_state_and_only_flips_while_active() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let capture = TestCapture {
        calls: Arc::clone(&calls),
        current_frame: 17,
        ..TestCapture::default()
    };
    let mut profile = RiveProfile::new(Box::new(capture));

    profile.init();
    profile.frame();
    profile.end_frame();
    profile.start();
    assert!(profile.is_active());
    assert_eq!(profile.resolve_string_id("first session"), 0);
    profile.end_frame();
    profile.stop();
    assert!(!profile.is_active());
    profile.end_frame();
    profile.start();
    assert!(profile.string_table().is_empty());
    assert_eq!(profile.resolve_string_id("second session"), 0);
    profile.stop();

    assert_eq!(
        *calls.lock().unwrap(),
        [
            "init",
            "frame",
            "enable",
            "end_frame",
            "disable",
            "enable",
            "disable"
        ]
    );
}

#[test]
fn profile_header_and_frame_payload_match_the_pinned_cpp_byte_format() {
    let mut frames = BTreeMap::new();
    frames.insert(
        3,
        ProfileCaptureFrame {
            frame_start_cpu: 100,
            next_frame_start_cpu: 140,
            events: vec![
                ProfileCaptureEvent {
                    event_type: 0,
                    timer_index: 0,
                    tick: 5,
                },
                ProfileCaptureEvent {
                    event_type: 1,
                    timer_index: 0,
                    tick: 35,
                },
            ],
        },
    );
    let capture = TestCapture {
        current_frame: 3,
        frames,
        ..TestCapture::default()
    };
    let mut profile = RiveProfile::new(Box::new(capture));
    profile.start();
    profile.end_frame();
    profile.end_frame();
    let mut bytes = Vec::new();
    profile.flush_frame_data_to(&mut bytes);

    assert_eq!(
        bytes,
        [
            70, 82, 80, 82, 2, 0, 0, 0, 64, 66, 15, 0, 0, 0, 0, 0, 1, 7, 97, 100, 118, 97, 110, 99,
            101, 2, 0, 0, 0, 51, 34, 17, 0, 1, 7, 114, 117, 110, 116, 105, 109, 101, 1, 37, 100, 0,
            0, 0, 0, 0, 0, 0, 140, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0, 35,
            0, 0, 0, 0, 0, 0, 0,
        ]
    );
}
