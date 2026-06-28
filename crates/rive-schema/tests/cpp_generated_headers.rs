use rive_schema::{
    CoreRegistryFieldKind, FieldKind, StoredFieldInitializer,
    core_registry_field_kind_by_property_key, core_registry_getter_field_kind_by_property_key,
    core_registry_setter_field_kind_by_property_key, generated::DEFINITIONS,
    is_callback_property_key, object_supports_property,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn reference_runtime_dir() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

#[test]
fn generated_rust_schema_matches_cpp_type_keys() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let constants = parse_u16_constants(&header);

        assert_eq!(
            constants.get("typeKey").copied(),
            Some(definition.type_key.int),
            "{} typeKey in {}",
            definition.name,
            header.display()
        );
    }
}

#[test]
fn generated_rust_schema_matches_cpp_property_keys() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let constants = parse_u16_constants(&header);

        for property in definition.properties {
            let cpp_name = format!("{}PropertyKey", property.name);
            assert_eq!(
                constants.get(&cpp_name).copied(),
                Some(property.key.int),
                "{}.{} property key in {}",
                definition.name,
                property.name,
                header.display()
            );

            for alternate in property.alternates {
                let cpp_name = format!("{}PropertyKey", alternate.name);
                assert_eq!(
                    constants.get(&cpp_name).copied(),
                    Some(alternate.int),
                    "{}.{} alternate property key in {}",
                    definition.name,
                    alternate.name,
                    header.display()
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_stored_field_members() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let cpp_members = parse_cpp_member_initializers(&header)
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let rust_members = definition
            .properties
            .iter()
            .filter(|property| property.stores_field)
            .map(|property| cpp_member_name(property.name))
            .collect::<BTreeSet<_>>();

        assert_eq!(
            rust_members,
            cpp_members,
            "{} stored field members in {}",
            definition.name,
            header.display()
        );

        for property in definition.properties {
            assert_eq!(
                cpp_members.contains(&cpp_member_name(property.name)),
                property.stores_field,
                "{}.{} stores_field",
                definition.name,
                property.name
            );
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_stored_field_initializers() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let initializers = parse_cpp_member_initializers(&header);

        for property in definition.properties {
            let Some(rust_initializer) = property.stored_field_initializer() else {
                continue;
            };

            let member = cpp_member_name(property.name);
            let cpp_value = initializers.get(&member).unwrap_or_else(|| {
                panic!(
                    "{}.{} missing C++ member initializer m_{} in {}",
                    definition.name,
                    property.name,
                    member,
                    header.display()
                )
            });

            assert_eq!(
                rust_initializer,
                parse_cpp_stored_field_initializer(
                    property.runtime_type,
                    cpp_value,
                    &format!("{}.{}", definition.name, property.name),
                ),
                "{}.{} stored-field initializer",
                definition.name,
                property.name
            );
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_value_setters() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let cpp_notified_properties = cpp_notified_property_names(&header);
        let rust_setter_properties = definition
            .properties
            .iter()
            .filter(|property| property.cpp_generates_value_setter_body())
            .map(|property| property.name.to_owned())
            .collect::<BTreeSet<_>>();

        for property_name in &cpp_notified_properties {
            assert!(
                rust_setter_properties.contains(property_name),
                "{} notifyPropertyChanged call for property without generated setter body: {} in {}",
                definition.name,
                property_name,
                header.display()
            );
        }

        let cpp_changed_hooks = cpp_changed_hook_names(&header);

        for property in definition.properties {
            assert_eq!(
                cpp_changed_hooks.contains(property.name),
                property.cpp_generates_changed_hook(),
                "{}.{} generated Changed hook in {}",
                definition.name,
                property.name,
                header.display()
            );

            let setter_body = cpp_property_setter_body(&header, property.name);
            assert_eq!(
                setter_body.is_some(),
                property.cpp_generates_value_setter_body(),
                "{}.{} generated setter body presence in {}",
                definition.name,
                property.name,
                header.display()
            );

            let Some(setter_body) = setter_body else {
                continue;
            };
            let actual = cpp_trimmed_lines(&setter_body);

            if property.cpp_setter_uses_passthrough() {
                assert_cpp_passthrough_setter_body(
                    &actual,
                    property.name,
                    definition.name,
                    &header,
                );
            } else {
                assert!(
                    property.cpp_setter_uses_stored_field(),
                    "{}.{} generated setter body is neither stored-field nor passthrough in {}",
                    definition.name,
                    property.name,
                    header.display()
                );
                assert_cpp_stored_field_setter_body(
                    &actual,
                    property.name,
                    &cpp_member_name(property.name),
                    definition.name,
                    &header,
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_value_getters() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);

        for property in definition.properties {
            let getter = cpp_property_getter(&header, property.name);
            assert_eq!(
                getter.is_some(),
                property.cpp_generates_stored_field_getter_body()
                    || property.cpp_generates_passthrough_getter_declaration(),
                "{}.{} generated getter presence in {}",
                definition.name,
                property.name,
                header.display()
            );

            let Some(getter) = getter else {
                continue;
            };

            if property.cpp_generates_passthrough_getter_declaration() {
                assert_cpp_passthrough_getter_declaration(
                    &getter,
                    property.name,
                    definition.name,
                    &header,
                );
            } else {
                assert!(
                    property.cpp_generates_stored_field_getter_body(),
                    "{}.{} generated getter is neither stored-field nor passthrough in {}",
                    definition.name,
                    property.name,
                    header.display()
                );
                assert_cpp_stored_field_getter_body(
                    &getter,
                    property.name,
                    &cpp_member_name(property.name),
                    property.virtual_ || property.pure_virtual,
                    property.override_get,
                    definition.name,
                    &header,
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_property_declarations() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let declarations = cpp_zero_declarations(&header);

        for property in definition.properties {
            let value_declaration = cpp_zero_declaration(&declarations, property.name);
            assert_eq!(
                value_declaration.is_some(),
                property.cpp_generates_pure_virtual_value_setter_declaration(),
                "{}.{} pure-virtual value setter declaration in {}",
                definition.name,
                property.name,
                header.display()
            );
            if let Some(declaration) = value_declaration {
                assert_cpp_value_setter_declaration(
                    &declaration,
                    property.name,
                    property.override_set,
                    definition.name,
                    &header,
                );
            }

            let member = cpp_member_name(property.name);
            let decode_declaration =
                cpp_zero_declaration(&declarations, &format!("decode{member}"));
            assert_eq!(
                decode_declaration.is_some(),
                property.cpp_generates_encoded_decode_hook(),
                "{}.{} encoded decode declaration in {}",
                definition.name,
                property.name,
                header.display()
            );
            if let Some(declaration) = decode_declaration {
                assert_cpp_encoded_decode_declaration(
                    &declaration,
                    &member,
                    property.override_set,
                    definition.name,
                    property.name,
                    &header,
                );
            }

            let copy_declaration = cpp_zero_declaration(&declarations, &format!("copy{member}"));
            assert_eq!(
                copy_declaration.is_some(),
                property.cpp_generates_encoded_copy_hook(),
                "{}.{} encoded copy declaration in {}",
                definition.name,
                property.name,
                header.display()
            );
            if let Some(declaration) = copy_declaration {
                assert_cpp_encoded_copy_declaration(
                    &declaration,
                    &member,
                    property.override_set,
                    definition.name,
                    property.name,
                    &header,
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_bitmask_passthrough_constants() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let constants = parse_u32_constants(&header);

        for property in definition.properties {
            let bitmask_name = format!("{}Bitmask", property.name);
            assert_eq!(
                constants.get(&bitmask_name).copied(),
                property.cpp_bitmask_passthrough_bitmask_constant(),
                "{}.{} generated bitmask passthrough bitmask constant in {}",
                definition.name,
                property.name,
                header.display()
            );

            let bit_offset_name = format!("{}BitOffset", property.name);
            assert_eq!(
                constants.get(&bit_offset_name).copied(),
                property.cpp_bitmask_passthrough_bit_offset_constant(),
                "{}.{} generated bitmask passthrough bit offset constant in {}",
                definition.name,
                property.name,
                header.display()
            );

            let field_mask_name = format!("{}FieldMask", property.name);
            assert_eq!(
                constants.get(&field_mask_name).copied(),
                property.cpp_bitmask_passthrough_field_mask_constant(),
                "{}.{} generated bitmask passthrough field mask constant in {}",
                definition.name,
                property.name,
                header.display()
            );
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_core_registry_property_field_ids() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);
    let cpp_field_ids = parse_cpp_core_registry_property_field_ids(&runtime_dir, &constants);

    for (key, cpp_kind) in &cpp_field_ids {
        assert_eq!(
            core_registry_field_kind_by_property_key(*key),
            Some(*cpp_kind),
            "property key {key} CoreRegistry::propertyFieldId kind"
        );
    }

    for definition in DEFINITIONS {
        for property in definition.properties {
            let expected = CoreRegistryFieldKind::from_property(property);
            assert_eq!(
                cpp_field_ids.get(&property.key.int).copied(),
                expected,
                "{}.{} propertyFieldId entry",
                definition.name,
                property.name
            );

            for alternate in property.alternates {
                assert_eq!(
                    cpp_field_ids.get(&alternate.int).copied(),
                    expected,
                    "{}.{} alternate propertyFieldId entry",
                    definition.name,
                    alternate.name
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_core_registry_setter_families() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);
    let cpp_setters = parse_cpp_core_registry_setter_field_kinds(&runtime_dir, &constants);

    for (key, cpp_kind) in &cpp_setters {
        assert_eq!(
            core_registry_setter_field_kind_by_property_key(*key),
            Some(*cpp_kind),
            "property key {key} CoreRegistry setter kind"
        );
    }

    for definition in DEFINITIONS {
        for property in definition.properties {
            let expected = core_registry_setter_field_kind_by_property_key(property.key.int);
            assert_eq!(
                cpp_setters.get(&property.key.int).copied(),
                expected,
                "{}.{} setter entry",
                definition.name,
                property.name
            );

            for alternate in property.alternates {
                assert_eq!(
                    cpp_setters.get(&alternate.int).copied(),
                    expected,
                    "{}.{} alternate setter entry",
                    definition.name,
                    alternate.name
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_core_registry_getter_families() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);
    let cpp_getters = parse_cpp_core_registry_getter_field_kinds(&runtime_dir, &constants);

    for (key, cpp_kind) in &cpp_getters {
        assert_eq!(
            core_registry_getter_field_kind_by_property_key(*key),
            Some(*cpp_kind),
            "property key {key} CoreRegistry getter kind"
        );
    }

    for definition in DEFINITIONS {
        for property in definition.properties {
            let expected = core_registry_getter_field_kind_by_property_key(property.key.int);
            assert_eq!(
                cpp_getters.get(&property.key.int).copied(),
                expected,
                "{}.{} getter entry",
                definition.name,
                property.name
            );

            for alternate in property.alternates {
                assert_eq!(
                    cpp_getters.get(&alternate.int).copied(),
                    expected,
                    "{}.{} alternate getter entry",
                    definition.name,
                    alternate.name
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_core_registry_callbacks() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);
    let cpp_callback_keys =
        parse_cpp_core_registry_callback_property_keys(&runtime_dir, &constants);

    for key in &cpp_callback_keys {
        assert!(
            is_callback_property_key(*key),
            "property key {key} CoreRegistry::isCallback entry"
        );
    }

    for definition in DEFINITIONS {
        for property in definition.properties {
            let expected = property.runtime_type == FieldKind::Callback;
            assert_eq!(
                cpp_callback_keys.contains(&property.key.int),
                expected,
                "{}.{} isCallback entry",
                definition.name,
                property.name
            );

            for alternate in property.alternates {
                assert!(
                    !cpp_callback_keys.contains(&alternate.int),
                    "{}.{} alternate key unexpectedly appears in CoreRegistry::isCallback",
                    definition.name,
                    alternate.name
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_core_registry_object_supports_property() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);
    let cpp_supports = parse_cpp_core_registry_object_supports_property(&runtime_dir, &constants);

    for (property_key, owner_class) in &cpp_supports {
        let owner = owner_class.strip_suffix("Base").unwrap_or(owner_class);

        for definition in DEFINITIONS {
            let expected =
                definition.name == owner || definition.ancestors.iter().any(|name| *name == owner);
            assert_eq!(
                object_supports_property(definition.type_key.int, *property_key),
                expected,
                "{} support for property key {property_key} owned by {owner_class}",
                definition.name
            );
        }
    }

    for definition in DEFINITIONS {
        for property in definition.properties {
            assert_eq!(
                cpp_supports.contains_key(&property.key.int),
                !property.encoded,
                "{}.{} objectSupportsProperty entry",
                definition.name,
                property.name
            );

            for alternate in property.alternates {
                assert_eq!(
                    cpp_supports.contains_key(&alternate.int),
                    !property.encoded,
                    "{}.{} alternate objectSupportsProperty entry",
                    definition.name,
                    alternate.name
                );
            }
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_deserialize_switches() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let constants = parse_u16_constants(&header);
        let cpp_deserialized_keys = parse_cpp_deserialize_property_keys(&header, &constants);

        for property in definition.properties {
            assert_eq!(
                cpp_deserialized_keys.contains(&property.key.int),
                property.deserializes,
                "{}.{} deserialize switch entry in {}",
                definition.name,
                property.name,
                header.display()
            );

            for alternate in property.alternates {
                assert!(
                    !cpp_deserialized_keys.contains(&alternate.int),
                    "{}.{} alternate key unexpectedly deserializes in {}",
                    definition.name,
                    alternate.name,
                    header.display()
                );
            }
        }

        for key in cpp_deserialized_keys {
            assert!(
                definition
                    .properties
                    .iter()
                    .any(|property| property.key.int == key),
                "{} deserialize switch contains key {key} missing from Rust schema",
                definition.name
            );
        }
    }
}

#[test]
fn generated_rust_schema_matches_cpp_is_type_of_switches() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let cpp_type_keys = parse_cpp_is_type_of_type_keys(&header, &constants);
        let expected_type_keys = std::iter::once(definition.type_key.int)
            .chain(definition.ancestors.iter().map(|ancestor| {
                DEFINITIONS
                    .iter()
                    .find(|definition| definition.name == *ancestor)
                    .unwrap_or_else(|| panic!("missing ancestor definition {ancestor}"))
                    .type_key
                    .int
            }))
            .collect::<BTreeSet<_>>();

        assert_eq!(
            cpp_type_keys,
            expected_type_keys,
            "{} isTypeOf switch in {}",
            definition.name,
            header.display()
        );
    }
}

#[test]
fn generated_rust_schema_matches_cpp_make_core_instance_registry() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let constants = parse_generated_class_constants(&runtime_dir);
    let constructible_type_keys = parse_cpp_make_core_instance_type_keys(&runtime_dir, &constants);

    for definition in DEFINITIONS {
        assert_eq!(
            constructible_type_keys.contains(&definition.type_key.int),
            !definition.abstract_,
            "{} constructible in CoreRegistry::makeCoreInstance",
            definition.name
        );
        assert_eq!(
            definition.cloneable, !definition.abstract_,
            "{} cloneable follows constructibility for now",
            definition.name
        );
    }

    for key in constructible_type_keys {
        assert!(
            DEFINITIONS
                .iter()
                .any(|definition| definition.type_key.int == key),
            "CoreRegistry::makeCoreInstance contains type key {key} missing from Rust schema"
        );
    }
}

#[test]
fn generated_rust_schema_matches_cpp_clone_declarations() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let cpp_cloneable = cpp_header_declares_clone(&header);

        assert_eq!(
            definition.cloneable,
            cpp_cloneable,
            "{} clone declaration in {}",
            definition.name,
            header.display()
        );
    }
}

#[test]
fn generated_rust_schema_matches_cpp_clone_implementations() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let source = cpp_generated_source_for(&runtime_dir, definition.file);
        if !source.exists() {
            assert!(
                !definition.cloneable,
                "{} is cloneable but generated C++ clone source is missing at {}",
                definition.name,
                source.display()
            );
            continue;
        }

        let cpp_clone_body = cpp_clone_method_body(&source, definition.name);
        if !definition.cloneable {
            assert!(
                cpp_clone_body.is_none(),
                "{} is abstract but generated C++ clone body exists in {}",
                definition.name,
                source.display()
            );
            continue;
        }

        let cpp_clone_body = cpp_clone_body.unwrap_or_else(|| {
            panic!(
                "{} is cloneable but generated C++ clone body is missing in {}",
                definition.name,
                source.display()
            )
        });

        assert_eq!(
            cpp_clone_constructed_type(&cpp_clone_body),
            Some(definition.name.to_owned()),
            "{} generated clone constructed type in {}",
            definition.name,
            source.display()
        );
        assert!(
            cpp_clone_body
                .lines()
                .any(|line| line.trim() == "cloned->copy(*this);"),
            "{} generated clone body does not copy source object in {}",
            definition.name,
            source.display()
        );
        assert!(
            cpp_clone_body
                .lines()
                .any(|line| line.trim() == "return cloned;"),
            "{} generated clone body does not return cloned object in {}",
            definition.name,
            source.display()
        );
    }
}

#[test]
fn generated_rust_schema_matches_cpp_copy_methods() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    for definition in DEFINITIONS {
        let header = cpp_header_for(&runtime_dir, definition.file);
        let rust_stored_members = definition
            .properties
            .iter()
            .filter(|property| property.stores_field)
            .map(|property| cpp_member_name(property.name))
            .collect::<BTreeSet<_>>();
        let rust_encoded_copy_hooks = definition
            .properties
            .iter()
            .filter(|property| property.encoded)
            .map(|property| cpp_member_name(property.name))
            .collect::<BTreeSet<_>>();

        let Some(copy_body) = cpp_copy_method_body(&header, definition.name) else {
            assert!(
                rust_stored_members.is_empty(),
                "{} has stored members but no generated copy method in {}",
                definition.name,
                header.display()
            );
            assert!(
                rust_encoded_copy_hooks.is_empty(),
                "{} has encoded properties but no generated copy method in {}",
                definition.name,
                header.display()
            );
            continue;
        };

        let cpp_copied_members = cpp_copy_member_assignments(&copy_body);

        assert_eq!(
            cpp_copied_members,
            rust_stored_members,
            "{} copy stored members in {}",
            definition.name,
            header.display()
        );

        let cpp_encoded_copy_hooks = cpp_copy_encoded_property_hooks(&copy_body);

        assert_eq!(
            cpp_encoded_copy_hooks,
            rust_encoded_copy_hooks,
            "{} encoded property copy hooks in {}",
            definition.name,
            header.display()
        );

        let cpp_parent_copy_calls = cpp_copy_parent_calls(&copy_body);
        let rust_parent_copy_calls = definition
            .runtime_parent
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            cpp_parent_copy_calls,
            rust_parent_copy_calls,
            "{} parent copy call in {}",
            definition.name,
            header.display()
        );
    }
}

fn cpp_header_for(runtime_dir: &Path, definition_file: &str) -> PathBuf {
    let stem = definition_file
        .strip_suffix(".json")
        .expect("definition file has .json suffix");
    let header = runtime_dir
        .join("include/rive/generated")
        .join(format!("{stem}_base.hpp"));

    assert!(
        header.exists(),
        "generated C++ header missing for {definition_file}: {}",
        header.display()
    );

    header
}

fn cpp_generated_source_for(runtime_dir: &Path, definition_file: &str) -> PathBuf {
    let stem = definition_file
        .strip_suffix(".json")
        .expect("definition file has .json suffix");
    runtime_dir
        .join("src/generated")
        .join(format!("{stem}_base.cpp"))
}

fn cpp_copy_method_body(header: &Path, definition_name: &str) -> Option<String> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let signature = format!("void copy(const {definition_name}Base& object)");
    let signature_start = source.find(&signature)?;
    let after_signature = &source[signature_start + signature.len()..];
    let body_open_offset = after_signature.find('{').unwrap_or_else(|| {
        panic!(
            "missing copy method body for {signature} in {}",
            header.display()
        )
    });
    let body_start = signature_start + signature.len() + body_open_offset + 1;
    let mut brace_depth = 1usize;

    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Some(source[body_start..body_start + offset].to_owned());
                }
            }
            _ => {}
        }
    }

    panic!(
        "unterminated copy method body for {signature} in {}",
        header.display()
    );
}

fn cpp_clone_method_body(source: &Path, definition_name: &str) -> Option<String> {
    let source_text = std::fs::read_to_string(source)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source.display()));
    let signature = format!("Core* {definition_name}Base::clone() const");
    let signature_start = source_text.find(&signature)?;
    let after_signature = &source_text[signature_start + signature.len()..];
    let body_open_offset = after_signature.find('{').unwrap_or_else(|| {
        panic!(
            "missing clone method body for {signature} in {}",
            source.display()
        )
    });
    let body_start = signature_start + signature.len() + body_open_offset + 1;
    let mut brace_depth = 1usize;

    for (offset, ch) in source_text[body_start..].char_indices() {
        match ch {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Some(source_text[body_start..body_start + offset].to_owned());
                }
            }
            _ => {}
        }
    }

    panic!(
        "unterminated clone method body for {signature} in {}",
        source.display()
    );
}

fn cpp_header_declares_clone(header: &Path) -> bool {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    source
        .lines()
        .any(|line| line.trim() == "Core* clone() const override;")
}

fn cpp_property_setter_body(header: &Path, property_name: &str) -> Option<String> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let signature = format!("void {property_name}(");
    let signature_start = source.find(&signature)?;
    let after_signature = &source[signature_start + signature.len()..];
    let params_end_offset = after_signature.find(')').unwrap_or_else(|| {
        panic!(
            "missing setter parameter list for {signature} in {}",
            header.display()
        )
    });
    let after_params = &after_signature[params_end_offset + 1..];
    let body_open_offset = after_params.find('{')?;
    if after_params
        .find(';')
        .is_some_and(|semicolon_offset| semicolon_offset < body_open_offset)
    {
        return None;
    }

    let body_start =
        signature_start + signature.len() + params_end_offset + 1 + body_open_offset + 1;
    let mut brace_depth = 1usize;

    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Some(source[body_start..body_start + offset].to_owned());
                }
            }
            _ => {}
        }
    }

    panic!(
        "unterminated setter body for {signature} in {}",
        header.display()
    );
}

fn assert_cpp_stored_field_setter_body(
    actual: &[String],
    property_name: &str,
    member: &str,
    definition_name: &str,
    header: &Path,
) {
    let prefix = vec![
        format!("if (m_{member} == value)"),
        "{".to_owned(),
        "return;".to_owned(),
        "}".to_owned(),
        format!("m_{member} = value;"),
        format!("{property_name}Changed();"),
    ];
    assert_cpp_setter_body_with_optional_notify(
        actual,
        &prefix,
        property_name,
        definition_name,
        header,
    );
}

fn assert_cpp_passthrough_setter_body(
    actual: &[String],
    property_name: &str,
    definition_name: &str,
    header: &Path,
) {
    let prefix = vec![
        format!("if ({property_name}() == value)"),
        "{".to_owned(),
        "return;".to_owned(),
        "}".to_owned(),
        format!("set{}(value);", cpp_member_name(property_name)),
        format!("{property_name}Changed();"),
    ];
    assert_cpp_setter_body_with_optional_notify(
        actual,
        &prefix,
        property_name,
        definition_name,
        header,
    );
}

fn assert_cpp_setter_body_with_optional_notify(
    actual: &[String],
    prefix: &[String],
    property_name: &str,
    definition_name: &str,
    header: &Path,
) {
    let mut with_notify = prefix.to_vec();
    with_notify.push(format!(
        "notifyPropertyChanged({property_name}PropertyKey);"
    ));

    assert!(
        actual == prefix || actual == with_notify,
        "{}.{} generated setter body in {}",
        definition_name,
        property_name,
        header.display()
    );
}

#[derive(Debug)]
enum CppGetter {
    Body {
        signature: String,
        body: Vec<String>,
    },
    Declaration(String),
}

fn cpp_property_getter(header: &Path, property_name: &str) -> Option<CppGetter> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let getter = format!(" {property_name}()");
    let lines = source.lines().map(str::trim).collect::<Vec<_>>();

    for (index, line) in lines.iter().enumerate() {
        if !line.contains(&getter) {
            continue;
        }

        if line.ends_with(';') {
            return Some(CppGetter::Declaration((*line).to_owned()));
        }

        if let Some((signature, body_start)) = line.split_once('{') {
            let body_start = body_start.trim();
            if let Some(body) = body_start.strip_suffix('}') {
                return Some(CppGetter::Body {
                    signature: signature.trim().to_owned(),
                    body: cpp_trimmed_lines(body),
                });
            }

            let mut body = cpp_trimmed_lines(body_start);
            for next_line in &lines[index + 1..] {
                if *next_line == "}" {
                    return Some(CppGetter::Body {
                        signature: signature.trim().to_owned(),
                        body,
                    });
                }
                body.push((*next_line).to_owned());
            }

            panic!(
                "unterminated getter body for {property_name} in {}",
                header.display()
            );
        }

        assert_eq!(
            lines.get(index + 1).copied(),
            Some("{"),
            "missing getter body for {property_name} in {}",
            header.display()
        );
        let mut body = Vec::new();
        for next_line in &lines[index + 2..] {
            if *next_line == "}" {
                return Some(CppGetter::Body {
                    signature: (*line).to_owned(),
                    body,
                });
            }
            body.push((*next_line).to_owned());
        }

        panic!(
            "unterminated getter body for {property_name} in {}",
            header.display()
        );
    }

    None
}

fn assert_cpp_stored_field_getter_body(
    actual: &CppGetter,
    property_name: &str,
    member: &str,
    is_virtual: bool,
    is_override: bool,
    definition_name: &str,
    header: &Path,
) {
    let CppGetter::Body { signature, body } = actual else {
        panic!(
            "{}.{} generated getter is not a body in {}: {actual:?}",
            definition_name,
            property_name,
            header.display()
        );
    };
    assert!(
        signature.starts_with(if is_virtual { "virtual " } else { "inline " }),
        "{}.{} generated getter storage class in {}: {signature}",
        definition_name,
        property_name,
        header.display()
    );
    assert!(
        signature.contains(&format!(" {property_name}() const")),
        "{}.{} generated getter signature in {}: {signature}",
        definition_name,
        property_name,
        header.display()
    );
    assert_eq!(
        signature.contains(" override"),
        is_override,
        "{}.{} generated getter override marker in {}: {signature}",
        definition_name,
        property_name,
        header.display()
    );
    assert_eq!(
        body,
        &vec![format!("return m_{member};")],
        "{}.{} generated getter return member in {}",
        definition_name,
        property_name,
        header.display()
    );
}

fn assert_cpp_passthrough_getter_declaration(
    actual: &CppGetter,
    property_name: &str,
    definition_name: &str,
    header: &Path,
) {
    let CppGetter::Declaration(actual) = actual else {
        panic!(
            "{}.{} generated passthrough getter is not a declaration in {}: {actual:?}",
            definition_name,
            property_name,
            header.display()
        );
    };
    assert!(
        actual.starts_with("virtual "),
        "{}.{} generated passthrough getter storage class in {}: {actual}",
        definition_name,
        property_name,
        header.display()
    );
    assert!(
        actual.contains(&format!(" {property_name}() = 0;")),
        "{}.{} generated passthrough getter declaration in {}: {actual}",
        definition_name,
        property_name,
        header.display()
    );
    assert!(
        !actual.contains(" const "),
        "{}.{} generated passthrough getter unexpectedly const in {}: {actual}",
        definition_name,
        property_name,
        header.display()
    );
}

fn cpp_zero_declarations(header: &Path) -> BTreeSet<String> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let mut declarations = BTreeSet::new();
    let mut current = Vec::<String>::new();

    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        if current.is_empty()
            && !((line.starts_with("virtual ") || line.starts_with("void "))
                && line.contains('(')
                && !line.contains('{'))
        {
            continue;
        }

        current.push(line.to_owned());
        if !line.ends_with(';') {
            continue;
        }

        let declaration = cpp_normalize_whitespace(&current.join(" "));
        current.clear();

        if declaration.ends_with("= 0;") {
            declarations.insert(declaration);
        }
    }

    declarations
}

