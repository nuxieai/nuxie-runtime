#![allow(clippy::unwrap_used, reason = "fixture assertions")]
use super::tests::fixture;
use super::*;
use nuxie_render_api::{PersistentFactory, RecordingFactory};

fn entry(
    property: &str,
    kind: RuntimeValueRuleKind,
    mode: RuntimeValueRuleMode,
    code: &str,
) -> RuntimeValueRule {
    RuntimeValueRule {
        model: "Values".into(),
        property: property.into(),
        kind,
        mode,
        code: code.into(),
        message: format!("message {code}"),
    }
}
fn table() -> RuntimeRuleGroup {
    RuntimeRuleGroup {
        model: "Values".into(),
        valid: "valid".into(),
        members: ["n", "text"]
            .map(|property| RuntimeRuleGroupMember {
                property: property.into(),
                errors_path: format!("{property}_errors"),
                item_model: "ErrorEntry".into(),
                code_property: "code".into(),
                message_property: "message".into(),
            })
            .into(),
    }
}
fn errors(root: &RuntimeOwnedViewModelHandle, property: &str) -> Vec<(String, String)> {
    root.list_items_by_property_name_path(property)
        .unwrap()
        .iter()
        .map(|item| {
            let get = |name| {
                String::from_utf8(
                    item.borrow()
                        .string_value_by_property_name(name)
                        .unwrap()
                        .to_vec(),
                )
                .unwrap()
            };
            (get("code"), get("message"))
        })
        .collect()
}
fn expected(codes: &[&str]) -> Vec<(String, String)> {
    codes
        .iter()
        .map(|code| ((*code).into(), format!("message {code}")))
        .collect()
}
fn run(
    policy: &RuntimeValuePolicy,
    roots: &[RuntimeOwnedViewModelHandle],
    action: impl FnOnce(),
    commit: bool,
) -> usize {
    let transaction = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(roots).unwrap();
    operation
        .apply_initial_groups(policy, roots, |_, _| Ok(()))
        .unwrap();
    action();
    let mut settled = false;
    let mut cursor = 0;
    for _ in 0..16 {
        if !operation
            .needs_policy_pass(policy, &capture, cursor)
            .unwrap()
        {
            settled = true;
            break;
        }
        let rules = operation.apply(policy, &capture, roots).unwrap();
        cursor = capture.write_count().unwrap();
        let retained = operation.retained_roots();
        let markers = policy
            .apply_markers(&capture, &retained, |owner, index, value| {
                Ok(owner
                    .borrow_mut()
                    .set_boolean_by_property_index(index, value))
            })
            .unwrap();
        let groups = operation
            .apply_groups(policy, roots, |_, _| Ok(()))
            .unwrap();
        if !rules && !markers && !groups {
            settled = true;
            break;
        }
    }
    assert!(settled, "native policy passes must reach a fixed point");
    let rows = RuntimeOwnedViewModelHandle::resolve_change_capture_across_with_owners(
        &operation.retained_roots(),
        capture,
    )
    .unwrap();
    if commit {
        transaction.commit();
        operation.commit_groups(policy);
    }
    rows.len()
}

