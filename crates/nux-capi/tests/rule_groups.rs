#![allow(clippy::unwrap_used, reason = "handwritten C ABI assertions")]
use nux_capi::*;
#[path = "support/rule_groups.rs"]
mod fixture;
use std::ptr;
fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}
struct Handles {
    file: *mut NuxFile,
    first: *mut NuxViewModelInstance,
    second: *mut NuxViewModelInstance,
    artboard: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
}
impl Handles {
    fn new() -> Self {
        let b = fixture::fixture();
        let mut file = ptr::null_mut();
        let mut first = ptr::null_mut();
        let mut second = ptr::null_mut();
        let mut artboard = ptr::null_mut();
        let mut player = ptr::null_mut();
        unsafe {
            assert_eq!(
                nux_file_import(
                    b.as_ptr(),
                    b.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_instance_new(file, 0, &mut first),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_instance_new(file, 1, &mut second),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(artboard, first),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        }
        Self {
            file,
            first,
            second,
            artboard,
            player,
        }
    }
    fn install(&self, reverse: bool) {
        let mut markers = Vec::new();
        let mut rules = Vec::new();
        for model in ["First", "Second"] {
            for (value, marker) in [
                ("trip_days", "trip_days_set"),
                ("italian_level", "italian_level_set"),
                ("wants_reminder", "wants_reminder_set"),
            ] {
                markers.push(NuxValueMarker {
                    model: view(model),
                    value: view(value),
                    marker: view(marker),
                });
                rules.push(NuxValueRule {
                    model: view(model),
                    property: view(value),
                    kind: NUX_VALUE_RULE_REQUIRED,
                    code: view("required"),
                    message: view("Answer required"),
                    ..Default::default()
                });
            }
            rules.push(NuxValueRule {
                model: view(model),
                property: view("trip_days"),
                kind: NUX_VALUE_RULE_NUMBER_MINIMUM,
                number_bound: 1.0,
                code: view("min"),
                message: view("At least one"),
                ..Default::default()
            });
            rules.push(NuxValueRule {
                model: view(model),
                property: view("trip_days"),
                kind: NUX_VALUE_RULE_NUMBER_MAXIMUM,
                number_bound: 365.0,
                mode: NUX_VALUE_RULE_REFUSE,
                code: view("max"),
                message: view("At most 365"),
                ..Default::default()
            });
            let max = NuxValueRule {
                model: view(model),
                property: view("email"),
                kind: NUX_VALUE_RULE_LENGTH,
                bound_flags: NUX_VALUE_RULE_HAS_MAXIMUM,
                maximum: 2,
                mode: NUX_VALUE_RULE_REFUSE,
                code: view("length"),
                message: view("At most two"),
                ..Default::default()
            };
            let pattern = NuxValueRule {
                model: view(model),
                property: view("email"),
                kind: NUX_VALUE_RULE_PATTERN,
                text: view("[0-9]+"),
                code: view("pattern"),
                message: view("Digits only"),
                ..Default::default()
            };
            rules.extend(if reverse {
                [pattern, max]
            } else {
                [max, pattern]
            });
        }
        unsafe {
            assert_eq!(
                nux_file_set_value_markers(self.file, markers.as_ptr(), markers.len()),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_file_set_value_rules(self.file, rules.as_ptr(), rules.len()),
                NuxStatus::Ok
            );
        }
        let paths = [
            "errors/trip_days",
            "errors/italian_level",
            "errors/wants_reminder",
            "errors/email",
        ];
        let members = ["trip_days", "italian_level", "wants_reminder", "email"]
            .iter()
            .zip(paths)
            .map(|(property, path)| NuxRuleGroupMember {
                property: view(property),
                errors_path: view(path),
                item_model: view("ErrorEntry"),
                code_property: view("rule"),
                message_property: view("message"),
            })
            .collect::<Vec<_>>();
        let groups = ["First", "Second"].map(|model| NuxRuleGroup {
            model: view(model),
            valid: view("valid"),
            members: members.as_ptr(),
            member_count: members.len(),
        });
        unsafe {
            assert_eq!(
                nux_file_set_rule_groups(self.file, groups.as_ptr(), groups.len()),
                NuxStatus::Ok
            );
        }
    }
    fn step(&self) -> NuxPlayerStepInfo {
        self.pointer_step(false)
    }
    fn pointer_step(&self, press: bool) -> NuxPlayerStepInfo {
        let mut result = ptr::null_mut();
        let mut info = NuxPlayerStepInfo::default();
        let pointer = NuxPlayerPointerEvent {
            kind: NUX_PLAYER_POINTER_KIND_DOWN,
            x: 10.0,
            y: 5.0,
            pointer_id: 1,
            timestamp_seconds: 0.0,
        };
        let step = NuxPlayerStep {
            pointers: if press { &pointer } else { ptr::null() },
            pointer_count: usize::from(press),
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_player_step(self.player, &step, &mut result),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_step_result_info(result, &mut info),
                NuxStatus::Ok
            );
            nux_player_step_result_free(result);
        }
        info
    }
    fn mutate(&self, writes: &[NuxViewModelMutation]) -> NuxViewModelMutationResultInfo {
        let mut result = ptr::null_mut();
        let mut info = NuxViewModelMutationResultInfo::default();
        unsafe {
            assert_eq!(
                nux_view_model_mutate(
                    &NuxViewModelMutationBatch {
                        mutations: writes.as_ptr(),
                        mutation_count: writes.len(),
                        ..Default::default()
                    },
                    &mut result
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_mutation_result_info(result, &mut info),
                NuxStatus::Ok
            );
            nux_view_model_mutation_result_free(result);
        }
        info
    }
    fn number(&self, value: f32) -> NuxViewModelMutation {
        NuxViewModelMutation {
            instance: self.first,
            path: view("trip_days"),
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            number_value: value,
            ..Default::default()
        }
    }
    fn boolean(&self, path: &str, value: bool) -> NuxViewModelMutation {
        NuxViewModelMutation {
            instance: self.first,
            path: view(path),
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL,
            bool_value: u32::from(value),
            ..Default::default()
        }
    }
    fn text(&self, value: &str) -> NuxViewModelMutation {
        NuxViewModelMutation {
            instance: self.first,
            path: view("email"),
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
            bytes_value: NuxByteView {
                data: value.as_ptr(),
                len: value.len(),
            },
            ..Default::default()
        }
    }
}
impl Drop for Handles {
    fn drop(&mut self) {
        unsafe {
            nux_player_free(self.player);
            nux_artboard_instance_free(self.artboard);
            nux_view_model_instance_free(self.first);
            nux_view_model_instance_free(self.second);
            nux_file_free(self.file);
        }
    }
}
struct Snapshot {
    handle: *mut NuxViewModelSnapshot,
    root: u64,
    values: Vec<NuxViewModelSnapshotValueView>,
}
impl Snapshot {
    fn new(root: *mut NuxViewModelInstance) -> Self {
        let mut handle = ptr::null_mut();
        let mut identity = 0;
        let mut info = NuxViewModelSnapshotInfo::default();
        unsafe {
            assert_eq!(
                nux_view_model_instance_identity(root, &mut identity),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_instance_snapshot(root, &mut handle),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_snapshot_info(handle, &mut info),
                NuxStatus::Ok
            );
        }
        let values = (0..info.value_count)
            .map(|index| {
                let mut value = NuxViewModelSnapshotValueView::default();
                unsafe {
                    assert_eq!(
                        nux_view_model_snapshot_value(handle, index, &mut value),
                        NuxStatus::Ok
                    );
                }
                value
            })
            .collect();
        Self {
            handle,
            root: identity,
            values,
        }
    }
    fn get(&self, owner: u64, name: &str) -> &NuxViewModelSnapshotValueView {
        self.values
            .iter()
            .find(|value| {
                value.owner_instance_id == owner
                    && unsafe {
                        std::slice::from_raw_parts(value.name.data.cast::<u8>(), value.name.len)
                    } == name.as_bytes()
            })
            .unwrap()
    }
    fn bytes(&self, owner: u64, name: &str) -> String {
        let view = self.get(owner, name).bytes_value;
        if view.len == 0 {
            return String::new();
        }
        String::from_utf8(unsafe { std::slice::from_raw_parts(view.data, view.len) }.to_vec())
            .unwrap()
    }
    fn errors(&self, field: &str) -> Vec<(String, String)> {
        let owner = self.get(self.root, "errors").referenced_instance_id;
        let list = self.get(owner, field);
        (list.first_list_item..list.first_list_item + list.list_item_count)
            .map(|index| {
                let mut item = 0;
                unsafe {
                    assert_eq!(
                        nux_view_model_snapshot_list_item(self.handle, index, &mut item),
                        NuxStatus::Ok
                    );
                }
                (self.bytes(item, "rule"), self.bytes(item, "message"))
            })
            .collect()
    }
    fn valid(&self) -> bool {
        self.get(self.root, "valid").bool_value != 0
    }
}
impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe {
            nux_view_model_snapshot_free(self.handle);
        }
    }
}
#[test]
fn computed_nested_lists_match_kept_values_and_declared_order() {
    for reverse in [false, true] {
        let h = Handles::new();
        h.install(reverse);
        assert_eq!(
            h.step().rule_report_count,
            0,
            "initial failures compute state without write reports"
        );
        let s = Snapshot::new(h.first);
        assert!(!s.valid());
        assert_eq!(
            s.errors("trip_days"),
            vec![("required".into(), "Answer required".into())]
        );
        drop(s);
        h.mutate(&[
            h.number(365.0),
            NuxViewModelMutation {
                instance: h.first,
                path: view("italian_level"),
                kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM,
                integer_value: 1,
                ..Default::default()
            },
            h.boolean("wants_reminder", false),
            h.boolean("saving", true),
            h.boolean("saved", true),
            NuxViewModelMutation {
                path: view("saveError"),
                ..h.text("offline")
            },
        ]);
        assert!(
            Snapshot::new(h.first).valid(),
            "false with marker on is answered"
        );
        h.mutate(&[h.number(366.0)]);
        let s = Snapshot::new(h.first);
        assert!(s.valid());
        assert_eq!(s.get(s.root, "trip_days").number_value, 365.0);
        assert_eq!(
            s.errors("trip_days"),
            vec![("max".into(), "At most 365".into())]
        );
        drop(s);
        h.step();
        assert_eq!(Snapshot::new(h.first).errors("trip_days").len(), 1);
        h.mutate(&[h.number(365.0)]);
        assert!(Snapshot::new(h.first).errors("trip_days").is_empty());
        h.mutate(&[h.text("a")]);
        let s = Snapshot::new(h.first);
        assert_eq!(s.bytes(s.root, "email"), "a");
        assert_eq!(
            s.errors("email"),
            vec![("pattern".into(), "Digits only".into())]
        );
        assert!(!s.valid());
        drop(s);
        h.mutate(&[h.text("ab"), h.text("abc")]);
        let s = Snapshot::new(h.first);
        assert!(!s.valid());
        let errors = s.errors("email");
        assert_eq!(
            errors,
            if reverse {
                vec![
                    ("pattern".into(), "Digits only".into()),
                    ("length".into(), "At most two".into()),
                ]
            } else {
                vec![
                    ("length".into(), "At most two".into()),
                    ("pattern".into(), "Digits only".into()),
                ]
            }
        );
        drop(s);
        let listener = h.pointer_step(true);
        assert!(listener.pointer_result_count > 0);
        assert!(
            listener.view_model_change_count >= 2,
            "listener true and computed false are both journaled"
        );
        assert!(
            !Snapshot::new(h.first).valid(),
            "listener's true is corrected before step returns"
        );
        h.mutate(&[h.boolean("valid", true)]);
        assert!(!Snapshot::new(h.first).valid());
        h.mutate(&[NuxViewModelMutation {
            instance: h.second,
            ..h.number(2.0)
        }]);
        assert_eq!(Snapshot::new(h.first).errors("email"), errors);
        h.mutate(&[h.number(0.0), h.boolean("trip_days_set", false)]);
        let s = Snapshot::new(h.first);
        assert_eq!(
            s.errors("trip_days"),
            vec![("required".into(), "Answer required".into())]
        );
        assert_eq!(s.get(s.root, "saving").bool_value, 1);
        assert_eq!(s.get(s.root, "saved").bool_value, 1);
        assert_eq!(s.bytes(s.root, "saveError"), "offline");
        drop(s);
        let quiet = h.step();
        assert_eq!(quiet.view_model_change_count, 0);
        unsafe {
            assert_eq!(
                nux_file_set_rule_groups(h.file, ptr::null(), 1),
                NuxStatus::NullArgument
            );
            assert_eq!(
                nux_file_set_rule_groups(h.file, ptr::null(), 4097),
                NuxStatus::LimitExceeded
            );
        }
        assert_eq!(h.step().view_model_change_count, 0);
        let wrong_target = NuxRuleGroupMember {
            property: view("email"),
            errors_path: view("email"),
            item_model: view("ErrorEntry"),
            code_property: view("rule"),
            message_property: view("message"),
        };
        let bad_group = NuxRuleGroup {
            model: view("First"),
            valid: view("valid"),
            members: &wrong_target,
            member_count: 1,
        };
        unsafe {
            assert_eq!(
                nux_file_set_rule_groups(h.file, &bad_group, 1),
                NuxStatus::InvalidArgument
            );
        }
        assert_eq!(
            h.step().view_model_change_count,
            0,
            "bad whole-table replacement keeps previous groups"
        );
    }
}

