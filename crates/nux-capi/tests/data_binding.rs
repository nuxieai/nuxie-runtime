#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "bounded C ABI fixture assertions"
)]

use nux_capi::*;
use std::path::PathBuf;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let root = std::env::var_os("NUX_RUNTIME_DIR")
        .or_else(|| std::env::var_os("RIVE_RUNTIME_DIR"))
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    std::fs::read(
        PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
    .expect("read upstream fixture")
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

fn f32_value(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn bool_value(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: bool) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.push(u8::from(value));
}

fn color(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn scalar_fixture() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 0x554e_4957, 0] {
        push_var_uint(&mut bytes, value);
    }
    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Values")
    });
    object(&mut bytes, "DataEnumCustom", |bytes| {
        string(bytes, "DataEnumCustom", "name", "Choice")
    });
    for (key, label) in [("first", "First"), ("second", "Second")] {
        object(&mut bytes, "DataEnumValue", |bytes| {
            string(bytes, "DataEnumValue", "key", key);
            string(bytes, "DataEnumValue", "value", label);
        });
    }
    for (ty, name) in [
        ("ViewModelPropertyString", "text"),
        ("ViewModelPropertyNumber", "number"),
        ("ViewModelPropertyBoolean", "enabled"),
        ("ViewModelPropertyColor", "tint"),
        ("ViewModelPropertyTrigger", "fire"),
        ("ViewModelPropertyAssetImage", "image"),
    ] {
        object(&mut bytes, ty, |bytes| string(bytes, ty, "name", name));
    }
    object(&mut bytes, "ViewModelPropertyEnumCustom", |bytes| {
        string(bytes, "ViewModelPropertyEnumCustom", "name", "choice");
        uint(bytes, "ViewModelPropertyEnumCustom", "enumId", 0);
    });
    object(&mut bytes, "Backboard", |_| {});
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "defaults");
        uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    object(&mut bytes, "ViewModelInstanceString", |bytes| {
        uint(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
        string(bytes, "ViewModelInstanceString", "propertyValue", "old");
    });
    object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 1);
        f32_value(bytes, "ViewModelInstanceNumber", "propertyValue", 3.0);
    });
    object(&mut bytes, "ViewModelInstanceBoolean", |bytes| {
        uint(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 2);
        bool_value(bytes, "ViewModelInstanceBoolean", "propertyValue", false);
    });
    object(&mut bytes, "ViewModelInstanceColor", |bytes| {
        uint(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 3);
        color(
            bytes,
            "ViewModelInstanceColor",
            "propertyValue",
            0x0102_0304,
        );
    });
    object(&mut bytes, "ViewModelInstanceTrigger", |bytes| {
        uint(bytes, "ViewModelInstanceTrigger", "viewModelPropertyId", 4);
        uint(bytes, "ViewModelInstanceTrigger", "propertyValue", 2);
    });
    object(&mut bytes, "ViewModelInstanceAssetImage", |bytes| {
        uint(
            bytes,
            "ViewModelInstanceAssetImage",
            "viewModelPropertyId",
            5,
        );
        uint(bytes, "ViewModelInstanceAssetImage", "propertyValue", 7);
    });
    object(&mut bytes, "ViewModelInstanceEnum", |bytes| {
        uint(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 6);
        uint(bytes, "ViewModelInstanceEnum", "propertyValue", 0);
    });
    object(&mut bytes, "Artboard", |_| {});
    bytes
}

fn shared_nested_list_fixture() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 0x554e_4956, 0] {
        push_var_uint(&mut bytes, value);
    }
    object(&mut bytes, "Backboard", |_| {});
    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Root")
    });
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "root");
        uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
        uint(
            bytes,
            "ViewModelInstanceViewModel",
            "viewModelPropertyId",
            0,
        );
        uint(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
    });
    object(&mut bytes, "ViewModelInstanceList", |bytes| {
        uint(bytes, "ViewModelInstanceList", "viewModelPropertyId", 1)
    });
    object(&mut bytes, "ViewModelInstanceListItem", |bytes| {
        uint(bytes, "ViewModelInstanceListItem", "viewModelId", 1);
        uint(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
    });
    object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
        string(bytes, "ViewModelPropertyViewModel", "name", "child");
        uint(
            bytes,
            "ViewModelPropertyViewModel",
            "viewModelReferenceId",
            1,
        );
    });
    object(&mut bytes, "ViewModelPropertyList", |bytes| {
        string(bytes, "ViewModelPropertyList", "name", "items")
    });
    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Child")
    });
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "shared");
        uint(bytes, "ViewModelInstance", "viewModelId", 1);
    });
    object(&mut bytes, "ViewModelInstanceString", |bytes| {
        uint(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
        string(
            bytes,
            "ViewModelInstanceString",
            "propertyValue",
            "shared value",
        );
    });
    object(&mut bytes, "ViewModelPropertyString", |bytes| {
        string(bytes, "ViewModelPropertyString", "name", "label")
    });
    object(&mut bytes, "Artboard", |_| {});
    bytes
}

