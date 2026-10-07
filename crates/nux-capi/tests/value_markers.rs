#![allow(
    clippy::unwrap_used,
    clippy::arithmetic_side_effects,
    reason = "bounded C API fixture assertions"
)]
use nux_capi::*;
use std::ptr;
#[path = "support/value_markers.rs"]
mod fixture;
fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}
fn marker(value: &str, target: &str) -> NuxValueMarker {
    NuxValueMarker {
        model: view("Values"),
        value: view(value),
        marker: view(target),
    }
}
fn install(file: *mut NuxFile, entries: &[NuxValueMarker]) -> NuxStatus {
    unsafe { nux_file_set_value_markers(file, entries.as_ptr(), entries.len()) }
}
fn import(bytes: &[u8]) -> *mut NuxFile {
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
fn model(file: *mut NuxFile) -> *mut NuxViewModelInstance {
    let mut value = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut value) },
        NuxStatus::Ok
    );
    value
}
fn mutation(model: *mut NuxViewModelInstance, path: &str, kind: u32) -> NuxViewModelMutation {
    NuxViewModelMutation {
        instance: model,
        path: view(path),
        kind,
        ..Default::default()
    }
}
fn mutate(writes: &[NuxViewModelMutation]) -> Vec<(usize, u32, f32, u32)> {
    let batch = NuxViewModelMutationBatch {
        mutations: writes.as_ptr(),
        mutation_count: writes.len(),
        ..Default::default()
    };
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::Ok
    );
    let mut rows = Vec::new();
    let mut index = 0;
    loop {
        let mut row = NuxViewModelChangeView::default();
        let status = unsafe { nux_view_model_mutation_result_change(result, index, &mut row) };
        if status == NuxStatus::NotFound {
            break;
        }
        assert_eq!(status, NuxStatus::Ok);
        rows.push((
            row.property_index,
            row.kind,
            row.number_value,
            row.bool_value,
        ));
        index += 1;
    }
    unsafe {
        nux_view_model_mutation_result_free(result);
    }
    rows
}
fn state(model: *mut NuxViewModelInstance) -> (f32, u32) {
    let mut snapshot = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_snapshot(model, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut n = NuxViewModelSnapshotValueView::default();
    let mut set = NuxViewModelSnapshotValueView::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_value(snapshot, 0, &mut n) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_snapshot_value(snapshot, 1, &mut set) },
        NuxStatus::Ok
    );
    unsafe {
        nux_view_model_snapshot_free(snapshot);
    }
    (n.number_value, set.bool_value)
}
struct Player {
    artboard: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
}
impl Player {
    fn new(file: *mut NuxFile, value: *mut NuxViewModelInstance) -> Self {
        Self::at(file, value, 0)
    }
    fn at(file: *mut NuxFile, value: *mut NuxViewModelInstance, index: usize) -> Self {
        let mut a = ptr::null_mut();
        let mut p = ptr::null_mut();
        assert_eq!(
            unsafe { nux_artboard_instance_new(file, index, &mut a) },
            NuxStatus::Ok
        );
        if !value.is_null() {
            assert_eq!(
                unsafe { nux_artboard_instance_bind_view_model(a, value) },
                NuxStatus::Ok
            );
        }
        assert_eq!(unsafe { nux_player_new_default(a, &mut p) }, NuxStatus::Ok);
        assert_eq!(unsafe { nux_player_enable_semantics(p) }, NuxStatus::Ok);
        Self {
            artboard: a,
            player: p,
        }
    }
    fn step(&self, step: &NuxPlayerStep) -> Vec<(usize, u32, f32, u32)> {
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(self.player, step, &mut result) },
            NuxStatus::Ok
        );
        let mut info = NuxPlayerStepInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_info(result, &mut info) },
            NuxStatus::Ok
        );
        let mut rows = Vec::new();
        for index in 0..info.view_model_change_count {
            let mut row = NuxViewModelChangeView::default();
            assert_eq!(
                unsafe { nux_player_step_result_view_model_change(result, index, &mut row) },
                NuxStatus::Ok
            );
            rows.push((
                row.property_index,
                row.kind,
                row.number_value,
                row.bool_value,
            ));
        }
        let mut info = NuxPlayerSchedulingInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_scheduling(result, &mut info) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_acknowledge_presented(self.player, info.render_revision) },
            NuxStatus::Ok
        );
        unsafe {
            nux_player_step_result_free(result);
        }
        rows
    }
    fn label(&self) -> String {
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(self.player, &mut snapshot) },
            NuxStatus::Ok
        );
        let mut node = NuxSemanticNodeView {
            struct_size: size_of::<NuxSemanticNodeView>() as u32,
            ..Default::default()
        };
        assert_eq!(
            unsafe { nux_semantic_snapshot_node(snapshot, 0, &mut node) },
            NuxStatus::Ok
        );
        let label = String::from_utf8(
            unsafe { std::slice::from_raw_parts(node.label.data.cast(), node.label.len) }.to_vec(),
        )
        .unwrap();
        unsafe {
            nux_semantic_snapshot_free(snapshot);
        }
        label
    }
    fn press(&self) -> Vec<(usize, u32, f32, u32)> {
        let pointer = NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 10.0,
            y: 5.0,
            pointer_id: 1,
            timestamp_seconds: 0.0,
        };
        self.step(&NuxPlayerStep {
            pointers: &pointer,
            pointer_count: 1,
            ..Default::default()
        })
    }
}
impl Drop for Player {
    fn drop(&mut self) {
        unsafe {
            nux_player_free(self.player);
            nux_artboard_instance_free(self.artboard);
        }
    }
}
#[test]
fn host_mutations_set_markers_and_respect_both_explicit_orders() {
    let file = import(&fixture::fixture(None, &[], false));
    let v = model(file);
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    let zero = mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER);
    let clear = mutation(v, "n_set", NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL);
    assert_eq!(
        mutate(&[zero]).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(state(v), (0.0, 1));
    mutate(&[zero, clear]);
    assert_eq!(state(v), (0.0, 0));
    mutate(&[clear, zero]);
    assert_eq!(state(v), (0.0, 1));
    assert_eq!(install(file, &[]), NuxStatus::Ok);
    mutate(&[clear, zero]);
    assert_eq!(state(v), (0.0, 0));
    unsafe {
        nux_view_model_instance_free(v);
        nux_file_free(file);
    }
}
#[test]
fn refused_installs_leave_the_previous_table_in_force() {
    let file = import(&fixture::fixture(None, &[], false));
    let v = model(file);
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    for (entries, status) in [
        (vec![marker("absent", "n_set")], NuxStatus::NotFound),
        (vec![marker("text", "n_set")], NuxStatus::InvalidArgument),
        (vec![marker("n", "n")], NuxStatus::InvalidArgument),
        (
            vec![marker("n", "n_set"), marker("b", "n_set")],
            NuxStatus::InvalidArgument,
        ),
        (
            vec![marker("n", "b"), marker("b", "b_set")],
            NuxStatus::InvalidArgument,
        ),
    ] {
        assert_eq!(install(file, &entries), status);
    }
    mutate(&[mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER)]);
    assert_eq!(state(v), (0.0, 1));
    unsafe {
        nux_view_model_instance_free(v);
        nux_file_free(file);
    }
}
#[test]
fn listener_value_and_marker_rows_follow_authored_order() {
    for actions in [
        vec![fixture::Action::Number(5.0)],
        vec![fixture::Action::Number(5.0), fixture::Action::Marker(true)],
    ] {
        let file = import(&fixture::fixture(None, &actions, false));
        let v = model(file);
        assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
        let p = Player::new(file, v);
        p.step(&NuxPlayerStep::default());
        assert_eq!(
            p.press().iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(state(v), (5.0, 1));
        assert_eq!(
            p.label(),
            "1",
            "marker binding is current before the next frame"
        );
        drop(p);
        unsafe {
            nux_view_model_instance_free(v);
            nux_file_free(file);
        }
    }
}
#[test]
fn listener_clear_preserves_its_changed_marker_and_no_table_is_inert() {
    for installed in [false, true] {
        let file = import(&fixture::fixture(
            None,
            &[fixture::Action::Number(0.0), fixture::Action::Marker(false)],
            false,
        ));
        let v = model(file);
        if installed {
            assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
        }
        let mut number = mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER);
        number.number_value = 5.0;
        let mut set = mutation(v, "n_set", NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL);
        set.bool_value = 1;
        mutate(&[number, set]);
        let p = Player::new(file, v);
        p.step(&NuxPlayerStep::default());
        assert_eq!(
            p.press().iter().map(|r| r.0).collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(state(v), (0.0, 0));
        drop(p);
        unsafe {
            nux_view_model_instance_free(v);
            nux_file_free(file);
        }
    }
}

#[test]
fn native_typing_sets_marker_and_unchanged_zero_requires_compiler_marker_binding() {
    for (text, expected) in [("5", (5.0, 1)), ("0", (0.0, 0))] {
        let file = import(&fixture::fixture(None, &[], true));
        let v = model(file);
        assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
        let p = Player::new(file, v);
        p.step(&NuxPlayerStep::default());
        let focus = NuxPlayerFocusInput {
            kind: NUX_PLAYER_FOCUS_KIND_NEXT,
            ..Default::default()
        };
        p.step(&NuxPlayerStep {
            focus_inputs: &focus,
            focus_input_count: 1,
            ..Default::default()
        });
        let mut focus_state = NuxPlayerFocusState::default();
        assert_eq!(
            unsafe { nux_player_focus_state(p.player, &mut focus_state) },
            NuxStatus::Ok
        );
        assert_eq!(focus_state.has_focus, 1);
        assert_eq!(focus_state.expects_keyboard_input, 1);
        let input = NuxPlayerFocusInput {
            kind: NUX_PLAYER_FOCUS_KIND_TEXT,
            text: view(text),
            ..Default::default()
        };
        p.step(&NuxPlayerStep {
            focus_inputs: &input,
            focus_input_count: 1,
            ..Default::default()
        });
        // An unchanged native number write has no journal entry. The compiler
        // must emit its paired marker binding to represent this zero as set.
        assert_eq!(state(v), expected, "typed {text}");
        drop(p);
        unsafe {
            nux_view_model_instance_free(v);
            nux_file_free(file);
        }
    }
}

#[test]
fn field_string_edit_is_seen_by_the_next_step() {
    let file = import(&fixture::fixture(None, &[], true));
    let v = model(file);
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    let p = Player::new(file, v);
    p.step(&NuxPlayerStep::default());
    let mut snapshot = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_semantic_snapshot(p.player, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut node = NuxSemanticNodeView {
        struct_size: size_of::<NuxSemanticNodeView>() as u32,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_node(snapshot, 0, &mut node) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe {
            nux_player_field_string_set(p.player, snapshot, node.id, view("editable"), view("5"))
        },
        NuxStatus::Ok
    );
    unsafe {
        nux_semantic_snapshot_free(snapshot);
    }
    p.step(&NuxPlayerStep::default());
    assert_eq!(state(v), (5.0, 1));
    drop(p);
    unsafe {
        nux_view_model_instance_free(v);
        nux_file_free(file);
    }
}

#[test]
fn marker_table_is_retained_by_models_after_the_file_handle_is_freed() {
    let file = import(&fixture::fixture(None, &[], false));
    let a = model(file);
    let b = model(file);
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    unsafe {
        nux_file_free(file);
    }
    for v in [a, b] {
        mutate(&[mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER)]);
        assert_eq!(state(v), (0.0, 1));
        unsafe {
            nux_view_model_instance_free(v);
        }
    }
}

#[test]
fn file_with_uninstalled_markers_reports_only_the_native_value_change() {
    let file = import(&fixture::fixture(
        None,
        &[fixture::Action::Number(5.0)],
        false,
    ));
    let v = model(file);
    let p = Player::new(file, v);
    p.step(&NuxPlayerStep::default());
    assert_eq!(p.press().iter().map(|r| r.0).collect::<Vec<_>>(), vec![0]);
    assert_eq!(state(v), (5.0, 0));
    drop(p);
    unsafe {
        nux_view_model_instance_free(v);
        nux_file_free(file);
    }
}

#[cfg(feature = "scripting")]
mod scripts {
    use super::*;
    fn payload() -> Vec<u8> {
        let source = br#"return function(context)
            return { init = function() return true end,
                performAction = function() context:viewModel().n.value = 0 end }
        end"#;
        compile(source)
    }
    pub(super) fn compile(source: &[u8]) -> Vec<u8> {
        luaur_common::set_all_flags(true);
        let mut size = 0;
        let code = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            ptr::null_mut(),
            &mut size,
        );
        assert!(!code.is_null());
        let mut result = vec![0];
        result.extend_from_slice(unsafe { std::slice::from_raw_parts(code.cast(), size) });
        result
    }
    pub(super) fn trusted(bytes: &[u8]) -> *mut NuxFile {
        let mut file = ptr::null_mut();
        let mut result = ptr::null_mut();
        let config = NuxHostCommandImportConfig {
            module_name: view("bridge"),
            ..Default::default()
        };
        assert_eq!(
            unsafe {
                nux_file_import_trusted_with_host_commands(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &config,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        unsafe {
            nux_capi_result_free(result);
        }
        file
    }
    #[test]
    fn trusted_script_unchanged_zero_sets_the_marker() {
        let file = trusted(&fixture::fixture(
            Some(&payload()),
            &[fixture::Action::Script],
            false,
        ));
        let v = model(file);
        assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
        let p = Player::new(file, v);
        p.step(&NuxPlayerStep::default());
        assert_eq!(p.press().iter().map(|r| r.0).collect::<Vec<_>>(), vec![1]);
        assert_eq!(state(v), (0.0, 1));
        drop(p);
        unsafe {
            nux_view_model_instance_free(v);
            nux_file_free(file);
        }
    }
    #[test]
    fn script_after_listener_clear_counts_even_when_number_is_unchanged() {
        let file = trusted(&fixture::fixture(
            Some(&payload()),
            &[
                fixture::Action::Number(0.0),
                fixture::Action::Marker(false),
                fixture::Action::Script,
            ],
            false,
        ));
        let v = model(file);
        assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
        let mut number = mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER);
        number.number_value = 5.0;
        mutate(&[number]);
        let p = Player::new(file, v);
        p.step(&NuxPlayerStep::default());
        p.press();
        assert_eq!(state(v), (0.0, 1));
        drop(p);
        unsafe {
            nux_view_model_instance_free(v);
            nux_file_free(file);
        }
    }
    #[test]
    fn listener_changed_clear_after_script_preserves_the_clear() {
        let file = trusted(&fixture::fixture(
            Some(&payload()),
            &[
                fixture::Action::Script,
                fixture::Action::Number(0.0),
                fixture::Action::Marker(false),
            ],
            false,
        ));
        let v = model(file);
        assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
        let mut number = mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER);
        number.number_value = 5.0;
        mutate(&[number]);
        let p = Player::new(file, v);
        p.step(&NuxPlayerStep::default());
        p.press();
        assert_eq!(state(v), (0.0, 0));
        drop(p);
        unsafe {
            nux_view_model_instance_free(v);
            nux_file_free(file);
        }
    }
}

#[test]
fn marker_journal_overflow_rolls_back_the_entire_host_batch() {
    let file = import(&fixture::fixture(None, &[], false));
    let v = model(file);
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    let writes = (0..1024)
        .map(|i| {
            let mut m = mutation(v, "n", NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER);
            m.number_value = i as f32;
            m
        })
        .collect::<Vec<_>>();
    let batch = NuxViewModelMutationBatch {
        mutations: writes.as_ptr(),
        mutation_count: writes.len(),
        ..Default::default()
    };
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::LimitExceeded
    );
    assert_eq!(state(v), (0.0, 0));
    let mut row = NuxViewModelChangeView::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_change(result, 0, &mut row) },
        NuxStatus::NotFound
    );
    unsafe {
        nux_view_model_mutation_result_free(result);
        nux_view_model_instance_free(v);
        nux_file_free(file);
    }
}

