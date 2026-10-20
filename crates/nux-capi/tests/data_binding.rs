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
    let callbacks = NuxRenderCallbacks::default();
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &callbacks, &mut file) },
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
    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 0)
    });
    // Exact File::viewModelInstanceListItem resolves the list item's implicit
    // artboard by the inserted instance's view-model id.
    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 1)
    });
    bytes
}

fn nested_replacement_fixture() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 0x554e_4955, 0] {
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
        uint(bytes, "ViewModelInstanceListItem", "viewModelId", 2);
        uint(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
    });
    object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
        string(bytes, "ViewModelPropertyViewModel", "name", "container");
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
        string(bytes, "ViewModel", "name", "Container")
    });
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "container");
        uint(bytes, "ViewModelInstance", "viewModelId", 1);
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
    object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
        uint(
            bytes,
            "ViewModelInstanceViewModel",
            "viewModelPropertyId",
            1,
        );
        uint(bytes, "ViewModelInstanceViewModel", "propertyValue", 999);
    });
    object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
        string(bytes, "ViewModelPropertyViewModel", "name", "selected");
        uint(
            bytes,
            "ViewModelPropertyViewModel",
            "viewModelReferenceId",
            2,
        );
    });
    object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
        string(bytes, "ViewModelPropertyViewModel", "name", "peer");
        uint(
            bytes,
            "ViewModelPropertyViewModel",
            "viewModelReferenceId",
            1,
        );
    });

    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Product")
    });
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "product");
        uint(bytes, "ViewModelInstance", "viewModelId", 2);
    });
    object(&mut bytes, "ViewModelInstanceString", |bytes| {
        uint(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
        string(
            bytes,
            "ViewModelInstanceString",
            "propertyValue",
            "authored",
        );
    });
    object(&mut bytes, "ViewModelPropertyString", |bytes| {
        string(bytes, "ViewModelPropertyString", "name", "sku")
    });

    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 0)
    });
    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 2)
    });
    bytes
}

fn import_bytes(bytes: &[u8]) -> *mut NuxFile {
    let callbacks = NuxRenderCallbacks::default();
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_file_import(bytes.as_ptr(), bytes.len(), &callbacks, &mut file) },
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