fn cpp_zero_declaration(declarations: &BTreeSet<String>, method_name: &str) -> Option<String> {
    let needle = format!("void {method_name}(");
    declarations
        .iter()
        .find(|declaration| declaration.contains(&needle))
        .cloned()
}

fn assert_cpp_value_setter_declaration(
    declaration: &str,
    property_name: &str,
    is_override: bool,
    definition_name: &str,
    header: &Path,
) {
    assert_cpp_void_zero_declaration_prefix(
        declaration,
        is_override,
        definition_name,
        property_name,
        header,
    );
    assert!(
        declaration.contains(&format!("void {property_name}(")),
        "{}.{} value setter declaration signature in {}: {declaration}",
        definition_name,
        property_name,
        header.display()
    );
}

fn assert_cpp_encoded_decode_declaration(
    declaration: &str,
    member: &str,
    is_override: bool,
    definition_name: &str,
    property_name: &str,
    header: &Path,
) {
    assert_cpp_void_zero_declaration_prefix(
        declaration,
        is_override,
        definition_name,
        property_name,
        header,
    );
    assert!(
        declaration.contains(&format!("void decode{member}(")) && declaration.contains(" value)"),
        "{}.{} encoded decode declaration signature in {}: {declaration}",
        definition_name,
        property_name,
        header.display()
    );
}

