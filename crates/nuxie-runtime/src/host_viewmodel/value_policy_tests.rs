#![allow(
    clippy::unwrap_used,
    clippy::arithmetic_side_effects,
    reason = "bounded fixture assertions"
)]
use crate::{
    File, RuntimeFactoryHandle, RuntimeFileHandle, RuntimeOwnedViewModelGraphTransaction,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance, RuntimeValueMarker,
    RuntimeValuePolicy, RuntimeValuePolicyError, RuntimeValueRule, RuntimeValueRuleKind,
    RuntimeValueRuleMode, RuntimeViewModelChangeCapture, RuntimeViewModelChangeValue,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
#[path = "../../tests/support/value_policy_fixture.rs"]
mod fixture;
use fixture::fixture;

fn setup() -> (
    RuntimeValuePolicy,
    RuntimeOwnedViewModelHandle,
    RuntimeFileHandle,
    PersistentFactory<RecordingFactory>,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&fixture(), retained, None, None, None).unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    (RuntimeValuePolicy::new(file.clone()), root, file, factory)
}
fn pair(value: &str, marker: &str) -> RuntimeValueMarker {
    RuntimeValueMarker {
        model: "Values".into(),
        value: value.into(),
        marker: marker.into(),
    }
}
fn run(
    policy: &RuntimeValuePolicy,
    root: &RuntimeOwnedViewModelHandle,
    action: impl FnOnce(),
) -> Vec<crate::RuntimeViewModelChange> {
    let roots = std::slice::from_ref(root);
    let transaction = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    action();
    policy
        .apply_markers(&capture, roots, |owner, index, value| {
            Ok(owner
                .borrow_mut()
                .set_boolean_by_property_index(index, value))
        })
        .unwrap();
    let changes = root.resolve_change_capture(capture).unwrap();
    transaction.commit();
    changes
}
#[test]
fn value_write_then_marker_and_explicit_order() {
    let (mut policy, root, _, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    let changes = run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 5.0);
    });
    assert_eq!(
        changes
            .iter()
            .map(|c| (c.property_index, c.value.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0, RuntimeViewModelChangeValue::Number(5.0)),
            (1, RuntimeViewModelChangeValue::Boolean(true))
        ]
    );
    run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 0.0);
        root.borrow_mut()
            .set_boolean_by_property_name("n_set", false);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
    run(&policy, &root, || {
        root.borrow_mut()
            .set_boolean_by_property_name("n_set", true);
        root.borrow_mut()
            .set_boolean_by_property_name("n_set", false);
        root.borrow_mut().set_number_by_property_name("n", 4.0);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
}
#[test]
fn replacement_is_atomic_and_empty_removes_markers() {
    let (mut policy, root, _, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    for (entries, error) in [
        (
            vec![pair("missing", "n_set")],
            RuntimeValuePolicyError::NotFound,
        ),
        (
            vec![pair("text", "n_set")],
            RuntimeValuePolicyError::InvalidArgument,
        ),
        (
            vec![pair("n", "b"), pair("b", "b_set")],
            RuntimeValuePolicyError::InvalidArgument,
        ),
        (
            vec![pair("n", "n_set"), pair("n", "b_set")],
            RuntimeValuePolicyError::InvalidArgument,
        ),
        (
            vec![pair("n", "n_set"), pair("b", "n_set")],
            RuntimeValuePolicyError::InvalidArgument,
        ),
        (
            vec![pair("b", "b")],
            RuntimeValuePolicyError::InvalidArgument,
        ),
    ] {
        assert_eq!(policy.set_markers(&entries), Err(error));
    }
    run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 1.0);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
    policy.set_markers(&[]).unwrap();
    root.borrow_mut()
        .set_boolean_by_property_name("n_set", false);
    let changes = run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 2.0);
    });
    assert_eq!(changes.len(), 1);
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
}
#[test]
fn marker_writes_share_journal_limit_and_checkpoint() {
    let (mut policy, root, _, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    let roots = std::slice::from_ref(&root);
    let transaction = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin_bounded(1, 1024).unwrap();
    root.borrow_mut().set_number_by_property_name("n", 5.0);
    assert_eq!(
        policy.apply_markers(&capture, roots, |owner, index, value| Ok(owner
            .borrow_mut()
            .set_boolean_by_property_index(index, value))),
        Err(RuntimeValuePolicyError::LimitExceeded)
    );
    drop(capture);
    drop(transaction);
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
}
#[test]
fn every_instance_of_the_file_uses_the_table() {
    let (mut policy, first, file, _factory) = setup();
    let second =
        RuntimeOwnedViewModelHandle::new(RuntimeOwnedViewModelInstance::new(file, 0).unwrap());
    policy
        .set_markers(&[
            pair("b", "b_set"),
            pair("c", "c_set"),
            pair("choice", "choice_set"),
        ])
        .unwrap();
    for root in [&first, &second] {
        run(&policy, root, || {
            root.borrow_mut().set_boolean_by_property_name("b", true);
            root.borrow_mut()
                .set_color_by_property_name("c", 0x11223344);
            root.borrow_mut().set_enum_by_property_name("choice", 1);
        });
        assert_eq!(
            root.borrow().boolean_value_by_property_name("b_set"),
            Some(true)
        );
        assert_eq!(
            root.borrow().boolean_value_by_property_name("c_set"),
            Some(true)
        );
        assert_eq!(
            root.borrow().boolean_value_by_property_name("choice_set"),
            Some(true)
        );
    }
}

#[test]
fn unchanged_host_values_count_but_do_not_publish_duplicate_value_rows() {
    let (mut policy, root, _, _factory) = setup();
    policy
        .set_markers(&[
            pair("n", "n_set"),
            pair("b", "b_set"),
            pair("c", "c_set"),
            pair("choice", "choice_set"),
        ])
        .unwrap();
    let color = root.borrow().color_value_by_property_name("c").unwrap();
    let changes = run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 0.0);
        root.borrow_mut().set_boolean_by_property_name("b", false);
        root.borrow_mut().set_color_by_property_name("c", color);
        root.borrow_mut().set_enum_by_property_name("choice", 0);
    });
    assert_eq!(
        changes
            .iter()
            .map(|change| change.property_index)
            .collect::<Vec<_>>(),
        vec![1, 3, 5, 8]
    );
    for name in ["n_set", "b_set", "c_set", "choice_set"] {
        assert_eq!(
            root.borrow().boolean_value_by_property_name(name),
            Some(true)
        );
    }
}
#[test]
fn unchanged_explicit_clear_marker_wins_only_after_the_last_value_write() {
    let (mut policy, root, _, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 0.0);
        root.borrow_mut()
            .set_boolean_by_property_name("n_set", false);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
    run(&policy, &root, || {
        root.borrow_mut()
            .set_boolean_by_property_name("n_set", false);
        root.borrow_mut().set_number_by_property_name("n", 0.0);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
}

#[test]
fn unchanged_script_adapter_writes_set_markers_in_operation_order() {
    let (mut policy, root, file, _factory) = setup();
    policy
        .set_markers(&[
            pair("n", "n_set"),
            pair("b", "b_set"),
            pair("c", "c_set"),
            pair("choice", "choice_set"),
        ])
        .unwrap();
    let script =
        crate::scripting::ScriptViewModel::from_native(root.native_handle(), file).unwrap();
    let color = root.borrow().color_value_by_property_name("c").unwrap();
    let changes = run(&policy, &root, || {
        assert!(!script.set_number("n", 0.0));
        assert!(!script.set_boolean("b", false));
        assert!(!script.set_color("c", color));
        assert!(!script.set_enum_value("choice", "first"));
    });
    assert_eq!(
        changes
            .iter()
            .map(|change| change.property_index)
            .collect::<Vec<_>>(),
        vec![1, 3, 5, 8]
    );
    root.borrow_mut()
        .set_boolean_by_property_name("n_set", false);
    run(&policy, &root, || {
        script.set_number("n", 0.0);
        script.set_boolean("n_set", false);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
    run(&policy, &root, || {
        script.set_boolean("n_set", false);
        script.set_number("n", 0.0);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
}

#[test]
fn no_table_keeps_unchanged_writes_out_of_the_bounded_journal() {
    let (policy, root, _, _factory) = setup();
    let capture = RuntimeViewModelChangeCapture::begin_bounded(0, 0).unwrap();
    policy.prepare_capture(&capture);
    root.borrow_mut().set_number_by_property_name("n", 0.0);
    assert!(root.resolve_change_capture(capture).unwrap().is_empty());
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
}

#[test]
fn removed_list_item_keeps_its_write_order_until_publication() {
    let (mut policy, _, file, _factory) = setup();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file, 1, 0).unwrap(),
    );
    let child = root.list_items_by_property_name_path("rows").unwrap()[0].clone();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    let retained = root.reachable_change_owner_snapshot().unwrap();
    let transaction = RuntimeOwnedViewModelGraphTransaction::begin(&retained, 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    child.borrow_mut().set_number_by_property_name("n", 5.0);
    assert!(root.remove_list_item_by_property_name_path("rows", 0));
    policy
        .apply_markers(&capture, &retained, |owner, index, value| {
            Ok(owner
                .borrow_mut()
                .set_boolean_by_property_index(index, value))
        })
        .unwrap();
    let changes =
        RuntimeOwnedViewModelHandle::resolve_change_capture_across(&retained, capture).unwrap();
    assert_eq!(
        child.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
    assert_eq!(root.list_item_count_by_property_name_path("rows"), Some(0));
    assert!(changes.iter().any(|change| change.owner_instance_identity
        == child.instance_identity()
        && change.property_index == 1
        && change.value == RuntimeViewModelChangeValue::Boolean(true)));
    transaction.commit();
}

#[test]
fn replacement_with_fewer_entries_stops_tracking_removed_pairs() {
    let (mut policy, root, _, _factory) = setup();
    policy
        .set_markers(&[pair("n", "n_set"), pair("b", "b_set")])
        .unwrap();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    run(&policy, &root, || {
        root.borrow_mut().set_number_by_property_name("n", 5.0);
        root.borrow_mut().set_boolean_by_property_name("b", true);
    });
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
    assert_eq!(
        root.borrow().boolean_value_by_property_name("b_set"),
        Some(false)
    );
}

#[test]
fn markers_do_not_validate_number_contents() {
    let (mut policy, root, _, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    for number in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.0] {
        root.borrow_mut()
            .set_boolean_by_property_name("n_set", false);
        run(&policy, &root, || {
            root.borrow_mut().set_number_by_property_name("n", number);
        });
        assert_eq!(
            root.borrow().boolean_value_by_property_name("n_set"),
            Some(true)
        );
    }
}

fn installed_rule(
    property: &str,
    kind: RuntimeValueRuleKind,
    mode: RuntimeValueRuleMode,
) -> RuntimeValueRule {
    RuntimeValueRule {
        model: "Values".into(),
        property: property.into(),
        kind,
        mode,
        code: "bound".into(),
        message: "Installed message".into(),
    }
}

#[test]
fn rules_restore_last_accepted_write_and_omit_refused_rows() {
    for writes in [[300.0, 400.0], [400.0, 300.0], [400.0, 400.0]] {
        let (mut policy, root, _file, _factory) = setup();
        policy.set_markers(&[pair("n", "n_set")]).unwrap();
        policy
            .set_rules(&[installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMaximum(365.0),
                RuntimeValueRuleMode::Refuse,
            )])
            .unwrap();
        let roots = std::slice::from_ref(&root);
        let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
        let mut operation = policy.begin_rules(roots).unwrap();
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        policy.prepare_capture(&capture);
        for value in writes {
            root.borrow_mut().set_number_by_property_name("n", value);
            root.borrow_mut()
                .set_boolean_by_property_name("n_set", true);
        }
        operation.apply(&policy, &capture, roots).unwrap();
        policy
            .apply_markers(&capture, roots, |owner, index, value| {
                Ok(owner
                    .borrow_mut()
                    .set_boolean_by_property_index(index, value))
            })
            .unwrap();
        let accepted = writes.contains(&300.0);
        assert_eq!(
            root.borrow().number_value_by_property_name("n"),
            Some(if accepted { 300.0 } else { 0.0 })
        );
        assert_eq!(
            root.borrow().boolean_value_by_property_name("n_set"),
            Some(accepted)
        );
        assert_eq!(
            operation
                .reports()
                .iter()
                .filter(|report| report.refused)
                .count(),
            if accepted { 1 } else { 2 }
        );
        let changes = root.resolve_change_capture(capture).unwrap();
        assert!(
            !changes
                .iter()
                .any(|change| change.value == RuntimeViewModelChangeValue::Number(400.0))
        );
        if accepted {
            assert!(
                changes
                    .iter()
                    .any(|change| change.value == RuntimeViewModelChangeValue::Number(300.0))
            );
            assert!(
                changes
                    .iter()
                    .any(|change| change.value == RuntimeViewModelChangeValue::Boolean(true))
            );
        } else {
            assert!(changes.is_empty());
        }
        checkpoint.commit();
    }
}

#[test]
fn rule_install_is_atomic_and_marking_keeps_the_write() {
    let (mut policy, root, _file, _factory) = setup();
    let rule = installed_rule(
        "n",
        RuntimeValueRuleKind::NumberMinimum(10.0),
        RuntimeValueRuleMode::Mark,
    );
    policy.set_rules(std::slice::from_ref(&rule)).unwrap();
    let mut bad = rule.clone();
    bad.property = "missing".into();
    assert_eq!(
        policy.set_rules(&[bad]),
        Err(RuntimeValuePolicyError::NotFound)
    );
    assert_eq!(policy.rule(0).unwrap().property, "n");
    assert_eq!(
        policy.set_rules(&[installed_rule(
            "b",
            RuntimeValueRuleKind::NumberMinimum(0.0),
            RuntimeValueRuleMode::Refuse
        )]),
        Err(RuntimeValuePolicyError::InvalidArgument)
    );
    assert_eq!(
        policy.set_rules(&[
            rule,
            installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMaximum(5.0),
                RuntimeValueRuleMode::Refuse
            )
        ]),
        Err(RuntimeValuePolicyError::InvalidArgument)
    );
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    root.borrow_mut().set_number_by_property_name("n", 1.0);
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(1.0));
    assert_eq!(operation.reports().len(), 1);
    assert!(!operation.reports()[0].refused);
    assert_eq!(root.resolve_change_capture(capture).unwrap().len(), 1);
    checkpoint.commit();
}

#[test]
fn picked_limit_restores_only_the_third_item_and_list_limit_restores_membership() {
    let (mut policy, first, file, _factory) = setup();
    let container = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 1, 0).unwrap(),
    );
    container.clear_list_items_by_property_name_path("rows");
    let second = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::new(file.clone(), 0).unwrap(),
    );
    let third =
        RuntimeOwnedViewModelHandle::new(RuntimeOwnedViewModelInstance::new(file, 0).unwrap());
    for item in [&first, &second, &third] {
        assert!(container.push_list_item_by_property_name_path("rows", item));
    }
    policy
        .set_rules(&[RuntimeValueRule {
            model: "Container".into(),
            property: "rows".into(),
            kind: RuntimeValueRuleKind::PickedCount {
                property: "b".into(),
                minimum: 0,
                maximum: 2,
            },
            mode: RuntimeValueRuleMode::Refuse,
            code: "limit".into(),
            message: "Two at most".into(),
        }])
        .unwrap();
    let roots = std::slice::from_ref(&container);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    for item in [&first, &second, &third] {
        item.borrow_mut().set_boolean_by_property_name("b", true);
    }
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(
        first.borrow().boolean_value_by_property_name("b"),
        Some(true)
    );
    assert_eq!(
        second.borrow().boolean_value_by_property_name("b"),
        Some(true)
    );
    assert_eq!(
        third.borrow().boolean_value_by_property_name("b"),
        Some(false)
    );
    assert_eq!(operation.reports().len(), 1);
    assert_eq!(
        operation.reports()[0].owner_instance_identity,
        third.instance_identity()
    );
    assert_eq!(operation.reports()[0].property_index, 2);
    assert_eq!(container.resolve_change_capture(capture).unwrap().len(), 2);
    checkpoint.commit();

    policy
        .set_rules(&[RuntimeValueRule {
            model: "Container".into(),
            property: "rows".into(),
            kind: RuntimeValueRuleKind::ItemCount {
                minimum: 2,
                maximum: 3,
            },
            mode: RuntimeValueRuleMode::Refuse,
            code: "count".into(),
            message: "At least two".into(),
        }])
        .unwrap();
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    assert!(container.remove_list_item_by_property_name_path("rows", 2));
    assert!(container.remove_list_item_by_property_name_path("rows", 1));
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(
        container.list_item_count_by_property_name_path("rows"),
        Some(2)
    );
    let kept = container.list_items_by_property_name_path("rows").unwrap();
    assert_eq!(kept[0].instance_identity(), first.instance_identity());
    assert_eq!(kept[1].instance_identity(), second.instance_identity());
    assert_eq!(operation.reports().len(), 1);
    assert_eq!(container.resolve_change_capture(capture).unwrap().len(), 1);
    checkpoint.commit();
}