fn snapshot_edges(snapshot: *const NuxViewModelSnapshot) -> Vec<(u64, String, u64, Vec<u64>)> {
    let mut info = NuxViewModelSnapshotInfo::default();
    assert_eq!(
        unsafe { nux_view_model_snapshot_info(snapshot, &mut info) },
        NuxStatus::Ok
    );
    let mut edges = Vec::new();
    for index in 0..info.value_count {
        let mut value = NuxViewModelSnapshotValueView::default();
        assert_eq!(
            unsafe { nux_view_model_snapshot_value(snapshot, index, &mut value) },
            NuxStatus::Ok
        );
        if value.kind != NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL
            && value.kind != NUX_VIEW_MODEL_VALUE_KIND_LIST
        {
            continue;
        }
        let mut items = Vec::new();
        for list_index in value.first_list_item..value.first_list_item + value.list_item_count {
            let mut id = 0;
            assert_eq!(
                unsafe { nux_view_model_snapshot_list_item(snapshot, list_index, &mut id) },
                NuxStatus::Ok
            );
            items.push(id);
        }
        edges.push((
            value.owner_instance_id,
            owned_string(value.name),
            value.referenced_instance_id,
            items,
        ));
    }
    edges.sort();
    edges
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

    let successful_mutations = [
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            instance,
            path: NuxStringView {
                data: valid_path.as_ptr().cast(),
                len: valid_path.len(),
            },
            number_value: 45.0,
            ..NuxViewModelMutation::default()
        },
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
    ];
    let correlation_id = 0xa11c_eu64;
    let batch = NuxViewModelMutationBatch {
        mutations: successful_mutations.as_ptr(),
        mutation_count: successful_mutations.len(),
        correlation_id,
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
    assert_eq!(result_info.applied_count, 2);
    assert_eq!(result_info.correlation_id, correlation_id);
    assert_eq!(result_info.change_count, 2);
    let after_success = snapshot(shared);
    assert_eq!(number(after_success, "num"), 91.0);

    for (index, expected) in [45.0, 91.0].into_iter().enumerate() {
        let mut change = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_view_model_mutation_result_change(result, index, &mut change) },
            NuxStatus::Ok
        );
        assert_eq!(change.origin, NUX_VIEW_MODEL_CHANGE_ORIGIN_CALLER);
        assert_eq!(change.correlation_id, correlation_id);
        assert_eq!(change.owner_instance_id, identity);
        assert_eq!(change.property_index, 1);
        assert_eq!(change.kind, NUX_VIEW_MODEL_VALUE_KIND_NUMBER);
        assert_eq!(change.number_value, expected);
    }

    unsafe {
        nux_view_model_snapshot_free(after_success);
        nux_view_model_snapshot_free(after_failure);
        nux_view_model_snapshot_free(before);
        nux_view_model_instance_free(shared);
        nux_view_model_instance_free(instance);
        nux_view_model_catalog_free(catalog);
        nux_file_free(file);
    }
    // The journal owns typed after-values and durable identities rather than
    // borrowing the file or live view-model graph.
    let mut retained = NuxViewModelChangeView::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_change(result, 1, &mut retained) },
        NuxStatus::Ok
    );
    assert_eq!(retained.owner_instance_id, identity);
    assert_eq!(retained.number_value, 91.0);
    unsafe { nux_view_model_mutation_result_free(result) };
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
fn linked_child_and_list_item_mutations_invalidate_the_bound_root_occurrence() {
    let bytes = shared_nested_list_fixture();
    let file = import_bytes(&bytes);
    let mut root = std::ptr::null_mut();
    let mut sibling_root = std::ptr::null_mut();
    let mut child = std::ptr::null_mut();
    let mut list_item = std::ptr::null_mut();
    let mut replacement = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, 0, &mut root) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_new(file, 0, &mut sibling_root) },
        NuxStatus::Ok
    );
    for out in [&mut child, &mut list_item, &mut replacement] {
        assert_eq!(
            unsafe { nux_view_model_instance_new(file, 1, out) },
            NuxStatus::Ok
        );
    }
    let mut artboard = std::ptr::null_mut();
    let mut player = std::ptr::null_mut();
    let mut sibling_artboard = std::ptr::null_mut();
    let mut sibling_player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, root) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_new_static(artboard, &mut player) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut sibling_artboard) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(sibling_artboard, sibling_root) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_new_static(sibling_artboard, &mut sibling_player) },
        NuxStatus::Ok
    );

    let step_and_ack = |player| {
        let mut result = std::ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(player, &NuxPlayerStep::default(), &mut result) },
            NuxStatus::Ok
        );
        let mut scheduling = NuxPlayerSchedulingInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_scheduling(result, &mut scheduling) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_acknowledge_presented(player, scheduling.render_revision) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_step_result_free(result) },
            NuxStatus::Ok
        );
        scheduling.render_revision
    };
    let apply = |mutation: NuxViewModelMutation| {
        let batch = NuxViewModelMutationBatch {
            mutations: &mutation,
            mutation_count: 1,
            ..NuxViewModelMutationBatch::default()
        };
        let mut result = std::ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_mutate(&batch, &mut result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_view_model_mutation_result_free(result) },
            NuxStatus::Ok
        );
    };

    let initial_revision = step_and_ack(player);
    let child_path = b"child";
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
        instance: root,
        path: NuxStringView {
            data: child_path.as_ptr().cast(),
            len: child_path.len(),
        },
        related_instance: child,
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, initial_revision) },
        NuxStatus::HandleMismatch
    );
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
        instance: sibling_root,
        path: NuxStringView {
            data: child_path.as_ptr().cast(),
            len: child_path.len(),
        },
        related_instance: child,
        ..NuxViewModelMutation::default()
    });
    let shared_root_revision = step_and_ack(player);
    let shared_sibling_revision = step_and_ack(sibling_player);
    let nested_path = b"child/label";
    let shared_value = b"shared nested change";
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
        instance: root,
        path: NuxStringView {
            data: nested_path.as_ptr().cast(),
            len: nested_path.len(),
        },
        bytes_value: NuxByteView {
            data: shared_value.as_ptr(),
            len: shared_value.len(),
        },
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, shared_root_revision) },
        NuxStatus::HandleMismatch
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(sibling_player, shared_sibling_revision) },
        NuxStatus::HandleMismatch,
        "a nested mutation through one root invalidates every root sharing the changed owner"
    );

    let no_op_root_revision = step_and_ack(player);
    let no_op_sibling_revision = step_and_ack(sibling_player);
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
        instance: root,
        path: NuxStringView {
            data: nested_path.as_ptr().cast(),
            len: nested_path.len(),
        },
        bytes_value: NuxByteView {
            data: shared_value.as_ptr(),
            len: shared_value.len(),
        },
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, no_op_root_revision) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(sibling_player, no_op_sibling_revision) },
        NuxStatus::Ok,
        "a no-op batch preserves every current presentation acknowledgement"
    );

    let legacy_no_op_root_revision = step_and_ack(player);
    let legacy_no_op_sibling_revision = step_and_ack(sibling_player);
    let nested_path_c = std::ffi::CString::new("child/label").unwrap();
    let shared_value_c = std::ffi::CString::new("shared nested change").unwrap();
    assert_eq!(
        unsafe {
            nux_view_model_instance_set_string(
                root,
                nested_path_c.as_ptr(),
                shared_value_c.as_ptr(),
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, legacy_no_op_root_revision) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(sibling_player, legacy_no_op_sibling_revision) },
        NuxStatus::Ok,
        "a no-op legacy setter preserves every current presentation acknowledgement"
    );
    let legacy_nested_root_revision = step_and_ack(player);
    let legacy_nested_sibling_revision = step_and_ack(sibling_player);
    let legacy_nested_value = std::ffi::CString::new("legacy nested actual change").unwrap();
    assert_eq!(
        unsafe {
            nux_view_model_instance_set_string(
                root,
                nested_path_c.as_ptr(),
                legacy_nested_value.as_ptr(),
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, legacy_nested_root_revision) },
        NuxStatus::HandleMismatch
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(sibling_player, legacy_nested_sibling_revision) },
        NuxStatus::HandleMismatch,
        "a changed legacy nested setter invalidates every root sharing the changed owner"
    );
    let label = std::ffi::CString::new("label").unwrap();
    let linked_revision = step_and_ack(player);
    let batch_child_value = b"batch child change";
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
        instance: child,
        path: NuxStringView {
            data: b"label".as_ptr().cast(),
            len: b"label".len(),
        },
        bytes_value: NuxByteView {
            data: batch_child_value.as_ptr(),
            len: batch_child_value.len(),
        },
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, linked_revision) },
        NuxStatus::HandleMismatch,
        "an atomic child mutation invalidates its bound root"
    );

    let before_legacy_child = step_and_ack(player);
    let legacy_value = std::ffi::CString::new("legacy child change").unwrap();
    assert_eq!(
        unsafe { nux_view_model_instance_set_string(child, label.as_ptr(), legacy_value.as_ptr()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, before_legacy_child) },
        NuxStatus::HandleMismatch,
        "a legacy child setter invalidates its bound root"
    );

    let items_path = b"items";
    let before_alias_list = step_and_ack(player);
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
        instance: root,
        path: NuxStringView {
            data: items_path.as_ptr().cast(),
            len: items_path.len(),
        },
        related_instance: child,
        index: 0,
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, before_alias_list) },
        NuxStatus::HandleMismatch
    );

    let before_replace = step_and_ack(player);
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
        instance: root,
        path: NuxStringView {
            data: child_path.as_ptr().cast(),
            len: child_path.len(),
        },
        related_instance: replacement,
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, before_replace) },
        NuxStatus::HandleMismatch
    );
    let after_replace = step_and_ack(player);
    let still_linked_value = std::ffi::CString::new("still linked through list").unwrap();
    assert_eq!(
        unsafe {
            nux_view_model_instance_set_string(child, label.as_ptr(), still_linked_value.as_ptr())
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, after_replace) },
        NuxStatus::Ok,
        "exact C++ parent bookkeeping deduplicates equal parents, then removing either edge removes that parent even while another edge remains"
    );

    let before_detach = step_and_ack(player);
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
        instance: root,
        path: NuxStringView {
            data: items_path.as_ptr().cast(),
            len: items_path.len(),
        },
        index: 0,
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, before_detach) },
        NuxStatus::HandleMismatch
    );
    let detached_revision = step_and_ack(player);
    let detached_value = std::ffi::CString::new("detached child change").unwrap();
    assert_eq!(
        unsafe {
            nux_view_model_instance_set_string(child, label.as_ptr(), detached_value.as_ptr())
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, detached_revision) },
        NuxStatus::Ok,
        "the runtime parent relay stops invalidating a root after unlink"
    );

    let before_list = step_and_ack(player);
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
        instance: root,
        path: NuxStringView {
            data: items_path.as_ptr().cast(),
            len: items_path.len(),
        },
        related_instance: list_item,
        index: 0,
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, before_list) },
        NuxStatus::HandleMismatch
    );
    let list_revision = step_and_ack(player);
    let batch_value = b"batch list-item change";
    apply(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
        instance: list_item,
        path: NuxStringView {
            data: b"label".as_ptr().cast(),
            len: b"label".len(),
        },
        bytes_value: NuxByteView {
            data: batch_value.as_ptr(),
            len: batch_value.len(),
        },
        ..NuxViewModelMutation::default()
    });
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, list_revision) },
        NuxStatus::HandleMismatch,
        "an atomic list-item mutation invalidates its bound root"
    );

    let before_legacy_list_item = step_and_ack(player);
    let legacy_list_value = std::ffi::CString::new("legacy list-item change").unwrap();
    assert_eq!(
        unsafe {
            nux_view_model_instance_set_string(
                list_item,
                label.as_ptr(),
                legacy_list_value.as_ptr(),
            )
        },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, before_legacy_list_item) },
        NuxStatus::HandleMismatch,
        "a legacy list-item setter invalidates its bound root"
    );

    unsafe {
        nux_player_free(sibling_player);
        nux_artboard_instance_free(sibling_artboard);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_view_model_instance_free(list_item);
        nux_view_model_instance_free(replacement);
        nux_view_model_instance_free(child);
        nux_view_model_instance_free(sibling_root);
        nux_view_model_instance_free(root);
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
    let mut result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.change_count, success.len());
    let mut replacement_id = 0;
    let mut third_id = 0;
    assert_eq!(
        unsafe { nux_view_model_instance_identity(replacement, &mut replacement_id) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_identity(third, &mut third_id) },
        NuxStatus::Ok
    );
    let mut linked = NuxViewModelChangeView::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_change(result, 0, &mut linked) },
        NuxStatus::Ok
    );
    assert_eq!(linked.kind, NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL);
    assert_eq!(linked.referenced_instance_id, replacement_id);
    for (change_index, expected_items) in [
        vec![replacement_id],
        vec![replacement_id, third_id],
        vec![third_id, replacement_id],
        vec![replacement_id, third_id],
        vec![replacement_id],
    ]
    .into_iter()
    .enumerate()
    {
        let mut change = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_view_model_mutation_result_change(result, change_index + 1, &mut change) },
            NuxStatus::Ok
        );
        assert_eq!(change.kind, NUX_VIEW_MODEL_VALUE_KIND_LIST);
        assert_eq!(change.list_item_count, expected_items.len());
        for (item_index, expected_id) in expected_items.into_iter().enumerate() {
            let mut item_id = 0;
            assert_eq!(
                unsafe {
                    nux_view_model_mutation_result_change_list_item(
                        result,
                        change_index + 1,
                        item_index,
                        &mut item_id,
                    )
                },
                NuxStatus::Ok
            );
            assert_eq!(item_id, expected_id);
        }
    }
    unsafe { nux_view_model_mutation_result_free(result) };
    let after_success = snapshot(root);
    let success_ids = nested_and_list_ids(after_success);
    assert_eq!(success_ids.1.len(), 1);
    assert_eq!(success_ids.0, success_ids.1[0]);
    assert_eq!(
        success_ids.0, replacement_id,
        "snapshot references retain native identity after replacement and list edits"
    );
    assert!(
        !success_ids.1.contains(&third_id),
        "removed list entries cannot survive in the current snapshot"
    );

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
    result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.change_count, 0);
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
fn nested_view_model_replacement_is_atomic_cycle_safe_and_preserves_shared_identity() {
    let bytes = nested_replacement_fixture();
    let file = import_bytes(&bytes);
    let mut root = std::ptr::null_mut();
    let mut replacement = std::ptr::null_mut();
    let mut third = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 0, 0, &mut root) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_new_authored(file, 2, 0, &mut replacement) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_new_schema_default(file, 2, &mut third) },
        NuxStatus::Ok
    );

    let mut root_id = 0;
    let mut replacement_id = 0;
    assert_eq!(
        unsafe { nux_view_model_instance_identity(root, &mut root_id) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_identity(replacement, &mut replacement_id) },
        NuxStatus::Ok
    );
    let selected = b"container/selected";
    let items = b"items";
    let success = [
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
            instance: root,
            related_instance: replacement,
            path: NuxStringView {
                data: selected.as_ptr().cast(),
                len: selected.len(),
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
    ];
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_view_model_mutate(
                &NuxViewModelMutationBatch {
                    mutations: success.as_ptr(),
                    mutation_count: success.len(),
                    ..NuxViewModelMutationBatch::default()
                },
                &mut result,
            )
        },
        NuxStatus::Ok
    );
    let mut result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.change_count, success.len());
    let mut replacement_change = None;
    let mut list_change = None;
    for change_index in 0..result_info.change_count {
        let mut change = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_view_model_mutation_result_change(result, change_index, &mut change) },
            NuxStatus::Ok
        );
        match change.kind {
            NUX_VIEW_MODEL_VALUE_KIND_VIEW_MODEL => replacement_change = Some(change),
            NUX_VIEW_MODEL_VALUE_KIND_LIST => list_change = Some(change),
            kind => panic!("unexpected structural change kind {kind}"),
        }
    }
    let replacement_change = replacement_change.expect("nested replacement change");
    assert_ne!(replacement_change.owner_instance_id, root_id);
    assert_eq!(replacement_change.property_index, 0);
    assert_eq!(replacement_change.referenced_instance_id, replacement_id);
    let list_change = list_change.expect("root list change");
    assert_eq!(list_change.owner_instance_id, root_id);
    assert_eq!(list_change.property_index, 1);
    unsafe { nux_view_model_mutation_result_free(result) };

    let after_success = snapshot(root);
    let success_edges = snapshot_edges(after_success);
    let container_id = success_edges
        .iter()
        .find(|(owner, name, _, _)| *owner == root_id && name == "container")
        .map(|(_, _, child, _)| *child)
        .expect("root retains its immediate container");
    let selected_id = success_edges
        .iter()
        .find(|(owner, name, _, _)| *owner == container_id && name == "selected")
        .map(|(_, _, child, _)| *child)
        .expect("container retains its selected product");
    let list_ids = success_edges
        .iter()
        .find(|(owner, name, _, _)| *owner == root_id && name == "items")
        .map(|(_, _, _, list)| list.as_slice())
        .expect("root retains its product list");
    assert_ne!(selected_id, 0);
    assert_eq!(list_ids, [selected_id]);

    let invalid = [
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
            instance: root,
            related_instance: third,
            path: NuxStringView {
                data: selected.as_ptr().cast(),
                len: selected.len(),
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
    result = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_view_model_mutate(
                &NuxViewModelMutationBatch {
                    mutations: invalid.as_ptr(),
                    mutation_count: invalid.len(),
                    ..NuxViewModelMutationBatch::default()
                },
                &mut result,
            )
        },
        NuxStatus::InvalidArgument
    );
    result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.change_count, 0);
    unsafe { nux_view_model_mutation_result_free(result) };
    let after_failure = snapshot(root);
    assert_eq!(snapshot_edges(after_failure), success_edges);

    let wrong_schema = NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
        instance: root,
        related_instance: root,
        path: NuxStringView {
            data: selected.as_ptr().cast(),
            len: selected.len(),
        },
        ..NuxViewModelMutation::default()
    };
    result = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_view_model_mutate(
                &NuxViewModelMutationBatch {
                    mutations: &wrong_schema,
                    mutation_count: 1,
                    ..NuxViewModelMutationBatch::default()
                },
                &mut result,
            )
        },
        NuxStatus::InvalidArgument
    );
    result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.change_count, 0);
    unsafe { nux_view_model_mutation_result_free(result) };
    let after_wrong_schema = snapshot(root);
    assert_eq!(snapshot_edges(after_wrong_schema), success_edges);

    let mut first_container = std::ptr::null_mut();
    let mut second_container = std::ptr::null_mut();
    for out in [&mut first_container, &mut second_container] {
        assert_eq!(
            unsafe { nux_view_model_instance_new_authored(file, 1, 0, out) },
            NuxStatus::Ok
        );
    }
    let root_container = b"container";
    let peer = b"peer";
    for (owner, path, child) in [
        (root, root_container.as_slice(), first_container),
        (second_container, peer.as_slice(), first_container),
    ] {
        let mutation = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
            instance: owner,
            related_instance: child,
            path: NuxStringView {
                data: path.as_ptr().cast(),
                len: path.len(),
            },
            ..NuxViewModelMutation::default()
        };
        result = std::ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_view_model_mutate(
                    &NuxViewModelMutationBatch {
                        mutations: &mutation,
                        mutation_count: 1,
                        ..NuxViewModelMutationBatch::default()
                    },
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        unsafe { nux_view_model_mutation_result_free(result) };
    }
    let before_cycle = snapshot(root);
    let before_cycle_edges = snapshot_edges(before_cycle);
    let nested_peer = b"container/peer";
    let cycle = NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_VIEW_MODEL,
        instance: root,
        related_instance: second_container,
        path: NuxStringView {
            data: nested_peer.as_ptr().cast(),
            len: nested_peer.len(),
        },
        ..NuxViewModelMutation::default()
    };
    result = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_view_model_mutate(
                &NuxViewModelMutationBatch {
                    mutations: &cycle,
                    mutation_count: 1,
                    ..NuxViewModelMutationBatch::default()
                },
                &mut result,
            )
        },
        NuxStatus::InvalidArgument
    );
    result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.change_count, 0);
    unsafe { nux_view_model_mutation_result_free(result) };
    let after_cycle = snapshot(root);
    assert_eq!(snapshot_edges(after_cycle), before_cycle_edges);

    unsafe {
        nux_view_model_snapshot_free(after_cycle);
        nux_view_model_snapshot_free(before_cycle);
        nux_view_model_snapshot_free(after_wrong_schema);
        nux_view_model_snapshot_free(after_failure);
        nux_view_model_snapshot_free(after_success);
        nux_view_model_instance_free(second_container);
        nux_view_model_instance_free(first_container);
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
    let final_text = b"final text";
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
    mutations.push(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
        instance,
        path: NuxStringView {
            data: b"text".as_ptr().cast(),
            len: b"text".len(),
        },
        bytes_value: NuxByteView {
            data: final_text.as_ptr(),
            len: final_text.len(),
        },
        ..NuxViewModelMutation::default()
    });
    mutations.push(NuxViewModelMutation {
        kind: NUX_VIEW_MODEL_MUTATION_KIND_FIRE_TRIGGER,
        instance,
        path: NuxStringView {
            data: b"fire".as_ptr().cast(),
            len: b"fire".len(),
        },
        ..NuxViewModelMutation::default()
    });
    let correlation_id = 0x51a1_7u64;
    let batch = NuxViewModelMutationBatch {
        mutations: mutations.as_ptr(),
        mutation_count: mutations.len(),
        correlation_id,
        ..NuxViewModelMutationBatch::default()
    };
    let mut result = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_mutate(&batch, &mut result) },
        NuxStatus::Ok
    );
    let mut result_info = NuxViewModelMutationResultInfo::default();
    assert_eq!(
        unsafe { nux_view_model_mutation_result_info(result, &mut result_info) },
        NuxStatus::Ok
    );
    assert_eq!(result_info.correlation_id, correlation_id);
    assert_eq!(result_info.change_count, mutations.len());
    for (index, kind, expected_bytes, expected_integer) in [
        (
            0,
            NUX_VIEW_MODEL_VALUE_KIND_STRING,
            Some(text.as_slice()),
            0,
        ),
        (4, NUX_VIEW_MODEL_VALUE_KIND_TRIGGER, None, 3),
        (
            7,
            NUX_VIEW_MODEL_VALUE_KIND_STRING,
            Some(final_text.as_slice()),
            0,
        ),
        (8, NUX_VIEW_MODEL_VALUE_KIND_TRIGGER, None, 4),
    ] {
        let mut change = NuxViewModelChangeView::default();
        assert_eq!(
            unsafe { nux_view_model_mutation_result_change(result, index, &mut change) },
            NuxStatus::Ok
        );
        assert_eq!(change.origin, NUX_VIEW_MODEL_CHANGE_ORIGIN_CALLER);
        assert_eq!(change.correlation_id, correlation_id);
        assert_eq!(change.kind, kind);
        if let Some(expected) = expected_bytes {
            assert_eq!(
                unsafe {
                    std::slice::from_raw_parts(change.bytes_value.data, change.bytes_value.len)
                },
                expected
            );
        } else {
            assert_eq!(change.integer_value, expected_integer);
        }
    }
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
                assert_eq!(bytes, final_text);
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
                assert_eq!(value.integer_value, 4);
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
    let mut player = std::ptr::null_mut();
    assert_eq!(
        unsafe { nux_player_new_static(artboard, &mut player) },
        NuxStatus::Ok
    );
    let step_and_ack = |player| {
        let mut result = std::ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(player, &NuxPlayerStep::default(), &mut result) },
            NuxStatus::Ok
        );
        let mut scheduling = NuxPlayerSchedulingInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_scheduling(result, &mut scheduling) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_acknowledge_presented(player, scheduling.render_revision) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_step_result_free(result) },
            NuxStatus::Ok
        );
        scheduling.render_revision
    };
    let initial_revision = step_and_ack(player);
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
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, initial_revision) },
        NuxStatus::Ok,
        "a rejected text batch preserves current render demand"
    );

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
        unsafe { nux_player_acknowledge_presented(player, initial_revision) },
        NuxStatus::HandleMismatch,
        "a committed text change invalidates the acknowledged occurrence revision"
    );
    let changed_revision = step_and_ack(player);
    assert_eq!(
        unsafe { nux_artboard_instance_set_text_runs(artboard, &batch, &mut changed) },
        NuxStatus::Ok
    );
    assert_eq!(changed, 0);
    assert_eq!(
        unsafe { nux_player_acknowledge_presented(player, changed_revision) },
        NuxStatus::Ok,
        "a no-op text batch preserves current render demand"
    );

    unsafe {
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

// A reverse measurement feeds another target and a second observation. This
// models layout-dependent paint without fonts, a GPU, or an SDK wrapper.
fn same_frame_feedback_fixture() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 0x554e_4956, 0] {
        push_var_uint(&mut bytes, value);
    }
    object(&mut bytes, "Backboard", |_| {});
    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Feedback")
    });
    for name in ["requested", "measured", "observed"] {
        object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
            string(bytes, "ViewModelPropertyNumber", "name", name)
        });
    }
    object(&mut bytes, "ViewModelInstance", |bytes| {
        uint(bytes, "ViewModelInstance", "viewModelId", 0);
        string(bytes, "ViewModelInstance", "name", "defaults");
    });
    for (index, value) in [10.0, 0.0, 0.0].into_iter().enumerate() {
        object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
            uint(
                bytes,
                "ViewModelInstanceNumber",
                "viewModelPropertyId",
                index as u64,
            );
            f32_value(bytes, "ViewModelInstanceNumber", "propertyValue", value);
        });
    }
    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 0);
        f32_value(bytes, "Artboard", "width", 200.0);
        f32_value(bytes, "Artboard", "height", 200.0);
    });
    let binding = |bytes: &mut Vec<u8>, target: u16, source: u8, reverse: bool| {
        object(bytes, "DataBindContext", |bytes| {
            uint(bytes, "DataBindContext", "propertyKey", u64::from(target));
            uint(bytes, "DataBindContext", "flags", u64::from(reverse));
            push_var_uint(
                bytes,
                u64::from(property_key("DataBindContext", "sourcePathIds")),
            );
            push_var_uint(bytes, 2);
            bytes.extend_from_slice(&[0, source]);
        });
    };
    object(&mut bytes, "Shape", |bytes| {
        uint(bytes, "Node", "parentId", 0)
    });
    object(&mut bytes, "Rectangle", |bytes| {
        uint(bytes, "Node", "parentId", 1);
        f32_value(bytes, "ParametricPath", "width", 10.0);
        f32_value(bytes, "ParametricPath", "height", 10.0);
    });
    binding(
        &mut bytes,
        property_key("ParametricPath", "width"),
        0,
        false,
    );
    binding(&mut bytes, property_key("ParametricPath", "width"), 1, true);
    object(&mut bytes, "Node", |bytes| {
        uint(bytes, "Node", "parentId", 0)
    });
    binding(&mut bytes, property_key("Node", "x"), 1, false);
    binding(&mut bytes, property_key("Node", "x"), 2, true);
    object(&mut bytes, "StateMachine", |bytes| {
        string(bytes, "StateMachine", "name", "Feedback")
    });
    bytes
}

