#![allow(
    clippy::unwrap_used,
    clippy::arithmetic_side_effects,
    reason = "bounded fixture assertions"
)]
use crate::{
    File, RuntimeFactoryHandle, RuntimeFileHandle, RuntimeOwnedViewModelGraphTransaction,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance, RuntimeValueMarker,
    RuntimeValuePolicy, RuntimeValuePolicyError, RuntimeViewModelChangeCapture,
    RuntimeViewModelChangeValue,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).unwrap();
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap()
        .key
        .int
}

fn object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .unwrap()
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn fixture() -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for v in [7, 0, 3593, 0] {
        push_var_uint(&mut b, v);
    }
    object(&mut b, "Backboard", |_| {});
    object(&mut b, "ViewModel", |b| {
        string(b, "ViewModel", "name", "Values")
    });
    for (kind, name) in [
        ("Number", "n"),
        ("Boolean", "n_set"),
        ("Boolean", "b"),
        ("Boolean", "b_set"),
        ("Color", "c"),
        ("Boolean", "c_set"),
        ("String", "text"),
    ] {
        let kind = format!("ViewModelProperty{kind}");
        object(&mut b, &kind, |b| string(b, &kind, "name", name));
    }
    object(&mut b, "DataEnumCustom", |b| {
        string(b, "DataEnumCustom", "name", "Options")
    });
    for key in ["first", "second"] {
        object(&mut b, "DataEnumValue", |b| {
            string(b, "DataEnumValue", "key", key);
            string(b, "DataEnumValue", "value", key);
        });
    }
    object(&mut b, "ViewModelPropertyEnumCustom", |b| {
        string(b, "ViewModelPropertyEnumCustom", "name", "choice");
        uint(b, "ViewModelPropertyEnumCustom", "enumId", 0);
    });
    object(&mut b, "ViewModelPropertyBoolean", |b| {
        string(b, "ViewModelPropertyBoolean", "name", "choice_set")
    });
    object(&mut b, "ViewModelInstance", |b| {
        uint(b, "ViewModelInstance", "viewModelId", 0)
    });
    for (i, kind) in [
        "Number", "Boolean", "Boolean", "Boolean", "Color", "Boolean", "String", "Enum", "Boolean",
    ]
    .iter()
    .enumerate()
    {
        let kind = format!("ViewModelInstance{kind}");
        object(&mut b, &kind, |b| {
            uint(b, &kind, "viewModelPropertyId", i as u64)
        });
    }
    b
}
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