fn import_bytes(bytes: &[u8]) -> *mut NuxFile {
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &mut file) },
        NuxStatus::Ok
    );
    file
}

fn owned_string(view: NuxStringView) -> String {
    if view.data.is_null() {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) };
    std::str::from_utf8(bytes).unwrap().to_owned()
}

fn find_property(
    catalog: *const NuxViewModelCatalog,
    info: NuxViewModelCatalogInfo,
    name: &str,
) -> (usize, NuxViewModelPropertyView) {
    for index in 0..info.property_count {
        let mut property = NuxViewModelPropertyView::default();
        assert_eq!(
            unsafe { nux_view_model_catalog_property(catalog, index, &mut property) },
            NuxStatus::Ok
        );
        if owned_string(property.name) == name {
            return (index, property);
        }
    }
    panic!("missing property {name}")
}

fn number(snapshot: *const NuxViewModelSnapshot, name: &str) -> f32 {
    let mut info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    for index in 0..info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        if owned_string(value.name) == name {
            assert_eq!(value.kind, NUX_VIEW_MODEL_VALUE_KIND_NUMBER);
            return value.number_value;
        }
    }
    panic!("missing snapshot value {name}")
}

fn snapshot(instance: *const NuxViewModelInstance) -> *mut NuxViewModelSnapshot {
    let mut snapshot = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_snapshot(instance, &mut snapshot) },
        NuxStatus::Ok
    );
    snapshot
}

fn nested_and_list_ids(snapshot: *const NuxViewModelSnapshot) -> (u64, Vec<u64>) {
    let mut info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    let mut nested = 0;
    let mut list = Vec::new();
    for index in 0..info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        match owned_string(value.name).as_str() {
            "child" => nested = value.referenced_instance_id,
            "items" => {
                for list_index in
                    value.first_list_item..value.first_list_item + value.list_item_count
                {
                    let mut id = 0;
                    assert_eq!(
                        unsafe { nux_view_model_snapshot_list_item(snapshot, list_index, &mut id) },
                        NuxStatus::Ok
                    );
                    list.push(id);
                }
            }
            _ => {}
        }
    }
    (nested, list)
}

#[test]
fn catalog_and_snapshot_are_owned_flat_projections() {
    let file = import("data_binding_test_2.riv");
    let mut catalog = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_view_model_catalog(file, &mut catalog) },
        NuxStatus::Ok
    );
    let mut info = NuxViewModelCatalogInfo::default();
    assert_eq!(
        unsafe { nux_view_model_catalog_info(catalog, &mut info) },
        NuxStatus::Ok
    );
    assert!(info.schema_count > 0);
    assert!(info.authored_instance_count > 0);
    let (_, number_property) = find_property(catalog, info, "num");

    let mut authored = NuxViewModelAuthoredInstanceView::default();
    let mut authored_catalog_index = None;
    for index in 0..info.authored_instance_count {
        assert_eq!(
            unsafe { nux_view_model_catalog_authored_instance(catalog, index, &mut authored) },
            NuxStatus::Ok
        );
        if authored.schema_index == number_property.schema_index {
            authored_catalog_index = Some(index);
            break;
        }
    }
    assert!(authored_catalog_index.is_some());

    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_view_model_instance_new_authored(
                file,
                authored.schema_index,
                authored.instance_index,
                &mut instance,
            )
        },
        NuxStatus::Ok
    );

    // Both projections retain/own everything they need after the import handle.
    assert_eq!(unsafe { nux_file_free(file) }, NuxStatus::Ok);
    let mut schema = NuxViewModelSchemaView::default();
    assert_eq!(
        unsafe { nux_view_model_catalog_schema(catalog, authored.schema_index, &mut schema) },
        NuxStatus::Ok
    );
    assert!(!owned_string(schema.name).is_empty());

    let snapshot = snapshot(instance);
    let original = number(snapshot, "num");
    assert!(original.is_finite());
    assert_eq!(
        unsafe { nux_view_model_instance_free(instance) },
        NuxStatus::Ok
    );
    assert_eq!(number(snapshot, "num"), original);

    unsafe {
        nux_view_model_snapshot_free(snapshot);
        nux_view_model_catalog_free(catalog);
    }
}