#[test]
fn refusal_correction_does_not_consume_the_write_budget_or_repeat_a_report() {
    let (mut policy, root, _file, _factory) = setup();
    policy
        .set_rules(&[installed_rule(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
        )])
        .unwrap();
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin_bounded(1, 1024).unwrap();
    policy.prepare_capture(&capture);
    root.borrow_mut().set_number_by_property_name("n", 400.0);
    assert!(operation.apply(&policy, &capture, roots).unwrap());
    assert!(!operation.apply(&policy, &capture, roots).unwrap());
    assert_eq!(operation.reports().len(), 1);
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    assert!(root.resolve_change_capture(capture).unwrap().is_empty());
    checkpoint.commit();
}

#[test]
fn explicit_empty_clear_lands_and_only_selected_properties_have_rules() {
    let (mut policy, root, _file, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    policy
        .set_rules(&[
            installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMinimum(10.0),
                RuntimeValueRuleMode::Refuse,
            ),
            installed_rule(
                "n",
                RuntimeValueRuleKind::Required,
                RuntimeValueRuleMode::Mark,
            ),
        ])
        .unwrap();
    root.borrow_mut().set_number_by_property_name("n", 20.0);
    root.borrow_mut()
        .set_boolean_by_property_name("n_set", true);
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    root.borrow_mut().set_number_by_property_name("n", 0.0);
    root.borrow_mut()
        .set_boolean_by_property_name("n_set", false);
    root.borrow_mut()
        .set_string_by_property_name("text", b"unrestricted");
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
    assert!(
        operation
            .reports()
            .iter()
            .all(|report| !report.refused && report.rule_index == 1)
    );
    assert_eq!(root.resolve_change_capture(capture).unwrap().len(), 3);
    checkpoint.commit();
}