#[test]
fn listener_initial_value_pair_retains_latest_refusal_and_settles() {
    let h = Handles::new();
    h.install(false);
    h.step();
    let info = h.pointer_step(true);
    assert_eq!(info.rule_report_count, 1);
    let state = Snapshot::new(h.first);
    assert_eq!(state.get(state.root, "trip_days").number_value, 300.0);
    assert_eq!(state.get(state.root, "trip_days_set").bool_value, 1);
    assert_eq!(
        state.errors("trip_days"),
        vec![("max".into(), "At most 365".into())]
    );
    assert!(!state.valid(), "the other required fields are still empty");
    drop(state);
    assert_eq!(h.step().view_model_change_count, 0);
}
#[test]
fn empty_group_is_valid_and_removal_stops_computation() {
    let h = Handles::new();
    let group = NuxRuleGroup {
        model: view("First"),
        valid: view("valid"),
        members: ptr::null(),
        member_count: 0,
    };
    // Every rule holds when the group has no members. No rule/marker table
    // is needed, and the first step must initialize the computed boolean.
    assert_eq!(
        unsafe { nux_file_set_rule_groups(h.file, &group, 1) },
        NuxStatus::Ok
    );
    h.step();
    assert!(Snapshot::new(h.first).valid());
    assert_eq!(h.step().view_model_change_count, 0);
    h.mutate(&[h.boolean("valid", false)]);
    assert!(Snapshot::new(h.first).valid());
    assert_eq!(
        unsafe { nux_file_set_rule_groups(h.file, ptr::null(), 0) },
        NuxStatus::Ok
    );
    h.mutate(&[h.boolean("valid", false)]);
    assert!(!Snapshot::new(h.first).valid());
}
