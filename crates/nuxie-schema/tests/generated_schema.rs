use nuxie_schema::{
    CoreRegistryFieldKind, FieldKind, StoredFieldInitializer, UintStorage,
    core_registry_field_kind_by_property_key, core_registry_getter_field_kind_by_property_key,
    core_registry_setter_field_kind_by_property_key, definition_by_name, definition_by_type_key,
    generated::{DEFINITIONS, ObjectKind},
    is_callback_property_key, object_supports_property,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn reference_runtime_dir() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

#[test]
fn generated_schema_exposes_current_runtime_definition_set() {
    assert_eq!(DEFINITIONS.len(), 342);

    let runtime_property_count = DEFINITIONS
        .iter()
        .flat_map(|definition| definition.properties)
        .count();
    assert_eq!(runtime_property_count, 596);

    let animatable_property_count = DEFINITIONS
        .iter()
        .flat_map(|definition| definition.properties)
        .filter(|property| property.animates)
        .count();
    assert_eq!(animatable_property_count, 218);

    let grouped_property_count = DEFINITIONS
        .iter()
        .flat_map(|definition| definition.properties)
        .filter(|property| property.group.is_some())
        .count();
    assert_eq!(grouped_property_count, 71);

    let non_coop_property_count = DEFINITIONS
        .iter()
        .flat_map(|definition| definition.properties)
        .filter(|property| !property.coop)
        .count();
    assert_eq!(non_coop_property_count, 8);

    let described_property_count = DEFINITIONS
        .iter()
        .flat_map(|definition| definition.properties)
        .filter(|property| property.description.is_some())
        .count();
    assert_eq!(described_property_count, 446);
}

#[test]
fn generated_schema_metadata_matches_cpp_defs_json() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let defs = read_defs_json(&runtime_dir.join("dev/defs"));
    let expected_files = defs
        .iter()
        .filter_map(|(file, json)| {
            (json_bool_or_default(json, "runtime", true) && json_key_fits_u16(json))
                .then_some(file.clone())
        })
        .collect::<BTreeSet<_>>();
    let actual_files = DEFINITIONS
        .iter()
        .map(|definition| definition.file.to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_files, expected_files,
        "generated definition set should exactly match runtime dev/defs files"
    );

    for definition in DEFINITIONS {
        let json = defs
            .get(definition.file)
            .unwrap_or_else(|| panic!("missing JSON definition for {}", definition.file));

        assert_eq!(
            definition.name,
            json_str(json, "name").unwrap_or(""),
            "{} name",
            definition.file
        );
        assert_eq!(
            (definition.type_key.int, definition.type_key.name),
            json_key_tuple(json),
            "{} key",
            definition.file
        );
        assert_eq!(
            definition.raw_parent_file,
            json_str(json, "extends"),
            "{} raw parent",
            definition.name
        );
        assert_eq!(
            definition.runtime_parent,
            runtime_parent_name(&defs, definition.file).as_deref(),
            "{} runtime parent",
            definition.name
        );
        assert_eq!(
            definition.mixins,
            json_string_array(json, "mixins").as_slice(),
            "{} mixins",
            definition.name
        );
        assert_eq!(
            definition.generic,
            json_str(json, "generic"),
            "{} generic",
            definition.name
        );
        assert_eq!(
            definition.generic_pass_through,
            json_str(json, "genericPassThrough"),
            "{} genericPassThrough",
            definition.name
        );
        assert_eq!(
            definition.exports_with_context,
            json_bool_or_default(json, "exportsWithContext", false),
            "{} exportsWithContext",
            definition.name
        );
        assert_eq!(
            definition.abstract_,
            json_bool_or_default(json, "abstract", false),
            "{} abstract",
            definition.name
        );
        assert_eq!(
            definition.cloneable, !definition.abstract_,
            "{} cloneable",
            definition.name
        );

        let expected_properties = json_runtime_properties(json);
        let actual_property_names = definition
            .properties
            .iter()
            .map(|property| property.name.to_owned())
            .collect::<BTreeSet<_>>();
        let expected_property_names = expected_properties.keys().cloned().collect::<BTreeSet<_>>();
        assert_eq!(
            actual_property_names, expected_property_names,
            "{} runtime properties",
            definition.name
        );

        for property in definition.properties {
            let property_json = expected_properties.get(property.name).unwrap_or_else(|| {
                panic!(
                    "missing JSON property {}.{}",
                    definition.name, property.name
                )
            });
            let label = format!("{}.{}", definition.name, property.name);

            assert_eq!(
                (property.key.int, property.key.name),
                json_key_tuple(property_json),
                "{label} key"
            );
            assert_eq!(
                property
                    .alternates
                    .iter()
                    .map(|key| (key.int, key.name))
                    .collect::<Vec<_>>(),
                json_alternates(property_json),
                "{label} alternates"
            );
            assert_eq!(
                property.declared_type,
                json_str(property_json, "type").unwrap_or(""),
                "{label} type"
            );
            assert_eq!(
                property.runtime_type,
                json_field_kind(property_json),
                "{label} runtime type"
            );
            assert_eq!(
                property.description,
                json_str(property_json, "description"),
                "{label} description"
            );
            assert_eq!(
                property.initial_value,
                json_str(property_json, "initialValue"),
                "{label} initialValue"
            );
            assert_eq!(
                property.initial_value_runtime,
                json_str(property_json, "initialValueRuntime"),
                "{label} initialValueRuntime"
            );
            assert_eq!(
                property.group,
                json_str(property_json, "group"),
                "{label} group"
            );
            assert_eq!(
                property.nullable,
                json_bool_or_default(property_json, "nullable", false),
                "{label} nullable"
            );
            assert_eq!(
                property.override_set,
                json_bool_or_default(property_json, "overrideSet", false),
                "{label} overrideSet"
            );
            assert_eq!(
                property.override_get,
                json_bool_or_default(property_json, "overrideGet", false),
                "{label} overrideGet"
            );
            assert_eq!(
                property.virtual_,
                json_bool_or_default(property_json, "virtual", false),
                "{label} virtual"
            );
            assert_eq!(
                property.editor_only,
                json_bool_or_default(property_json, "editorOnly", false),
                "{label} editorOnly"
            );
            assert_eq!(
                property.coop,
                json_effective_coop(property_json),
                "{label} coop"
            );
            assert_eq!(
                property.with_rive_tools_only,
                json_bool_or_default(property_json, "withRiveToolsOnly", false),
                "{label} withRiveToolsOnly"
            );
            assert_eq!(
                property.encoded,
                json_bool_or_default(property_json, "encoded", false),
                "{label} encoded"
            );
            assert_eq!(
                property.bindable,
                json_bool_or_default(property_json, "bindable", false),
                "{label} bindable"
            );
            assert_eq!(
                property.animates,
                json_bool_or_default(property_json, "animates", false),
                "{label} animates"
            );
            assert_eq!(
                property.computed,
                json_bool_or_default(property_json, "computed", false),
                "{label} computed"
            );
            assert_eq!(
                property.journal,
                json_bool(property_json, "journal"),
                "{label} journal"
            );
            assert_eq!(
                property.parentable,
                json_u64(property_json, "parentable"),
                "{label} parentable"
            );
            assert_eq!(
                property.records,
                json_bool(property_json, "records"),
                "{label} records"
            );
            assert_eq!(
                property.exports_to_runtime_conditionally,
                json_bool_or_default(property_json, "exportsToRuntimeConditionally", false),
                "{label} exportsToRuntimeConditionally"
            );
            assert_eq!(
                property.pure_virtual,
                json_bool_or_default(property_json, "pureVirtual", false),
                "{label} pureVirtual"
            );
            assert_eq!(
                property.passthrough,
                json_bool_or_default(property_json, "passthrough", false),
                "{label} passthrough"
            );
            assert_eq!(
                property.bitmask_passthrough.map(|bitmask| (
                    bitmask.target,
                    bitmask.bit,
                    bitmask.width
                )),
                json_bitmask_passthrough(property_json),
                "{label} bitmask passthrough"
            );
        }
    }
}

