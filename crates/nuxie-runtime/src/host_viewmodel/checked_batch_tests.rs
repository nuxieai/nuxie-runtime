#![allow(
    clippy::unwrap_used,
    reason = "literal public-boundary fixture assertions"
)]
use super::*;
use crate::{
    File, RuntimeFactoryHandle, RuntimeOwnedViewModelGraphTransaction,
    RuntimeOwnedViewModelTransaction,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
#[path = "../../tests/support/value_policy_fixture.rs"]
mod fixture;

fn entry(root: &str, path: &str, value: RuntimeCheckedValueInput) -> RuntimeCheckedValueBatchEntry {
    RuntimeCheckedValueBatchEntry {
        root_name: root.into(),
        path: path.into(),
        value,
    }
}

type Roots = BTreeMap<String, RuntimeOwnedViewModelHandle>;
fn setup() -> (
    RuntimeValuePolicy,
    RuntimeOwnedViewModelHandle,
    [RuntimeOwnedViewModelHandle; 3],
    Roots,
    PersistentFactory<RecordingFactory>,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::group_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let options = ["a", "b", "c"].map(|_| {
        RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file.clone(), 0).unwrap(),
        )
    });
    for option in &options {
        assert!(root.push_list_item_by_property_name_path("picks", option));
    }
    options[0]
        .borrow_mut()
        .set_boolean_by_property_name("b", true);
    let mut policy = RuntimeValuePolicy::new(file);
    policy
        .set_rules(&[RuntimeValueRule {
            model: "Values".into(),
            property: "picks".into(),
            kind: RuntimeValueRuleKind::PickedCount {
                property: "b".into(),
                minimum: 0,
                maximum: 1,
            },
            mode: RuntimeValueRuleMode::Refuse,
            code: "maxItems".into(),
            message: "Choose fewer options.".into(),
        }])
        .unwrap();
    policy
        .set_groups(&[RuntimeRuleGroup {
            model: "Values".into(),
            valid: "valid".into(),
            members: vec![RuntimeRuleGroupMember {
                property: "picks".into(),
                errors_path: "picks_errors".into(),
                item_model: "ErrorEntry".into(),
                code_property: "code".into(),
                message_property: "message".into(),
            }],
        }])
        .unwrap();
    let roots = BTreeMap::from([
        ("form".into(), root.clone()),
        ("a".into(), options[0].clone()),
        ("b".into(), options[1].clone()),
        ("c".into(), options[2].clone()),
    ]);
    (policy, root, options, roots, factory)
}

#[test]
fn checked_batch_refuses_whole_several_choice_replacement_with_native_message() {
    let (policy, root, options, roots, _factory) = setup();
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    // An earlier independent write in this same operation must survive.
    root.borrow_mut().set_number_by_property_name("n", 42.0);
    let result = runtime_checked_value_write_batch(
        &policy,
        &mut operation,
        &roots,
        vec![
            entry("a", "b", RuntimeCheckedValueInput::Boolean(false)),
            entry("b", "b", RuntimeCheckedValueInput::Boolean(true)),
            entry("c", "b", RuntimeCheckedValueInput::Boolean(true)),
        ],
    )
    .unwrap();
    assert_eq!(
        result,
        RuntimeCheckedValueBatchResult {
            applied: false,
            refusal: Some(RuntimeCheckedValueRefusal {
                rule_index: 0,
                code: "maxItems".into(),
                message: "Choose fewer options.".into()
            })
        }
    );
    assert_eq!(
        options
            .each_ref()
            .map(|option| option.borrow().boolean_value_by_property_name("b").unwrap()),
        [true, false, false]
    );
    operation
        .apply(&policy, &capture, std::slice::from_ref(&root))
        .unwrap();
    operation
        .apply_groups(&policy, std::slice::from_ref(&root), |_, _| Ok(()))
        .unwrap();
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(42.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("valid"),
        Some(true)
    );
    let errors = root
        .list_items_by_property_name_path("picks_errors")
        .unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0]
            .borrow()
            .string_value_by_property_name("code")
            .unwrap()
            .as_ref(),
        b"maxItems"
    );
    assert_eq!(
        errors[0]
            .borrow()
            .string_value_by_property_name("message")
            .unwrap()
            .as_ref(),
        b"Choose fewer options."
    );
    assert_eq!(operation.reports().len(), 1);
    assert!(operation.reports()[0].refused);
    // The report names the selection that breaks the maximum, not the deselect.
    assert_eq!(
        operation.reports()[0].owner_instance_identity,
        options[1].instance_identity()
    );
    assert_eq!(
        operation.reports()[0].attempted,
        RuntimeViewModelChangeValue::Boolean(true)
    );
    let changes = RuntimeOwnedViewModelHandle::resolve_change_capture_across_with_owners(
        &operation.retained_roots(),
        capture,
    )
    .unwrap();
    assert!(!changes.iter().any(|(owner, change)| {
        options
            .iter()
            .any(|o| o.instance_identity() == owner.instance_identity())
            && change.property_index == 2
    }));
    transaction.commit();
    operation.commit_groups(&policy);
}

