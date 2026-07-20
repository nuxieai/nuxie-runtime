use nuxie_binary::{
    AuthoringProperty, AuthoringRecord, AuthoringValue, HeaderFieldKind, RuntimeFile,
    RuntimeImportStatus, SUPPORTED_MAJOR_VERSION, SUPPORTED_MINOR_VERSION,
};

fn uint(key: u16, value: u64) -> AuthoringProperty {
    AuthoringProperty {
        key,
        value: AuthoringValue::Uint(value),
    }
}

fn double(key: u16, value: f32) -> AuthoringProperty {
    AuthoringProperty {
        key,
        value: AuthoringValue::Double(value),
    }
}

fn string(key: u16, value: &str) -> AuthoringProperty {
    AuthoringProperty {
        key,
        value: AuthoringValue::String(value.to_owned()),
    }
}

fn bool_value(key: u16, value: bool) -> AuthoringProperty {
    AuthoringProperty {
        key,
        value: AuthoringValue::Bool(value),
    }
}

fn bytes(key: u16, value: Vec<u8>) -> AuthoringProperty {
    AuthoringProperty {
        key,
        value: AuthoringValue::Bytes(value),
    }
}

#[test]
fn authoring_records_build_an_importable_runtime_file() {
    let file = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 0)],
        },
        AuthoringRecord {
            type_key: 7,
            properties: vec![uint(5, 1)],
        },
        AuthoringRecord {
            type_key: 20,
            properties: vec![uint(5, 1)],
        },
        AuthoringRecord {
            type_key: 18,
            properties: vec![
                uint(5, 3),
                AuthoringProperty {
                    key: 37,
                    value: AuthoringValue::Color(0xff_33_66_cc),
                },
            ],
        },
    ])
    .expect("valid authoring records should build a runtime file");

    assert_eq!(SUPPORTED_MAJOR_VERSION, 7);
    assert_eq!(SUPPORTED_MINOR_VERSION, 2);
    assert_eq!(file.header.major_version, 7);
    assert_eq!(file.header.minor_version, 2);
    assert_eq!(file.header.file_id, 0);
    assert_eq!(file.object_count(), 6);
    assert_eq!(file.known_object_count(), 6);
    assert_eq!(file.imported_object_count(), 6);
    assert_eq!(
        file.header.property_field_ids.get(&5),
        Some(&HeaderFieldKind::Uint)
    );
    assert_eq!(
        file.header.property_field_ids.get(&37),
        Some(&HeaderFieldKind::Color)
    );

    let expected_types = [
        "Backboard",
        "Artboard",
        "Shape",
        "Rectangle",
        "Fill",
        "SolidColor",
    ];
    for (id, expected_type) in expected_types.into_iter().enumerate() {
        let object = file
            .object(id)
            .unwrap_or_else(|| panic!("missing authored object {id}"));
        assert_eq!(object.id, id as u32);
        assert_eq!(object.type_name, expected_type);
        assert_eq!(file.import_status(id), Some(RuntimeImportStatus::Imported));
    }

    assert_eq!(
        file.object(2)
            .and_then(|object| object.uint_property("parentId")),
        Some(0)
    );
    assert_eq!(
        file.object(3)
            .and_then(|object| object.uint_property("parentId")),
        Some(1)
    );
    assert_eq!(
        file.object(4)
            .and_then(|object| object.uint_property("parentId")),
        Some(1)
    );
    assert_eq!(
        file.object(5)
            .and_then(|object| object.uint_property("parentId")),
        Some(3)
    );
    assert_eq!(
        file.object(5)
            .and_then(|object| object.color_property("colorValue")),
        Some(0xff_33_66_cc)
    );
}

#[test]
fn artboard_geometry_preserves_local_order_and_shape_ownership() {
    let file = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 0)],
        },
        AuthoringRecord {
            type_key: 7,
            properties: vec![uint(5, 1)],
        },
        AuthoringRecord {
            type_key: 20,
            properties: vec![uint(5, 1)],
        },
        AuthoringRecord {
            type_key: 18,
            properties: vec![uint(5, 3)],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 0)],
        },
        AuthoringRecord {
            type_key: 4,
            properties: vec![uint(5, 5)],
        },
        AuthoringRecord {
            type_key: 24,
            properties: vec![uint(5, 5)],
        },
        AuthoringRecord {
            type_key: 18,
            properties: vec![uint(5, 7)],
        },
    ])
    .expect("valid authored shapes should build a runtime file");

    let geometry = file
        .artboard_geometry(0)
        .expect("the authored artboard should have indexed geometry");

    assert_eq!(
        geometry
            .paths
            .iter()
            .map(|path| path.local_id)
            .collect::<Vec<_>>(),
        vec![2, 6]
    );
    assert_eq!(
        geometry
            .shapes
            .iter()
            .map(|shape| shape.local_id)
            .collect::<Vec<_>>(),
        vec![1, 5]
    );
    assert_eq!(
        geometry
            .shapes
            .iter()
            .map(|shape| {
                shape
                    .paths
                    .iter()
                    .map(|path| path.local_id)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![2], vec![6]]
    );
    assert_eq!(
        geometry
            .shapes
            .iter()
            .map(|shape| {
                shape
                    .paints
                    .iter()
                    .map(|paint| (paint.local_id, paint.mutator_local_id))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(3, Some(4))], vec![(7, Some(8))]]
    );
}