fn assert_cpp_encoded_copy_declaration(
    declaration: &str,
    member: &str,
    is_override: bool,
    definition_name: &str,
    property_name: &str,
    header: &Path,
) {
    assert_cpp_void_zero_declaration_prefix(
        declaration,
        is_override,
        definition_name,
        property_name,
        header,
    );
    assert!(
        declaration.contains(&format!("void copy{member}("))
            && declaration.contains(&format!("const {definition_name}Base& object)")),
        "{}.{} encoded copy declaration signature in {}: {declaration}",
        definition_name,
        property_name,
        header.display()
    );
}

fn assert_cpp_void_zero_declaration_prefix(
    declaration: &str,
    is_override: bool,
    definition_name: &str,
    property_name: &str,
    header: &Path,
) {
    assert!(
        declaration.starts_with(if is_override {
            "void "
        } else {
            "virtual void "
        }),
        "{}.{} pure-virtual declaration prefix in {}: {declaration}",
        definition_name,
        property_name,
        header.display()
    );
    assert!(
        declaration.ends_with("= 0;"),
        "{}.{} pure-virtual declaration terminator in {}: {declaration}",
        definition_name,
        property_name,
        header.display()
    );
    assert_eq!(
        declaration.contains(" override "),
        is_override,
        "{}.{} pure-virtual declaration override marker in {}: {declaration}",
        definition_name,
        property_name,
        header.display()
    );
}