#[test]
fn checked_batch_accepts_final_selection_in_either_order_and_outer_rollback_restores_it() {
    for reverse in [false, true] {
        for commit in [false, true] {
            let (policy, root, options, roots, _factory) = setup();
            let transaction =
                RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096)
                    .unwrap();
            let capture = RuntimeViewModelChangeCapture::begin().unwrap();
            policy.prepare_capture(&capture);
            let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
            // Create a refusal first, then show that an accepted replacement clears
            // group errors without erasing the earlier report from this operation.
            assert!(
                !runtime_checked_value_write_batch(
                    &policy,
                    &mut operation,
                    &roots,
                    vec![entry("b", "b", RuntimeCheckedValueInput::Boolean(true))]
                )
                .unwrap()
                .applied
            );
            let mut entries = vec![
                entry("b", "b", RuntimeCheckedValueInput::Boolean(true)),
                entry("a", "b", RuntimeCheckedValueInput::Boolean(false)),
            ];
            if reverse {
                entries.reverse();
            }
            assert_eq!(
                runtime_checked_value_write_batch(&policy, &mut operation, &roots, entries)
                    .unwrap(),
                RuntimeCheckedValueBatchResult {
                    applied: true,
                    refusal: None
                }
            );
            operation
                .apply(&policy, &capture, std::slice::from_ref(&root))
                .unwrap();
            operation
                .apply_groups(&policy, std::slice::from_ref(&root), |_, _| Ok(()))
                .unwrap();
            assert!(
                root.list_items_by_property_name_path("picks_errors")
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(operation.reports().len(), 1);
            // An ordinary later write is evaluated against the accepted batch.
            options[2]
                .borrow_mut()
                .set_boolean_by_property_name("b", true);
            operation
                .apply(&policy, &capture, std::slice::from_ref(&root))
                .unwrap();
            assert_eq!(
                options
                    .each_ref()
                    .map(|o| o.borrow().boolean_value_by_property_name("b").unwrap()),
                [false, true, false]
            );
            assert_eq!(operation.reports().len(), 2);
            assert_eq!(
                operation.reports()[1].owner_instance_identity,
                options[2].instance_identity()
            );
            let changes = RuntimeOwnedViewModelHandle::resolve_change_capture_across_with_owners(
                &operation.retained_roots(),
                capture,
            )
            .unwrap();
            let picks = changes
                .iter()
                .filter(|(_, c)| c.property_index == 2)
                .map(|(o, c)| (o.instance_identity(), c.value.clone()))
                .collect::<Vec<_>>();
            let mut expected = vec![
                (
                    options[1].instance_identity(),
                    RuntimeViewModelChangeValue::Boolean(true),
                ),
                (
                    options[0].instance_identity(),
                    RuntimeViewModelChangeValue::Boolean(false),
                ),
            ];
            if reverse {
                expected.reverse();
            }
            assert_eq!(picks, expected);
            if commit {
                transaction.commit();
                operation.commit_groups(&policy);
            } else {
                drop(transaction);
            }
            assert_eq!(
                options
                    .each_ref()
                    .map(|o| o.borrow().boolean_value_by_property_name("b").unwrap()),
                if commit {
                    [false, true, false]
                } else {
                    [true, false, false]
                }
            );
        }
    }
}