#[test]
fn player_step_settles_reverse_binding_feedback_before_publication() {
    let file = import_bytes(&same_frame_feedback_fixture());
    let mut artboard = std::ptr::null_mut();
    let mut model = std::ptr::null_mut();
    let mut player = std::ptr::null_mut();
    unsafe {
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(artboard, &mut model),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, model),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
    }
    for requested in [10.0, 80.0, 25.0, 10.0] {
        let mutation = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_NUMBER,
            instance: model,
            path: NuxStringView {
                data: b"requested".as_ptr().cast(),
                len: 9,
            },
            number_value: requested,
            ..NuxViewModelMutation::default()
        };
        let batch = NuxViewModelMutationBatch {
            mutations: &mutation,
            mutation_count: 1,
            ..NuxViewModelMutationBatch::default()
        };
        let mut mutation_result = std::ptr::null_mut();
        let mut step_result = std::ptr::null_mut();
        unsafe {
            assert_eq!(
                nux_view_model_mutate(&batch, &mut mutation_result),
                NuxStatus::Ok
            );
            nux_view_model_mutation_result_free(mutation_result);
            assert_eq!(
                nux_player_step(player, &NuxPlayerStep::default(), &mut step_result),
                NuxStatus::Ok
            );
            nux_player_step_result_free(step_result);
        }
        let values = snapshot(model);
        assert_eq!(number(values, "measured"), requested);
        assert_eq!(
            number(values, "observed"),
            requested,
            "reverse feedback must settle in the same step"
        );
        unsafe {
            nux_view_model_snapshot_free(values);
        }
    }
    unsafe {
        nux_player_free(player);
        nux_view_model_instance_free(model);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn frame_checkpoints_preserve_external_observer_commit_and_rollback() {
    use nuxie::runtime::viewmodel::viewmodel_instance_value::{
        ViewModelInstanceValueDelegate, ViewModelInstanceValueDelegateHandle,
    };
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    struct Observer(Rc<Cell<usize>>);
    impl ViewModelInstanceValueDelegate for Observer {
        fn value_changed(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let mut factory = nuxie::PersistentFactory::new(nuxie::render_api::RecordingFactory::new());
    let file = nuxie::import_native(
        &same_frame_feedback_fixture(),
        &mut factory,
        None,
        nuxie::FileImportLimits::default(),
    )
    .unwrap();
    let source = file.with_file(|file| file.artboard_at_source(0)).unwrap();
    let native = file
        .with_file(|file| file.create_default_view_model_instance_for_artboard(source))
        .unwrap();
    let model = nuxie::RuntimeOwnedViewModelHandle::from_native(file, native).unwrap();
    let value = model
        .native_handle()
        .with(|owner| owner.as_view_model_instance().unwrap().property_values()[0].clone())
        .unwrap();
    let notifications = Rc::new(Cell::new(0));
    let observer: ViewModelInstanceValueDelegateHandle =
        Rc::new(RefCell::new(Observer(notifications.clone())));
    value.with_mut(|value| {
        value
            .as_view_model_instance_value_mut()
            .unwrap()
            .add_delegate(&observer)
    });

    let checkpoint =
        nuxie::RuntimeOwnedViewModelGraphTransaction::begin(&[model.clone()], 32).unwrap();
    model
        .borrow_mut()
        .set_number_by_property_name("requested", 90.0);
    assert_eq!(
        notifications.get(),
        0,
        "a frame may not publish before commit"
    );
    drop(checkpoint);
    assert_eq!(
        model.borrow().number_value_by_property_name("requested"),
        Some(10.0)
    );
    assert_eq!(
        notifications.get(),
        0,
        "rolled-back frame observations stay private"
    );

    let checkpoint =
        nuxie::RuntimeOwnedViewModelGraphTransaction::begin(&[model.clone()], 32).unwrap();
    model
        .borrow_mut()
        .set_number_by_property_name("requested", 80.0);
    assert_eq!(notifications.get(), 0);
    checkpoint.commit();
    assert_eq!(notifications.get(), 1);
    assert_eq!(
        model.borrow().number_value_by_property_name("requested"),
        Some(80.0)
    );

    let mut batch = nuxie::RuntimeOwnedViewModelTransaction::begin().unwrap();
    assert!(batch.set_number(&model, "requested", 25.0));
    assert_eq!(
        notifications.get(),
        1,
        "host batches retain atomic publication"
    );
    drop(batch);
    assert_eq!(
        model.borrow().number_value_by_property_name("requested"),
        Some(80.0)
    );
    assert_eq!(notifications.get(), 1);

    let mut batch = nuxie::RuntimeOwnedViewModelTransaction::begin().unwrap();
    assert!(batch.set_number(&model, "requested", 25.0));
    batch.commit();
    assert_eq!(notifications.get(), 2);
    assert_eq!(
        model.borrow().number_value_by_property_name("requested"),
        Some(25.0)
    );
}

/// Output bindings on each component-list row. Every row's default machine
/// runs a two-second 0 to 1000 timeline on its marker node.
#[derive(Clone, Copy, Debug)]
enum RowBindings {
    /// No output bindings: the row's machine never writes a view model.
    Quiet,
    /// The row artboard's computed width reads back into `rowWidth`.
    Readback,
    /// The animated marker x writes into `probe` on every timed frame.
    Marker,
    /// Both outputs, into separate properties.
    Active,
}

impl RowBindings {
    const ALL: [Self; 4] = [Self::Quiet, Self::Readback, Self::Marker, Self::Active];

    fn readback(self) -> bool {
        matches!(self, Self::Readback | Self::Active)
    }

    fn marker(self) -> bool {
        matches!(self, Self::Marker | Self::Active)
    }

    /// `probe` after `seconds` of the row's marker timeline (500 per second).
    fn probe_after(self, seconds: f32) -> f32 {
        if self.marker() { seconds * 500.0 } else { 0.0 }
    }

    /// `rowWidth` once the 100-wide row has mounted.
    fn mounted_width(self) -> f32 {
        if self.readback() { 100.0 } else { 0.0 }
    }
}

/// Row artboards A and B (schemas RowA and RowB) hosted by LayerList's
/// `screens` ArtboardComponentList, bound to the Host schema's `screens` list.
/// Byte-identical to the iOS SDK's removal-probe-*.riv reproducers for
/// UNIV-5186.
fn component_list_rows_fixture(bindings: RowBindings) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 1, 0] {
        push_var_uint(&mut bytes, value);
    }
    let bind = |bytes: &mut Vec<u8>, target: u16, reverse: bool, source: [u8; 2]| {
        object(bytes, "DataBindContext", |bytes| {
            uint(bytes, "DataBindContext", "propertyKey", u64::from(target));
            if reverse {
                uint(bytes, "DataBindContext", "flags", 1);
            }
            push_var_uint(
                bytes,
                u64::from(property_key("DataBindContext", "sourcePathIds")),
            );
            push_var_uint(bytes, 2);
            bytes.extend_from_slice(&source);
        });
    };
    let states = |bytes: &mut Vec<u8>, layer: &str, animation: Option<u64>| {
        object(bytes, "StateMachineLayer", |bytes| {
            string(bytes, "StateMachineLayer", "name", layer)
        });
        object(bytes, "EntryState", |_| {});
        if animation.is_some() {
            object(bytes, "StateTransition", |bytes| {
                uint(bytes, "StateTransition", "stateToId", 3)
            });
        }
        object(bytes, "AnyState", |_| {});
        object(bytes, "ExitState", |_| {});
        if let Some(animation) = animation {
            object(bytes, "AnimationState", |bytes| {
                uint(bytes, "AnimationState", "animationId", animation)
            });
        }
    };
    object(&mut bytes, "Backboard", |_| {});
    for schema in ["RowA", "RowB"] {
        object(&mut bytes, "ViewModel", |bytes| {
            string(bytes, "ViewModel", "name", schema)
        });
        for property in ["probe", "rowWidth"] {
            object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
                string(bytes, "ViewModelPropertyNumber", "name", property)
            });
        }
    }
    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Host")
    });
    object(&mut bytes, "ViewModelPropertyList", |bytes| {
        string(bytes, "ViewModelPropertyList", "name", "screens")
    });
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "host");
        uint(bytes, "ViewModelInstance", "viewModelId", 2);
    });
    object(&mut bytes, "ViewModelInstanceList", |bytes| {
        uint(bytes, "ViewModelInstanceList", "viewModelPropertyId", 0)
    });
    for (schema, name) in ["A", "B"].into_iter().enumerate() {
        let schema = schema as u8;
        object(&mut bytes, "Artboard", |bytes| {
            string(bytes, "Artboard", "name", name);
            f32_value(bytes, "Artboard", "width", 100.0);
            f32_value(bytes, "Artboard", "height", 100.0);
            uint(bytes, "Artboard", "viewModelId", u64::from(schema));
            uint(bytes, "Artboard", "defaultStateMachineId", 0);
        });
        if bindings.readback() {
            bind(
                &mut bytes,
                property_key("Node", "computedWidth"),
                true,
                [schema, 1],
            );
        }
        object(&mut bytes, "Node", |bytes| {
            string(bytes, "Node", "name", "Marker");
            uint(bytes, "Node", "parentId", 0);
        });
        if bindings.marker() {
            bind(&mut bytes, property_key("Node", "x"), true, [schema, 0]);
        }
        object(&mut bytes, "LinearAnimation", |bytes| {
            string(bytes, "LinearAnimation", "name", "Marker");
            uint(bytes, "LinearAnimation", "fps", 60);
            uint(bytes, "LinearAnimation", "duration", 120);
            uint(bytes, "LinearAnimation", "loopValue", 0);
        });
        object(&mut bytes, "KeyedObject", |bytes| {
            uint(bytes, "KeyedObject", "objectId", 1)
        });
        object(&mut bytes, "KeyedProperty", |bytes| {
            uint(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key("Node", "x")),
            )
        });
        for (frame, value) in [(0, 0.0), (120, 1000.0)] {
            object(&mut bytes, "KeyFrameDouble", |bytes| {
                uint(bytes, "KeyFrameDouble", "frame", frame);
                uint(bytes, "KeyFrameDouble", "interpolationType", 1);
                f32_value(bytes, "KeyFrameDouble", "value", value);
            });
        }
        object(&mut bytes, "StateMachine", |bytes| {
            string(bytes, "StateMachine", "name", "Row")
        });
        states(&mut bytes, "Marker", Some(0));
    }
    object(&mut bytes, "Artboard", |bytes| {
        string(bytes, "Artboard", "name", "LayerList");
        f32_value(bytes, "Artboard", "width", 100.0);
        f32_value(bytes, "Artboard", "height", 100.0);
        uint(bytes, "Artboard", "viewModelId", 2);
        uint(bytes, "Artboard", "defaultStateMachineId", 0);
    });
    object(&mut bytes, "Node", |bytes| {
        uint(bytes, "Node", "parentId", 0)
    });
    object(&mut bytes, "ArtboardComponentList", |bytes| {
        string(bytes, "ArtboardComponentList", "name", "screens");
        uint(bytes, "ArtboardComponentList", "parentId", 1);
    });
    bind(
        &mut bytes,
        property_key("ArtboardComponentList", "listSource"),
        false,
        [2, 0],
    );
    object(&mut bytes, "StateMachine", |bytes| {
        string(bytes, "StateMachine", "name", "Host")
    });
    states(&mut bytes, "Rows", None);
    bytes
}