#[test]
fn script_created_detached_owner_uses_its_value_before_the_first_write() {
    let (mut policy, root, file, _factory) = setup();
    policy
        .set_rules(&[installed_rule(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
        )])
        .unwrap();
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let created = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::new(file.clone(), 0).unwrap(),
    );
    let script = crate::ScriptViewModel::from_native(created.native_handle(), file).unwrap();
    assert!(script.set_number("n", 400.0));
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(
        created.borrow().number_value_by_property_name("n"),
        Some(0.0)
    );
    assert_eq!(operation.reports().len(), 1);
    assert_eq!(
        operation.reports()[0].owner_instance_identity,
        created.instance_identity()
    );
    assert!(
        RuntimeOwnedViewModelHandle::resolve_change_capture_across(
            &operation.retained_roots(),
            capture
        )
        .unwrap()
        .is_empty()
    );
    checkpoint.commit();
}

#[test]
fn checked_write_refuses_before_mutation_and_keeps_other_writes() {
    let (mut policy, root, _file, _factory) = setup();
    policy.set_markers(&[pair("n", "n_set")]).unwrap();
    policy
        .set_rules(&[installed_rule(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
        )])
        .unwrap();
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let result = operation
        .checked_write(
            &policy,
            &root,
            "n",
            RuntimeViewModelChangeValue::Number(400.0),
            None,
            || panic!("a refused writer must not run"),
        )
        .unwrap();
    assert!(!result.applied);
    assert_eq!(result.rule_indices, vec![0]);
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    let marker = operation
        .checked_write(
            &policy,
            &root,
            "n_set",
            RuntimeViewModelChangeValue::Boolean(true),
            None,
            || panic!("the refused value's marker must not land"),
        )
        .unwrap();
    assert!(!marker.applied);
    let result = operation
        .checked_write(
            &policy,
            &root,
            "b",
            RuntimeViewModelChangeValue::Boolean(true),
            None,
            || {
                root.borrow_mut().set_boolean_by_property_name("b", true);
                Ok(())
            },
        )
        .unwrap();
    assert!(result.applied);
    assert!(result.rule_indices.is_empty());
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(operation.reports().len(), 1);
    let changes = root.resolve_change_capture(capture).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].property_index, 2);
    checkpoint.commit();
}