#[test]
fn object_kind_and_type_key_lookup_agree() {
    let artboard = ObjectKind::Artboard.definition();
    assert_eq!(artboard.name, "Artboard");
    assert_eq!(artboard.type_key.int, 1);
    assert_eq!(artboard.rust_variant, "Artboard");
    assert!(artboard.cloneable);

    let by_key = definition_by_type_key(1).expect("Artboard type key exists");
    assert_eq!(by_key.name, "Artboard");
    assert!(by_key.is_a("Component"));
    assert!(by_key.is_a("Drawable"));
    assert!(by_key.is_a("WorldTransformComponent"));

    let component = definition_by_name("Component").expect("Component exists");
    assert_eq!(component.type_key.int, 10);
    assert!(component.abstract_);
    assert!(!component.cloneable);
}

#[test]
fn inherited_property_lookup_follows_runtime_parent_chain() {
    let artboard = definition_by_name("Artboard").expect("Artboard exists");

    let name = artboard
        .property_by_key_in_hierarchy(4)
        .expect("Artboard inherits Component.name");
    assert_eq!(name.name, "name");
    assert_eq!(name.runtime_type, FieldKind::String);
    assert!(name.deserializes);

    let width = artboard
        .property_by_key_in_hierarchy(7)
        .expect("Artboard inherits LayoutComponent.width");
    assert_eq!(width.name, "width");
    assert_eq!(width.runtime_type, FieldKind::Double);

    let x = artboard
        .property_by_key_in_hierarchy(9)
        .expect("Artboard accepts Node.xArtboard alternate key");
    assert_eq!(x.name, "x");
    assert_eq!(x.key.int, 13);
    assert!(x.alternates.iter().any(|alternate| alternate.int == 9));
}