#[test]
fn shared_identity_and_atomic_batches_preserve_the_live_graph_on_error() {
    let file = import("data_binding_test_2.riv");
    let mut catalog = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_view_model_catalog(file, &mut catalog) },
        NuxStatus::Ok
    );
    let mut info = NuxViewModelCatalogInfo::default();
    assert_eq!(
        unsafe { nux_view_model_catalog_info(catalog, &mut info) },
        NuxStatus::Ok
    );
    let (_, property) = find_property(catalog, info, "num");
    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, property.schema_index, &mut instance) },
        NuxStatus::Ok
    );
    let mut shared = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_share(instance, &mut shared) },
        NuxStatus::Ok
    );
    let (mut identity, mut shared_identity) = (0, 0);
    assert_eq!(
        unsafe { nux_view_model_instance_identity(instance, &mut identity) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_identity(shared, &mut shared_identity) },
        NuxStatus::Ok
    );
    assert_ne!(identity, 0);
    assert_eq!(identity, shared_identity);

    let before = snapshot(instance);
    let original = number(before, "num");
    let valid_path = b"num";
    let missing_path = b"missing";
    let mutations = [
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            instance,
            path: NuxStringView {
                data: valid_path.as_ptr().cast(),
                len: valid_path.len(),
            },
            number_value: 91.0,
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            instance,
            path: NuxStringView {
                data: missing_path.as_ptr().cast(),
                len: missing_path.len(),
            },
            number_value: 12.0,
            ..NuxViewModelMutation::default()
        },
    ];
    let batch = NuxViewModelMutationBatch {
        mutations: mutations.as_ptr(),
        mutation_count: mutations.len(),
        ..NuxViewModelMutationBatch::default()
    };
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::NotFound
    );
    let mut result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.status, NuxStatus::NotFound);
    assert_eq!(result_info.applied_count, 0);
    assert!(!owned_string(result_info.code).is_empty());
    unsafe { nux_view_model_mutation_result_free(result) };

    let after_failure = snapshot(shared);
    assert_eq!(number(after_failure, "num"), original);

    let mutation = NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
        instance,
        path: NuxStringView {
            data: valid_path.as_ptr().cast(),
            len: valid_path.len(),
        },
        number_value: 91.0,
        ..NuxViewModelMutation::default()
    };
    let batch = NuxViewModelMutationBatch {
        mutations: &mutation,
        mutation_count: 1,
        ..NuxViewModelMutationBatch::default()
    };
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.applied_count, 1);
    let after_success = snapshot(shared);
    assert_eq!(number(after_success, "num"), 91.0);

    unsafe {
        nux_view_model_mutation_result_free(result);
        nux_view_model_snapshot_free(after_success);
        nux_view_model_snapshot_free(after_failure);
        nux_view_model_snapshot_free(before);
        nux_view_model_instance_free(shared);
        nux_view_model_instance_free(instance);
        nux_view_model_catalog_free(catalog);
        nux_file_free(file);
    }
}

#[test]
fn mutation_input_is_bounded_and_results_are_published_for_rejections() {
    let oversized = NuxViewModelMutationBatch {
        mutations: std::ptr::null(),
        mutation_count: 1_025,
        ..NuxViewModelMutationBatch::default()
    };
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&oversized, &mut result) },
        NuxStatus::LimitExceeded
    );
    assert!(!result.is_null());
    let mut info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut info) },
        NuxStatus::Ok
    );
    assert_eq!(info.status, NuxStatus::LimitExceeded);
    assert_eq!(info.applied_count, 0);
    unsafe { nux_view_model_mutation_result_free(result) };
}

