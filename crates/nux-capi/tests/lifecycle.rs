#![allow(
    clippy::arithmetic_side_effects,
    clippy::field_reassign_with_default,
    clippy::unwrap_used,
    reason = "lifecycle tests deliberately mutate caller-owned ABI prefixes and use bounded fixture counters"
)]

use nux_capi::*;
use std::ffi::{CString, c_void};
use std::path::PathBuf;

const SMI_FIXTURE: &str = "smi_test.riv";
const SMI_ARTBOARD: usize = 1;

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

fn import(name: &str) -> *mut NuxFile {
    let bytes = fixture_bytes(name);
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
        NuxStatus::Ok
    );
    file
}

fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn owned(view: NuxStringView) -> String {
    if view.len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) };
    std::str::from_utf8(bytes).unwrap().to_owned()
}

fn artboard(file: *mut NuxFile, index: usize) -> *mut NuxArtboardInstance {
    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, index, &mut instance) },
        NuxStatus::Ok
    );
    instance
}

fn player_info(player: *mut NuxPlayer) -> NuxPlayerInfo {
    let mut info = NuxPlayerInfo::default();
    assert_eq!(unsafe { nux_player_info(player, &mut info) }, NuxStatus::Ok);
    info
}

#[test]
fn exact_selectors_have_no_case_or_kind_fallback_and_copy_player_metadata() {
    let file = import(SMI_FIXTURE);

    let mut named_artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_artboard_instance_new_named(file, view("artboard to nest"), &mut named_artboard)
        },
        NuxStatus::Ok
    );
    unsafe { nux_artboard_instance_free(named_artboard) };
    assert_eq!(
        unsafe {
            nux_artboard_instance_new_named(file, view("Artboard To Nest"), &mut named_artboard)
        },
        NuxStatus::NotFound
    );
    assert!(named_artboard.is_null());

    let instance = artboard(file, SMI_ARTBOARD);
    let mut state_player = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_new_state_machine_named(instance, view("State Machine 1"), &mut state_player)
        },
        NuxStatus::Ok
    );
    let info = player_info(state_player);
    assert_eq!(info.kind, NuxPlayerKind::StateMachine);
    assert_eq!(info.index, 0);
    assert_eq!(owned(info.name), "State Machine 1");

    let mut missing = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_new_state_machine_named(instance, view("state machine 1"), &mut missing)
        },
        NuxStatus::NotFound
    );
    assert!(missing.is_null());
    assert_eq!(
        unsafe { nux_player_new_state_machine_named(instance, view(""), &mut missing) },
        NuxStatus::NotFound
    );

    let mut animation_player = std::ptr::null_mut();
    let mut animation_name = NuxStringView::default();
    assert_eq!(
        unsafe { nux_file_artboard_animation_name(file, SMI_ARTBOARD, 0, &mut animation_name) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe {
            nux_player_new_linear_animation_named(instance, animation_name, &mut animation_player)
        },
        NuxStatus::Ok
    );
    let info = player_info(animation_player);
    assert_eq!(info.kind, NuxPlayerKind::LinearAnimation);
    assert_eq!(info.index, 0);
    assert_eq!(owned(info.name), "Timeline 1");

    let mut static_player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(instance, &mut static_player) },
        NuxStatus::Ok
    );
    let info = player_info(static_player);
    assert_eq!(info.kind, NuxPlayerKind::StaticArtboard);
    assert_eq!(info.index, usize::MAX);
    assert_eq!(owned(info.name), "");

    let mut default_player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_default(instance, &mut default_player) },
        NuxStatus::Ok
    );
    assert_eq!(
        player_info(default_player).kind,
        NuxPlayerKind::StateMachine
    );

    unsafe {
        nux_player_free(default_player);
        nux_player_free(static_player);
        nux_player_free(animation_player);
        nux_player_free(state_player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn default_selection_matches_authored_then_valid_fallback_order() {
    for (fixture, index, expected_kind) in [
        // Authored valid default state machine.
        (SMI_FIXTURE, SMI_ARTBOARD, NuxPlayerKind::StateMachine),
        // Its sole state-machine definition cannot instantiate and there is no
        // animation, so selection reaches the static fallback.
        ("shapetest.riv", 0, NuxPlayerKind::StaticArtboard),
        // No state machines; linear-animation zero is selected.
        ("circle_clips.riv", 0, NuxPlayerKind::LinearAnimation),
    ] {
        let file = import(fixture);
        let instance = artboard(file, index);
        let mut player = std::ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_new_default(instance, &mut player) },
            NuxStatus::Ok
        );
        assert_eq!(player_info(player).kind, expected_kind, "fixture {fixture}");
        unsafe {
            nux_player_free(player);
            nux_artboard_instance_free(instance);
            nux_file_free(file);
        }
    }
}