/// The host side of a component list: LayerList's Host machine bound to its
/// default Host instance, plus the row instances the host keeps alive.
struct RowHost {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    root: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
    rows: Vec<*mut NuxViewModelInstance>,
}

impl RowHost {
    fn new(bindings: RowBindings) -> Self {
        let file = import_bytes(&component_list_rows_fixture(bindings));
        let mut artboard = std::ptr::null_mut();
        let mut root = std::ptr::null_mut();
        let mut player = std::ptr::null_mut();
        let name = "LayerList";
        let machine = "Host";
        unsafe {
            assert_eq!(
                nux_artboard_instance_new_named(
                    file,
                    NuxStringView {
                        data: name.as_ptr().cast(),
                        len: name.len(),
                    },
                    &mut artboard,
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_instance_new_default(artboard, &mut root),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(artboard, root),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_new_state_machine_named(
                    artboard,
                    NuxStringView {
                        data: machine.as_ptr().cast(),
                        len: machine.len(),
                    },
                    &mut player,
                ),
                NuxStatus::Ok
            );
        }
        Self {
            file,
            artboard,
            root,
            player,
            rows: Vec::new(),
        }
    }

    /// A schema-default row instance (0 is RowA, 1 is RowB) the host retains.
    fn row(&mut self, schema: usize) -> *mut NuxViewModelInstance {
        let mut row = std::ptr::null_mut();
        assert_eq!(
            unsafe { nux_view_model_instance_new_schema_default(self.file, schema, &mut row) },
            NuxStatus::Ok
        );
        self.rows.push(row);
        row
    }

    fn mutate(&self, mutations: &[NuxViewModelMutation]) {
        let batch = NuxViewModelMutationBatch {
            mutations: mutations.as_ptr(),
            mutation_count: mutations.len(),
            ..NuxViewModelMutationBatch::default()
        };
        let mut result = std::ptr::null_mut();
        unsafe {
            assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
            assert_eq!(nux_view_model_mutation_result_free(result), NuxStatus::Ok);
        }
    }

    fn insert(&self, row: *mut NuxViewModelInstance, index: usize) -> NuxViewModelMutation {
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
            instance: self.root,
            path: NuxStringView {
                data: b"screens".as_ptr().cast(),
                len: 7,
            },
            related_instance: row,
            index,
            ..NuxViewModelMutation::default()
        }
    }