#[test]
fn generic_instances_bind_only_to_compatible_occurrences_from_the_same_file() {
    let file = import("data_binding_test_2.riv");
    let other_file = import("data_binding_test_2.riv");
    let mut catalog = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_view_model_catalog(file, &mut catalog) },
        NuxStatus::Ok
    );
    let mut info = NuxViewModelCatalogInfo::default();
    assert_eq!(
        unsafe { nux_view_model_catalog_info(catalog, &mut info) },
        NuxStatus::Ok
    );
    let schema = find_property(catalog, info, "num").1.schema_index;
    let mut view_model = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, schema, &mut view_model) },
        NuxStatus::Ok
    );
    let (mut artboard, mut other_artboard) = (std::ptr::null_mut(), std::ptr::null_mut());
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_new(other_file, 0, &mut other_artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(other_artboard, view_model) },
        NuxStatus::HandleMismatch
    );
    unsafe {
        nux_artboard_instance_free(other_artboard);
        nux_artboard_instance_free(artboard);
        nux_view_model_instance_free(view_model);
        nux_view_model_catalog_free(catalog);
        nux_file_free(other_file);
        nux_file_free(file);
    }
}

#[test]
fn flattened_snapshot_preserves_shared_nested_and_list_identity() {
    let bytes = shared_nested_list_fixture();
    let file = import_bytes(&bytes);
    let mut root = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut root) },
        NuxStatus::Ok
    );
    let snapshot = snapshot(root);
    let mut info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    assert_eq!(info.instance_count, 2, "shared child appears once");
    assert_eq!(info.list_item_count, 1);
    let mut child_id = 0;
    let mut list_id = 0;
    for index in 0..info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        match owned_string(value.name).as_str() {
            "child" => child_id = value.referenced_instance_id,
            "items" => {
                assert_eq!(value.list_item_count, 1);
                assert_eq!(
                    unsafe {
                        nux_view_model_snapshot_list_item(
                            snapshot,
                            value.first_list_item,
                            &mut list_id,
                        )
                    },
                    NuxStatus::Ok
                );
            }
            _ => {}
        }
    }
    assert_ne!(child_id, 0);
    assert_eq!(
        child_id, list_id,
        "property and list preserve the same graph node"
    );

    unsafe {
        nux_view_model_snapshot_free(snapshot);
        nux_view_model_instance_free(root);
        nux_file_free(file);
    }
}

#[test]
fn structural_mutations_preserve_identity_and_failed_batches_preserve_topology() {
    let bytes = shared_nested_list_fixture();
    let file = import_bytes(&bytes);
    let mut root = std::ptr::null_mut();
    let mut replacement = std::ptr::null_mut();
    let mut third = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut root) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 1, 0, &mut replacement) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, 1, &mut third) },
        NuxStatus::Ok
    );

    let child = b"child";
    let items = b"items";
    let success = [
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
            instance: root,
            related_instance: replacement,
            path: NuxStringView {
                data: child.as_ptr().cast(),
                len: child.len(),
            },
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_SET,
            instance: root,
            related_instance: replacement,
            path: NuxStringView {
                data: items.as_ptr().cast(),
                len: items.len(),
            },
            index: 0,
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
            instance: root,
            related_instance: third,
            path: NuxStringView {
                data: items.as_ptr().cast(),
                len: items.len(),
            },
            index: 1,
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_SWAP,
            instance: root,
            path: NuxStringView {
                data: items.as_ptr().cast(),
                len: items.len(),
            },
            index: 0,
            second_index: 1,
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE,
            instance: root,
            path: NuxStringView {
                data: items.as_ptr().cast(),
                len: items.len(),
            },
            index: 0,
            second_index: 1,
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
            instance: root,
            path: NuxStringView {
                data: items.as_ptr().cast(),
                len: items.len(),
            },
            index: 1,
            ..NuxViewModelMutation::default()
        },
    ];
    let mut result = std::ptr::null_mut();
    let success_batch = NuxViewModelMutationBatch {
        mutations: success.as_ptr(),
        mutation_count: success.len(),
        ..NuxViewModelMutationBatch::default()
    };
    assert_eq!(
        unsafe { nux_view_model_mutate(&success_batch, &mut result) },
        NuxStatus::Ok
    );
    unsafe { nux_view_model_mutation_result_free(result) };
    let after_success = snapshot(root);
    let success_ids = nested_and_list_ids(after_success);
    assert_eq!(success_ids.1.len(), 1);
    assert_eq!(success_ids.0, success_ids.1[0]);

    let invalid = [
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
            instance: root,
            related_instance: third,
            path: NuxStringView {
                data: child.as_ptr().cast(),
                len: child.len(),
            },
            ..NuxViewModelMutation::default()
        },
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
            instance: root,
            path: NuxStringView {
                data: items.as_ptr().cast(),
                len: items.len(),
            },
            index: 99,
            ..NuxViewModelMutation::default()
        },
    ];
    let invalid_batch = NuxViewModelMutationBatch {
        mutations: invalid.as_ptr(),
        mutation_count: invalid.len(),
        ..NuxViewModelMutationBatch::default()
    };
    assert_eq!(
        unsafe { nux_view_model_mutate(&invalid_batch, &mut result) },
        NuxStatus::InvalidArgument
    );
    unsafe { nux_view_model_mutation_result_free(result) };
    let after_failure = snapshot(root);
    assert_eq!(nested_and_list_ids(after_failure), success_ids);

    unsafe {
        nux_view_model_snapshot_free(after_failure);
        nux_view_model_snapshot_free(after_success);
        nux_view_model_instance_free(third);
        nux_view_model_instance_free(replacement);
        nux_view_model_instance_free(root);
        nux_file_free(file);
    }
}

