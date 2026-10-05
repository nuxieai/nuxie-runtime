#![allow(
    clippy::unwrap_used,
    clippy::arithmetic_side_effects,
    reason = "bounded independent C API fixture assertions"
)]
use nux_capi::*;
use std::ptr;
#[path = "support/global_values.rs"]
mod global_values;
fn view(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
fn import() -> *mut NuxFile {
    import_bytes(global_values::fixture(false))
}
fn import_bytes(bytes: Vec<u8>) -> *mut NuxFile {
    let mut file = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file,
            )
        },
        NuxStatus::Ok
    );
    file
}
fn player(file: *mut NuxFile, index: usize) -> (*mut NuxArtboardInstance, *mut NuxPlayer) {
    let mut a = ptr::null_mut();
    let mut p = ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, index, &mut a) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_player_new_default(a, &mut p) }, NuxStatus::Ok);
    assert_eq!(unsafe { nux_player_enable_semantics(p) }, NuxStatus::Ok);
    (a, p)
}
fn model(file: *mut NuxFile, index: usize) -> *mut NuxViewModelInstance {
    let mut v = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, index, &mut v) },
        NuxStatus::Ok
    );
    v
}
fn write(v: *mut NuxViewModelInstance, value: bool) {
    let m = NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL,
        instance: v,
        path: view("enabled"),
        bool_value: value.into(),
        ..NuxViewModelMutation::default()
    };
    let b = NuxViewModelMutationBatch {
        mutations: &m,
        mutation_count: 1,
        ..NuxViewModelMutationBatch::default()
    };
    let mut r = ptr::null_mut();
    assert_eq!(unsafe { nux_view_model_mutate(&b, &mut r) }, NuxStatus::Ok);
    unsafe {
        nux_view_model_mutation_result_free(r);
    }
}
fn step_and_present(p: *mut NuxPlayer) -> u64 {
    unsafe {
        let mut r = ptr::null_mut();
        assert_eq!(
            nux_player_step(p, &NuxPlayerStep::default(), &mut r),
            NuxStatus::Ok
        );
        let mut result_info = NuxPlayerSchedulingInfo::default();
        assert_eq!(
            nux_player_step_result_scheduling(r, &mut result_info),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_acknowledge_presented(p, result_info.render_revision),
            NuxStatus::Ok
        );
        nux_player_step_result_free(r);
        result_info.render_revision
    }
}
fn labels(p: *mut NuxPlayer) -> Vec<String> {
    step_and_present(p);
    unsafe {
        let mut s = ptr::null_mut();
        assert_eq!(nux_player_semantic_snapshot(p, &mut s), NuxStatus::Ok);
        let mut info = NuxSemanticSnapshotInfo {
            struct_size: size_of::<NuxSemanticSnapshotInfo>() as u32,
            render_revision: 0,
            tree_version: 0,
            node_count: 0,
        };
        assert_eq!(nux_semantic_snapshot_info(s, &mut info), NuxStatus::Ok);
        let mut values = Vec::new();
        for i in 0..info.node_count {
            let mut n = NuxSemanticNodeView {
                struct_size: size_of::<NuxSemanticNodeView>() as u32,
                ..Default::default()
            };
            assert_eq!(nux_semantic_snapshot_node(s, i, &mut n), NuxStatus::Ok);
            if n.label.len != 0 {
                values.push(
                    String::from_utf8(
                        std::slice::from_raw_parts(n.label.data.cast(), n.label.len).to_vec(),
                    )
                    .unwrap(),
                );
            }
        }
        nux_semantic_snapshot_free(s);
        values
    }
}
#[test]
fn global_set_and_clear_reach_screen_copy_and_list_row() {
    // Pinned 7acbdfecb data_converter_to_string.cpp:203-207 renders bool as 1/0.
    let f = import();
    let (a, p) = player(f, 1);
    let v = model(f, 0);
    write(v, true);
    assert_eq!(
        unsafe { nux_player_set_global_view_model(p, view("Flags"), v) },
        NuxStatus::Ok
    );
    assert_eq!(labels(p), ["1", "1", "1"]);
    // The player owns the live instance after its public handle is released.
    unsafe {
        nux_view_model_instance_free(v);
        nux_file_free(f);
        nux_artboard_instance_free(a);
    }
    assert_eq!(labels(p), ["1", "1", "1"]);
    assert_eq!(
        unsafe { nux_player_set_global_view_model(p, view("Flags"), ptr::null()) },
        NuxStatus::Ok
    );
    assert_eq!(labels(p), ["0", "0", "0"]);
    unsafe {
        nux_player_free(p);
    }
}
#[test]
fn global_refusals_preserve_the_bound_instance() {
    let f = import();
    let (a, p) = player(f, 1);
    let v = model(f, 0);
    let wrong = model(f, 1);
    let other_f = import();
    let other = model(other_f, 0);
    let (static_a, temporary) = player(f, 0);
    unsafe {
        nux_player_free(temporary);
    }
    let mut static_p = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(static_a, &mut static_p) },
        NuxStatus::Ok
    );
    let mut linear_p = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_linear_animation_named(a, view("Timeline"), &mut linear_p) },
        NuxStatus::Ok
    );
    write(v, true);
    assert_eq!(
        unsafe { nux_player_set_global_view_model(p, view("Flags"), v) },
        NuxStatus::Ok
    );
    for (target, name, value, status) in [
        (p, "missing", v, NuxStatus::NotFound),
        (p, "Root", wrong, NuxStatus::NotFound),
        (p, "Flags", wrong, NuxStatus::HandleMismatch),
        (p, "Flags", other, NuxStatus::HandleMismatch),
        (static_p, "Flags", v, NuxStatus::NotFound),
        (linear_p, "Flags", v, NuxStatus::NotFound),
    ] {
        assert_eq!(
            unsafe { nux_player_set_global_view_model(target, view(name), value) },
            status,
            "name {name}, target {target:p}"
        );
    }
    assert_eq!(labels(p), ["1", "1", "1"]);
    unsafe {
        nux_player_free(p);
        nux_player_free(static_p);
        nux_player_free(linear_p);
        nux_artboard_instance_free(a);
        nux_artboard_instance_free(static_a);
        nux_view_model_instance_free(v);
        nux_view_model_instance_free(wrong);
        nux_view_model_instance_free(other);
        nux_file_free(f);
        nux_file_free(other_f);
    }
}