#[test]
fn passthrough_properties_are_known_but_not_deserialized_fields() {
    let node = definition_by_name("Node").expect("Node exists");
    let computed_world_x = node
        .property_by_key_in_hierarchy(808)
        .expect("computedWorldX exists");

    assert_eq!(computed_world_x.name, "computedWorldX");
    assert_eq!(computed_world_x.runtime_type, FieldKind::Double);
    assert!(computed_world_x.bindable);
    assert!(!computed_world_x.animates);
    assert!(computed_world_x.passthrough);
    assert!(!computed_world_x.deserializes);
    assert!(!computed_world_x.stores_field);
    assert!(computed_world_x.cpp_generates_changed_hook());
    assert!(computed_world_x.cpp_generates_value_setter_body());
    assert!(!computed_world_x.cpp_generates_stored_field_getter_body());
    assert!(computed_world_x.cpp_generates_passthrough_getter_declaration());
    assert!(!computed_world_x.cpp_generates_pure_virtual_value_setter_declaration());
    assert!(!computed_world_x.cpp_generates_encoded_decode_hook());
    assert!(!computed_world_x.cpp_generates_encoded_copy_hook());
    assert!(computed_world_x.cpp_setter_uses_passthrough());
}

#[test]
fn animates_metadata_matches_cpp_defs_surface() {
    let node = definition_by_name("Node").expect("Node exists");
    let x = node.property_by_key(13).expect("Node.x");
    assert_eq!(x.group, Some("position"));
    assert!(x.override_get);
    assert!(x.animates);
    assert!(node.property_by_key(14).expect("Node.y").animates);

    let event = definition_by_name("Event").expect("Event exists");
    let trigger = event.property_by_key(395).expect("Event.trigger");
    assert_eq!(trigger.runtime_type, FieldKind::Callback);
    assert!(trigger.animates);

    let component = definition_by_name("Component").expect("Component exists");
    assert!(
        !component
            .property_by_key(4)
            .expect("Component.name")
            .animates
    );
}