#[test]
fn authoring_records_use_the_wire_canonical_property_order() {
    let file = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![
                double(18, 0.5),
                string(4, "Shape"),
                double(13, 12.0),
                uint(5, 0),
            ],
        },
    ])
    .expect("valid authoring records should build a runtime file");

    let property_keys = file
        .object(2)
        .expect("authored Shape should exist")
        .properties
        .iter()
        .map(|property| property.key)
        .collect::<Vec<_>>();
    assert_eq!(property_keys, vec![4, 5, 13, 18]);
}

#[test]
fn authoring_header_uses_only_wire_compatible_property_field_types() {
    let file = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 0), string(4, "Shape"), double(13, 12.0)],
        },
        AuthoringRecord {
            type_key: 7,
            properties: vec![uint(5, 1), bool_value(164, true)],
        },
        AuthoringRecord {
            type_key: 20,
            properties: vec![uint(5, 1)],
        },
        AuthoringRecord {
            type_key: 18,
            properties: vec![
                uint(5, 3),
                AuthoringProperty {
                    key: 37,
                    value: AuthoringValue::Color(0xff_33_66_cc),
                },
            ],
        },
        AuthoringRecord {
            type_key: 92,
            properties: vec![uint(5, 0), bytes(582, Vec::new())],
        },
    ])
    .expect("all authored property kinds should build an importable runtime file");

    let property_field_ids = file
        .header
        .property_field_ids
        .iter()
        .map(|(key, kind)| (*key, *kind))
        .collect::<Vec<_>>();
    assert_eq!(
        property_field_ids,
        vec![
            (4, HeaderFieldKind::StringOrBytes),
            (5, HeaderFieldKind::Uint),
            (13, HeaderFieldKind::Double),
            (37, HeaderFieldKind::Color),
        ]
    );
}

#[test]
fn authoring_records_reject_objects_the_runtime_would_drop() {
    let error = RuntimeFile::from_authoring_records(vec![AuthoringRecord {
        type_key: 1,
        properties: vec![],
    }])
    .expect_err("an Artboard without a preceding Backboard must be rejected");

    assert!(
        error.to_string().contains("would be dropped"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_reject_an_invalid_component_parent() {
    let error = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 99)],
        },
    ])
    .expect_err("a component parent must resolve inside its artboard");

    assert!(
        error.to_string().contains("invalid artboard-local object"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_reject_an_unknown_type() {
    let error = RuntimeFile::from_authoring_records(vec![AuthoringRecord {
        type_key: u16::MAX,
        properties: vec![],
    }])
    .expect_err("unknown type keys must be rejected");

    assert!(
        error
            .to_string()
            .contains("unknown authoring object type key"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_reject_an_abstract_type() {
    let error = RuntimeFile::from_authoring_records(vec![AuthoringRecord {
        type_key: 9,
        properties: vec![],
    }])
    .expect_err("abstract schema types cannot be authored as records");

    assert!(
        error.to_string().contains("is abstract"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_reject_non_deserializable_properties() {
    let error = RuntimeFile::from_authoring_records(vec![AuthoringRecord {
        type_key: 122,
        properties: vec![uint(401, 0)],
    }])
    .expect_err("callback-only properties cannot be authored as stored record values");

    assert!(
        error.to_string().contains("is not deserializable"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_reject_the_wrong_property_value_kind() {
    let error = RuntimeFile::from_authoring_records(vec![AuthoringRecord {
        type_key: 18,
        properties: vec![uint(37, 0xff_33_66_cc)],
    }])
    .expect_err("SolidColor.colorValue requires a color value");

    let message = error.to_string();
    assert!(
        message.contains("expects Color"),
        "unexpected error: {error:#}"
    );
    assert!(message.contains("got uint"), "unexpected error: {error:#}");
}

#[test]
fn authoring_records_reject_duplicate_property_keys() {
    let error = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 0), uint(5, 0)],
        },
    ])
    .expect_err("an authored property key must occur at most once per object");

    assert!(
        error
            .to_string()
            .contains("duplicate authoring property key 5"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_reject_uints_that_cannot_round_trip_through_the_wire_format() {
    let error = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 3,
            properties: vec![uint(5, 0)],
        },
        AuthoringRecord {
            type_key: 20,
            properties: vec![uint(5, 1), uint(40, u64::from(u32::MAX) + 1)],
        },
    ])
    .expect_err("authored uints must remain import/export round-trippable");

    assert!(
        error
            .to_string()
            .contains("does not fit in C++ unsigned int"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn authoring_records_preserve_uint64_library_scope_values() {
    let file = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 558,
            properties: vec![uint(798, u64::MAX), uint(799, u64::from(u32::MAX) + 1)],
        },
    ])
    .expect("LibraryAsset uint64 values should retain the full wire range");

    let library = file.object(1).expect("authored LibraryAsset");
    assert_eq!(library.uint_property("libraryId"), Some(u64::MAX));
    assert_eq!(
        library.uint_property("libraryVersionId"),
        Some(u64::from(u32::MAX) + 1)
    );
    assert_eq!(
        file.header.property_field_ids.get(&798),
        Some(&HeaderFieldKind::Uint)
    );
    assert_eq!(
        file.header.property_field_ids.get(&799),
        Some(&HeaderFieldKind::Uint)
    );
}

#[test]
fn authoring_records_apply_uint8_member_truncation_after_uint_wire_validation() {
    let file = RuntimeFile::from_authoring_records(vec![
        AuthoringRecord {
            type_key: 23,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 1,
            properties: vec![],
        },
        AuthoringRecord {
            type_key: 420,
            properties: vec![uint(596, 0x1ff)],
        },
    ])
    .expect("uint8 aliases should accept uint wire values");

    assert_eq!(
        file.object(2)
            .expect("authored LayoutComponentStyle")
            .uint_property("displayValue"),
        Some(0xff)
    );
}
