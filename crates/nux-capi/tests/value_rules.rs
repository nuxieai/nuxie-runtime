#![allow(clippy::unwrap_used, reason = "handwritten C ABI rule assertions")]
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
fn rule(property: &str, kind: u32, mode: u32) -> NuxValueRule {
    NuxValueRule {
        model: view("Values"),
        property: view(property),
        kind,
        mode,
        code: view("authored-code"),
        message: view("Authored message"),
        ..Default::default()
    }
}
struct Handles {
    file: *mut NuxFile,
    value: *mut NuxViewModelInstance,
}
impl Handles {
    fn new() -> Self {
        let bytes = fixture::fixture(None, &[], false);
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
        let mut value = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut value) },
            NuxStatus::Ok
        );
        Self { file, value }
    }
    fn install(&self, rules: &[NuxValueRule]) -> NuxStatus {
        unsafe { nux_file_set_value_rules(self.file, rules.as_ptr(), rules.len()) }
    }
    fn write(&self, path: &str, number: f32) -> NuxViewModelMutation {
        NuxViewModelMutation {
            instance: self.value,
            path: view(path),
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            number_value: number,
            ..Default::default()
        }
    }
    fn mutate(&self, writes: &[NuxViewModelMutation]) -> *mut NuxViewModelMutationResult {
        let batch = NuxViewModelMutationBatch {
            mutations: writes.as_ptr(),
            mutation_count: writes.len(),
            correlation_id: 73,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_mutate(&batch, &mut result) },
            NuxStatus::Ok
        );
        result
    }
}
impl Drop for Handles {
    fn drop(&mut self) {
        unsafe {
            nux_view_model_instance_free(self.value);
            nux_file_free(self.file);
        }
    }
}
fn result_info(result: *mut NuxViewModelMutationResult) -> NuxViewModelMutationResultInfo {
    let mut info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    info
}
fn report(result: *mut NuxViewModelMutationResult, index: usize) -> NuxValueRuleReportView {
    let mut report = NuxValueRuleReportView::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_rule_report(result, index, &mut report) },
        NuxStatus::Ok
    );
    report
}
#[test]
fn refusal_keeps_other_writes_and_reports_attempts_in_order() {
    let h = Handles::new();
    let mut minimum = rule("n", NUX_VALUE_RULE_NUMBER_MINIMUM, NUX_VALUE_RULE_MARK);
    minimum.number_bound = 10.0;
    let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
    maximum.number_bound = 365.0;
    assert_eq!(h.install(&[minimum, maximum]), NuxStatus::Ok);
    let other = NuxViewModelMutation {
        instance: h.value,
        path: view("b"),
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL,
        bool_value: 1,
        ..Default::default()
    };
    let result = h.mutate(&[
        h.write("n", 5.0),
        h.write("n", 400.0),
        other,
        h.write("n", 300.0),
    ]);
    let info = result_info(result);
    assert_eq!(
        (
            info.applied_count,
            info.change_count,
            info.rule_report_count
        ),
        (3, 3, 2)
    );
    let first = report(result, 0);
    let second = report(result, 1);
    assert_eq!(
        (
            first.rule_index,
            first.refused,
            first.attempted.number_value
        ),
        (0, 0, 5.0)
    );
    assert_eq!(
        (
            second.rule_index,
            second.refused,
            second.attempted.number_value
        ),
        (1, 1, 400.0)
    );
    assert_eq!(second.attempted.correlation_id, 73);
    assert_eq!(second.attempted.property_index, 0);
    for index in 0..info.change_count {
        let mut row = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_view_model_mutation_result_change(result, index, &mut row) },
            NuxStatus::Ok
        );
        assert_ne!(row.number_value, 400.0);
    }
    unsafe {
        nux_view_model_mutation_result_free(result);
    }
}
#[test]
fn marker_replacement_preserves_rules_and_refusal_does_not_set_marker() {
    let h = Handles::new();
    let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
    maximum.number_bound = 365.0;
    assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
    let marker = NuxValueMarker {
        model: view("Values"),
        value: view("n"),
        marker: view("n_set"),
    };
    assert_eq!(
        unsafe { nux_file_set_value_markers(h.file, &marker, 1) },
        NuxStatus::Ok
    );
    let marker_write = NuxViewModelMutation {
        instance: h.value,
        path: view("n_set"),
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL,
        bool_value: 1,
        ..Default::default()
    };
    let result = h.mutate(&[h.write("n", 400.0), marker_write]);
    let info = result_info(result);
    assert_eq!(
        (
            info.applied_count,
            info.change_count,
            info.rule_report_count
        ),
        (0, 0, 1)
    );
    unsafe {
        nux_view_model_mutation_result_free(result);
    }
    assert_eq!(
        unsafe { nux_file_set_value_rules(h.file, ptr::null(), 0) },
        NuxStatus::Ok
    );
    let result = h.mutate(&[h.write("n", 400.0)]);
    let info = result_info(result);
    assert_eq!(
        (
            info.applied_count,
            info.change_count,
            info.rule_report_count
        ),
        (1, 2, 0)
    );
    unsafe {
        nux_view_model_mutation_result_free(result);
    }
}
#[test]
fn invalid_install_is_atomic_and_invalid_pattern_is_rejected() {
    let h = Handles::new();
    let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
    maximum.number_bound = 3.0;
    assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
    let wrong = rule("missing", NUX_VALUE_RULE_REQUIRED, NUX_VALUE_RULE_MARK);
    assert_eq!(h.install(&[wrong]), NuxStatus::NotFound);
    assert_eq!(
        h.install(&[rule("b", NUX_VALUE_RULE_LENGTH, NUX_VALUE_RULE_MARK)]),
        NuxStatus::InvalidArgument
    );
    let result = h.mutate(&[h.write("n", 4.0)]);
    assert_eq!(result_info(result).rule_report_count, 1);
    unsafe {
        nux_view_model_mutation_result_free(result);
    }
    let mut pattern = rule("text", NUX_VALUE_RULE_PATTERN, NUX_VALUE_RULE_MARK);
    pattern.text = view("[");
    assert_eq!(h.install(&[pattern]), NuxStatus::InvalidArgument);
    let result = h.mutate(&[h.write("n", 4.0)]);
    assert_eq!(result_info(result).applied_count, 0);
    assert_eq!(result_info(result).rule_report_count, 1);
    unsafe { nux_view_model_mutation_result_free(result) };
}