fn numeric_rules(policy: &mut RuntimeValuePolicy, mode: RuntimeValueRuleMode) {
    policy.set_groups(&[]).unwrap();
    policy
        .set_markers(&[RuntimeValueMarker {
            model: "Values".into(),
            value: "n".into(),
            marker: "n_set".into(),
        }])
        .unwrap();
    policy
        .set_rules(&[
            RuntimeValueRule {
                model: "Values".into(),
                property: "n".into(),
                kind: RuntimeValueRuleKind::NumberMaximum(10.0),
                mode,
                code: "max".into(),
                message: "Use ten or less.".into(),
            },
            RuntimeValueRule {
                model: "Values".into(),
                property: "text".into(),
                kind: RuntimeValueRuleKind::Length {
                    minimum: 0,
                    maximum: 2,
                },
                mode,
                code: "length".into(),
                message: "Use two letters.".into(),
            },
        ])
        .unwrap();
}

#[test]
fn checked_batch_marking_breaches_apply_and_clear_keeps_marker_and_journal_order() {
    let (mut policy, root, _, roots, _factory) = setup();
    numeric_rules(&mut policy, RuntimeValueRuleMode::Mark);
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    assert_eq!(
        runtime_checked_value_write_batch(
            &policy,
            &mut operation,
            &roots,
            vec![
                entry("form", "n", RuntimeCheckedValueInput::Number(11.0)),
                entry(
                    "form",
                    "text",
                    RuntimeCheckedValueInput::Text(b"abc".to_vec())
                )
            ]
        )
        .unwrap(),
        RuntimeCheckedValueBatchResult {
            applied: true,
            refusal: None
        }
    );
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(11.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
    operation
        .apply(&policy, &capture, std::slice::from_ref(&root))
        .unwrap();
    policy
        .apply_markers(
            &capture,
            &operation.retained_roots(),
            |owner, index, value| {
                Ok(owner
                    .borrow_mut()
                    .set_boolean_by_property_index(index, value))
            },
        )
        .unwrap();
    operation
        .apply(&policy, &capture, std::slice::from_ref(&root))
        .unwrap();
    assert_eq!(
        operation
            .reports()
            .iter()
            .map(|r| (r.rule_index, r.refused))
            .collect::<Vec<_>>(),
        vec![(0, false), (1, false)]
    );
    assert!(
        runtime_checked_value_write_batch(
            &policy,
            &mut operation,
            &roots,
            vec![
                entry("form", "n", RuntimeCheckedValueInput::Clear),
                entry("form", "text", RuntimeCheckedValueInput::Clear)
            ]
        )
        .unwrap()
        .applied
    );
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
    operation
        .apply(&policy, &capture, std::slice::from_ref(&root))
        .unwrap();
    let changes = root.resolve_change_capture(capture).unwrap();
    assert_eq!(
        changes
            .iter()
            .map(|c| (c.property_index, c.value.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0, RuntimeViewModelChangeValue::Number(11.0)),
            (1, RuntimeViewModelChangeValue::Boolean(true)),
            (
                6,
                RuntimeViewModelChangeValue::String(Arc::from(b"abc".as_slice()))
            ),
            (0, RuntimeViewModelChangeValue::Number(0.0)),
            (1, RuntimeViewModelChangeValue::Boolean(false)),
            (
                6,
                RuntimeViewModelChangeValue::String(Arc::from(b"".as_slice()))
            ),
        ]
    );
    transaction.commit();
}

#[test]
fn checked_batch_first_refusal_and_suppressed_marker_survive_until_a_new_value() {
    let (mut policy, root, _, roots, _factory) = setup();
    numeric_rules(&mut policy, RuntimeValueRuleMode::Refuse);
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    let result = runtime_checked_value_write_batch(
        &policy,
        &mut operation,
        &roots,
        vec![
            entry(
                "form",
                "text",
                RuntimeCheckedValueInput::Text(b"abc".to_vec()),
            ),
            entry("form", "n", RuntimeCheckedValueInput::Number(11.0)),
        ],
    )
    .unwrap();
    assert_eq!(
        result.refusal.unwrap(),
        RuntimeCheckedValueRefusal {
            rule_index: 1,
            code: "length".into(),
            message: "Use two letters.".into()
        }
    );
    assert_eq!(
        operation
            .reports()
            .iter()
            .map(|r| (r.rule_index, r.refused))
            .collect::<Vec<_>>(),
        vec![(1, true), (0, true)]
    );
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(false)
    );
    assert!(
        !runtime_checked_value_write_batch(
            &policy,
            &mut operation,
            &roots,
            vec![entry(
                "form",
                "n_set",
                RuntimeCheckedValueInput::Boolean(true)
            )]
        )
        .unwrap()
        .applied
    );
    assert_eq!(operation.reports().len(), 2);
    assert!(
        runtime_checked_value_write_batch(
            &policy,
            &mut operation,
            &roots,
            vec![entry("form", "n", RuntimeCheckedValueInput::Number(5.0))]
        )
        .unwrap()
        .applied
    );
    root.borrow_mut().set_number_by_property_name("n", 6.0);
    operation
        .apply(&policy, &capture, std::slice::from_ref(&root))
        .unwrap();
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(6.0));
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
    let changes = root.resolve_change_capture(capture).unwrap();
    assert_eq!(
        changes
            .iter()
            .map(|c| (c.property_index, c.value.clone()))
            .collect::<Vec<_>>(),
        vec![
            (0, RuntimeViewModelChangeValue::Number(5.0)),
            (1, RuntimeViewModelChangeValue::Boolean(true)),
            (0, RuntimeViewModelChangeValue::Number(6.0))
        ]
    );
    transaction.commit();
}