#[test]
fn checked_reports_follow_attempt_order_across_early_script_flushes() {
    let (mut policy, root, _file, _factory) = setup();
    policy
        .set_rules(&[
            installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMinimum(10.0),
                RuntimeValueRuleMode::Mark,
            ),
            installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMaximum(365.0),
                RuntimeValueRuleMode::Refuse,
            ),
        ])
        .unwrap();
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    for (value, accepted, indices) in [
        (5.0, true, vec![0]),
        (400.0, false, vec![1]),
        (20.0, true, vec![]),
    ] {
        operation.apply_pending_writes(&policy, roots).unwrap();
        let result = operation
            .checked_write(
                &policy,
                &root,
                "n",
                RuntimeViewModelChangeValue::Number(value),
                None,
                || {
                    root.borrow_mut().set_number_by_property_name("n", value);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(result.applied, accepted);
        assert_eq!(result.rule_indices, indices);
    }
    operation.apply(&policy, &capture, roots).unwrap();
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(20.0));
    assert_eq!(
        operation
            .reports()
            .iter()
            .map(|report| (report.rule_index, report.refused))
            .collect::<Vec<_>>(),
        vec![(0, false), (1, true)]
    );
    let changes = root.resolve_change_capture(capture).unwrap();
    assert_eq!(changes.len(), 2);
    assert!(matches!(
        changes[0].value,
        RuntimeViewModelChangeValue::Number(5.0)
    ));
    assert!(matches!(
        changes[1].value,
        RuntimeViewModelChangeValue::Number(20.0)
    ));
    checkpoint.commit();
}