fn cpp_normalize_whitespace(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cpp_clone_constructed_type(clone_body: &str) -> Option<String> {
    clone_body.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix("auto cloned = new ")?;
        Some(rest.strip_suffix("();").unwrap_or(rest).to_owned())
    })
}

fn cpp_copy_member_assignments(copy_body: &str) -> BTreeSet<String> {
    copy_body
        .split(';')
        .filter_map(|statement| {
            let statement = statement.trim();
            let rest = statement.strip_prefix("m_")?;
            let (member, copied_member) = rest
                .split_once(" = object.m_")
                .unwrap_or_else(|| panic!("bad generated member copy statement: {statement:?}"));
            let copied_member = copied_member.trim();
            assert_eq!(
                member, copied_member,
                "generated copy statement copies a different member: {statement:?}"
            );
            Some(member.to_owned())
        })
        .collect()
}

fn cpp_copy_encoded_property_hooks(copy_body: &str) -> BTreeSet<String> {
    copy_body
        .split(';')
        .filter_map(|statement| {
            let statement = statement.trim();
            if statement.contains("::") {
                return None;
            }
            let Some(rest) = statement.strip_prefix("copy") else {
                return None;
            };
            let Some(property_suffix) = rest.strip_suffix("(object)") else {
                return None;
            };
            Some(property_suffix.to_owned())
        })
        .collect()
}