#[test]
fn cpp_generated_value_setter_metadata_matches_generator_shapes() {
    let node = definition_by_name("Node").expect("Node exists");
    let x = node.property_by_key(13).expect("Node.x");
    assert!(x.cpp_generates_changed_hook());
    assert!(x.cpp_generates_value_setter_body());
    assert!(x.cpp_generates_stored_field_getter_body());
    assert!(!x.cpp_generates_passthrough_getter_declaration());
    assert!(!x.cpp_generates_pure_virtual_value_setter_declaration());
    assert!(!x.cpp_generates_encoded_decode_hook());
    assert!(!x.cpp_generates_encoded_copy_hook());
    assert!(x.cpp_setter_uses_stored_field());
    assert!(!x.cpp_setter_uses_passthrough());

    let file_asset_contents =
        definition_by_name("FileAssetContents").expect("FileAssetContents exists");
    let bytes = file_asset_contents
        .property_by_key(212)
        .expect("FileAssetContents.bytes");
    assert!(bytes.encoded);
    assert!(bytes.cpp_generates_changed_hook());
    assert!(!bytes.cpp_generates_value_setter_body());
    assert!(!bytes.cpp_generates_stored_field_getter_body());
    assert!(!bytes.cpp_generates_passthrough_getter_declaration());
    assert!(!bytes.cpp_generates_pure_virtual_value_setter_declaration());
    assert!(bytes.cpp_generates_encoded_decode_hook());
    assert!(bytes.cpp_generates_encoded_copy_hook());

    let custom_property_trigger =
        definition_by_name("CustomPropertyTrigger").expect("CustomPropertyTrigger exists");
    let fire = custom_property_trigger
        .property_by_key(869)
        .expect("CustomPropertyTrigger.fire");
    assert_eq!(fire.runtime_type, FieldKind::Callback);
    assert!(!fire.cpp_generates_changed_hook());
    assert!(!fire.cpp_generates_value_setter_body());
    assert!(!fire.cpp_generates_stored_field_getter_body());
    assert!(!fire.cpp_generates_passthrough_getter_declaration());
    assert!(fire.cpp_generates_pure_virtual_value_setter_declaration());
    assert!(!fire.cpp_generates_encoded_decode_hook());
    assert!(!fire.cpp_generates_encoded_copy_hook());

    let nested_bool = definition_by_name("NestedBool").expect("NestedBool exists");
    let nested_value = nested_bool
        .property_by_key(238)
        .expect("NestedBool.nestedValue");
    assert!(nested_value.pure_virtual);
    assert!(nested_value.cpp_generates_changed_hook());
    assert!(!nested_value.cpp_generates_value_setter_body());
    assert!(nested_value.cpp_generates_stored_field_getter_body());
    assert!(!nested_value.cpp_generates_passthrough_getter_declaration());
    assert!(nested_value.cpp_generates_pure_virtual_value_setter_declaration());
    assert!(!nested_value.cpp_generates_encoded_decode_hook());
    assert!(!nested_value.cpp_generates_encoded_copy_hook());

    let text = definition_by_name("Text").expect("Text exists");
    let vertical_trim_top = text
        .property_by_key(1027)
        .expect("Text.verticalTrimTopValue");
    assert!(vertical_trim_top.bitmask_passthrough.is_some());
    assert!(!vertical_trim_top.cpp_generates_changed_hook());
    assert!(!vertical_trim_top.cpp_generates_value_setter_body());
    assert!(!vertical_trim_top.cpp_generates_stored_field_getter_body());
    assert!(!vertical_trim_top.cpp_generates_passthrough_getter_declaration());
    assert!(!vertical_trim_top.cpp_generates_pure_virtual_value_setter_declaration());
    assert!(!vertical_trim_top.cpp_generates_encoded_decode_hook());
    assert!(!vertical_trim_top.cpp_generates_encoded_copy_hook());
    assert_eq!(
        vertical_trim_top.cpp_bitmask_passthrough_bitmask_constant(),
        None
    );
    assert_eq!(
        vertical_trim_top.cpp_bitmask_passthrough_bit_offset_constant(),
        Some(0)
    );
    assert_eq!(
        vertical_trim_top.cpp_bitmask_passthrough_field_mask_constant(),
        Some(0xFF)
    );

    let semantic_data = definition_by_name("SemanticData").expect("SemanticData exists");
    let is_expandable = semantic_data
        .property_by_key(989)
        .expect("SemanticData.isExpandable");
    assert_eq!(
        is_expandable.cpp_bitmask_passthrough_bitmask_constant(),
        Some(1)
    );
    assert_eq!(
        is_expandable.cpp_bitmask_passthrough_bit_offset_constant(),
        None
    );
    assert_eq!(
        is_expandable.cpp_bitmask_passthrough_field_mask_constant(),
        None
    );
}

