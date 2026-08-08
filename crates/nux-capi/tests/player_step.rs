#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "C-ABI conformance tests use bounded fixture counters and explicit teardown"
)]

use nux_capi::*;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let fixture = PathBuf::from(
        std::env::var_os("NUX_RUNTIME_DIR")
            .or_else(|| std::env::var_os("RIVE_RUNTIME_DIR"))
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    std::fs::read(fixture).expect("read fixture")
}

fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn owned(value: NuxStringView) -> String {
    if value.len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.data.cast::<u8>(), value.len) };
    std::str::from_utf8(bytes).unwrap().to_owned()
}

fn import(name: &str) -> *mut NuxFile {
    let bytes = fixture_bytes(name);
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
        NuxStatus::Ok
    );
    file
}

fn artboard(file: *mut NuxFile, index: usize) -> *mut NuxArtboardInstance {
    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, index, &mut instance) },
        NuxStatus::Ok
    );
    instance
}

fn state_player(instance: *mut NuxArtboardInstance, name: &str) -> *mut NuxPlayer {
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_state_machine_named(instance, view(name), &mut player) },
        NuxStatus::Ok
    );
    player
}

fn step(
    player: *mut NuxPlayer,
    inputs: &[NuxPlayerInputChange],
    pointers: &[NuxPlayerPointerEvent],
    elapsed_seconds: f32,
) -> (NuxStatus, *mut NuxPlayerStepResult) {
    let operation = NuxPlayerStep {
        struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
        correlation_id: 0,
        inputs: inputs.as_ptr(),
        input_count: inputs.len(),
        pointers: pointers.as_ptr(),
        pointer_count: pointers.len(),
        elapsed_seconds,
    };
    let mut result = std::ptr::null_mut();
    let status = unsafe { nux_player_step(player, &operation, &mut result) };
    (status, result)
}

fn info(result: *mut NuxPlayerStepResult) -> NuxPlayerStepInfo {
    let mut info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    info
}

fn animation_name(file: *const NuxFile, artboard_index: usize, index: usize) -> String {
    let mut name = NuxStringView::default();
    assert_eq!(
        unsafe { nux_file_artboard_animation_name(file, artboard_index, index, &mut name) },
        NuxStatus::Ok
    );
    owned(name)
}