#[derive(Default)]
struct RenderLifetime {
    next: u64,
    made: usize,
    released: usize,
}

unsafe extern "C" fn make_path(
    user_data: *mut c_void,
    _path: *const NuxRawPathView,
    _fill_rule: u8,
) -> u64 {
    let state = unsafe { &mut *user_data.cast::<RenderLifetime>() };
    state.next += 1;
    state.made += 1;
    state.next
}

unsafe extern "C" fn make_object(user_data: *mut c_void) -> u64 {
    let state = unsafe { &mut *user_data.cast::<RenderLifetime>() };
    state.next += 1;
    state.made += 1;
    state.next
}

unsafe extern "C" fn release_object(user_data: *mut c_void, handle: u64) {
    assert_ne!(handle, 0);
    unsafe { &mut *user_data.cast::<RenderLifetime>() }.released += 1;
}

#[test]
fn player_retains_artboard_file_and_renderer_binding_until_last_owner() {
    let file = import(SMI_FIXTURE);
    let instance = artboard(file, SMI_ARTBOARD);
    let mut state = RenderLifetime::default();
    let callbacks = NuxRenderCallbacks {
        user_data: (&mut state as *mut RenderLifetime).cast(),
        make_render_path: Some(make_path),
        make_empty_render_path: Some(make_object),
        make_render_paint: Some(make_object),
        release_render_path: Some(release_object),
        release_render_paint: Some(release_object),
        release_render_shader: Some(release_object),
        ..NuxRenderCallbacks::default()
    };
    assert_eq!(
        unsafe { nux_artboard_instance_draw(instance, &callbacks) },
        NuxStatus::Ok
    );
    assert!(state.made > 0);
    let released_after_draw = state.released;

    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_default(instance, &mut player) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    assert_eq!(
        unsafe { nux_artboard_instance_free(instance) },
        NuxStatus::Ok
    );
    assert_eq!(
        state.released, released_after_draw,
        "player retains renderer-created objects"
    );
    assert_eq!(player_info(player).kind, NuxPlayerKind::StateMachine);
    assert_eq!(unsafe { nux_player_free(player) }, NuxStatus::Ok);
    assert_eq!(state.made, state.released);
}

#[test]
fn wrong_thread_wrong_kind_and_wrong_origin_are_stable_errors() {
    let file = import(SMI_FIXTURE);
    let first = artboard(file, SMI_ARTBOARD);
    let second = artboard(file, SMI_ARTBOARD);
    let mut machine = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_state_machine_instance_new(first, 0, &mut machine) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_state_machine_instance_advance(second, machine, 0.0, std::ptr::null_mut()) },
        NuxStatus::HandleMismatch
    );
    assert_eq!(
        unsafe { nux_artboard_instance_advance(file.cast(), 0.0, std::ptr::null_mut()) },
        NuxStatus::HandleMismatch
    );

    let file_address = file as usize;
    let (use_status, free_status) = std::thread::spawn(move || {
        let file = file_address as *mut NuxFile;
        let mut count = 99;
        (
            unsafe { nux_file_artboard_count(file, &mut count) },
            unsafe { nux_file_free(file) },
        )
    })
    .join()
    .unwrap();
    assert_eq!(use_status, NuxStatus::WrongThread);
    assert_eq!(free_status, NuxStatus::WrongThread);
    let mut count = 0;
    assert_eq!(
        unsafe { nux_file_artboard_count(file, &mut count) },
        NuxStatus::Ok
    );

    unsafe {
        nux_state_machine_instance_free(machine);
        nux_artboard_instance_free(second);
        nux_artboard_instance_free(first);
        nux_file_free(file);
    }
}