fn cpp_copy_parent_calls(copy_body: &str) -> BTreeSet<String> {
    copy_body
        .split(';')
        .filter_map(|statement| {
            let statement = statement.trim();
            let (parent, method) = statement.split_once("::")?;
            (method == "copy(object)").then(|| parent.to_owned())
        })
        .collect()
}

fn cpp_notified_property_names(header: &Path) -> BTreeSet<String> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));

    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let property = line.strip_prefix("notifyPropertyChanged(")?;
            let property = property.strip_suffix("PropertyKey);").unwrap_or_else(|| {
                panic!(
                    "bad generated notifyPropertyChanged call in {}: {line}",
                    header.display()
                )
            });
            Some(property.to_owned())
        })
        .collect()
}

fn cpp_changed_hook_names(header: &Path) -> BTreeSet<String> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));

    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let property = line.strip_prefix("virtual void ")?;
            let property = property.strip_suffix("Changed() {}")?;
            Some(property.to_owned())
        })
        .collect()
}

fn cpp_trimmed_lines(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_cpp_make_core_instance_type_keys(
    runtime_dir: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeSet<u16> {
    let path = runtime_dir.join("include/rive/generated/core_registry.hpp");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut keys = BTreeSet::new();
    let mut in_function = false;

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("static Core* makeCoreInstance") {
            in_function = true;
            continue;
        }
        if !in_function {
            continue;
        }
        if line.starts_with("static void setUint") {
            break;
        }

        let Some(rest) = line.strip_prefix("case ") else {
            continue;
        };
        let case = rest.trim_end_matches(':').trim();
        let key = constants
            .get(case)
            .copied()
            .unwrap_or_else(|| panic!("missing generated constant {case} in {}", path.display()));
        keys.insert(key);
    }

    keys
}

fn parse_cpp_is_type_of_type_keys(
    header: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeSet<u16> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let mut keys = BTreeSet::new();
    let mut in_is_type_of = false;

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("bool isTypeOf(uint16_t typeKey) const override") {
            in_is_type_of = true;
            continue;
        }
        if !in_is_type_of {
            continue;
        }
        if line.starts_with("uint16_t coreType()") {
            break;
        }

        let Some(rest) = line.strip_prefix("case ") else {
            continue;
        };
        let case = rest.trim_end_matches(':').trim();
        let key = constants
            .get(case)
            .copied()
            .unwrap_or_else(|| panic!("missing generated constant {case} in {}", header.display()));
        keys.insert(key);
    }

    keys
}