#[test]
fn invalid_input_batch_is_rejected_before_any_value_is_committed() {
    let file = import("smi_test.riv");
    let instance = artboard(file, 1);
    let player = state_player(instance, "State Machine 1");

    let rejected = [
        NuxPlayerInputChange {
            kind: NuxPlayerInputKind::Bool as u32,
            name: view("bool"),
            bool_value: 1,
            number_value: 0.0,
        },
        NuxPlayerInputChange {
            kind: NuxPlayerInputKind::Number as u32,
            name: view("missing"),
            bool_value: 0,
            number_value: 42.0,
        },
    ];
    let (status, result) = step(player, &rejected, &[], 0.0);
    assert_eq!(status, NuxStatus::NotFound);
    assert!(!result.is_null());
    let mut result_status = NuxStatus::Ok;
    assert_eq!(
        unsafe { nux_player_step_result_status(result, &mut result_status) },
        NuxStatus::Ok
    );
    assert_eq!(result_status, NuxStatus::NotFound);
    unsafe { nux_player_step_result_free(result) };

    // Compare the next operation with an untouched player over a second
    // occurrence. If the first bool leaked from the rejected batch, resetting
    // it would produce a different advance/state-change result.
    let control_instance = artboard(file, 1);
    let control_player = state_player(control_instance, "State Machine 1");
    let reset = [NuxPlayerInputChange {
        kind: NuxPlayerInputKind::Bool as u32,
        name: view("bool"),
        bool_value: 0,
        number_value: 0.0,
    }];
    let (status, result) = step(player, &reset, &[], 0.0);
    assert_eq!(status, NuxStatus::Ok);
    let mut info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    assert_eq!(info.state_change_count, 1);
    for index in 0..info.state_change_count {
        let mut change = NuxPlayerStateChangeView::default();
        assert_eq!(
            unsafe { nux_player_step_result_state_change(result, index, &mut change) },
            NuxStatus::Ok
        );
        assert_eq!(change.layer_index, index);
        assert_eq!(
            change.state_core_type,
            u32::from(
                nuxie_schema::definition_by_name("AnimationState")
                    .unwrap()
                    .type_key
                    .int
            )
        );
    }
    let (control_status, control_result) = step(control_player, &reset, &[], 0.0);
    assert_eq!(control_status, NuxStatus::Ok);
    let mut control_info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(control_result, &mut control_info) },
        NuxStatus::Ok
    );
    assert_eq!(info.keep_going, control_info.keep_going);
    assert_eq!(info.state_change_count, control_info.state_change_count);
    assert_eq!(info.event_count, control_info.event_count);

    unsafe {
        nux_player_step_result_free(result);
        nux_player_step_result_free(control_result);
        nux_player_free(control_player);
        nux_artboard_instance_free(control_instance);
        nux_player_free(player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn pointer_step_returns_cpp_ordered_reported_events_through_owned_views() {
    let file = import("click_event.riv");
    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new_named(file, view("art-1"), &mut instance) },
        NuxStatus::Ok
    );
    let player = state_player(instance, "sm-1");
    let (status, initialization) = step(player, &[], &[], 0.0);
    assert_eq!(status, NuxStatus::Ok);
    unsafe { nux_player_step_result_free(initialization) };
    let pointers = [
        NuxPlayerPointerEvent {
            kind: NuxPlayerPointerKind::Down as u32,
            x: 75.0,
            y: 75.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
        NuxPlayerPointerEvent {
            kind: NuxPlayerPointerKind::Up as u32,
            x: 75.0,
            y: 75.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
    ];
    let (status, result) = step(player, &[], &pointers, 0.0);
    assert_eq!(status, NuxStatus::Ok);
    let mut info = NuxPlayerStepInfo::default();
    assert_eq!(
        unsafe { nux_player_step_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    assert!(info.keep_going);
    assert_eq!(info.pointer_result_count, 2);
    assert_eq!(info.event_count, 1);
    let mut first_hit = NUX_PLAYER_POINTER_HIT_NONE;
    let mut second_hit = NUX_PLAYER_POINTER_HIT_NONE;
    assert_eq!(
        unsafe { nux_player_step_result_pointer(result, 0, &mut first_hit) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_step_result_pointer(result, 1, &mut second_hit) },
        NuxStatus::Ok
    );
    assert_ne!(first_hit, NUX_PLAYER_POINTER_HIT_NONE);
    assert_ne!(second_hit, NUX_PLAYER_POINTER_HIT_NONE);

    let mut event = NuxPlayerEventView::default();
    assert_eq!(
        unsafe { nux_player_step_result_event(result, 0, &mut event) },
        NuxStatus::Ok
    );
    assert!(!owned(event.name).is_empty());
    assert_eq!(event.property_count, 0);
    assert_eq!(
        unsafe { nux_player_step_result_event(result, 1, &mut event) },
        NuxStatus::NotFound
    );

    // Pinned C++ `hittest_test.cpp:284-310` reports exactly one cumulative
    // click event for this down/up sequence. The facade-level Rust oracle has
    // the same fixture assertion in nuxie-runtime's cpp_probe suite.
    unsafe {
        nux_player_step_result_free(result);
        nux_player_free(player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn step_validates_bounds_and_player_kind_without_mutating() {
    let file = import("circle_clips.riv");
    let instance = artboard(file, 0);
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_default(instance, &mut player) },
        NuxStatus::Ok
    );
    let input = [NuxPlayerInputChange {
        kind: NuxPlayerInputKind::Trigger as u32,
        name: view("anything"),
        bool_value: 0,
        number_value: 0.0,
    }];
    let (status, result) = step(player, &input, &[], 0.0);
    assert_eq!(status, NuxStatus::NotFound);
    assert!(!result.is_null());
    unsafe { nux_player_step_result_free(result) };

    let (status, result) = step(player, &[], &[], f32::NAN);
    assert_eq!(status, NuxStatus::InvalidArgument);
    assert!(!result.is_null());
    unsafe {
        nux_player_step_result_free(result);
        nux_player_free(player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn step_rejects_invalid_wire_values_and_bounds_before_dispatch() {
    let file = import("smi_test.riv");
    let instance = artboard(file, 1);
    let player = state_player(instance, "State Machine 1");

    for input in [
        NuxPlayerInputChange {
            kind: u32::MAX,
            name: view("bool"),
            bool_value: 0,
            number_value: 0.0,
        },
        NuxPlayerInputChange {
            kind: NUX_PLAYER_INPUT_KIND_BOOL,
            name: view("bool"),
            bool_value: 2,
            number_value: 0.0,
        },
    ] {
        let (status, result) = step(player, &[input], &[], 0.0);
        assert_eq!(status, NuxStatus::InvalidArgument);
        assert_eq!(
            unsafe { nux_player_step_result_free(result) },
            NuxStatus::Ok
        );
    }

    let oversized_name = vec![b'x'; NUX_PLAYER_STEP_MAX_INPUT_NAME_BYTES + 1];
    let input = NuxPlayerInputChange {
        kind: NUX_PLAYER_INPUT_KIND_BOOL,
        name: NuxStringView {
            data: oversized_name.as_ptr().cast(),
            len: oversized_name.len(),
        },
        bool_value: 1,
        number_value: 0.0,
    };
    let (status, result) = step(player, &[input], &[], 0.0);
    assert_eq!(status, NuxStatus::LimitExceeded);
    assert_eq!(
        unsafe { nux_player_step_result_free(result) },
        NuxStatus::Ok
    );

    for pointer in [
        NuxPlayerPointerEvent {
            kind: u32::MAX,
            x: 0.0,
            y: 0.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
        NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 0.0,
            y: 0.0,
            pointer_id: 0,
            timestamp_seconds: 1.0,
        },
    ] {
        let (status, result) = step(player, &[], &[pointer], 0.0);
        assert_eq!(status, NuxStatus::InvalidArgument);
        assert_eq!(
            unsafe { nux_player_step_result_free(result) },
            NuxStatus::Ok
        );
    }

    let oversized = NuxPlayerStep {
        pointer_count: NUX_PLAYER_STEP_MAX_POINTERS + 1,
        ..NuxPlayerStep::default()
    };
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_step(player, &oversized, &mut result) },
        NuxStatus::LimitExceeded
    );
    assert_eq!(
        unsafe { nux_player_step_result_free(result) },
        NuxStatus::Ok
    );

    let undersized = NuxPlayerStep {
        struct_size: (NUX_PLAYER_STEP_V3_MIN_SIZE - 1) as u32,
        ..NuxPlayerStep::default()
    };
    assert_eq!(
        unsafe { nux_player_step(player, &undersized, &mut result) },
        NuxStatus::InvalidStructSize
    );
    assert_eq!(
        unsafe { nux_player_step_result_free(result) },
        NuxStatus::Ok
    );

    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn linear_static_and_nested_steps_match_runtime_oracles() {
    let bytes = fixture_bytes("smi_test.riv");
    let file = import("smi_test.riv");

    let linear_instance = artboard(file, 1);
    let timeline = animation_name(file, 1, 0);
    let mut linear_player = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_new_linear_animation_named(
                linear_instance,
                view(&timeline),
                &mut linear_player,
            )
        },
        NuxStatus::Ok
    );
    let (status, linear_result) = step(linear_player, &[], &[], 0.25);
    assert_eq!(status, NuxStatus::Ok);

    let rust_file = Arc::new(nuxie::File::import(&bytes).unwrap());
    let mut rust_linear =
        nuxie::OwnedArtboardInstance::instantiate(Arc::clone(&rust_file), 1).unwrap();
    let mut rust_animation = rust_linear
        .linear_animation_instance_named(&timeline)
        .unwrap();
    let mut rust_events = Vec::new();
    let more = rust_linear
        .raw_mut()
        .advance_linear_animation_instance_with_events(&mut rust_animation, 0.25, &mut rust_events);
    let _ = rust_linear
        .raw_mut()
        .apply_linear_animation_instance(&rust_animation, 1.0);
    let artboard_more = rust_linear.advance(0.25);
    let rust_keep_going = more
        || artboard_more
        || rust_linear
            .raw()
            .linear_animation_instance_keep_going(&rust_animation);
    assert_eq!(info(linear_result).keep_going, rust_keep_going);

    let static_instance = artboard(file, 1);
    let mut static_player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(static_instance, &mut static_player) },
        NuxStatus::Ok
    );
    let static_pointer = [NuxPlayerPointerEvent {
        kind: NUX_PLAYER_POINTER_KIND_DOWN,
        x: 5.0,
        y: 6.0,
        pointer_id: 7,
        timestamp_seconds: 0.0,
    }];
    let (status, static_result) = step(static_player, &[], &static_pointer, 123.0);
    assert_eq!(status, NuxStatus::Ok);
    let static_info = info(static_result);
    assert!(
        static_info.keep_going,
        "C++ StaticScene always returns true"
    );
    assert_eq!(static_info.pointer_result_count, 1);
    let mut static_hit = u32::MAX;
    assert_eq!(
        unsafe { nux_player_step_result_pointer(static_result, 0, &mut static_hit) },
        NuxStatus::Ok
    );
    assert_eq!(static_hit, NUX_PLAYER_POINTER_HIT_NONE);

    let nested_instance = artboard(file, 0);
    let mut nested_player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_default(nested_instance, &mut nested_player) },
        NuxStatus::Ok
    );
    let (status, nested_result) = step(nested_player, &[], &[], 0.016);
    assert_eq!(status, NuxStatus::Ok);
    let mut rust_nested = nuxie::OwnedArtboardInstance::instantiate(rust_file, 0).unwrap();
    let mut rust_machine = rust_nested.default_state_machine_instance().unwrap();
    let expected_nested = rust_nested
        .try_advance_with_state_machine(&mut rust_machine, 0.016)
        .unwrap();
    assert_eq!(info(nested_result).keep_going, expected_nested);

    unsafe {
        nux_player_step_result_free(nested_result);
        nux_player_free(nested_player);
        nux_artboard_instance_free(nested_instance);
        nux_player_step_result_free(static_result);
        nux_player_free(static_player);
        nux_artboard_instance_free(static_instance);
        nux_player_step_result_free(linear_result);
        nux_player_free(linear_player);
        nux_artboard_instance_free(linear_instance);
        nux_file_free(file);
    }
}

#[test]
fn live_pinned_cpp_player_step_oracle_matches_c_and_rust() {
    const PINNED_RIVE_RUNTIME_REVISION: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
    let Some(probe) = std::env::var_os("RIVE_CPP_PROBE").map(PathBuf::from) else {
        eprintln!("skipping live C++ player-step oracle; run `make capi-player-step-oracle`");
        return;
    };
    let runtime_dir = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    );
    let revision = Command::new("git")
        .args(["-C", runtime_dir.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .expect("read pinned rive-runtime revision");
    assert!(
        revision.status.success(),
        "read pinned rive-runtime revision"
    );
    assert_eq!(
        String::from_utf8(revision.stdout).unwrap().trim(),
        PINNED_RIVE_RUNTIME_REVISION,
        "the C++ oracle must come from the repository's provenance pin"
    );
    let fixture = runtime_dir.join("tests/unit_tests/assets/click_event.riv");
    let output = Command::new(&probe)
        .args([
            "--no-advance",
            "--instance-artboards",
            "--runtime-update",
            "--runtime-advance-and-apply-state-machine",
            "0",
            "0",
            "--runtime-pointer-down-state-machine",
            "0",
            "75",
            "75",
            "--runtime-pointer-up-state-machine",
            "0",
            "75",
            "75",
            "--runtime-advance-and-apply-state-machine",
            "0",
            "0",
            "--file",
        ])
        .arg(&fixture)
        .output()
        .expect("run provenance-checked C++ probe");
    assert!(
        output.status.success(),
        "C++ probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cpp: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let reports = cpp["artboards"][0]["runtimeStateMachineAdvances"]
        .as_array()
        .expect("C++ art-1 state-machine reports");
    let cpp_down_hit = reports[1]["pointerHitResult"].as_u64().unwrap() as u32;
    let cpp_event_count = reports[2]["reportedEventCount"].as_u64().unwrap() as usize;
    let cpp_event_name = reports[2]["reportedEvents"][0]["eventName"]
        .as_str()
        .unwrap();
    let cpp_final_keep_going = reports[3]["advanced"].as_bool().unwrap();
    let cpp_final_state_types = reports[3]["changedStateCoreTypes"].as_array().unwrap();

    let file = import("click_event.riv");
    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new_named(file, view("art-1"), &mut instance) },
        NuxStatus::Ok
    );
    let player = state_player(instance, "sm-1");
    let (status, initialization) = step(player, &[], &[], 0.0);
    assert_eq!(status, NuxStatus::Ok);
    unsafe { nux_player_step_result_free(initialization) };
    let pointers = [
        NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 75.0,
            y: 75.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
        NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_UP,
            x: 75.0,
            y: 75.0,
            pointer_id: 0,
            timestamp_seconds: 0.0,
        },
    ];
    let (status, result) = step(player, &[], &pointers, 0.0);
    assert_eq!(status, NuxStatus::Ok);
    let c_info = info(result);
    let mut c_down_hit = NUX_PLAYER_POINTER_HIT_NONE;
    assert_eq!(
        unsafe { nux_player_step_result_pointer(result, 0, &mut c_down_hit) },
        NuxStatus::Ok
    );
    let mut c_event = NuxPlayerEventView::default();
    assert_eq!(
        unsafe { nux_player_step_result_event(result, 0, &mut c_event) },
        NuxStatus::Ok
    );
    assert_eq!(c_down_hit, cpp_down_hit);
    assert_eq!(c_info.event_count, cpp_event_count);
    assert_eq!(owned(c_event.name), cpp_event_name);
    assert_eq!(c_info.keep_going, cpp_final_keep_going);
    assert_eq!(c_info.state_change_count, cpp_final_state_types.len());

    let bytes = fixture_bytes("click_event.riv");
    let rust_file = Arc::new(nuxie::File::import(&bytes).unwrap());
    let mut rust_instance = nuxie::OwnedArtboardInstance::instantiate(rust_file, 0).unwrap();
    let mut rust_machine = rust_instance.state_machine_instance_named("sm-1").unwrap();
    let _ = rust_instance
        .try_advance_with_state_machine(&mut rust_machine, 0.0)
        .unwrap();
    let rust_down_hit =
        rust_machine.pointer_down_hit_result(rust_instance.raw_mut(), 75.0, 75.0, 0, None);
    let _ = rust_machine.pointer_up_hit_result(rust_instance.raw_mut(), 75.0, 75.0, 0, None);
    let rust_immediate_event_count = rust_machine.reported_event_count();
    let rust_keep_going = rust_instance
        .try_advance_with_state_machine(&mut rust_machine, 0.0)
        .unwrap();
    let rust_down_hit = match rust_down_hit {
        nuxie::RuntimeHitResult::None => NUX_PLAYER_POINTER_HIT_NONE,
        nuxie::RuntimeHitResult::Hit => NUX_PLAYER_POINTER_HIT_HIT,
        nuxie::RuntimeHitResult::HitOpaque => NUX_PLAYER_POINTER_HIT_HIT_OPAQUE,
    };
    assert_eq!(rust_down_hit, cpp_down_hit);
    assert_eq!(rust_immediate_event_count, cpp_event_count);
    assert_eq!(rust_keep_going, cpp_final_keep_going);
    assert_eq!(
        rust_machine.changed_state_count(),
        cpp_final_state_types.len()
    );

    unsafe {
        nux_player_step_result_free(result);
        nux_player_free(player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}