#[test]
fn one_global_instance_updates_two_players_and_invalidates_both() {
    let f = import();
    let (a, first) = player(f, 1);
    let (b, second) = player(f, 1);
    let v = model(f, 0);
    write(v, true);
    for p in [first, second] {
        assert_eq!(
            unsafe { nux_player_set_global_view_model(p, view("Flags"), v) },
            NuxStatus::Ok
        );
        assert_eq!(labels(p), ["1", "1", "1"]);
    }
    let first_revision = step_and_present(first);
    let second_revision = step_and_present(second);
    write(v, false);
    for (p, revision) in [(first, first_revision), (second, second_revision)] {
        assert_eq!(
            unsafe { nux_player_acknowledge_presented(p, revision) },
            NuxStatus::HandleMismatch
        );
        assert_eq!(labels(p), ["0", "0", "0"]);
    }
    assert_eq!(
        unsafe { nux_player_set_global_view_model(first, view("Flags"), ptr::null()) },
        NuxStatus::Ok
    );
    write(v, true);
    assert_eq!(labels(first), ["0", "0", "0"]);
    assert_eq!(labels(second), ["1", "1", "1"]);
    unsafe {
        nux_player_free(first);
        nux_artboard_instance_free(a);
        nux_file_free(f);
    }
    write(v, false);
    assert_eq!(labels(second), ["0", "0", "0"]);
    unsafe {
        nux_player_free(second);
        nux_artboard_instance_free(b);
        nux_view_model_instance_free(v);
    }
}