#[test]
fn report_payloads_share_the_operation_byte_bound() {
    let (mut policy, root, _file, _factory) = setup();
    let rules = (0..1025)
        .map(|_| {
            installed_rule(
                "text",
                RuntimeValueRuleKind::Length {
                    minimum: 0,
                    maximum: 1,
                },
                RuntimeValueRuleMode::Mark,
            )
        })
        .collect::<Vec<_>>();
    policy.set_rules(&rules).unwrap();
    let roots = std::slice::from_ref(&root);
    let _checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let mut operation = policy.begin_rules(roots).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    root.borrow_mut()
        .set_string_by_property_name("text", &vec![b'x'; 8192]);
    assert_eq!(
        operation.apply(&policy, &capture, roots),
        Err(RuntimeValuePolicyError::LimitExceeded)
    );
}

#[test]
fn value_and_its_marker_report_a_marking_breach_once() {
    for explicit in [true, false] {
        let (mut policy, root, _file, _factory) = setup();
        policy.set_markers(&[pair("n", "n_set")]).unwrap();
        policy
            .set_rules(&[installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMinimum(10.0),
                RuntimeValueRuleMode::Mark,
            )])
            .unwrap();
        let roots = std::slice::from_ref(&root);
        let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
        let mut operation = policy.begin_rules(roots).unwrap();
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        policy.prepare_capture(&capture);
        root.borrow_mut().set_number_by_property_name("n", 5.0);
        if explicit {
            root.borrow_mut()
                .set_boolean_by_property_name("n_set", true);
        }
        operation.apply(&policy, &capture, roots).unwrap();
        policy
            .apply_markers(&capture, roots, |owner, index, value| {
                Ok(owner
                    .borrow_mut()
                    .set_boolean_by_property_index(index, value))
            })
            .unwrap();
        operation.apply(&policy, &capture, roots).unwrap();
        assert_eq!(operation.reports().len(), 1, "explicit marker: {explicit}");
        drop(capture);
        checkpoint.commit();
    }
}