#[test]
fn kept_validity_ordered_errors_and_latest_refusal() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = crate::File::import(
        &fixture::group_fixture(),
        crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let second = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let roots = [root.clone(), second.clone()];
    let mut policy = RuntimeValuePolicy::new(file);
    policy
        .set_markers(&[RuntimeValueMarker {
            model: "Values".into(),
            value: "n".into(),
            marker: "n_set".into(),
        }])
        .unwrap();
    let numeric = vec![
        entry(
            "n",
            RuntimeValueRuleKind::Required,
            RuntimeValueRuleMode::Mark,
            "required",
        ),
        entry(
            "n",
            RuntimeValueRuleKind::NumberMinimum(1.0),
            RuntimeValueRuleMode::Mark,
            "min",
        ),
        entry(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
            "max",
        ),
    ];
    for reverse in [false, true] {
        let maximum = entry(
            "text",
            RuntimeValueRuleKind::Length {
                minimum: 0,
                maximum: 2,
            },
            RuntimeValueRuleMode::Refuse,
            "length",
        );
        let pattern = entry(
            "text",
            RuntimeValueRuleKind::Pattern("[0-9]+".into()),
            RuntimeValueRuleMode::Mark,
            "pattern",
        );
        let mut rules = numeric.clone();
        rules.extend(if reverse {
            [pattern, maximum]
        } else {
            [maximum, pattern]
        });
        policy.set_rules(&rules).unwrap();
        policy.set_groups(&[table()]).unwrap();
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut().set_number_by_property_name("n", 0.0);
                root.borrow_mut()
                    .set_boolean_by_property_name("n_set", false);
                root.borrow_mut().set_string_by_property_name("text", b"");
            },
            true,
        );
        assert_eq!(errors(&root, "n_errors"), expected(&["required"]));
        use crate::mechanical_port::source::{
            artboard::Artboard,
            viewmodel::{
                viewmodel_instance_list::ViewModelInstanceList,
                viewmodel_instance_list_item::ViewModelInstanceListItem,
            },
        };
        let list = root.borrow().property_by_path(&[10]).unwrap();
        let item = list
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items()[0].clone())
            .unwrap();
        let artboard = item
            .with_downcast::<ViewModelInstanceListItem, _>(ViewModelInstanceListItem::artboard)
            .flatten();
        assert!(
            artboard.is_some(),
            "generated error row uses the authored matching artboard"
        );
        assert_eq!(
            artboard
                .unwrap()
                .with_downcast::<Artboard, _>(|artboard| artboard.base.view_model_id()),
            Some(2)
        );

        assert_eq!(
            root.borrow().boolean_value_by_property_name("valid"),
            Some(false)
        );
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut().set_number_by_property_name("n", 365.0);
                root.borrow_mut().set_number_by_property_name("n", 366.0);
            },
            true,
        );
        assert_eq!(
            root.borrow().boolean_value_by_property_name("valid"),
            Some(true)
        );
        assert_eq!(
            errors(&root, "n_errors"),
            expected(&["max"]),
            "derived marker cannot erase the newest refusal"
        );
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut().set_number_by_property_name("n", 366.0);
            },
            true,
        );
        assert_eq!(
            root.borrow().number_value_by_property_name("n"),
            Some(365.0)
        );
        assert_eq!(errors(&root, "n_errors"), expected(&["max"]));
        assert_eq!(
            root.borrow().boolean_value_by_property_name("valid"),
            Some(true)
        );
        assert_eq!(
            run(&policy, &roots, || {}, true),
            0,
            "quiet preserves latest refusal without rows"
        );
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut().set_number_by_property_name("n", 365.0);
            },
            true,
        );
        assert!(
            errors(&root, "n_errors").is_empty(),
            "accepted unchanged host write clears refusal"
        );
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut().set_string_by_property_name("text", b"ab");
                root.borrow_mut()
                    .set_string_by_property_name("text", b"abc");
            },
            true,
        );
        assert_eq!(
            errors(&root, "text_errors"),
            expected(&["length", "pattern"])
        );
        assert_eq!(
            root.borrow().boolean_value_by_property_name("valid"),
            Some(false)
        );
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut()
                    .set_boolean_by_property_name("valid", true);
                root.clear_list_items_by_property_name_path("text_errors");
            },
            true,
        );
        assert_eq!(
            root.borrow().boolean_value_by_property_name("valid"),
            Some(false)
        );
        assert_eq!(errors(&root, "text_errors").len(), 2);
        let before = errors(&root, "text_errors");
        run(
            &policy,
            &roots,
            || {
                root.borrow_mut().set_string_by_property_name("text", b"1");
            },
            false,
        );
        assert_eq!(
            errors(&root, "text_errors"),
            before,
            "failed native transaction restores output lists"
        );
        run(&policy, &roots, || {}, true);
        assert_eq!(
            errors(&root, "text_errors"),
            before,
            "failed operation does not clear refusal history"
        );
        run(
            &policy,
            &roots,
            || {
                second.borrow_mut().set_number_by_property_name("n", 2.0);
            },
            true,
        );
        assert_eq!(
            errors(&root, "text_errors"),
            before,
            "other instance cannot alter this instance's errors"
        );
        let mut bad = table();
        bad.members[0].errors_path = "missing".into();
        assert_eq!(
            policy.set_groups(&[bad]),
            Err(RuntimeValuePolicyError::NotFound)
        );
        assert_eq!(
            run(&policy, &roots, || {}, true),
            0,
            "bad replacement retained previous table"
        );
    }
    let mut number_group = table();
    number_group.members.truncate(1);
    let mut text_group = table();
    text_group.valid = "valid_text".into();
    text_group.members.remove(0);
    policy.set_groups(&[number_group, text_group]).unwrap();
    run(
        &policy,
        &roots,
        || {
            root.borrow_mut().set_number_by_property_name("n", 2.0);
            root.borrow_mut().set_string_by_property_name("text", b"ab");
        },
        true,
    );
    assert_eq!(
        root.borrow().boolean_value_by_property_name("valid"),
        Some(true)
    );
    assert_eq!(
        root.borrow().boolean_value_by_property_name("valid_text"),
        Some(false)
    );
    let text_errors = root
        .list_items_by_property_name_path("text_errors")
        .unwrap();
    run(
        &policy,
        &roots,
        || {
            root.borrow_mut().set_number_by_property_name("n", 0.0);
        },
        true,
    );
    assert_eq!(
        root.borrow().boolean_value_by_property_name("valid"),
        Some(false)
    );
    assert_eq!(
        root.borrow().boolean_value_by_property_name("valid_text"),
        Some(false)
    );
    assert_eq!(
        root.list_items_by_property_name_path("text_errors")
            .unwrap()[0]
            .instance_identity(),
        text_errors[0].instance_identity(),
        "another group's write does not replace this list"
    );
}