fn parse_cpp_deserialize_property_keys(
    header: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeSet<u16> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let mut keys = BTreeSet::new();
    let mut in_deserialize = false;

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("bool deserialize(uint16_t propertyKey") {
            in_deserialize = true;
            continue;
        }
        if !in_deserialize {
            continue;
        }
        if line.starts_with("protected:") || line.starts_with("};") {
            break;
        }

        let Some(rest) = line.strip_prefix("case ") else {
            continue;
        };
        let case = rest.trim_end_matches(':').trim();
        let key = constants
            .get(case)
            .copied()
            .unwrap_or_else(|| panic!("missing generated constant {case} in {}", header.display()));
        keys.insert(key);
    }

    keys
}

fn parse_generated_class_constants(runtime_dir: &Path) -> BTreeMap<String, u16> {
    let mut constants = BTreeMap::new();
    collect_generated_class_constants(&runtime_dir.join("include/rive/generated"), &mut constants);
    constants
}

fn collect_generated_class_constants(dir: &Path, constants: &mut BTreeMap<String, u16>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read dir entry: {err}"));
        let path = entry.path();
        if path.is_dir() {
            collect_generated_class_constants(&path, constants);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("hpp") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if !stem.ends_with("_base") {
            continue;
        }

        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let class_name = source
            .lines()
            .find_map(|line| {
                let line = line.trim();
                let rest = line.strip_prefix("class ")?;
                rest.split_whitespace().next()
            })
            .unwrap_or_else(|| panic!("missing class declaration in {}", path.display()));

        for (name, value) in parse_u16_constants_from_source(&source, &path) {
            constants.insert(format!("{class_name}::{name}"), value);
        }
    }
}