#[test]
fn released_instance_identity_is_never_reused() {
    let bytes = shared_nested_list_fixture();
    let file = import_bytes(&bytes);
    let mut first = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, 1, &mut first) },
        NuxStatus::Ok
    );
    let mut first_id = 0;
    assert_eq!(
        unsafe { nux_view_model_instance_identity(first, &mut first_id) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_free(first) },
        NuxStatus::Ok
    );
    let mut second = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, 1, &mut second) },
        NuxStatus::Ok
    );
    let mut second_id = 0;
    assert_eq!(
        unsafe { nux_view_model_instance_identity(second, &mut second_id) },
        NuxStatus::Ok
    );
    assert_ne!(first_id, second_id);
    unsafe {
        nux_view_model_instance_free(second);
        nux_file_free(file);
    }
}

#[test]
fn scalar_text_trigger_enum_and_image_mutations_round_trip_through_owned_snapshot() {
    let bytes = scalar_fixture();
    let file = import_bytes(&bytes);
    let mut catalog = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_view_model_catalog(file, &mut catalog) },
        NuxStatus::Ok
    );
    let mut catalog_info = NuxViewModelCatalogInfo::default();
    assert_eq!(
        unsafe { nux_view_model_catalog_info(catalog, &mut catalog_info) },
        NuxStatus::Ok
    );
    let expected = [
        ("text", NUX_VIEW_MODEL_VALUE_KIND_STRING),
        ("number", NUX_VIEW_MODEL_VALUE_KIND_NUMBER),
        ("enabled", NUX_VIEW_MODEL_VALUE_KIND_BOOL),
        ("tint", NUX_VIEW_MODEL_VALUE_KIND_COLOR),
        ("fire", NUX_VIEW_MODEL_VALUE_KIND_TRIGGER),
        ("image", NUX_VIEW_MODEL_VALUE_KIND_IMAGE),
        ("choice", NUX_VIEW_MODEL_VALUE_KIND_ENUM),
    ];
    for (name, kind) in expected {
        assert_eq!(find_property(catalog, catalog_info, name).1.kind, kind);
    }
    assert_eq!(catalog_info.enum_label_count, 2);

    let mut instance = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_schema_default(file, 0, &mut instance) },
        NuxStatus::Ok
    );
    let text = b"updated text";
    let paths: [&[u8]; 7] = [
        b"text", b"number", b"enabled", b"tint", b"fire", b"image", b"choice",
    ];
    let kinds = [
        NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
        NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
        NUX_VIEW_MODEL_MUTATION_KIND_SET_BOOL,
        NUX_VIEW_MODEL_MUTATION_KIND_SET_COLOR,
        NUX_VIEW_MODEL_MUTATION_KIND_FIRE_TRIGGER,
        NUX_VIEW_MODEL_MUTATION_KIND_SET_IMAGE,
        NUX_VIEW_MODEL_MUTATION_KIND_SET_ENUM,
    ];
    let mut mutations = Vec::new();
    for (index, (path, kind)) in paths.into_iter().zip(kinds).enumerate() {
        mutations.push(NuxViewModelMutation {
            kind,
            instance,
            path: NuxStringView {
                data: path.as_ptr().cast(),
                len: path.len(),
            },
            bytes_value: if index == 0 {
                NuxByteView {
                    data: text.as_ptr(),
                    len: text.len(),
                }
            } else {
                NuxByteView::default()
            },
            number_value: 11.5,
            bool_value: 1,
            integer_value: if index == 5 { 42 } else { 1 },
            ..NuxViewModelMutation::default()
        });
    }
    let batch = NuxViewModelMutationBatch {
        mutations: mutations.as_ptr(),
        mutation_count: mutations.len(),
        ..NuxViewModelMutationBatch::default()
    };
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::Ok
    );
    let snapshot = snapshot(instance);
    let mut snapshot_info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut snapshot_info) },
        NuxStatus::Ok
    );
    let mut observed = 0;
    for index in 0..snapshot_info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        match owned_string(value.name).as_str() {
            "text" => {
                let bytes = unsafe {
                    std::slice::from_raw_parts(value.bytes_value.data, value.bytes_value.len)
                };
                assert_eq!(bytes, text);
                observed += 1;
            }
            "number" => {
                assert_eq!(value.number_value, 11.5);
                observed += 1;
            }
            "enabled" => {
                assert_eq!(value.bool_value, 1);
                observed += 1;
            }
            "tint" | "choice" => {
                assert_eq!(value.integer_value, 1);
                observed += 1;
            }
            "fire" => {
                assert_eq!(value.integer_value, 3);
                observed += 1;
            }
            "image" => {
                assert_eq!(value.integer_value, 42);
                observed += 1;
            }
            _ => {}
        }
    }
    assert_eq!(observed, 7);
    unsafe {
        nux_view_model_snapshot_free(snapshot);
        nux_view_model_mutation_result_free(result);
        nux_view_model_instance_free(instance);
        nux_view_model_catalog_free(catalog);
        nux_file_free(file);
    }
}