struct Player {
    artboard: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
}
impl Player {
    fn new(h: &Handles) -> Self {
        Self::at(h, 0)
    }
    fn at(h: &Handles, index: usize) -> Self {
        let mut artboard = ptr::null_mut();
        let mut player = ptr::null_mut();
        assert_eq!(
            unsafe { nux_artboard_instance_new(h.file, index, &mut artboard) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_artboard_instance_bind_view_model(artboard, h.value) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_new_default(artboard, &mut player) },
            NuxStatus::Ok
        );
        Self { artboard, player }
    }
    fn step(&self, press: bool) -> (*mut NuxPlayerStepResult, NuxPlayerStepInfo) {
        self.step_at(press, 10.0)
    }
    fn step_at(&self, press: bool, x: f32) -> (*mut NuxPlayerStepResult, NuxPlayerStepInfo) {
        let pointer = NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x,
            y: 5.0,
            pointer_id: 1,
            timestamp_seconds: 0.0,
        };
        let step = NuxPlayerStep {
            pointers: if press { &pointer } else { ptr::null() },
            pointer_count: usize::from(press),
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(self.player, &step, &mut result) },
            NuxStatus::Ok
        );
        let mut info = NuxPlayerStepInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_info(result, &mut info) },
            NuxStatus::Ok
        );
        let mut scheduling = NuxPlayerSchedulingInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_scheduling(result, &mut scheduling) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_acknowledge_presented(self.player, scheduling.render_revision) },
            NuxStatus::Ok
        );
        (result, info)
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
fn handles_from_file(file: *mut NuxFile) -> Handles {
    let mut value = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut value) },
        NuxStatus::Ok
    );
    Handles { file, value }
}
fn assert_number(h: &Handles, expected: f32) {
    let mut snapshot = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_snapshot(h.value, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut value = NuxViewModelSnapshotValueView::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_value(snapshot, 0, &mut value) },
        NuxStatus::Ok
    );
    assert_eq!(value.number_value, expected);
    unsafe {
        nux_view_model_snapshot_free(snapshot);
    }
}
#[test]
fn listener_reports_refused_attempt_and_keeps_the_last_good_value() {
    for (actions, expected) in [
        (
            vec![
                fixture::Action::Number(300.0),
                fixture::Action::Number(400.0),
            ],
            300.0,
        ),
        (
            vec![
                fixture::Action::Number(400.0),
                fixture::Action::Number(300.0),
            ],
            300.0,
        ),
    ] {
        let bytes = fixture::fixture(None, &actions, false);
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
        let h = handles_from_file(file);
        let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
        maximum.number_bound = 365.0;
        assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
        let player = Player::new(&h);
        let (initial, info) = player.step(false);
        assert_eq!(info.rule_report_count, 0);
        unsafe {
            nux_player_step_result_free(initial);
        }
        let (result, info) = player.step(true);
        assert_eq!(info.rule_report_count, 1);
        let mut report = NuxValueRuleReportView::default();
        assert_eq!(
            unsafe { nux_player_step_result_rule_report(result, 0, &mut report) },
            NuxStatus::Ok
        );
        assert_eq!(
            (
                report.rule_index,
                report.refused,
                report.attempted.number_value
            ),
            (0, 1, 400.0)
        );
        assert_number(&h, expected);
        for index in 0..info.view_model_change_count {
            let mut row = NuxViewModelChangeView::default();
            assert_eq!(
                unsafe { nux_player_step_result_view_model_change(result, index, &mut row) },
                NuxStatus::Ok
            );
            assert_ne!(row.number_value, 400.0);
        }
        unsafe {
            nux_player_step_result_free(result);
        }
    }
}
#[cfg(feature = "scripting")]
#[test]
fn checked_script_call_is_synchronous_in_a_real_player_step() {
    let _flags = luaur_common::ScopedAllFlags::enter(true);
    let source = br#"local rules = require('value_rules')
        return function(context)
            return { init = function() return true end,
                performAction = function()
                    local ok, rule = rules.set('', 'n', 400)
                    assert(not ok and rule == 'authored-code')
                    assert(context:viewModel().n.value == 0)
                    local ok, rule = rules.set('', 'n', 300)
                    assert(ok and rule == nil)
                end }
        end"#;
    let mut size = 0;
    let code = luaur_compiler::functions::luau_compile::luau_compile(
        source.as_ptr().cast(),
        source.len(),
        ptr::null_mut(),
        &mut size,
    );
    assert!(!code.is_null());
    let mut payload = vec![0];
    payload.extend_from_slice(unsafe { std::slice::from_raw_parts(code.cast(), size) });
    let bytes = fixture::fixture(Some(&payload), &[fixture::Action::Script], false);
    let config = NuxHostCommandImportConfig {
        module_name: view("value_rules"),
        ..Default::default()
    };
    let mut file = ptr::null_mut();
    let mut import_result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_trusted_with_host_commands(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &config,
                &mut file,
                &mut import_result,
            )
        },
        NuxStatus::Ok
    );
    unsafe {
        nux_capi_result_free(import_result);
    }
    let h = handles_from_file(file);
    let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
    maximum.number_bound = 365.0;
    assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
    let player = Player::new(&h);
    let (initial, _) = player.step(false);
    unsafe {
        nux_player_step_result_free(initial);
    }
    let (result, info) = player.step(true);
    assert_eq!(info.rule_report_count, 1);
    assert_number(&h, 300.0);
    let mut report = NuxValueRuleReportView::default();
    assert_eq!(
        unsafe { nux_player_step_result_rule_report(result, 0, &mut report) },
        NuxStatus::Ok
    );
    assert_eq!((report.refused, report.attempted.number_value), (1, 400.0));
    unsafe {
        nux_player_step_result_free(result);
    }
}