#[test]
fn checked_batch_invalid_or_over_budget_candidates_leave_values_and_journal_usable() {
    let (mut policy, root, _, mut roots, _factory) = setup();
    policy.set_groups(&[]).unwrap();
    policy.set_rules(&[]).unwrap();
    roots.insert("alias".into(), root.clone());
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin_bounded(1, 1024).unwrap();
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    for (second, expected) in [
        (
            entry("form", "missing", RuntimeCheckedValueInput::Number(2.0)),
            RuntimeValuePolicyError::NotFound,
        ),
        (
            entry("form", "text", RuntimeCheckedValueInput::Boolean(true)),
            RuntimeValuePolicyError::InvalidArgument,
        ),
        (
            entry("alias", "n", RuntimeCheckedValueInput::Number(2.0)),
            RuntimeValuePolicyError::InvalidArgument,
        ),
        (
            entry(
                "form",
                "text",
                RuntimeCheckedValueInput::Text(b"ok".to_vec()),
            ),
            RuntimeValuePolicyError::LimitExceeded,
        ),
    ] {
        assert_eq!(
            runtime_checked_value_write_batch(
                &policy,
                &mut operation,
                &roots,
                vec![
                    entry("form", "n", RuntimeCheckedValueInput::Number(5.0)),
                    second
                ]
            )
            .unwrap_err(),
            expected
        );
        assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
        assert_eq!(capture.write_count().unwrap(), 0);
        assert!(operation.reports().is_empty());
    }
    assert!(
        runtime_checked_value_write_batch(
            &policy,
            &mut operation,
            &roots,
            vec![entry("form", "n", RuntimeCheckedValueInput::Number(8.0))]
        )
        .unwrap()
        .applied
    );
    assert_eq!(root.resolve_change_capture(capture).unwrap().len(), 1);
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(8.0));
    transaction.commit();
}