#[test]
fn root_text_run_batch_prevalidates_before_any_write() {
    let file = import("background_measure.riv");
    let mut artboard = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    let headline = b"nameRun";
    let missing = b"missing";
    let changed_text = b"changed";
    let invalid = [
        NuxTextRunMutation {
            name: NuxStringView {
                data: headline.as_ptr().cast(),
                len: headline.len(),
            },
            text: NuxByteView {
                data: changed_text.as_ptr(),
                len: changed_text.len(),
            },
        },
        NuxTextRunMutation {
            name: NuxStringView {
                data: missing.as_ptr().cast(),
                len: missing.len(),
            },
            text: NuxByteView {
                data: changed_text.as_ptr(),
                len: changed_text.len(),
            },
        },
    ];
    let invalid_batch = NuxTextRunMutationBatch {
        mutations: invalid.as_ptr(),
        mutation_count: invalid.len(),
        ..NuxTextRunMutationBatch::default()
    };
    let mut changed = 99;
    assert_eq!(
        unsafe { nux_artboard_instance_set_text_runs(artboard, &invalid_batch, &mut changed) },
        NuxStatus::NotFound
    );
    assert_eq!(changed, 0);

    let mutation = NuxTextRunMutation {
        name: NuxStringView {
            data: headline.as_ptr().cast(),
            len: headline.len(),
        },
        text: NuxByteView {
            data: changed_text.as_ptr(),
            len: changed_text.len(),
        },
    };
    let batch = NuxTextRunMutationBatch {
        mutations: &mutation,
        mutation_count: 1,
        ..NuxTextRunMutationBatch::default()
    };
    assert_eq!(
        unsafe { nux_artboard_instance_set_text_runs(artboard, &batch, &mut changed) },
        NuxStatus::Ok
    );
    assert_eq!(changed, 1, "failed batch left the original text intact");
    assert_eq!(
        unsafe { nux_artboard_instance_set_text_runs(artboard, &batch, &mut changed) },
        NuxStatus::Ok
    );
    assert_eq!(changed, 0);

    unsafe {
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