#[test]
fn field_edit_is_checked_before_the_next_step_returns() {
    let bytes = fixture::fixture(None, &[], true);
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
    let h = handles_from_file(file);
    let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
    maximum.number_bound = 365.0;
    assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
    let player = Player::new(&h);
    assert_eq!(
        unsafe { nux_player_enable_semantics(player.player) },
        NuxStatus::Ok
    );
    let (initial, _) = player.step(false);
    unsafe {
        nux_player_step_result_free(initial);
    }
    for (text, expected_reports) in [("300", 0), ("400", 1)] {
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(player.player, &mut snapshot) },
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
                nux_player_field_string_set(
                    player.player,
                    snapshot,
                    node.id,
                    view("editable"),
                    view(text),
                )
            },
            NuxStatus::Ok
        );
        unsafe {
            nux_semantic_snapshot_free(snapshot);
        }
        let (result, info) = player.step(false);
        assert_eq!(info.rule_report_count, expected_reports);
        assert_number(&h, 300.0);
        if expected_reports != 0 {
            let mut report = NuxValueRuleReportView::default();
            assert_eq!(
                unsafe { nux_player_step_result_rule_report(result, 0, &mut report) },
                NuxStatus::Ok
            );
            assert_eq!((report.refused, report.attempted.number_value), (1, 400.0));
        }
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(player.player, &mut snapshot) },
            NuxStatus::Ok
        );
        let mut kept = [0u8; 16];
        let mut length = 0;
        assert_eq!(
            unsafe {
                nux_player_field_string_copy(
                    player.player,
                    snapshot,
                    node.id,
                    view("editable"),
                    kept.as_mut_ptr(),
                    kept.len(),
                    &mut length,
                )
            },
            NuxStatus::Ok
        );
        // Upstream DataConverterToString::convertNumber uses std::to_string
        // without rounding flags (data_converter_to_string.cpp:45-58).
        assert_eq!(&kept[..length], b"300.000000");
        unsafe {
            nux_semantic_snapshot_free(snapshot);
            nux_player_step_result_free(result);
        }
    }
}