#[test]
fn cpp_generator_metadata_matches_defs_surface() {
    let artboard = definition_by_name("Artboard").expect("Artboard exists");
    assert_eq!(artboard.mixins, &["publishable.json"]);
    assert_eq!(artboard.generic, None);
    assert_eq!(artboard.generic_pass_through, None);
    assert!(!artboard.exports_with_context);

    let straight_vertex = definition_by_name("StraightVertex").expect("StraightVertex exists");
    assert_eq!(straight_vertex.generic, Some("bones/weight.json"));

    let path_vertex = definition_by_name("PathVertex").expect("PathVertex exists");
    assert_eq!(path_vertex.generic_pass_through, Some("bones/weight.json"));

    let cubic_interpolator =
        definition_by_name("CubicInterpolator").expect("CubicInterpolator exists");
    assert!(cubic_interpolator.exports_with_context);

    let file_asset = definition_by_name("FileAsset").expect("FileAsset exists");
    let cdn_uuid = file_asset.property_by_key(359).expect("FileAsset.cdnUuid");
    assert!(!cdn_uuid.coop);
    assert!(!cdn_uuid.editor_only);
    assert!(!cdn_uuid.with_rive_tools_only);

    let file_asset_contents =
        definition_by_name("FileAssetContents").expect("FileAssetContents exists");
    let bytes = file_asset_contents
        .property_by_key(212)
        .expect("FileAssetContents.bytes");
    assert_eq!(bytes.description, Some("Byte data of the file."));
    assert_eq!(bytes.records, Some(false));
    assert_eq!(bytes.journal, None);

    let signature = file_asset_contents
        .property_by_key(911)
        .expect("FileAssetContents.signature");
    assert_eq!(signature.journal, Some(false));
    assert_eq!(signature.records, None);

    let script_asset = definition_by_name("ScriptAsset").expect("ScriptAsset exists");
    let generator_function_ref = script_asset
        .property_by_key(893)
        .expect("ScriptAsset.generatorFunctionRef");
    assert!(generator_function_ref.exports_to_runtime_conditionally);
    assert_eq!(generator_function_ref.journal, Some(false));

    let view_model_instance =
        definition_by_name("ViewModelInstance").expect("ViewModelInstance exists");
    assert_eq!(
        view_model_instance
            .property_by_key(566)
            .expect("ViewModelInstance.viewModelId")
            .parentable,
        Some(1)
    );

    let semantic_data = definition_by_name("SemanticData").expect("SemanticData exists");
    assert!(
        semantic_data
            .property_by_key(998)
            .expect("SemanticData.isChecked")
            .description
            .expect("SemanticData.isChecked description")
            .contains('\u{2014}')
    );

    let shape_paint = definition_by_name("ShapePaint").expect("ShapePaint exists");
    assert!(
        shape_paint
            .properties
            .iter()
            .find(|property| property.name == "isVisible")
            .expect("ShapePaint.isVisible")
            .virtual_
    );

    let nested_bool = definition_by_name("NestedBool").expect("NestedBool exists");
    assert!(
        nested_bool
            .property_by_key(238)
            .expect("NestedBool.nestedValue")
            .pure_virtual
    );
}