#[test]
fn view_model_lineage_is_checked_and_handles_outlive_file_and_artboard() {
    let file = import("data_binding_test_2.riv");
    let first = artboard(file, 0);
    let second = artboard(file, 0);
    let mut view_model = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(first, &mut view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(second, view_model) },
        NuxStatus::HandleMismatch
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    assert_eq!(unsafe { nux_artboard_instance_free(first) }, NuxStatus::Ok);
    let property = CString::new("num").unwrap();
    assert_eq!(
        unsafe { nux_view_model_instance_set_number(view_model, property.as_ptr(), 42.0) },
        NuxStatus::Ok
    );
    unsafe {
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(second);
    }
}

#[test]
fn standalone_state_machine_survives_file_and_artboard_handle_release() {
    let file = import(SMI_FIXTURE);
    let instance = artboard(file, SMI_ARTBOARD);
    let mut machine = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_state_machine_instance_new(instance, 0, &mut machine) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    assert_eq!(
        unsafe { nux_artboard_instance_free(instance) },
        NuxStatus::Ok
    );
    let bool_name = CString::new("bool").unwrap();
    assert_eq!(
        unsafe { nux_state_machine_instance_set_bool(machine, bool_name.as_ptr(), true) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_state_machine_instance_free(machine) },
        NuxStatus::Ok
    );
}

#[repr(C)]
struct Large<T> {
    value: T,
    canary: u64,
}