#[test]
fn text_binding_rules_use_installed_modes_and_return_kept_text() {
    let options = [view("yes"), view("no")];
    let cases = [
        (
            NUX_VALUE_RULE_LENGTH,
            NUX_VALUE_RULE_REFUSE,
            "abc",
            "",
            "",
            0,
            2,
        ),
        (
            NUX_VALUE_RULE_LENGTH,
            NUX_VALUE_RULE_MARK,
            "a",
            "a",
            "",
            2,
            20,
        ),
        (
            NUX_VALUE_RULE_PATTERN,
            NUX_VALUE_RULE_MARK,
            "1",
            "1",
            "[a-z]+",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_URL,
            NUX_VALUE_RULE_MARK,
            "relative",
            "relative",
            "",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_DATE,
            NUX_VALUE_RULE_MARK,
            "2026-02-29",
            "2026-02-29",
            "",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_TEXT_MINIMUM,
            NUX_VALUE_RULE_MARK,
            "2025-12-31",
            "2025-12-31",
            "2026-01-01",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_TEXT_MAXIMUM,
            NUX_VALUE_RULE_REFUSE,
            "2027-01-01",
            "",
            "2026-12-31",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_ALLOWED_VALUES,
            NUX_VALUE_RULE_REFUSE,
            "maybe",
            "",
            "",
            0,
            0,
        ),
    ];
    for (kind, mode, attempted, kept, operand, minimum, maximum) in cases {
        let bytes = fixture::text_input_fixture();
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
        let h = handles_from_file(file);
        let mut entry = rule("text", kind, mode);
        entry.text = view(operand);
        if kind == NUX_VALUE_RULE_LENGTH {
            entry.minimum = minimum;
            entry.maximum = maximum;
            entry.bound_flags = NUX_VALUE_RULE_HAS_MINIMUM | NUX_VALUE_RULE_HAS_MAXIMUM;
        }
        if kind == NUX_VALUE_RULE_ALLOWED_VALUES {
            entry.values = options.as_ptr();
            entry.value_count = options.len();
        }
        assert_eq!(h.install(&[entry]), NuxStatus::Ok);
        let player = Player::new(&h);
        assert_eq!(
            unsafe { nux_player_enable_semantics(player.player) },
            NuxStatus::Ok
        );
        let (initial, info) = player.step(false);
        assert_eq!(info.rule_report_count, 0);
        unsafe {
            nux_player_step_result_free(initial);
        }
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(player.player, &mut snapshot) },
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
                nux_player_field_string_set(
                    player.player,
                    snapshot,
                    node.id,
                    view("editable"),
                    view(attempted),
                )
            },
            NuxStatus::Ok
        );
        unsafe {
            nux_semantic_snapshot_free(snapshot);
        }
        let (result, info) = player.step(false);
        assert_eq!(info.rule_report_count, 1, "rule {kind}");
        let mut report = NuxValueRuleReportView::default();
        assert_eq!(
            unsafe { nux_player_step_result_rule_report(result, 0, &mut report) },
            NuxStatus::Ok
        );
        assert_eq!(report.refused, u32::from(mode == NUX_VALUE_RULE_REFUSE));
        assert_eq!(
            unsafe {
                std::slice::from_raw_parts(
                    report.attempted.bytes_value.data,
                    report.attempted.bytes_value.len,
                )
            },
            attempted.as_bytes()
        );
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_semantic_snapshot(player.player, &mut snapshot) },
            NuxStatus::Ok
        );
        let mut buffer = [0u8; 32];
        let mut length = 0;
        assert_eq!(
            unsafe {
                nux_player_field_string_copy(
                    player.player,
                    snapshot,
                    node.id,
                    view("editable"),
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    &mut length,
                )
            },
            NuxStatus::Ok
        );
        assert_eq!(&buffer[..length], kept.as_bytes(), "rule {kind}");
        unsafe {
            nux_semantic_snapshot_free(snapshot);
            nux_player_step_result_free(result);
        }
    }
}