    fn remove(&self, index: usize) -> NuxViewModelMutation {
        NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
            instance: self.root,
            path: NuxStringView {
                data: b"screens".as_ptr().cast(),
                len: 7,
            },
            index,
            ..NuxViewModelMutation::default()
        }
    }

    /// One player step. Returns the published owner identities of its
    /// view-model changes, or the failure diagnostic.
    fn step(&self, elapsed_seconds: f32) -> Result<Vec<u64>, String> {
        let operation = NuxPlayerStep {
            elapsed_seconds,
            ..NuxPlayerStep::default()
        };
        let mut result = std::ptr::null_mut();
        let status = unsafe { nux_player_step(self.player, &operation, &mut result) };
        let outcome = if status == NuxStatus::Ok {
            let mut info = NuxPlayerStepInfo::default();
            assert_eq!(
                unsafe { nux_player_step_result_info(result, &mut info) },
                NuxStatus::Ok
            );
            Ok((0..info.view_model_change_count)
                .map(|index| {
                    let mut change = NuxViewModelChangeView::default();
                    assert_eq!(
                        unsafe {
                            nux_player_step_result_view_model_change(result, index, &mut change)
                        },
                        NuxStatus::Ok
                    );
                    change.owner_instance_id
                })
                .collect())
        } else {
            let mut diagnostic = NuxCapiDiagnosticView::default();
            assert_eq!(
                unsafe { nux_player_step_result_diagnostic(result, &mut diagnostic) },
                NuxStatus::Ok
            );
            Err(format!(
                "{status:?} {}: {}",
                owned_string(diagnostic.code),
                owned_string(diagnostic.message)
            ))
        };
        assert_eq!(
            unsafe { nux_player_step_result_free(result) },
            NuxStatus::Ok
        );
        outcome
    }

    fn identity(row: *mut NuxViewModelInstance) -> u64 {
        let mut identity = 0;
        assert_eq!(
            unsafe { nux_view_model_instance_identity(row, &mut identity) },
            NuxStatus::Ok
        );
        identity
    }

    fn value(row: *mut NuxViewModelInstance, name: &str) -> f32 {
        let values = snapshot(row);
        let value = number(values, name);
        unsafe {
            nux_view_model_snapshot_free(values);
        }
        value
    }
}