#[test]
fn native_default_instance_gets_markers_without_a_host_subscription() {
    let file = import(&fixture::fixture(
        None,
        &[fixture::Action::Number(5.0)],
        false,
    ));
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    let player = Player::new(file, ptr::null_mut());
    let mut auxiliary = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 1, 0, &mut auxiliary) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_set_global_view_model(player.player, view("Aux"), auxiliary) },
        NuxStatus::Ok
    );
    unsafe { nux_view_model_instance_free(auxiliary) };
    assert!(player.step(&NuxPlayerStep::default()).is_empty());
    assert_eq!(player.label(), "0");
    assert!(player.press().is_empty());
    assert_eq!(player.label(), "1");
    drop(player);
    unsafe { nux_file_free(file) };
}

#[cfg(feature = "scripting")]
#[test]
fn markers_follow_writes_in_native_component_copies_and_list_rows() {
    let script = scripts::compile(
        br#"return function(context)
        return { init = function() return true end,
            performAction = function()
                context:viewModel().n.value = 5
                context:rootViewModel().rows[1].n.value = 5
            end }
    end"#,
    );
    let file = scripts::trusted(&fixture::occurrences(&script));
    let mut root = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 1, 0, &mut root) },
        NuxStatus::Ok
    );
    assert_eq!(install(file, &[marker("n", "n_set")]), NuxStatus::Ok);
    let player = Player::at(file, root, 1);
    player.step(&NuxPlayerStep::default());
    let labels = || {
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(player.player, &mut snapshot) },
            NuxStatus::Ok
        );
        let mut labels = Vec::new();
        for index in 0..8 {
            let mut node = NuxSemanticNodeView {
                struct_size: size_of::<NuxSemanticNodeView>() as u32,
                ..Default::default()
            };
            let status = unsafe { nux_semantic_snapshot_node(snapshot, index, &mut node) };
            if status == NuxStatus::NotFound {
                break;
            }
            assert_eq!(status, NuxStatus::Ok);
            if node.label.len != 0 {
                labels.push(
                    String::from_utf8(
                        unsafe {
                            std::slice::from_raw_parts(node.label.data.cast(), node.label.len)
                        }
                        .to_vec(),
                    )
                    .unwrap(),
                );
            }
        }
        unsafe { nux_semantic_snapshot_free(snapshot) };
        labels
    };
    assert_eq!(labels(), vec!["0", "0"]);
    for (x, expected) in [(105.0, vec!["1", "1"])] {
        let pointer = NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x,
            y: 5.0,
            pointer_id: x as i32,
            timestamp_seconds: 0.0,
        };
        player.step(&NuxPlayerStep {
            pointers: &pointer,
            pointer_count: 1,
            ..Default::default()
        });
        assert_eq!(labels(), expected);
    }
    drop(player);
    unsafe {
        nux_view_model_instance_free(root);
        nux_file_free(file);
    }
}
