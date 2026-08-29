#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    reason = "bounded host-projection fixture assertions"
)]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeFactoryHandle, RuntimeFileHandle, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance,
};

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

fn boolean(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: bool) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.push(u8::from(value));
}

fn nested_list_fixture() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 0x554e_2813, 0] {
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
    for (name, instance_index) in [("primary", 0), ("secondary", 1)] {
        let property_index = instance_index as u64;
        object(&mut bytes, "ViewModelInstanceViewModel", |bytes| {
            uint(bytes, "ViewModelInstanceViewModel", "parentId", 0);
            uint(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                property_index,
            );
            uint(
                bytes,
                "ViewModelInstanceViewModel",
                "propertyValue",
                property_index,
            );
        });
    }
    for name in ["primary", "secondary"] {
        object(&mut bytes, "ViewModelPropertyViewModel", |bytes| {
            string(bytes, "ViewModelPropertyViewModel", "name", name);
            uint(
                bytes,
                "ViewModelPropertyViewModel",
                "viewModelReferenceId",
                1,
            );
        });
    }

    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Paywall")
    });
    object(&mut bytes, "ViewModelPropertyString", |bytes| {
        string(
            bytes,
            "ViewModelPropertyString",
            "name",
            "selectedProductId",
        )
    });
    object(&mut bytes, "ViewModelPropertyList", |bytes| {
        string(bytes, "ViewModelPropertyList", "name", "products")
    });
    for (instance_index, name) in ["primary", "secondary"].into_iter().enumerate() {
        object(&mut bytes, "ViewModelInstance", |bytes| {
            string(bytes, "ViewModelInstance", "name", name);
            uint(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        object(&mut bytes, "ViewModelInstanceString", |bytes| {
            uint(
                bytes,
                "ViewModelInstanceString",
                "parentId",
                instance_index as u64,
            );
            uint(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            string(bytes, "ViewModelInstanceString", "propertyValue", "");
        });
        object(&mut bytes, "ViewModelInstanceList", |bytes| {
            uint(
                bytes,
                "ViewModelInstanceList",
                "parentId",
                instance_index as u64,
            );
            uint(bytes, "ViewModelInstanceList", "viewModelPropertyId", 1);
        });
        for product_index in 0..2 {
            object(&mut bytes, "ViewModelInstanceListItem", |bytes| {
                uint(bytes, "ViewModelInstanceListItem", "viewModelId", 2);
                uint(
                    bytes,
                    "ViewModelInstanceListItem",
                    "viewModelInstanceId",
                    product_index,
                );
            });
        }
    }

    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Product")
    });
    object(&mut bytes, "ViewModelPropertyString", |bytes| {
        string(bytes, "ViewModelPropertyString", "name", "productId")
    });
    object(&mut bytes, "ViewModelPropertyBoolean", |bytes| {
        string(bytes, "ViewModelPropertyBoolean", "name", "isSelected")
    });
    for (instance_index, (name, product_id)) in
        [("Basic", "basic"), ("Pro", "pro")].into_iter().enumerate()
    {
        object(&mut bytes, "ViewModelInstance", |bytes| {
            string(bytes, "ViewModelInstance", "name", name);
            uint(bytes, "ViewModelInstance", "viewModelId", 2);
        });
        object(&mut bytes, "ViewModelInstanceString", |bytes| {
            uint(
                bytes,
                "ViewModelInstanceString",
                "parentId",
                instance_index as u64,
            );
            uint(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            string(
                bytes,
                "ViewModelInstanceString",
                "propertyValue",
                product_id,
            );
        });
        object(&mut bytes, "ViewModelInstanceBoolean", |bytes| {
            uint(
                bytes,
                "ViewModelInstanceBoolean",
                "parentId",
                instance_index as u64,
            );
            uint(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 1);
            boolean(bytes, "ViewModelInstanceBoolean", "propertyValue", false);
        });
    }

    object(&mut bytes, "ViewModel", |bytes| {
        string(bytes, "ViewModel", "name", "Other")
    });
    object(&mut bytes, "ViewModelPropertyString", |bytes| {
        string(bytes, "ViewModelPropertyString", "name", "productId")
    });
    object(&mut bytes, "ViewModelInstance", |bytes| {
        string(bytes, "ViewModelInstance", "name", "other");
        uint(bytes, "ViewModelInstance", "viewModelId", 3);
    });
    object(&mut bytes, "ViewModelInstanceString", |bytes| {
        uint(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
        string(bytes, "ViewModelInstanceString", "propertyValue", "other");
    });

    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 0)
    });
    object(&mut bytes, "Artboard", |bytes| {
        uint(bytes, "Artboard", "viewModelId", 3)
    });
    bytes
}