impl Drop for RowHost {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            for row in self.rows.drain(..) {
                assert_eq!(nux_view_model_instance_free(row), NuxStatus::Ok);
            }
            assert_eq!(nux_view_model_instance_free(self.root), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(self.artboard), NuxStatus::Ok);
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
        }
    }
}

#[test]
fn component_list_rows_fixture_matches_the_sdk_reproducers() {
    use sha2::Digest;
    for (bindings, sha256) in [
        (
            RowBindings::Quiet,
            "6657faf26a8b00660369a0b76024c049b108cd77669340f0fb1e6e6cbdec3c45",
        ),
        (
            RowBindings::Readback,
            "03b471786f99cdf2a9aee4a5299f583a2c55a93aeb3b1253a4d97d8860863d05",
        ),
        (
            RowBindings::Marker,
            "fb04c982a210723a2db32d16a70ca2e1d1d24c5e885335450940f124262240db",
        ),
        (
            RowBindings::Active,
            "c899e71efb8a615f125e9bd1c34b27bdb00aba112be6e1202375e04034670570",
        ),
    ] {
        let digest = sha2::Sha256::digest(component_list_rows_fixture(bindings));
        let hex = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(hex, sha256, "{bindings:?}");
    }
}

#[test]
fn timed_step_after_removing_a_component_list_row_keeps_the_player_working() {
    for bindings in RowBindings::ALL {
        for settle_first in [false, true] {
            let mut host = RowHost::new(bindings);
            let a = host.row(0);
            let a_id = RowHost::identity(a);
            host.mutate(&[host.insert(a, 0)]);
            host.step(0.0).unwrap();
            let mounted = host.step(0.25).unwrap();
            assert_eq!(
                mounted.contains(&a_id),
                bindings.marker(),
                "{bindings:?}: a mounted row's changes reach the host"
            );
            assert_eq!(RowHost::value(a, "probe"), bindings.probe_after(0.25));
            host.mutate(&[host.remove(0)]);
            if settle_first {
                host.step(0.0)
                    .unwrap_or_else(|error| panic!("{bindings:?} zero-time: {error}"));
            }
            let removed = host
                .step(0.5)
                .unwrap_or_else(|error| panic!("{bindings:?} settle={settle_first}: {error}"));
            assert!(
                !removed.contains(&a_id),
                "{bindings:?}: a departed row's changes stay out of the host's subscription"
            );
            // A timed step advances the outgoing row once more as it detaches,
            // and the retained instance keeps that final write. A zero-time
            // step detaches it first.
            let detached = bindings.probe_after(if settle_first { 0.25 } else { 0.75 });
            assert_eq!(RowHost::value(a, "probe"), detached, "{bindings:?}");
            assert_eq!(RowHost::value(a, "rowWidth"), bindings.mounted_width());
            host.step(0.5)
                .unwrap_or_else(|error| panic!("{bindings:?} after removal: {error}"));
            assert_eq!(
                RowHost::value(a, "probe"),
                detached,
                "{bindings:?}: a detached row no longer advances"
            );
        }
    }
}