#[test]
fn native_unchanged_acceptance_requires_an_observable_write() {
    use crate::mechanical_port::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = crate::File::import(
        &fixture::group_fixture(),
        crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let roots = std::slice::from_ref(&root);
    let mut policy = RuntimeValuePolicy::new(file);
    policy
        .set_rules(&[entry(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
            "max",
        )])
        .unwrap();
    let mut group = table();
    group.members.truncate(1);
    policy.set_groups(&[group]).unwrap();
    run(
        &policy,
        roots,
        || {
            root.borrow_mut().set_number_by_property_name("n", 300.0);
        },
        true,
    );
    run(
        &policy,
        roots,
        || {
            root.borrow_mut().set_number_by_property_name("n", 400.0);
        },
        true,
    );
    assert_eq!(errors(&root, "n_errors"), expected(&["max"]));
    run(
        &policy,
        roots,
        || {
            root.borrow()
                .property_by_path(&[0])
                .unwrap()
                .with_downcast_mut::<ViewModelInstanceNumber, _>(|number| number.set_value(300.0))
                .unwrap();
        },
        true,
    );
    // The native setter emits nothing for an unchanged value. Until a real
    // change occurs, the host cannot distinguish this from no write at all.
    assert_eq!(errors(&root, "n_errors"), expected(&["max"]));
    run(
        &policy,
        roots,
        || {
            root.borrow()
                .property_by_path(&[0])
                .unwrap()
                .with_downcast_mut::<ViewModelInstanceNumber, _>(|number| number.set_value(301.0))
                .unwrap();
        },
        true,
    );
    assert!(errors(&root, "n_errors").is_empty());
}

#[test]
fn optional_empty_picks_start_valid_and_quiet_steps_skip_passes() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = crate::File::import(
        &fixture::group_fixture(),
        crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let roots = std::slice::from_ref(&root);
    let mut policy = RuntimeValuePolicy::new(file);
    policy
        .set_rules(&[entry(
            "picks",
            RuntimeValueRuleKind::PickedCount {
                property: "b".into(),
                minimum: 1,
                maximum: 3,
            },
            RuntimeValueRuleMode::Mark,
            "minimum",
        )])
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
    assert!(run(&policy, roots, || {}, true) > 0);
    assert_eq!(
        root.borrow().boolean_value_by_property_name("valid"),
        Some(true)
    );
    assert!(errors(&root, "picks_errors").is_empty());
    assert!(RuntimeValuePolicy::take_test_pass_count() > 0);
    assert_eq!(run(&policy, roots, || {}, true), 0);
    assert_eq!(
        RuntimeValuePolicy::take_test_pass_count(),
        0,
        "quiet step performs no policy pass"
    );
    policy.invalidate();
    assert_eq!(run(&policy, roots, || {}, true), 0);
    assert_eq!(
        RuntimeValuePolicy::take_test_pass_count(),
        1,
        "invalidated roots recompute their initial outputs"
    );
}
#[test]
fn processed_script_refusal_still_updates_groups_with_an_empty_capture() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = crate::File::import(
        &fixture::group_fixture(),
        crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let roots = std::slice::from_ref(&root);
    let mut policy = RuntimeValuePolicy::new(file);
    policy
        .set_rules(&[entry(
            "n",
            RuntimeValueRuleKind::NumberMaximum(365.0),
            RuntimeValueRuleMode::Refuse,
            "max",
        )])
        .unwrap();
    let mut group = table();
    group.members.truncate(1);
    policy.set_groups(&[group]).unwrap();
    run(
        &policy,
        roots,
        || {
            root.borrow_mut().set_number_by_property_name("n", 300.0);
        },
        true,
    );
    let transaction = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let mut operation = policy.begin_rules(roots).unwrap();
    assert!(
        !operation
            .apply_initial_groups(&policy, roots, |_, _| Ok(()))
            .unwrap()
    );
    assert!(
        !operation
            .checked_write(
                &policy,
                &root,
                "n",
                RuntimeViewModelChangeValue::Number(400.0),
                None,
                || panic!("refused writer must not run")
            )
            .unwrap()
            .applied
    );
    operation.apply_pending_writes(&policy, roots).unwrap();
    assert_eq!(capture.write_count().unwrap(), 0);
    assert!(operation.needs_policy_pass(&policy, &capture, 0).unwrap());
    operation
        .apply_groups(&policy, roots, |_, _| Ok(()))
        .unwrap();
    assert_eq!(errors(&root, "n_errors"), expected(&["max"]));
    drop(capture);
    transaction.commit();
    operation.commit_groups(&policy);
}