fn list_handles() -> Handles {
    let bytes = fixture::list_fixture();
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
    handles_from_file(file)
}
#[test]
fn enum_rules_resolve_authored_labels_and_refuse_an_unknown_index() {
    let h = list_handles();
    let values = [view("second")];
    let mut entry = rule(
        "choice",
        NUX_VALUE_RULE_ALLOWED_VALUES,
        NUX_VALUE_RULE_REFUSE,
    );
    entry.values = values.as_ptr();
    entry.value_count = values.len();
    assert_eq!(h.install(&[entry]), NuxStatus::Ok);
    let writes = [1, 0, 9].map(|value| NuxViewModelMutation {
        instance: h.value,
        path: view("choice"),
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM,
        integer_value: value,
        ..Default::default()
    });
    let result = h.mutate(&writes);
    assert_eq!(
        (
            result_info(result).applied_count,
            result_info(result).rule_report_count
        ),
        (1, 2)
    );
    assert_eq!(report(result, 0).attempted.integer_value, 0);
    assert_eq!(report(result, 1).attempted.integer_value, 9);
    unsafe {
        nux_view_model_mutation_result_free(result);
    }
}
#[test]
fn list_refusal_retains_attempted_member_identities_until_result_free() {
    let h = list_handles();
    let mut container = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(h.file, 1, 0, &mut container) },
        NuxStatus::Ok
    );
    let entry = NuxValueRule {
        model: view("Container"),
        property: view("rows"),
        kind: NUX_VALUE_RULE_ITEM_COUNT,
        mode: NUX_VALUE_RULE_REFUSE,
        bound_flags: NUX_VALUE_RULE_HAS_MAXIMUM,
        maximum: 1,
        ..Default::default()
    };
    assert_eq!(h.install(&[entry]), NuxStatus::Ok);
    let write = NuxViewModelMutation {
        instance: container,
        path: view("rows"),
        kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
        index: 1,
        related_instance: h.value,
        ..Default::default()
    };
    let result = h.mutate(&[write]);
    assert_eq!(
        (
            result_info(result).applied_count,
            result_info(result).change_count,
            result_info(result).rule_report_count
        ),
        (0, 0, 1)
    );
    assert_eq!(report(result, 0).attempted.list_item_count, 2);
    let mut identity = 0;
    assert_eq!(
        unsafe {
            nux_view_model_mutation_result_rule_report_list_item(result, 0, 1, &mut identity)
        },
        NuxStatus::Ok
    );
    let mut expected = 0;
    assert_eq!(
        unsafe { nux_view_model_instance_identity(h.value, &mut expected) },
        NuxStatus::Ok
    );
    assert_eq!(identity, expected);
    unsafe {
        nux_view_model_instance_free(container);
        nux_view_model_mutation_result_free(result);
    }
}

#[cfg(feature = "scripting")]
#[test]
fn native_copy_and_list_row_bindings_read_kept_values_in_the_same_step() {
    let _flags = luaur_common::ScopedAllFlags::enter(true);
    let source = br#"local bridge = require('bridge')
    return function(context)
        return { init = function() return true end,
            performAction = function()
                context:viewModel().n.value = 300
                context:viewModel().n.value = 400
                if context:viewModel().n.value > 365 then
                    bridge.command('observed-attempt', {})
                end
                context:rootViewModel().rows[1].n.value = 300
                context:rootViewModel().rows[1].n.value = 400
            end }
    end"#;
    let mut size = 0;
    let code = luaur_compiler::functions::luau_compile::luau_compile(
        source.as_ptr().cast(),
        source.len(),
        ptr::null_mut(),
        &mut size,
    );
    assert!(!code.is_null());
    let mut payload = vec![0];
    payload.extend_from_slice(unsafe { std::slice::from_raw_parts(code.cast(), size) });
    let bytes = fixture::occurrences_values(&payload);
    let config = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..Default::default()
    };
    let mut file = ptr::null_mut();
    let mut import_result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_trusted_with_host_commands(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &config,
                &mut file,
                &mut import_result,
            )
        },
        NuxStatus::Ok
    );
    unsafe {
        nux_capi_result_free(import_result);
    }
    let mut value = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 1, 0, &mut value) },
        NuxStatus::Ok
    );
    let h = Handles { file, value };
    let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
    maximum.number_bound = 365.0;
    assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
    let player = Player::at(&h, 1);
    assert_eq!(
        unsafe { nux_player_enable_semantics(player.player) },
        NuxStatus::Ok
    );
    let (initial, _) = player.step(false);
    unsafe {
        nux_player_step_result_free(initial);
    }
    let (result, info) = player.step_at(true, 105.0);
    assert_eq!(info.rule_report_count, 2);
    assert_eq!(info.host_command_count, 1);
    let mut owners = Vec::new();
    for index in 0..2 {
        let mut report = NuxValueRuleReportView::default();
        assert_eq!(
            unsafe { nux_player_step_result_rule_report(result, index, &mut report) },
            NuxStatus::Ok
        );
        assert_eq!((report.refused, report.attempted.number_value), (1, 400.0));
        owners.push(report.attempted.owner_instance_id);
    }
    assert_ne!(owners[0], owners[1]);
    let mut snapshot = ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_semantic_snapshot(player.player, &mut snapshot) },
        NuxStatus::Ok
    );
    let mut labels: Vec<Vec<u8>> = Vec::new();
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
                unsafe { std::slice::from_raw_parts(node.label.data.cast(), node.label.len) }
                    .to_vec(),
            );
        }
    }
    assert_eq!(labels, vec![b"300.000000".to_vec(), b"300.000000".to_vec()]);
    for index in 0..info.view_model_change_count {
        let mut row = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_player_step_result_view_model_change(result, index, &mut row) },
            NuxStatus::Ok
        );
        assert_ne!(row.number_value, 400.0);
    }
    unsafe {
        nux_semantic_snapshot_free(snapshot);
        nux_player_step_result_free(result);
    }
}