#[test]
fn listener_global_write_is_reported_only_for_host_set_roots() {
    for host_set in [false, true] {
        let f = import_bytes(global_values::fixture(true));
        let (a, p) = player(f, 1);
        let v = model(f, 0);
        if host_set {
            assert_eq!(
                unsafe { nux_player_set_global_view_model(p, view("Flags"), v) },
                NuxStatus::Ok
            );
        } else {
            // Bind defaults through the same clear operation without subscribing.
            assert_eq!(
                unsafe { nux_player_set_global_view_model(p, view("Flags"), ptr::null()) },
                NuxStatus::Ok
            );
        }
        assert_eq!(labels(p), ["0", "0", "0"]);
        let mut identity = 0;
        assert_eq!(
            unsafe { nux_view_model_instance_identity(v, &mut identity) },
            NuxStatus::Ok
        );
        let pointer = NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 10.0,
            y: 5.0,
            pointer_id: 1,
            timestamp_seconds: 0.0,
        };
        let step = NuxPlayerStep {
            pointers: &pointer,
            pointer_count: 1,
            correlation_id: 3593,
            ..NuxPlayerStep::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(p, &step, &mut result) },
            NuxStatus::Ok
        );
        let mut info = NuxPlayerStepInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_info(result, &mut info) },
            NuxStatus::Ok
        );
        assert_eq!(info.view_model_change_count, usize::from(host_set));
        if host_set {
            let mut change = NuxViewModelChangeView::default();
            assert_eq!(
                unsafe { nux_player_step_result_view_model_change(result, 0, &mut change) },
                NuxStatus::Ok
            );
            assert_eq!(change.owner_instance_id, identity);
            assert_eq!(change.property_index, 0);
            assert_eq!(change.kind, NUX_VIEW_MODEL_VALUE_KIND_BOOL);
            assert_eq!(change.bool_value, 1);
            assert_eq!(change.origin, NUX_VIEW_MODEL_CHANGE_ORIGIN_RUNTIME);
            assert_eq!(change.correlation_id, 3593);
        }
        unsafe {
            nux_player_step_result_free(result);
        }
        assert_eq!(labels(p), ["1", "1", "1"]);
        unsafe {
            nux_view_model_instance_free(v);
            nux_player_free(p);
            nux_artboard_instance_free(a);
            nux_file_free(f);
        }
    }
}

#[test]
fn host_global_mutation_is_reported_once_in_the_mutation_result() {
    let f = import();
    let (a, p) = player(f, 1);
    let v = model(f, 0);
    unsafe {
        assert_eq!(
            nux_player_set_global_view_model(p, view("Flags"), v),
            NuxStatus::Ok
        );
        let mut identity = 0;
        assert_eq!(
            nux_view_model_instance_identity(v, &mut identity),
            NuxStatus::Ok
        );
        let mutation = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL,
            instance: v,
            path: view("enabled"),
            bool_value: 1,
            ..NuxViewModelMutation::default()
        };
        let batch = NuxViewModelMutationBatch {
            mutations: &mutation,
            mutation_count: 1,
            correlation_id: 42,
            ..NuxViewModelMutationBatch::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
        let mut change = NuxViewModelChangeView::default();
        assert_eq!(
            nux_view_model_mutation_result_change(result, 0, &mut change),
            NuxStatus::Ok
        );
        assert_eq!(change.owner_instance_id, identity);
        assert_eq!(change.bool_value, 1);
        assert_eq!(change.origin, NUX_VIEW_MODEL_CHANGE_ORIGIN_CALLER);
        assert_eq!(change.correlation_id, 42);
        assert_eq!(
            nux_view_model_mutation_result_change(result, 1, &mut change),
            NuxStatus::NotFound
        );
        nux_view_model_mutation_result_free(result);
        let mut step_result = ptr::null_mut();
        assert_eq!(
            nux_player_step(p, &NuxPlayerStep::default(), &mut step_result),
            NuxStatus::Ok
        );
        let mut info = NuxPlayerStepInfo::default();
        assert_eq!(
            nux_player_step_result_info(step_result, &mut info),
            NuxStatus::Ok
        );
        assert_eq!(info.view_model_change_count, 0);
        nux_player_step_result_free(step_result);
        assert_eq!(labels(p), ["1", "1", "1"]);
        nux_player_free(p);
        nux_artboard_instance_free(a);
        nux_view_model_instance_free(v);
        nux_file_free(f);
    }
}