#[test]
fn removing_one_component_list_row_keeps_the_other_row_advancing() {
    for bindings in RowBindings::ALL {
        let mut host = RowHost::new(bindings);
        let a = host.row(0);
        let b = host.row(1);
        let (a_id, b_id) = (RowHost::identity(a), RowHost::identity(b));
        host.mutate(&[host.insert(a, 0), host.insert(b, 1)]);
        host.step(0.0).unwrap();
        host.mutate(&[host.remove(0)]);
        let removed = host
            .step(0.5)
            .unwrap_or_else(|error| panic!("{bindings:?}: {error}"));
        assert!(!removed.contains(&a_id), "{bindings:?}");
        assert_eq!(removed.contains(&b_id), bindings.marker(), "{bindings:?}");
        assert_eq!(RowHost::value(a, "probe"), bindings.probe_after(0.5));
        assert_eq!(RowHost::value(b, "probe"), bindings.probe_after(0.5));
        let next = host
            .step(0.5)
            .unwrap_or_else(|error| panic!("{bindings:?} next: {error}"));
        assert_eq!(next.contains(&b_id), bindings.marker(), "{bindings:?}");
        assert_eq!(RowHost::value(a, "probe"), bindings.probe_after(0.5));
        assert_eq!(RowHost::value(b, "probe"), bindings.probe_after(1.0));
    }
}