#[cfg(feature = "scripting")]
#[test]
fn text_rules_cover_host_listener_and_both_script_write_paths() {
    let _flags = luaur_common::ScopedAllFlags::enter(true);
    let options = [view("yes"), view("no")];
    // The expected kept values are the same literal contract used by the
    // text-input test, independently of which native writer produces them.
    for (kind, mode, attempted, kept, operand, minimum, maximum) in [
        (
            NUX_VALUE_RULE_LENGTH,
            NUX_VALUE_RULE_REFUSE,
            "abc",
            "",
            "",
            0,
            2,
        ),
        (
            NUX_VALUE_RULE_LENGTH,
            NUX_VALUE_RULE_MARK,
            "a",
            "a",
            "",
            2,
            20,
        ),
        (
            NUX_VALUE_RULE_PATTERN,
            NUX_VALUE_RULE_MARK,
            "1",
            "1",
            "[a-z]+",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_URL,
            NUX_VALUE_RULE_MARK,
            "relative",
            "relative",
            "",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_DATE,
            NUX_VALUE_RULE_MARK,
            "2026-02-29",
            "2026-02-29",
            "",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_TEXT_MINIMUM,
            NUX_VALUE_RULE_MARK,
            "2025-12-31",
            "2025-12-31",
            "2026-01-01",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_TEXT_MAXIMUM,
            NUX_VALUE_RULE_REFUSE,
            "2027-01-01",
            "",
            "2026-12-31",
            0,
            0,
        ),
        (
            NUX_VALUE_RULE_ALLOWED_VALUES,
            NUX_VALUE_RULE_REFUSE,
            "maybe",
            "",
            "",
            0,
            0,
        ),
    ] {
        for writer in ["host", "listener", "script", "checked-script"] {
            let action = if writer == "checked-script" {
                format!(
                    "local ok, code = require('commands').set('', 'text', '{attempted}'); assert(ok == {} and code == {})",
                    mode == NUX_VALUE_RULE_MARK,
                    if mode == NUX_VALUE_RULE_MARK {
                        "nil"
                    } else {
                        "'authored-code'"
                    }
                )
            } else {
                format!("context:viewModel().text.value = '{attempted}'")
            };
            let source = format!(
                "return function(context) return {{ init = function() return true end, performAction = function() {action} end }} end"
            );
            let mut size = 0;
            let code = luaur_compiler::functions::luau_compile::luau_compile(
                source.as_ptr().cast(),
                source.len(),
                ptr::null_mut(),
                &mut size,
            );
            assert!(!code.is_null());
            let mut payload = vec![0];
            payload.extend_from_slice(unsafe { std::slice::from_raw_parts(code.cast(), size) });
            let actions = match writer {
                "host" => vec![],
                "listener" => vec![fixture::Action::Text(attempted)],
                _ => vec![fixture::Action::Script],
            };
            let bytes = fixture::fixture(Some(&payload), &actions, false);
            let mut file = ptr::null_mut();
            let mut import_result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_file_import_trusted_with_host_commands(
                        bytes.as_ptr(),
                        bytes.len(),
                        &NuxRenderCallbacks::default(),
                        &NuxHostCommandImportConfig {
                            module_name: view("commands"),
                            ..Default::default()
                        },
                        &mut file,
                        &mut import_result,
                    )
                },
                NuxStatus::Ok
            );
            unsafe {
                nux_capi_result_free(import_result);
            }
            let h = handles_from_file(file);
            let mut entry = rule("text", kind, mode);
            entry.text = view(operand);
            if kind == NUX_VALUE_RULE_LENGTH {
                entry.minimum = minimum;
                entry.maximum = maximum;
                entry.bound_flags = NUX_VALUE_RULE_HAS_MINIMUM | NUX_VALUE_RULE_HAS_MAXIMUM;
            }
            if kind == NUX_VALUE_RULE_ALLOWED_VALUES {
                entry.values = options.as_ptr();
                entry.value_count = options.len();
            }
            assert_eq!(h.install(&[entry]), NuxStatus::Ok);
            let player = Player::new(&h);
            let (initial, info) = player.step(false);
            assert_eq!(info.rule_report_count, 0);
            unsafe {
                nux_player_step_result_free(initial);
            }
            let observed = if writer == "host" {
                let result = h.mutate(&[NuxViewModelMutation {
                    instance: h.value,
                    path: view("text"),
                    kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
                    bytes_value: NuxByteView {
                        data: attempted.as_ptr(),
                        len: attempted.len(),
                    },
                    ..Default::default()
                }]);
                assert_eq!(result_info(result).rule_report_count, 1, "{writer}: {kind}");
                let observed = report(result, 0).refused;
                unsafe {
                    nux_view_model_mutation_result_free(result);
                }
                observed
            } else {
                let (result, info) = player.step(true);
                assert_eq!(info.rule_report_count, 1, "{writer}: {kind}");
                let mut report = NuxValueRuleReportView::default();
                assert_eq!(
                    unsafe { nux_player_step_result_rule_report(result, 0, &mut report) },
                    NuxStatus::Ok
                );
                let observed = report.refused;
                unsafe {
                    nux_player_step_result_free(result);
                }
                observed
            };
            assert_eq!(
                observed,
                u32::from(mode == NUX_VALUE_RULE_REFUSE),
                "{writer}: {kind}"
            );
            let mut snapshot = ptr::null_mut();
            assert_eq!(
                unsafe { nux_view_model_instance_snapshot(h.value, &mut snapshot) },
                NuxStatus::Ok
            );
            let mut value = NuxViewModelSnapshotValueView::default();
            assert_eq!(
                unsafe { nux_view_model_snapshot_value(snapshot, 6, &mut value) },
                NuxStatus::Ok
            );
            let observed = if value.bytes_value.len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(value.bytes_value.data, value.bytes_value.len) }
            };
            assert_eq!(observed, kept.as_bytes(), "{writer}: {kind}");
            unsafe {
                nux_view_model_snapshot_free(snapshot);
            }
        }
    }
}