#[test]
fn split_count_bounds_reject_an_empty_interval_atomically() {
    use RuntimeValueRuleKind::{ItemCount, Length, PickedCount};
    let (mut policy, _root, _file, _factory) = setup();
    let original = installed_rule(
        "n",
        RuntimeValueRuleKind::NumberMinimum(10.0),
        RuntimeValueRuleMode::Mark,
    );
    policy.set_rules(&[original]).unwrap();
    for (model, property, lower, upper) in [
        (
            "Values",
            "text",
            Length {
                minimum: 3,
                maximum: usize::MAX,
            },
            Length {
                minimum: 0,
                maximum: 2,
            },
        ),
        (
            "Container",
            "rows",
            ItemCount {
                minimum: 3,
                maximum: usize::MAX,
            },
            ItemCount {
                minimum: 0,
                maximum: 2,
            },
        ),
        (
            "Container",
            "rows",
            PickedCount {
                property: "b".into(),
                minimum: 3,
                maximum: usize::MAX,
            },
            PickedCount {
                property: "b".into(),
                minimum: 0,
                maximum: 2,
            },
        ),
    ] {
        let mut lower = installed_rule(property, lower, RuntimeValueRuleMode::Mark);
        lower.model = model.into();
        let mut upper = installed_rule(property, upper, RuntimeValueRuleMode::Refuse);
        upper.model = model.into();
        // No count can be both at least three and at most two, regardless
        // of whether those bounds occupy one entry or separate entries.
        assert_eq!(
            policy.set_rules(&[lower, upper]),
            Err(RuntimeValuePolicyError::InvalidArgument),
            "{model}.{property}"
        );
        assert_eq!(policy.rule(0).unwrap().property, "n");
        assert!(policy.rule(1).is_none());
    }
}