#[test]
fn stored_field_initializers_match_cpp_member_defaults() {
    let node = definition_by_name("Node").expect("Node exists");
    assert_eq!(
        node.property_by_key(13)
            .expect("Node.x")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Double(0.0))
    );
    assert_eq!(
        node.property_by_key_in_hierarchy(16)
            .expect("Node.scaleX")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Double(1.0))
    );

    let component = definition_by_name("Component").expect("Component exists");
    assert_eq!(
        component
            .property_by_key(4)
            .expect("Component.name")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::String(""))
    );
    assert_eq!(
        component
            .property_by_key(5)
            .expect("Component.parentId")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Uint(0))
    );

    let solid_color = definition_by_name("SolidColor").expect("SolidColor exists");
    assert_eq!(
        solid_color
            .property_by_key(37)
            .expect("SolidColor.colorValue")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Color(0xFF747474))
    );

    let draw_target = definition_by_name("DrawTarget").expect("DrawTarget exists");
    assert_eq!(
        draw_target
            .property_by_key(119)
            .expect("DrawTarget.drawableId")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Uint(u64::from(u32::MAX)))
    );

    let data_bind = definition_by_name("DataBind").expect("DataBind exists");
    assert_eq!(
        data_bind
            .property_by_key(586)
            .expect("DataBind.propertyKey")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Uint(0))
    );

    let focus_data = definition_by_name("FocusData").expect("FocusData exists");
    assert_eq!(
        focus_data
            .property_by_key(953)
            .expect("FocusData.canFocus")
            .stored_field_initializer(),
        None
    );
    assert_eq!(
        focus_data
            .property_by_key(1033)
            .expect("FocusData.focusFlags")
            .stored_field_initializer(),
        Some(StoredFieldInitializer::Uint(7))
    );

    assert_eq!(
        node.property_by_key(808)
            .expect("Node.computedWorldX passthrough")
            .stored_field_initializer(),
        None
    );
}

#[test]
fn uint_storage_width_preserves_alias_and_uint64_semantics() {
    let file_asset = definition_by_name("FileAsset").expect("FileAsset exists");
    let asset_id = file_asset
        .property_by_key(204)
        .expect("FileAsset.assetId exists");
    let scope_library_id = file_asset
        .property_by_key(1037)
        .expect("FileAsset.scopeLibraryId exists");
    assert_eq!(asset_id.uint_storage(), Some(UintStorage::Uint32));
    assert_eq!(scope_library_id.uint_storage(), Some(UintStorage::Uint64));
    assert_eq!(
        scope_library_id.stored_field_initializer(),
        Some(StoredFieldInitializer::Uint(0))
    );

    let compact_layout_field = definition_by_name("LayoutComponentStyle")
        .expect("LayoutComponentStyle exists")
        .property_by_key(596)
        .expect("LayoutComponentStyle.displayValue exists");
    assert_eq!(compact_layout_field.declared_type, "uint8");
    assert_eq!(compact_layout_field.runtime_type, FieldKind::Uint);
    assert_eq!(
        compact_layout_field.uint_storage(),
        Some(UintStorage::Uint8)
    );

    for key in [596, 1037] {
        assert_eq!(
            core_registry_field_kind_by_property_key(key),
            Some(CoreRegistryFieldKind::Uint)
        );
        assert_eq!(
            core_registry_setter_field_kind_by_property_key(key),
            Some(FieldKind::Uint)
        );
        assert_eq!(
            core_registry_getter_field_kind_by_property_key(key),
            Some(FieldKind::Uint)
        );
    }
}