#[test]
fn a_removed_component_list_row_keeps_its_values_and_remounts_on_reinsert() {
    for bindings in RowBindings::ALL {
        let mut host = RowHost::new(bindings);
        let a = host.row(0);
        let a_id = RowHost::identity(a);
        for cycle in 0..2 {
            host.mutate(&[host.insert(a, 0)]);
            host.step(0.0)
                .unwrap_or_else(|error| panic!("{bindings:?} cycle {cycle} mount: {error}"));
            let mounted = host
                .step(0.5)
                .unwrap_or_else(|error| panic!("{bindings:?} cycle {cycle} advance: {error}"));
            assert_eq!(mounted.contains(&a_id), bindings.marker(), "{bindings:?}");
            // Each mount starts a fresh row machine, so the marker restarts.
            assert_eq!(RowHost::value(a, "probe"), bindings.probe_after(0.5));
            host.mutate(&[host.remove(0)]);
            host.step(0.5)
                .unwrap_or_else(|error| panic!("{bindings:?} cycle {cycle} remove: {error}"));
            host.step(0.5)
                .unwrap_or_else(|error| panic!("{bindings:?} cycle {cycle} detached: {error}"));
            assert_eq!(RowHost::value(a, "probe"), bindings.probe_after(1.0));
            assert_eq!(RowHost::value(a, "rowWidth"), bindings.mounted_width());
        }
    }
}

#[test]
fn component_list_rows_mounted_and_removed_in_one_operation_keep_the_player_working() {
    for bindings in RowBindings::ALL {
        // Inserted and removed before the player ever mounts the row.
        for one_batch in [true, false] {
            let mut host = RowHost::new(bindings);
            let a = host.row(0);
            host.step(0.0).unwrap();
            if one_batch {
                host.mutate(&[host.insert(a, 0), host.remove(0)]);
            } else {
                host.mutate(&[host.insert(a, 0)]);
                host.mutate(&[host.remove(0)]);
            }
            host.step(0.5)
                .unwrap_or_else(|error| panic!("{bindings:?} one_batch={one_batch}: {error}"));
            assert_eq!(RowHost::value(a, "probe"), 0.0, "{bindings:?}");
            assert_eq!(RowHost::value(a, "rowWidth"), 0.0, "{bindings:?}");
            host.step(0.5).unwrap();
        }
        // One advance detaches the outgoing row and mounts the incoming one.
        let mut host = RowHost::new(bindings);
        let a = host.row(0);
        let b = host.row(1);
        let (a_id, b_id) = (RowHost::identity(a), RowHost::identity(b));
        host.mutate(&[host.insert(a, 0)]);
        host.step(0.0).unwrap();
        host.step(0.25).unwrap();
        host.mutate(&[host.remove(0), host.insert(b, 0)]);
        let swapped = host
            .step(0.5)
            .unwrap_or_else(|error| panic!("{bindings:?} swap: {error}"));
        assert!(!swapped.contains(&a_id), "{bindings:?}");
        // The incoming row's mount-time readback publishes in the same step.
        assert_eq!(swapped.contains(&b_id), bindings.readback(), "{bindings:?}");
        assert_eq!(RowHost::value(a, "probe"), bindings.probe_after(0.75));
        assert_eq!(RowHost::value(b, "rowWidth"), bindings.mounted_width());
        let next = host
            .step(0.5)
            .unwrap_or_else(|error| panic!("{bindings:?} after swap: {error}"));
        assert_eq!(next.contains(&b_id), bindings.marker(), "{bindings:?}");
        assert_eq!(RowHost::value(b, "probe"), bindings.probe_after(0.5));
    }
}