fn parse_cpp_core_registry_property_field_ids(
    runtime_dir: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeMap<u16, CoreRegistryFieldKind> {
    let path = runtime_dir.join("include/rive/generated/core_registry.hpp");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let mut in_function = false;
    let mut pending_cases = Vec::<String>::new();
    let mut pending_class = None::<String>;
    let mut field_ids = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("static int propertyFieldId") {
            in_function = true;
            continue;
        }
        if !in_function {
            continue;
        }
        if line.starts_with("static bool isCallback") {
            break;
        }

        if let Some(rest) = line.strip_prefix("case ") {
            let rest = rest.trim();
            if let Some(class) = rest.strip_suffix("::") {
                pending_class = Some(class.to_string());
            } else {
                let case = rest.trim_end_matches(':').trim();
                pending_cases.push(case.to_string());
            }
            continue;
        }

        if let Some(class) = pending_class.take() {
            if line.ends_with("PropertyKey:") {
                pending_cases.push(format!("{}::{}", class, line.trim_end_matches(':')));
                continue;
            }
        }

        let kind = if line.contains("return CoreUintType::id") {
            Some(CoreRegistryFieldKind::Uint)
        } else if line.contains("return CoreStringType::id")
            || line.contains("return CoreBytesType::id")
        {
            Some(CoreRegistryFieldKind::StringOrBytes)
        } else if line.contains("return CoreDoubleType::id") {
            Some(CoreRegistryFieldKind::Double)
        } else if line.contains("return CoreColorType::id") {
            Some(CoreRegistryFieldKind::Color)
        } else if line.contains("return CoreBoolType::id") {
            Some(CoreRegistryFieldKind::Bool)
        } else {
            None
        };

        if let Some(kind) = kind {
            for case in pending_cases.drain(..) {
                let key = constants
                    .get(&case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                let previous = field_ids.insert(key, kind);
                assert!(
                    previous.is_none() || previous == Some(kind),
                    "conflicting CoreRegistry field ids for property key {key}"
                );
            }
        }
    }

    field_ids
}

fn parse_cpp_core_registry_setter_field_kinds(
    runtime_dir: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeMap<u16, FieldKind> {
    let path = runtime_dir.join("include/rive/generated/core_registry.hpp");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let mut current_kind = None::<FieldKind>;
    let mut pending_class = None::<String>;
    let mut setters = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();

        current_kind = match line {
            line if line.starts_with("static void setUint") => Some(FieldKind::Uint),
            line if line.starts_with("static void setString") => Some(FieldKind::String),
            line if line.starts_with("static void setColor") => Some(FieldKind::Color),
            line if line.starts_with("static void setBool") => Some(FieldKind::Bool),
            line if line.starts_with("static void setDouble") => Some(FieldKind::Double),
            line if line.starts_with("static void setCallback") => Some(FieldKind::Callback),
            line if line.starts_with("static uint32_t getUint") => break,
            _ => current_kind,
        };

        let Some(kind) = current_kind else {
            continue;
        };

        if let Some(rest) = line.strip_prefix("case ") {
            let rest = rest.trim();
            if let Some(class) = rest.strip_suffix("::") {
                pending_class = Some(class.to_string());
            } else {
                let case = rest.trim_end_matches(':').trim();
                let key = constants
                    .get(case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                let previous = setters.insert(key, kind);
                assert!(
                    previous.is_none() || previous == Some(kind),
                    "conflicting CoreRegistry setter kinds for property key {key}"
                );
            }
            continue;
        }

        if let Some(class) = pending_class.take() {
            if line.ends_with("PropertyKey:") {
                let case = format!("{}::{}", class, line.trim_end_matches(':'));
                let key = constants
                    .get(&case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                let previous = setters.insert(key, kind);
                assert!(
                    previous.is_none() || previous == Some(kind),
                    "conflicting CoreRegistry setter kinds for property key {key}"
                );
                continue;
            }
        }
    }

    setters
}

fn parse_cpp_core_registry_getter_field_kinds(
    runtime_dir: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeMap<u16, FieldKind> {
    let path = runtime_dir.join("include/rive/generated/core_registry.hpp");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let mut current_kind = None::<FieldKind>;
    let mut pending_class = None::<String>;
    let mut getters = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();

        current_kind = match line {
            line if line.starts_with("static uint32_t getUint") => Some(FieldKind::Uint),
            line if line.starts_with("static std::string getString") => Some(FieldKind::String),
            line if line.starts_with("static int getColor") => Some(FieldKind::Color),
            line if line.starts_with("static bool getBool") => Some(FieldKind::Bool),
            line if line.starts_with("static float getDouble") => Some(FieldKind::Double),
            line if line.starts_with("static int propertyFieldId") => break,
            _ => current_kind,
        };

        let Some(kind) = current_kind else {
            continue;
        };

        if let Some(rest) = line.strip_prefix("case ") {
            let rest = rest.trim();
            if let Some(class) = rest.strip_suffix("::") {
                pending_class = Some(class.to_string());
            } else {
                let case = rest.trim_end_matches(':').trim();
                let key = constants
                    .get(case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                let previous = getters.insert(key, kind);
                assert!(
                    previous.is_none() || previous == Some(kind),
                    "conflicting CoreRegistry getter kinds for property key {key}"
                );
            }
            continue;
        }

        if let Some(class) = pending_class.take() {
            if line.ends_with("PropertyKey:") {
                let case = format!("{}::{}", class, line.trim_end_matches(':'));
                let key = constants
                    .get(&case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                let previous = getters.insert(key, kind);
                assert!(
                    previous.is_none() || previous == Some(kind),
                    "conflicting CoreRegistry getter kinds for property key {key}"
                );
                continue;
            }
        }
    }

    getters
}

fn parse_cpp_core_registry_callback_property_keys(
    runtime_dir: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeSet<u16> {
    let path = runtime_dir.join("include/rive/generated/core_registry.hpp");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let mut in_function = false;
    let mut pending_class = None::<String>;
    let mut callback_keys = BTreeSet::new();

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("static bool isCallback") {
            in_function = true;
            continue;
        }
        if !in_function {
            continue;
        }
        if line.starts_with("static bool objectSupportsProperty") {
            break;
        }

        if let Some(rest) = line.strip_prefix("case ") {
            let rest = rest.trim();
            if let Some(class) = rest.strip_suffix("::") {
                pending_class = Some(class.to_string());
            } else {
                let case = rest.trim_end_matches(':').trim();
                let key = constants
                    .get(case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                callback_keys.insert(key);
            }
            continue;
        }

        if let Some(class) = pending_class.take() {
            if line.ends_with("PropertyKey:") {
                let case = format!("{}::{}", class, line.trim_end_matches(':'));
                let key = constants
                    .get(&case)
                    .copied()
                    .unwrap_or_else(|| panic!("missing generated constant {case}"));
                callback_keys.insert(key);
                continue;
            }
        }
    }

    callback_keys
}

fn parse_cpp_core_registry_object_supports_property(
    runtime_dir: &Path,
    constants: &BTreeMap<String, u16>,
) -> BTreeMap<u16, String> {
    let path = runtime_dir.join("include/rive/generated/core_registry.hpp");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    let mut in_function = false;
    let mut pending_cases = Vec::<String>::new();
    let mut pending_class = None::<String>;
    let mut supports = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("static bool objectSupportsProperty") {
            in_function = true;
            continue;
        }
        if !in_function {
            continue;
        }
        if line == "return false;" {
            break;
        }

        if let Some(rest) = line.strip_prefix("case ") {
            let rest = rest.trim();
            if let Some(class) = rest.strip_suffix("::") {
                pending_class = Some(class.to_string());
            } else {
                let case = rest.trim_end_matches(':').trim();
                pending_cases.push(case.to_string());
            }
            continue;
        }

        if let Some(class) = pending_class.take() {
            if line.ends_with("PropertyKey:") {
                pending_cases.push(format!("{}::{}", class, line.trim_end_matches(':')));
                continue;
            }
        }

        let Some(rest) = line.strip_prefix("return object->is<") else {
            continue;
        };
        let owner = rest
            .strip_suffix(">();")
            .unwrap_or_else(|| panic!("bad objectSupportsProperty return line: {line}"))
            .to_string();

        for case in pending_cases.drain(..) {
            let key = constants
                .get(&case)
                .copied()
                .unwrap_or_else(|| panic!("missing generated constant {case}"));
            let previous = supports.insert(key, owner.clone());
            assert!(
                previous.is_none() || previous.as_ref() == Some(&owner),
                "conflicting objectSupportsProperty owners for property key {key}"
            );
        }
    }

    supports
}

fn parse_u16_constants(header: &Path) -> BTreeMap<String, u16> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    parse_u16_constants_from_source(&source, header)
}

fn parse_u32_constants(header: &Path) -> BTreeMap<String, u32> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let mut constants = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("static const uint32_t ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(" = ") else {
            continue;
        };
        let Some(value) = value.strip_suffix(';') else {
            continue;
        };
        constants.insert(
            name.to_string(),
            parse_u32_constant_value(value, &format!("{}::{name}", header.display())),
        );
    }

    constants
}

fn parse_cpp_member_initializers(header: &Path) -> BTreeMap<String, String> {
    let source = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let mut initializers = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();
        if !line.ends_with(';') {
            continue;
        }

        let Some((type_name, rest)) = line.split_once(" m_") else {
            continue;
        };
        if !matches!(
            type_name,
            "bool" | "float" | "int" | "std::string" | "uint32_t"
        ) {
            continue;
        }

        let Some((member, value)) = rest.split_once(" = ") else {
            continue;
        };
        let Some(value) = value.strip_suffix(';') else {
            continue;
        };

        initializers.insert(member.to_string(), value.to_string());
    }

    initializers
}

fn cpp_member_name(property_name: &str) -> String {
    let mut chars = property_name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut member = String::new();
    member.push(first.to_ascii_uppercase());
    member.extend(chars);
    member
}

fn parse_cpp_stored_field_initializer(
    kind: FieldKind,
    cpp_value: &str,
    label: &str,
) -> StoredFieldInitializer {
    match kind {
        FieldKind::Bool => StoredFieldInitializer::Bool(parse_bool_initializer(cpp_value, label)),
        FieldKind::Color => {
            StoredFieldInitializer::Color(parse_color_initializer(cpp_value, label))
        }
        FieldKind::Double => {
            StoredFieldInitializer::Double(parse_float_initializer(cpp_value, label))
        }
        FieldKind::String => {
            StoredFieldInitializer::String(parse_string_initializer(cpp_value, label))
        }
        FieldKind::Uint => StoredFieldInitializer::Uint(parse_uint_initializer(cpp_value, label)),
        FieldKind::Bytes | FieldKind::Callback => {
            panic!("{label} unexpectedly has a stored-field initializer for {kind:?}");
        }
    }
}

fn parse_bool_initializer(value: &str, label: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("{label} has bad bool initializer {value:?}"),
    }
}