#[test]
fn core_registry_field_kind_matches_cpp_fallback_families() {
    assert_eq!(
        core_registry_field_kind_by_property_key(5),
        Some(CoreRegistryFieldKind::Uint)
    );
    assert_eq!(
        core_registry_field_kind_by_property_key(4),
        Some(CoreRegistryFieldKind::StringOrBytes)
    );
    assert_eq!(
        core_registry_field_kind_by_property_key(212),
        Some(CoreRegistryFieldKind::StringOrBytes)
    );
    assert_eq!(
        core_registry_field_kind_by_property_key(13),
        Some(CoreRegistryFieldKind::Double)
    );
    assert_eq!(
        core_registry_field_kind_by_property_key(37),
        Some(CoreRegistryFieldKind::Color)
    );
    assert_eq!(
        core_registry_field_kind_by_property_key(245),
        Some(CoreRegistryFieldKind::Bool)
    );
    assert_eq!(core_registry_field_kind_by_property_key(1027), None);
    assert_eq!(core_registry_field_kind_by_property_key(401), None);
}

#[test]
fn callback_property_keys_match_cpp_registry() {
    assert!(is_callback_property_key(395));
    assert!(is_callback_property_key(401));
    assert!(is_callback_property_key(869));
    assert!(is_callback_property_key(1016));

    assert!(!is_callback_property_key(4));
    assert!(!is_callback_property_key(245));
    assert_eq!(core_registry_field_kind_by_property_key(401), None);
}

#[test]
fn core_registry_setter_field_kind_matches_cpp_setter_families() {
    assert_eq!(
        core_registry_setter_field_kind_by_property_key(5),
        Some(FieldKind::Uint)
    );
    assert_eq!(
        core_registry_setter_field_kind_by_property_key(4),
        Some(FieldKind::String)
    );
    assert_eq!(
        core_registry_setter_field_kind_by_property_key(37),
        Some(FieldKind::Color)
    );
    assert_eq!(
        core_registry_setter_field_kind_by_property_key(245),
        Some(FieldKind::Bool)
    );
    assert_eq!(
        core_registry_setter_field_kind_by_property_key(13),
        Some(FieldKind::Double)
    );
    assert_eq!(
        core_registry_setter_field_kind_by_property_key(401),
        Some(FieldKind::Callback)
    );

    assert_eq!(core_registry_setter_field_kind_by_property_key(212), None);
}

#[test]
fn core_registry_getter_field_kind_matches_cpp_getter_families() {
    assert_eq!(
        core_registry_getter_field_kind_by_property_key(5),
        Some(FieldKind::Uint)
    );
    assert_eq!(
        core_registry_getter_field_kind_by_property_key(4),
        Some(FieldKind::String)
    );
    assert_eq!(
        core_registry_getter_field_kind_by_property_key(37),
        Some(FieldKind::Color)
    );
    assert_eq!(
        core_registry_getter_field_kind_by_property_key(245),
        Some(FieldKind::Bool)
    );
    assert_eq!(
        core_registry_getter_field_kind_by_property_key(13),
        Some(FieldKind::Double)
    );
    assert_eq!(
        core_registry_getter_field_kind_by_property_key(1027),
        Some(FieldKind::Uint)
    );

    assert_eq!(core_registry_getter_field_kind_by_property_key(989), None);
    assert_eq!(core_registry_getter_field_kind_by_property_key(401), None);
    assert_eq!(core_registry_getter_field_kind_by_property_key(212), None);
}

#[test]
fn object_supports_property_follows_cpp_registry_semantics() {
    assert!(object_supports_property(1, 4)); // Artboard inherits Component.name.
    assert!(object_supports_property(1, 9)); // Artboard supports Node.xArtboard alternate.
    assert!(object_supports_property(122, 401)); // NestedTrigger.fire callback.

    assert!(!object_supports_property(122, 13)); // NestedTrigger is not a Node transform.
    assert!(!object_supports_property(106, 212)); // Encoded FileAssetContents.bytes payload.
    assert!(!object_supports_property(65_000, 4)); // Unknown object type.
}

fn read_defs_json(defs_dir: &Path) -> BTreeMap<String, Value> {
    let mut paths = Vec::new();
    collect_json_paths(defs_dir, &mut paths);

    paths
        .into_iter()
        .map(|path| {
            let file = path
                .strip_prefix(defs_dir)
                .unwrap_or_else(|err| panic!("failed to relativize {}: {err}", path.display()))
                .to_string_lossy()
                .replace('\\', "/");
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            let json = serde_json::from_str(&contents)
                .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
            (file, json)
        })
        .collect()
}