#[test]
fn host_created_owner_uses_its_value_before_the_first_write() {
    for constructor in ["new", "authored", "named"] {
        let (mut policy, root, file, _factory) = setup();
        policy
            .set_rules(&[installed_rule(
                "n",
                RuntimeValueRuleKind::NumberMaximum(365.0),
                RuntimeValueRuleMode::Refuse,
            )])
            .unwrap();
        let checkpoint =
            RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096)
                .unwrap();
        let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        policy.prepare_capture(&capture);
        let instance = match constructor {
            "new" => RuntimeOwnedViewModelInstance::new(file, 0).unwrap(),
            "authored" => RuntimeOwnedViewModelInstance::from_instance(file, 0, 0).unwrap(),
            _ => RuntimeOwnedViewModelInstance::from_instance_name(file, 0, "").unwrap(),
        };
        let created = RuntimeOwnedViewModelHandle::new(instance);
        assert_eq!(
            created.borrow().number_value_by_property_name("n"),
            Some(0.0),
            "{constructor}"
        );
        assert!(created.borrow_mut().set_number_by_property_name("n", 400.0));
        // The owner only enters the caller's graph after its first write.
        // Its checkpoint is the constructor's zero, never the attempted 400.
        operation
            .apply(&policy, &capture, &[root, created.clone()])
            .unwrap();
        assert_eq!(
            created.borrow().number_value_by_property_name("n"),
            Some(0.0),
            "{constructor}"
        );
        assert_eq!(operation.reports().len(), 1);
        assert!(operation.reports()[0].refused);
        assert!(
            RuntimeOwnedViewModelHandle::resolve_change_capture_across(
                &operation.retained_roots(),
                capture
            )
            .unwrap()
            .is_empty()
        );
        checkpoint.commit();
    }
}

#[test]
#[ignore = "Native creation checkpoint needs a port exception or compiler contract; project question pending"]
fn native_created_owner_needs_a_checkpoint_before_its_first_write() {
    use crate::mechanical_port::source::viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_number::ViewModelInstanceNumber,
    };
    let (mut policy, root, file, _factory) = setup();
    policy
        .set_rules(&[installed_rule(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
        )])
        .unwrap();
    let checkpoint =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    // This is the factory used by native number-to-list conversion and
    // nested-artboard default models. Neither path constructs a host adapter.
    let native = file
        .with_file(|file| file.create_default_view_model_instance(file.view_model(0).unwrap()))
        .unwrap();
    let number = native
        .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().first().cloned())
        .flatten()
        .unwrap();
    number
        .with_downcast_mut::<ViewModelInstanceNumber, _>(|value| value.set_value(400.0))
        .unwrap();
    let created = RuntimeOwnedViewModelHandle::from_native(file, native).unwrap();
    operation
        .apply(&policy, &capture, &[root, created.clone()])
        .unwrap();
    assert_eq!(
        created.borrow().number_value_by_property_name("n"),
        Some(0.0)
    );
    checkpoint.commit();
}