#[test]
fn versioned_structs_reject_short_prefixes_and_preserve_larger_tails() {
    let mut short_runtime = NuxRuntimeInfo::default();
    short_runtime.struct_size = 0;
    assert_eq!(
        unsafe { nux_capi_runtime_info(&mut short_runtime) },
        NuxStatus::InvalidStructSize
    );
    let mut large_runtime = Large {
        value: NuxRuntimeInfo::default(),
        canary: 0xA11C_E5AA_D15C_A11C,
    };
    large_runtime.value.struct_size = std::mem::size_of_val(&large_runtime) as u32;
    assert_eq!(
        unsafe { nux_capi_runtime_info(&mut large_runtime.value) },
        NuxStatus::Ok
    );
    assert_eq!(large_runtime.canary, 0xA11C_E5AA_D15C_A11C);

    let file = import(SMI_FIXTURE);
    let instance = artboard(file, SMI_ARTBOARD);
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(instance, &mut player) },
        NuxStatus::Ok
    );
    let mut short_player = NuxPlayerInfo::default();
    short_player.struct_size = 0;
    assert_eq!(
        unsafe { nux_player_info(player, &mut short_player) },
        NuxStatus::InvalidStructSize
    );
    let mut large_player = Large {
        value: NuxPlayerInfo::default(),
        canary: 0xBADC_0FFE_E0DD_F00D,
    };
    large_player.value.struct_size = std::mem::size_of_val(&large_player) as u32;
    assert_eq!(
        unsafe { nux_player_info(player, &mut large_player.value) },
        NuxStatus::Ok
    );
    assert_eq!(large_player.canary, 0xBADC_0FFE_E0DD_F00D);

    let mut short_callbacks = NuxRenderCallbacks::default();
    short_callbacks.struct_size = 0;
    assert_eq!(
        unsafe { nux_artboard_instance_draw(instance, &short_callbacks) },
        NuxStatus::InvalidStructSize
    );
    let mut large_callbacks = Large {
        value: NuxRenderCallbacks::default(),
        canary: 0x1234_5678_9ABC_DEF0,
    };
    large_callbacks.value.struct_size = std::mem::size_of_val(&large_callbacks) as u32;
    assert_eq!(
        unsafe { nux_artboard_instance_draw(instance, &large_callbacks.value) },
        NuxStatus::Ok
    );
    assert_eq!(large_callbacks.canary, 0x1234_5678_9ABC_DEF0);
    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn owned_results_cover_import_and_selection_errors() {
    let mut file = std::ptr::dangling_mut();
    let mut result = std::ptr::null_mut();
    let invalid = b"not a rive file";
    assert_eq!(
        unsafe {
            nux_file_import_with_result(invalid.as_ptr(), invalid.len(), &mut file, &mut result)
        },
        NuxStatus::ImportError
    );
    assert!(file.is_null());
    let mut result_status = NuxStatus::Ok;
    assert_eq!(
        unsafe { nux_capi_result_status(result, &mut result_status) },
        NuxStatus::Ok
    );
    assert_eq!(result_status, NuxStatus::ImportError);
    let mut diagnostic = NuxCapiDiagnosticView::default();
    assert_eq!(
        unsafe { nux_capi_result_diagnostic(result, &mut diagnostic) },
        NuxStatus::Ok
    );
    assert_eq!(diagnostic.status, NuxStatus::ImportError);
    assert_eq!(owned(diagnostic.code), "nux_capi.import_error");
    assert!(!owned(diagnostic.message).is_empty());
    unsafe { nux_capi_result_free(result) };

    let file = import(SMI_FIXTURE);
    let instance = artboard(file, SMI_ARTBOARD);
    let mut player = std::ptr::dangling_mut();
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_new_state_machine_named_with_result(
                instance,
                view("missing"),
                &mut player,
                &mut result,
            )
        },
        NuxStatus::NotFound
    );
    assert!(player.is_null());
    assert_eq!(
        unsafe { nux_capi_result_status(result, &mut result_status) },
        NuxStatus::Ok
    );
    assert_eq!(result_status, NuxStatus::NotFound);
    unsafe {
        nux_capi_result_free(result);
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

#[test]
fn result_bearing_apis_reject_aliased_output_slots_before_publication() {
    let bytes = fixture_bytes(SMI_FIXTURE);
    let mut shared_slot: *mut c_void = std::ptr::dangling_mut();
    let shared_slot_pointer = &mut shared_slot as *mut *mut c_void;
    assert_eq!(
        unsafe {
            nux_file_import_with_result(
                bytes.as_ptr(),
                bytes.len(),
                shared_slot_pointer.cast(),
                shared_slot_pointer.cast(),
            )
        },
        NuxStatus::InvalidArgument
    );
    assert!(shared_slot.is_null());

    let file = import(SMI_FIXTURE);
    let instance = artboard(file, SMI_ARTBOARD);
    shared_slot = std::ptr::dangling_mut();
    assert_eq!(
        unsafe {
            nux_player_new_default_with_result(
                instance,
                shared_slot_pointer.cast(),
                shared_slot_pointer.cast(),
            )
        },
        NuxStatus::InvalidArgument
    );
    assert!(shared_slot.is_null());
    unsafe {
        nux_artboard_instance_free(instance);
        nux_file_free(file);
    }
}

struct ReentrantFree {
    instance: *mut NuxArtboardInstance,
    status: NuxStatus,
    zero_handle_releases: usize,
}

unsafe extern "C" fn reentrant_make_path(
    user_data: *mut c_void,
    _path: *const NuxRawPathView,
    _fill_rule: u8,
) -> u64 {
    let state = unsafe { &mut *user_data.cast::<ReentrantFree>() };
    state.status = unsafe { nux_artboard_instance_free(state.instance) };
    0
}

unsafe extern "C" fn reentrant_save(user_data: *mut c_void) {
    let state = unsafe { &mut *user_data.cast::<ReentrantFree>() };
    state.status = unsafe { nux_artboard_instance_free(state.instance) };
}

unsafe extern "C" fn count_zero_handle_release(user_data: *mut c_void, _handle: u64) {
    unsafe { &mut *user_data.cast::<ReentrantFree>() }.zero_handle_releases += 1;
}

unsafe extern "C" fn count_save(user_data: *mut c_void) {
    unsafe { *user_data.cast::<usize>() += 1 };
}

#[test]
fn sibling_visual_occurrences_bind_distinct_callback_contexts_independently() {
    let file = import(SMI_FIXTURE);
    let first = artboard(file, SMI_ARTBOARD);
    let second = artboard(file, SMI_ARTBOARD);
    let mut first_saves = 0usize;
    let mut second_saves = 0usize;
    let first_callbacks = NuxRenderCallbacks {
        user_data: std::ptr::from_mut(&mut first_saves).cast(),
        save: Some(count_save),
        ..NuxRenderCallbacks::default()
    };
    let second_callbacks = NuxRenderCallbacks {
        user_data: std::ptr::from_mut(&mut second_saves).cast(),
        save: Some(count_save),
        ..NuxRenderCallbacks::default()
    };

    assert_eq!(
        unsafe { nux_artboard_instance_draw(first, &first_callbacks) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_draw(second, &second_callbacks) },
        NuxStatus::Ok
    );
    assert!(first_saves > 0);
    assert!(second_saves > 0);

    assert_eq!(unsafe { nux_artboard_instance_free(first) }, NuxStatus::Ok);
    let before = second_saves;
    assert_eq!(
        unsafe { nux_artboard_instance_draw(second, &second_callbacks) },
        NuxStatus::Ok
    );
    assert!(second_saves > before);
    assert_eq!(unsafe { nux_artboard_instance_free(second) }, NuxStatus::Ok);
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
}

#[test]
fn callback_time_free_is_rejected_and_second_callback_binding_must_match() {
    let file = import(SMI_FIXTURE);
    let instance = artboard(file, SMI_ARTBOARD);
    let mut state = ReentrantFree {
        instance,
        status: NuxStatus::Ok,
        zero_handle_releases: 0,
    };
    let callbacks = NuxRenderCallbacks {
        user_data: (&mut state as *mut ReentrantFree).cast(),
        make_render_path: Some(reentrant_make_path),
        release_render_path: Some(count_zero_handle_release),
        save: Some(reentrant_save),
        ..NuxRenderCallbacks::default()
    };
    assert_eq!(
        unsafe { nux_artboard_instance_draw(instance, &callbacks) },
        NuxStatus::Ok
    );
    assert_eq!(state.status, NuxStatus::ReentrantCall);
    let different_descriptor = callbacks;
    assert_eq!(
        unsafe { nux_artboard_instance_draw(instance, &different_descriptor) },
        NuxStatus::HandleMismatch
    );
    assert_eq!(
        unsafe { nux_artboard_instance_free(instance) },
        NuxStatus::Ok
    );
    assert_eq!(
        state.zero_handle_releases, 0,
        "zero renderer handles must never be released"
    );
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
}

#[test]
fn free_null_is_ok_and_immediate_double_free_is_best_effort_rejected() {
    assert_eq!(
        unsafe { nux_file_free(std::ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_free(std::ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_free(std::ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_state_machine_instance_free(std::ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_free(std::ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_capi_result_free(std::ptr::null_mut()) },
        NuxStatus::Ok
    );

    let file = import(SMI_FIXTURE);
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    // This diagnostic is intentionally only best effort. The public contract
    // declares all post-free pointer use invalid because allocator ABA can
    // reuse the same address for a later handle.
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::HandleMismatch);
}