fn collect_json_paths(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read dir entry: {err}"));
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            out.push(path);
        }
    }
}

fn json_runtime_properties(json: &Value) -> BTreeMap<String, &Value> {
    json.get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter(|(_, property)| {
            json_bool_or_default(property, "runtime", true) && json_key_fits_u16(property)
        })
        .map(|(name, property)| (name.to_owned(), property))
        .collect()
}

fn runtime_parent_name(defs: &BTreeMap<String, Value>, file: &str) -> Option<String> {
    let mut parent_file = json_str(defs.get(file)?, "extends")?;
    loop {
        let parent = defs.get(parent_file)?;
        if json_bool_or_default(parent, "runtime", true) && json_key_fits_u16(parent) {
            return json_str(parent, "name").map(str::to_owned);
        }
        parent_file = json_str(parent, "extends")?;
    }
}

fn json_str<'a>(json: &'a Value, key: &str) -> Option<&'a str> {
    json.get(key).and_then(Value::as_str)
}

fn json_bool(json: &Value, key: &str) -> Option<bool> {
    json.get(key).and_then(Value::as_bool)
}

fn json_bool_or_default(json: &Value, key: &str, default: bool) -> bool {
    json_bool(json, key).unwrap_or(default)
}

fn json_effective_coop(json: &Value) -> bool {
    json_bool(json, "coop").unwrap_or(!json_bool_or_default(json, "editorOnly", false))
}

fn json_u64(json: &Value, key: &str) -> Option<u64> {
    json.get(key).and_then(Value::as_u64)
}

fn json_key_fits_u16(json: &Value) -> bool {
    json_key_int(json).is_some()
}

fn json_key_tuple(json: &Value) -> (u16, &str) {
    let key = json
        .get("key")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("JSON value has no key object: {json:?}"));
    (
        json_key_int(json).expect("key int fits u16"),
        key.get("string")
            .and_then(Value::as_str)
            .expect("key string exists"),
    )
}

fn json_key_int(json: &Value) -> Option<u16> {
    let raw = json
        .get("key")
        .and_then(Value::as_object)?
        .get("int")
        .and_then(Value::as_u64)?;
    u16::try_from(raw).ok()
}

fn json_alternates(json: &Value) -> Vec<(u16, &str)> {
    json.get("key")
        .and_then(Value::as_object)
        .and_then(|key| key.get("alternates"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|alternate| {
            (
                alternate
                    .get("int")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
                    .expect("alternate key int fits u16"),
                alternate
                    .get("string")
                    .and_then(Value::as_str)
                    .expect("alternate key string exists"),
            )
        })
        .collect()
}

fn json_string_array<'a>(json: &'a Value, key: &str) -> Vec<&'a str> {
    json.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|value| value.as_str().expect("array entry is a string"))
        .collect()
}

fn json_field_kind(json: &Value) -> FieldKind {
    match json_str(json, "typeRuntime").or_else(|| json_str(json, "type")) {
        Some("bool") => FieldKind::Bool,
        Some("Bytes") => FieldKind::Bytes,
        Some("callback") => FieldKind::Callback,
        Some("Color") => FieldKind::Color,
        Some("double") => FieldKind::Double,
        Some("String") => FieldKind::String,
        Some("uint") | Some("uint8") | Some("uint64") | Some("Id") | Some("List<Id>") => {
            FieldKind::Uint
        }
        other => panic!("unsupported JSON runtime field kind {other:?}"),
    }
}

fn json_bitmask_passthrough(json: &Value) -> Option<(&str, u8, u8)> {
    Some((
        json_str(json, "passthroughForBitmask")?,
        u8::try_from(json_u64(json, "passthroughBit")?).expect("passthroughBit fits u8"),
        u8::try_from(json_u64(json, "passthroughBitWidth").unwrap_or(1))
            .expect("passthroughBitWidth fits u8"),
    ))
}