fn import_nested_list_fixture() -> (
    RuntimeFileHandle,
    PersistentFactory<RecordingFactory>,
    RuntimeOwnedViewModelHandle,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&nested_list_fixture(), retained, None, None, None).unwrap();
    let root = RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap();
    (file, factory, RuntimeOwnedViewModelHandle::new(root))
}

fn selected_values(root: &RuntimeOwnedViewModelHandle, child: &str) -> Vec<bool> {
    root.linked_view_model_by_property_name_path(child)
        .unwrap()
        .testing_list_items_by_property_name("products")
        .unwrap()
        .into_iter()
        .filter_map(|item| item.borrow().boolean_value_by_property_name("isSelected"))
        .collect()
}

#[test]
fn string_preflight_delegates_to_the_current_typed_occurrence() {
    let (_file, _factory, root) = import_nested_list_fixture();
    let nested = root
        .borrow()
        .string_source_handle_by_property_name_path("primary/selectedProductId")
        .unwrap();
    assert!(root.borrow().can_set_string_by_source_handle(&nested));
    assert!(!root.borrow().can_set_string_by_property_index(0));

    let primary = root
        .linked_view_model_by_property_name_path("primary")
        .unwrap();
    assert!(primary.borrow().can_set_string_by_property_index(0));
    assert!(!primary.borrow().can_set_string_by_property_index(1));
    assert!(
        root.borrow_mut()
            .set_string_by_source_handle(&nested, b"basic")
    );
    assert!(root.borrow().can_set_string_by_source_handle(&nested));
}

#[test]
fn nested_list_relation_revalidates_before_applying_any_boolean() {
    let (file, _factory, root) = import_nested_list_fixture();
    let primary = root
        .borrow()
        .list_string_match_boolean_handle_by_property_name_path(
            "primary/products",
            "productId",
            "isSelected",
        )
        .unwrap();
    let secondary = root
        .borrow()
        .list_string_match_boolean_handle_by_property_name_path(
            "secondary/products",
            "productId",
            "isSelected",
        )
        .unwrap();
    assert_ne!(primary, secondary, "the complete list path is retained");
    assert!(root.borrow().can_apply_list_string_match_boolean(&primary));
    assert_eq!(
        root.borrow_mut()
            .apply_list_string_match_boolean(&primary, b"basic"),
        Some(true)
    );
    assert_eq!(selected_values(&root, "primary"), vec![true, false]);
    assert_eq!(
        root.borrow_mut()
            .apply_list_string_match_boolean(&primary, b"basic"),
        Some(false),
        "a repeated projection is a no-op"
    );
    assert_eq!(
        root.borrow_mut()
            .apply_list_string_match_boolean(&primary, b"pro"),
        Some(true)
    );
    assert_eq!(selected_values(&root, "primary"), vec![false, true]);

    let incompatible = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file, 3, 0).unwrap(),
    );
    assert!(
        root.linked_view_model_by_property_name_path("primary")
            .unwrap()
            .push_list_item_by_property_name_path("products", &incompatible)
    );
    assert!(!root.borrow().can_apply_list_string_match_boolean(&primary));
    assert_eq!(
        root.borrow_mut()
            .apply_list_string_match_boolean(&primary, b"basic"),
        None
    );
    assert_eq!(
        selected_values(&root, "primary")[..2],
        [false, true],
        "an incompatible replacement cannot cause a partial update"
    );
    assert!(
        root.borrow()
            .can_apply_list_string_match_boolean(&secondary)
    );
}