#[test]
fn installed_minimum_mode_controls_host_and_listener_writes() {
    for mode in [NUX_VALUE_RULE_MARK, NUX_VALUE_RULE_REFUSE] {
        for writer in ["host", "listener"] {
            let bytes = fixture::fixture(None, &[fixture::Action::Number(0.0)], false);
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
            let h = handles_from_file(file);
            let mut entry = rule("n", NUX_VALUE_RULE_NUMBER_MINIMUM, mode);
            entry.number_bound = 1.0;
            assert_eq!(h.install(&[entry]), NuxStatus::Ok);
            let player = Player::new(&h);
            let (initial, info) = player.step(false);
            assert_eq!(info.rule_report_count, 0);
            unsafe {
                nux_player_step_result_free(initial);
            }
            let prime = h.mutate(&[h.write("n", 1.0)]);
            assert_eq!(result_info(prime).rule_report_count, 0);
            unsafe {
                nux_view_model_mutation_result_free(prime);
            }
            let refused = if writer == "host" {
                let result = h.mutate(&[h.write("n", 0.0)]);
                assert_eq!(result_info(result).rule_report_count, 1);
                assert_eq!(
                    result_info(result).applied_count,
                    usize::from(mode == NUX_VALUE_RULE_MARK)
                );
                let refused = report(result, 0).refused;
                unsafe {
                    nux_view_model_mutation_result_free(result);
                }
                refused
            } else {
                let (result, info) = player.step(true);
                assert_eq!(info.rule_report_count, 1);
                let mut report = NuxValueRuleReportView::default();
                assert_eq!(
                    unsafe { nux_player_step_result_rule_report(result, 0, &mut report) },
                    NuxStatus::Ok
                );
                let refused = report.refused;
                unsafe {
                    nux_player_step_result_free(result);
                }
                refused
            };
            // One step below the installed minimum stays at one in refuse
            // mode and lands at zero in mark mode. The installer decides.
            assert_eq!(refused, u32::from(mode == NUX_VALUE_RULE_REFUSE));
            assert_number(
                &h,
                if mode == NUX_VALUE_RULE_REFUSE {
                    1.0
                } else {
                    0.0
                },
            );
        }
    }
}