#[test]
fn checked_batch_refusal_blocks_only_the_markers_of_refused_entries() {
    let (mut policy, root, _, roots, _factory) = setup();
    numeric_rules(&mut policy, RuntimeValueRuleMode::Refuse);
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    let result = runtime_checked_value_write_batch(
        &policy,
        &mut operation,
        &roots,
        vec![
            entry("form", "n", RuntimeCheckedValueInput::Number(5.0)),
            entry(
                "form",
                "text",
                RuntimeCheckedValueInput::Text(b"abc".to_vec()),
            ),
        ],
    )
    .unwrap();
    assert_eq!(result.refusal.unwrap().code, "length");
    // Only the text breaks a rule, so only the text is reported.
    assert_eq!(
        operation
            .reports()
            .iter()
            .map(|r| (r.property_index, r.rule_index, r.refused))
            .collect::<Vec<_>>(),
        vec![(6, 1, true)]
    );
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));
    // The text's refusal does not block the number's marker.
    assert_eq!(
        runtime_checked_value_write_batch(
            &policy,
            &mut operation,
            &roots,
            vec![entry(
                "form",
                "n_set",
                RuntimeCheckedValueInput::Boolean(true)
            )]
        )
        .unwrap(),
        RuntimeCheckedValueBatchResult {
            applied: true,
            refusal: None
        }
    );
    assert_eq!(
        root.borrow().boolean_value_by_property_name("n_set"),
        Some(true)
    );
    transaction.commit();
}

fn foreign_root() -> (
    RuntimeOwnedViewModelHandle,
    PersistentFactory<RecordingFactory>,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::group_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file, 0, 0).unwrap(),
    );
    (root, factory)
}

#[test]
fn checked_batch_refuses_out_of_scope_or_oversized_batches_before_any_write() {
    let (policy, root, _, roots, _factory) = setup();
    let (foreign, _foreign_factory) = foreign_root();
    let mut with_foreign = roots.clone();
    with_foreign.insert("foreign".into(), foreign);
    let five = || vec![entry("form", "n", RuntimeCheckedValueInput::Number(5.0))];
    let unchanged = || assert_eq!(root.borrow().number_value_by_property_name("n"), Some(0.0));

    // No graph transaction: a capture and operation alone are not enough.
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(std::slice::from_ref(&root)).unwrap();
    assert_eq!(
        runtime_checked_value_write_batch(&policy, &mut operation, &roots, five()).unwrap_err(),
        RuntimeValuePolicyError::BorrowConflict
    );
    // A per-owner host transaction cannot restore batch writes it never captured.
    let host = RuntimeOwnedViewModelTransaction::begin().unwrap();
    assert_eq!(
        runtime_checked_value_write_batch(&policy, &mut operation, &roots, five()).unwrap_err(),
        RuntimeValuePolicyError::BorrowConflict
    );
    drop(host);
    unchanged();
    assert_eq!(capture.write_count().unwrap(), 0);
    drop(capture);

    // A graph transaction with no change capture.
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096).unwrap();
    assert_eq!(
        runtime_checked_value_write_batch(&policy, &mut operation, &roots, five()).unwrap_err(),
        RuntimeValuePolicyError::BorrowConflict
    );
    unchanged();

    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    for (roots, entries, expected) in [
        (
            &roots,
            (0..4097)
                .map(|_| entry("form", "n", RuntimeCheckedValueInput::Number(5.0)))
                .collect::<Vec<_>>(),
            RuntimeValuePolicyError::LimitExceeded,
        ),
        (
            &roots,
            vec![entry(
                "form",
                "text",
                RuntimeCheckedValueInput::Text(vec![b'a'; 8 * 1024 * 1024]),
            )],
            RuntimeValuePolicyError::LimitExceeded,
        ),
        (
            &with_foreign,
            vec![entry("foreign", "n", RuntimeCheckedValueInput::Number(5.0))],
            RuntimeValuePolicyError::InvalidArgument,
        ),
    ] {
        assert_eq!(
            runtime_checked_value_write_batch(&policy, &mut operation, roots, entries).unwrap_err(),
            expected
        );
        unchanged();
        assert_eq!(capture.write_count().unwrap(), 0);
        assert!(operation.reports().is_empty());
    }
    // The same operation still accepts an in-scope batch.
    assert!(
        runtime_checked_value_write_batch(&policy, &mut operation, &roots, five())
            .unwrap()
            .applied
    );
    assert_eq!(root.borrow().number_value_by_property_name("n"), Some(5.0));
    transaction.commit();
}
