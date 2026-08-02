use nuxie_runtime::{
    ListenerPerformChangeRecord, ProfileCapture, ProfileCaptureEvent, ProfileCaptureFrame,
    ProfileCaptureMetadata, ProfileCaptureTimer, ProfilePathSegment, RiveProfile, TransitionRecord,
};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppProfilerOracle {
    bytes: Vec<u8>,
    transition: CppTransitionRecord,
    listener: CppListenerRecord,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppTransitionRecord {
    artboard_id: u32,
    sm_id: u32,
    layer_id: u32,
    from_state_id: u32,
    to_state_id: u32,
    tick: u64,
    path: Vec<CppPathSegment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppPathSegment {
    #[serde(rename = "type")]
    segment_type: u8,
    name_id: u32,
    index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppListenerRecord {
    artboard_id: u32,
    sm_id: u32,
    listener_name_id: u32,
    listener_type: u32,
    hit_event: u32,
    pointer_id: u32,
    tick: u64,
}

struct OracleCapture {
    current_frame: u64,
    ticks: Vec<u64>,
    next_tick: usize,
}

impl ProfileCapture for OracleCapture {
    fn end_frame(&mut self) {
        self.current_frame += 1;
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
        (frame_index == 3).then(|| ProfileCaptureFrame {
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
        })
    }
}

#[test]
fn profiler_wire_bytes_and_record_layout_match_pinned_cpp_source_oracle() {
    let Some(probe) = std::env::var_os("RIVE_CPP_PROBE").map(PathBuf::from) else {
        eprintln!("skipping F12 profiler C++ source oracle; set RIVE_CPP_PROBE");
        return;
    };
    let output = Command::new(&probe)
        .arg("--f12-profiler-oracle")
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", probe.display()));
    assert!(
        output.status.success(),
        "F12 C++ profiler oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cpp: CppProfilerOracle = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("invalid F12 C++ profiler oracle JSON: {error}"));

    let transitions = Arc::new(Mutex::new(Vec::<TransitionRecord>::new()));
    let listeners = Arc::new(Mutex::new(Vec::<ListenerPerformChangeRecord>::new()));
    let mut profile = RiveProfile::new(Box::new(OracleCapture {
        current_frame: 3,
        ticks: vec![41, 42],
        next_tick: 0,
    }));
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
    profile.end_frame();
    profile.end_frame();
    let mut rust_bytes = Vec::new();
    profile.flush_frame_data_to(&mut rust_bytes);
    assert_eq!(rust_bytes, cpp.bytes);

    let transitions = transitions.lock().unwrap();
    let rust_transition = &transitions[0];
    assert_eq!(cpp.transition.artboard_id, rust_transition.artboard_id);
    assert_eq!(cpp.transition.sm_id, rust_transition.sm_id);
    assert_eq!(cpp.transition.layer_id, rust_transition.layer_id);
    assert_eq!(cpp.transition.from_state_id, rust_transition.from_state_id);
    assert_eq!(cpp.transition.to_state_id, rust_transition.to_state_id);
    assert_eq!(cpp.transition.tick, rust_transition.tick);
    assert_eq!(cpp.transition.path.len(), rust_transition.path.len());
    for (cpp_segment, rust_segment) in cpp.transition.path.iter().zip(&rust_transition.path) {
        assert_eq!(cpp_segment.segment_type, rust_segment.segment_type);
        assert_eq!(cpp_segment.name_id, rust_segment.name_id);
        assert_eq!(cpp_segment.index, rust_segment.index);
    }

    let listeners = listeners.lock().unwrap();
    let rust_listener = &listeners[0];
    assert_eq!(cpp.listener.artboard_id, rust_listener.artboard_id);
    assert_eq!(cpp.listener.sm_id, rust_listener.sm_id);
    assert_eq!(
        cpp.listener.listener_name_id,
        rust_listener.listener_name_id
    );
    assert_eq!(cpp.listener.listener_type, rust_listener.listener_type);
    assert_eq!(cpp.listener.hit_event, rust_listener.hit_event);
    assert_eq!(cpp.listener.pointer_id, rust_listener.pointer_id);
    assert_eq!(cpp.listener.tick, rust_listener.tick);
}