#[test]
fn invalid_pattern_install_returns_owned_rule_code_and_preserves_the_table() {
    for replace in [false, true] {
        let h = Handles::new();
        let mut maximum = rule("n", NUX_VALUE_RULE_NUMBER_MAXIMUM, NUX_VALUE_RULE_REFUSE);
        maximum.number_bound = 3.0;
        if replace {
            assert_eq!(h.install(&[maximum]), NuxStatus::Ok);
        }
        let code = String::from("authored-pattern-code");
        let mut pattern = rule("text", NUX_VALUE_RULE_PATTERN, NUX_VALUE_RULE_MARK);
        pattern.text = view("[");
        pattern.code = view(&code);
        let rules = [
            rule("text", NUX_VALUE_RULE_REQUIRED, NUX_VALUE_RULE_MARK),
            pattern,
        ];
        let mut diagnostic = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_set_value_rules_with_result(
                    h.file,
                    rules.as_ptr(),
                    rules.len(),
                    &mut diagnostic,
                )
            },
            NuxStatus::InvalidArgument
        );
        drop(code);
        let mut details = NuxCapiDiagnosticView::default();
        assert_eq!(
            unsafe { nux_capi_result_diagnostic(diagnostic, &mut details) },
            NuxStatus::Ok
        );
        assert_eq!(details.status, NuxStatus::InvalidArgument);
        assert_eq!(
            unsafe { std::slice::from_raw_parts(details.code.data.cast::<u8>(), details.code.len) },
            b"authored-pattern-code"
        );
        unsafe { nux_capi_result_free(diagnostic) };
        let result = h.mutate(&[h.write("n", 4.0)]);
        assert_eq!(result_info(result).applied_count, usize::from(!replace));
        assert_eq!(result_info(result).rule_report_count, usize::from(replace));
        unsafe { nux_view_model_mutation_result_free(result) };
        assert_eq!(
            unsafe {
                nux_file_set_value_rules_with_result(h.file, ptr::null(), 0, &mut diagnostic)
            },
            NuxStatus::Ok
        );
        assert!(diagnostic.is_null());
        assert_eq!(
            unsafe {
                nux_file_set_value_rules_with_result(
                    ptr::null_mut(),
                    ptr::null(),
                    0,
                    &mut diagnostic,
                )
            },
            NuxStatus::NullArgument
        );
        assert!(diagnostic.is_null());
    }
}

#[cfg(feature = "scripting")]
#[test]
fn file_value_rules_module_runs_without_host_tables() {
    let _flags = luaur_common::ScopedAllFlags::enter(true);
    fn compile(source: &[u8]) -> Vec<u8> {
        let mut size = 0;
        let code = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            ptr::null_mut(),
            &mut size,
        );
        assert!(!code.is_null());
        let mut payload = vec![0];
        payload.extend_from_slice(unsafe { std::slice::from_raw_parts(code.cast(), size) });
        payload
    }
    let module = compile(b"return { authored = function() return 37 end }");
    let action = compile(
        br#"return function(context)
            return { init = function() return true end,
                performAction = function()
                    context:viewModel().n.value = require('value_rules').authored()
                end }
        end"#,
    );
    let bytes = fixture::fixture_with_value_rules_module(&action, &module);
    let config = NuxHostCommandImportConfig {
        module_name: view("bridge"),
        ..Default::default()
    };
    let mut file = ptr::null_mut();
    let mut import_result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import_trusted_with_host_commands(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &config,
                &mut file,
                &mut import_result,
            )
        },
        NuxStatus::Ok
    );
    unsafe {
        nux_capi_result_free(import_result);
    }
    // No marker, rule, group or global table is installed.
    let handles = handles_from_file(file);
    let player = Player::new(&handles);
    let (initial, _) = player.step(false);
    unsafe {
        nux_player_step_result_free(initial);
    }
    let (result, info) = player.step(true);
    unsafe {
        nux_player_step_result_free(result);
    }
    assert_eq!(info.rule_report_count, 0);
    assert_number(&handles, 37.0);
}