fn parse_color_initializer(value: &str, label: &str) -> u32 {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16)
            .unwrap_or_else(|err| panic!("{label} has bad color initializer {value:?}: {err}"))
    } else {
        value
            .parse::<u32>()
            .unwrap_or_else(|err| panic!("{label} has bad color initializer {value:?}: {err}"))
    }
}

fn parse_float_initializer(value: &str, label: &str) -> f32 {
    value
        .strip_suffix('f')
        .unwrap_or(value)
        .parse::<f32>()
        .unwrap_or_else(|err| panic!("{label} has bad float initializer {value:?}: {err}"))
}

fn parse_string_initializer(value: &str, label: &str) -> &'static str {
    match value {
        "\"\"" => "",
        "\"https://public.rive.app/cdn/uuid\"" => "https://public.rive.app/cdn/uuid",
        _ => panic!("{label} has unsupported string initializer {value:?}"),
    }
}

fn parse_uint_initializer(value: &str, label: &str) -> u32 {
    match value {
        "-1" => u32::MAX,
        "Core::invalidPropertyKey" => 0,
        _ => value
            .parse::<u32>()
            .unwrap_or_else(|err| panic!("{label} has bad uint initializer {value:?}: {err}")),
    }
}

fn parse_u32_constant_value(value: &str, label: &str) -> u32 {
    if let Some((lhs, rhs)) = value.split_once(" << ") {
        let base = lhs
            .strip_suffix('u')
            .unwrap_or(lhs)
            .parse::<u32>()
            .unwrap_or_else(|err| panic!("{label} has bad u32 shift base {value:?}: {err}"));
        let shift = rhs
            .parse::<u32>()
            .unwrap_or_else(|err| panic!("{label} has bad u32 shift amount {value:?}: {err}"));
        return base
            .checked_shl(shift)
            .unwrap_or_else(|| panic!("{label} has overflowing u32 shift {value:?}"));
    }

    value
        .strip_suffix('u')
        .unwrap_or(value)
        .parse::<u32>()
        .unwrap_or_else(|err| panic!("{label} has bad u32 constant {value:?}: {err}"))
}

fn parse_u16_constants_from_source(source: &str, header: &Path) -> BTreeMap<String, u16> {
    let mut constants = BTreeMap::new();

    for line in source.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("static const uint16_t ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(" = ") else {
            continue;
        };
        let Some(value) = value.strip_suffix(';') else {
            continue;
        };
        let value = value.parse::<u16>().unwrap_or_else(|err| {
            panic!("bad u16 constant in {}: {line}: {err}", header.display())
        });
        constants.insert(name.to_string(), value);
    }

    constants
}
