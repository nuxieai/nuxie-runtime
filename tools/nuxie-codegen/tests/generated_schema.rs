use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference_runtime_dir() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

#[test]
fn generated_schema_is_reproducible_from_cpp_defs() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let expected_path = workspace_root().join("crates/nuxie-schema/src/generated/schema.rs");
    let actual_path = std::env::temp_dir().join(format!(
        "rive-rust-generated-schema-{}.rs",
        std::process::id()
    ));

    let output = Command::new(env!("CARGO_BIN_EXE_nuxie-codegen"))
        .arg("--defs")
        .arg(runtime_dir.join("dev/defs"))
        .arg("--out")
        .arg(&actual_path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run nuxie-codegen: {err}"));

    assert!(
        output.status.success(),
        "nuxie-codegen failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rustfmt = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(&actual_path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run rustfmt: {err}"));
    assert!(
        rustfmt.status.success(),
        "rustfmt failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&rustfmt.stdout),
        String::from_utf8_lossy(&rustfmt.stderr)
    );

    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", expected_path.display()));
    let actual = std::fs::read_to_string(&actual_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", actual_path.display()));
    let _ = std::fs::remove_file(&actual_path);

    if actual != expected {
        let mismatch = first_mismatch(&actual, &expected);
        panic!(
            "checked-in generated schema differs from formatted nuxie-codegen output: {mismatch}"
        );
    }
}

#[test]
fn cpp_defs_runtime_type_surface_is_explicitly_tracked() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let defs_dir = runtime_dir.join("dev/defs");
    let mut files = Vec::new();
    collect_json_paths(&defs_dir, &mut files);

    let mut runtime_definition_count = 0usize;
    let mut runtime_property_count = 0usize;
    let mut declared_types = BTreeSet::new();
    let mut runtime_types = BTreeSet::new();
    let mut type_pairs = BTreeMap::<(String, Option<String>), usize>::new();
    let mut mixins = BTreeSet::new();
    let mut generics = BTreeSet::new();
    let mut generic_pass_throughs = BTreeSet::new();
    let mut exports_with_context = BTreeSet::new();

    for path in files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let def: Value = serde_json::from_str(&contents)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));

        if !json_bool_or_default(&def, "runtime", true) || !json_key_fits_u16(&def) {
            continue;
        }
        let file = path
            .strip_prefix(&defs_dir)
            .unwrap_or_else(|err| panic!("failed to relativize {}: {err}", path.display()))
            .to_string_lossy()
            .replace('\\', "/");
        runtime_definition_count += 1;

        if let Some(raw_mixins) = def.get("mixins").and_then(Value::as_array) {
            mixins.insert((
                file.clone(),
                raw_mixins
                    .iter()
                    .map(|value| value.as_str().expect("mixin entry is a string").to_owned())
                    .collect::<Vec<_>>(),
            ));
        }
        if let Some(generic) = def.get("generic").and_then(Value::as_str) {
            generics.insert((file.clone(), generic.to_owned()));
        }
        if let Some(generic_pass_through) = def.get("genericPassThrough").and_then(Value::as_str) {
            generic_pass_throughs.insert((file.clone(), generic_pass_through.to_owned()));
        }
        if json_bool_or_default(&def, "exportsWithContext", false) {
            exports_with_context.insert(file.clone());
        }

        let Some(properties) = def.get("properties").and_then(Value::as_object) else {
            continue;
        };
        for property in properties.values() {
            if !json_bool_or_default(property, "runtime", true) || !json_key_fits_u16(property) {
                continue;
            }

            let declared = property
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{} has runtime property with no type", path.display()));
            let runtime = property
                .get("typeRuntime")
                .and_then(Value::as_str)
                .unwrap_or(declared);
            let runtime_override = property
                .get("typeRuntime")
                .and_then(Value::as_str)
                .map(str::to_owned);

            runtime_property_count += 1;
            declared_types.insert(declared.to_owned());
            runtime_types.insert(runtime.to_owned());
            *type_pairs
                .entry((declared.to_owned(), runtime_override))
                .or_default() += 1;
        }
    }

    assert_eq!(runtime_definition_count, 336);
    assert_eq!(runtime_property_count, 588);
    assert_eq!(
        declared_types,
        [
            "bool", "Bytes", "callback", "Color", "double", "Id", "List<Id>", "String", "uint",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>(),
        "dev/defs declared property type surface changed; audit FieldKind::from_property before refreshing this list"
    );
    assert_eq!(
        runtime_types,
        [
            "bool", "Bytes", "callback", "Color", "double", "String", "uint"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>(),
        "dev/defs runtime property type surface changed; audit FieldKind::from_property before refreshing this list"
    );

    let expected_pairs = [
        (("bool", None), 72usize),
        (("bool", Some("bool")), 1),
        (("Bytes", None), 4),
        (("callback", None), 4),
        (("Color", None), 7),
        (("double", None), 222),
        (("Id", Some("uint")), 83),
        (("List<Id>", Some("Bytes")), 8),
        (("String", None), 23),
        (("String", Some("String")), 1),
        (("uint", None), 159),
        (("uint", Some("uint")), 4),
    ]
    .into_iter()
    .map(|((declared, runtime), count)| ((declared.to_owned(), runtime.map(str::to_owned)), count))
    .collect::<BTreeMap<_, _>>();

    assert_eq!(
        type_pairs, expected_pairs,
        "dev/defs declared/runtime type pair counts changed; audit schema generation and binary decoding assumptions before refreshing this list"
    );

    let expected_mixins = [
        ("artboard.json", vec!["publishable.json"]),
        (
            "script_input_viewmodel_property.json",
            vec!["script_input_common.json"],
        ),
        ("viewmodel/viewmodel.json", vec!["publishable.json"]),
    ]
    .into_iter()
    .map(|(file, values)| {
        (
            file.to_owned(),
            values.into_iter().map(str::to_owned).collect::<Vec<_>>(),
        )
    })
    .collect::<BTreeSet<_>>();
    assert_eq!(
        mixins, expected_mixins,
        "dev/defs mixin surface changed; audit generated Definition::mixins metadata"
    );

    let expected_generics = [
        (
            "animation/blend_state_1d.json",
            "animation/blend_animation_1d.json",
        ),
        (
            "animation/blend_state_direct.json",
            "animation/blend_animation_direct.json",
        ),
        (
            "animation/nested_state_machine.json",
            "animation/state_machine_resolver.json",
        ),
        ("shapes/cubic_vertex.json", "bones/cubic_weight.json"),
        ("shapes/straight_vertex.json", "bones/weight.json"),
    ]
    .into_iter()
    .map(|(file, generic)| (file.to_owned(), generic.to_owned()))
    .collect::<BTreeSet<_>>();
    assert_eq!(
        generics, expected_generics,
        "dev/defs generic surface changed; audit generated Definition::generic metadata"
    );

    assert_eq!(
        generic_pass_throughs,
        [(
            "shapes/path_vertex.json".to_owned(),
            "bones/weight.json".to_owned()
        )]
        .into_iter()
        .collect::<BTreeSet<_>>(),
        "dev/defs genericPassThrough surface changed; audit generated Definition::generic_pass_through metadata"
    );

    let expected_exports_with_context = [
        "animation/cubic_ease_interpolator.json",
        "animation/cubic_interpolator.json",
        "animation/cubic_value_interpolator.json",
        "animation/elastic_interpolator.json",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(
        exports_with_context, expected_exports_with_context,
        "dev/defs exportsWithContext surface changed; audit generated Definition::exports_with_context metadata"
    );
}

#[test]
fn cpp_defs_runtime_property_metadata_surface_is_explicitly_tracked() {
    let runtime_dir = reference_runtime_dir();
    assert!(
        runtime_dir.exists(),
        "reference runtime not found at {}; set RIVE_RUNTIME_DIR",
        runtime_dir.display()
    );

    let properties = runtime_json_properties(&runtime_dir.join("dev/defs"));
    assert_eq!(properties.len(), 588);

    let encoded = properties
        .iter()
        .filter(|entry| json_bool_or_default(&entry.property, "encoded", false))
        .map(|entry| (entry.file.clone(), entry.name.clone(), entry.key))
        .collect::<BTreeSet<_>>();
    let expected_encoded = [
        (
            "animation/listener_types/listener_input_type_viewmodel.json",
            "viewModelPathIds",
            963,
        ),
        (
            "animation/state_machine_fire_trigger.json",
            "viewModelPathIds",
            871,
        ),
        (
            "animation/state_machine_listener_single.json",
            "viewModelPathIds",
            868,
        ),
        ("assets/file_asset.json", "cdnUuid", 359),
        ("assets/file_asset_contents.json", "bytes", 212),
        ("assets/file_asset_contents.json", "signature", 911),
        (
            "data_bind/converters/data_converter_operation_viewmodel.json",
            "sourcePathIds",
            711,
        ),
        ("data_bind/data_bind_context.json", "sourcePathIds", 588),
        ("data_bind/data_bind_path.json", "path", 920),
        ("nested_artboard.json", "dataBindPathIds", 582),
        (
            "script_input_viewmodel_property.json",
            "dataBindPathIds",
            866,
        ),
        ("shapes/mesh.json", "triangleIndexBytes", 223),
    ]
    .into_iter()
    .map(|(file, name, key)| (file.to_owned(), name.to_owned(), key))
    .collect::<BTreeSet<_>>();
    assert_eq!(
        encoded, expected_encoded,
        "dev/defs encoded property surface changed; audit generated decode hooks and binary buffer helpers"
    );

    let alternates = properties
        .iter()
        .flat_map(|entry| {
            entry
                .property
                .get("key")
                .and_then(Value::as_object)
                .and_then(|key| key.get("alternates"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|alternate| {
                    (
                        entry.file.clone(),
                        entry.name.clone(),
                        entry.key,
                        alternate
                            .get("int")
                            .and_then(Value::as_u64)
                            .expect("alternate key has an integer"),
                        alternate
                            .get("string")
                            .and_then(Value::as_str)
                            .expect("alternate key has a string")
                            .to_owned(),
                    )
                })
        })
        .collect::<BTreeSet<_>>();
    let expected_alternates = [
        ("node.json", "x", 13, 9, "xArtboard"),
        ("node.json", "y", 14, 10, "yArtboard"),
    ]
    .into_iter()
    .map(|(file, name, key, alternate_key, alternate_name)| {
        (
            file.to_owned(),
            name.to_owned(),
            key,
            alternate_key,
            alternate_name.to_owned(),
        )
    })
    .collect::<BTreeSet<_>>();
    assert_eq!(
        alternates, expected_alternates,
        "dev/defs alternate key surface changed; audit generated property lookup and C++ deserialize-switch parity"
    );

    let passthrough = properties
        .iter()
        .filter(|entry| json_bool_or_default(&entry.property, "passthrough", false))
        .map(|entry| (entry.file.clone(), entry.name.clone(), entry.key))
        .collect::<BTreeSet<_>>();
    let expected_passthrough = [
        (
            "constraints/scrolling/scroll_constraint.json",
            "scrollPercentX",
            761,
        ),
        (
            "constraints/scrolling/scroll_constraint.json",
            "scrollPercentY",
            762,
        ),
        (
            "constraints/scrolling/scroll_constraint.json",
            "scrollIndex",
            763,
        ),
        (
            "constraints/scrolling/scroll_constraint.json",
            "velocityX",
            1023,
        ),
        (
            "constraints/scrolling/scroll_constraint.json",
            "velocityY",
            1024,
        ),
        (
            "constraints/scrolling/scroll_constraint.json",
            "scrollActive",
            1025,
        ),
        ("node.json", "computedLocalX", 806),
        ("node.json", "computedLocalY", 807),
        ("node.json", "computedWorldX", 808),
        ("node.json", "computedWorldY", 809),
        ("node.json", "computedRootX", 864),
        ("node.json", "computedRootY", 865),
        ("node.json", "computedWidth", 810),
        ("node.json", "computedHeight", 811),
        ("shapes/shape.json", "length", 781),
    ]
    .into_iter()
    .map(|(file, name, key)| (file.to_owned(), name.to_owned(), key))
    .collect::<BTreeSet<_>>();
    assert_eq!(
        passthrough, expected_passthrough,
        "dev/defs passthrough surface changed; audit generated deserializes/stores_field metadata"
    );

    let bitmasks = properties
        .iter()
        .filter_map(|entry| {
            let target = entry
                .property
                .get("passthroughForBitmask")
                .and_then(Value::as_str)?;
            let bit = entry
                .property
                .get("passthroughBit")
                .and_then(Value::as_u64)
                .expect("bitmask passthrough has passthroughBit");
            let width = entry
                .property
                .get("passthroughBitWidth")
                .and_then(Value::as_u64)
                .unwrap_or(1);
            Some((
                entry.file.clone(),
                entry.name.clone(),
                entry.key,
                target.to_owned(),
                bit,
                width,
            ))
        })
        .collect::<BTreeSet<_>>();
    let expected_bitmasks = [
        (
            "semantic/semantic_data.json",
            "isCheckable",
            991,
            "traitFlags",
            2,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isChecked",
            998,
            "stateFlags",
            2,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isDisabled",
            1002,
            "stateFlags",
            6,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isEnablable",
            994,
            "traitFlags",
            5,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isExpandable",
            989,
            "traitFlags",
            0,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isExpanded",
            996,
            "stateFlags",
            0,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isFocusable",
            995,
            "traitFlags",
            6,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isFocused",
            1003,
            "stateFlags",
            7,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isHidden",
            1004,
            "stateFlags",
            8,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isLiveRegion",
            1005,
            "stateFlags",
            9,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isMixed",
            999,
            "stateFlags",
            3,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isModal",
            1007,
            "stateFlags",
            11,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isMultiline",
            1009,
            "stateFlags",
            13,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isObscured",
            1008,
            "stateFlags",
            12,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isReadOnly",
            1006,
            "stateFlags",
            10,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isRequirable",
            993,
            "traitFlags",
            4,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isRequired",
            1001,
            "stateFlags",
            5,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isSelectable",
            990,
            "traitFlags",
            1,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isSelected",
            997,
            "stateFlags",
            1,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isToggleable",
            992,
            "traitFlags",
            3,
            1,
        ),
        (
            "semantic/semantic_data.json",
            "isToggled",
            1000,
            "stateFlags",
            4,
            1,
        ),
        (
            "text/text.json",
            "verticalTrimBottomValue",
            1028,
            "verticalTrimValue",
            8,
            8,
        ),
        (
            "text/text.json",
            "verticalTrimTopValue",
            1027,
            "verticalTrimValue",
            0,
            8,
        ),
    ]
    .into_iter()
    .map(|(file, name, key, target, bit, width)| {
        (
            file.to_owned(),
            name.to_owned(),
            key,
            target.to_owned(),
            bit,
            width,
        )
    })
    .collect::<BTreeSet<_>>();
    assert_eq!(
        bitmasks, expected_bitmasks,
        "dev/defs bitmask passthrough surface changed; audit generated bitmask metadata and bool fallback behavior"
    );

    assert_eq!(
        properties
            .iter()
            .filter(|entry| json_bool_or_default(&entry.property, "bindable", false))
            .count(),
        247,
        "dev/defs bindable property count changed; audit generated bindable metadata"
    );

    assert_eq!(
        properties
            .iter()
            .filter(|entry| json_bool_or_default(&entry.property, "animates", false))
            .count(),
        212,
        "dev/defs animates property count changed; audit generated animates metadata and animation-keyed property support"
    );
    let animates_runtime_types = properties
        .iter()
        .filter(|entry| json_bool_or_default(&entry.property, "animates", false))
        .map(|entry| {
            entry
                .property
                .get("typeRuntime")
                .or_else(|| entry.property.get("type"))
                .and_then(Value::as_str)
                .unwrap_or_else(|| {
                    panic!(
                        "{}:{} has animates with no runtime type",
                        entry.file, entry.name
                    )
                })
        })
        .fold(BTreeMap::<String, usize>::new(), |mut counts, value| {
            *counts.entry(value.to_owned()).or_default() += 1;
            counts
        });
    let expected_animates_runtime_types = [
        ("bool", 25usize),
        ("callback", 4),
        ("Color", 4),
        ("double", 147),
        ("String", 6),
        ("uint", 26),
    ]
    .into_iter()
    .map(|(value, count)| (value.to_owned(), count))
    .collect::<BTreeMap<_, _>>();
    assert_eq!(
        animates_runtime_types, expected_animates_runtime_types,
        "dev/defs animates runtime-type surface changed; audit animation decoding and generated schema metadata"
    );

    assert_eq!(
        properties
            .iter()
            .filter(|entry| entry.property.get("group").is_some())
            .count(),
        71,
        "dev/defs group property count changed; audit generated group metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| json_bool_or_default(&entry.property, "overrideGet", false))
            .count(),
        4,
        "dev/defs overrideGet property count changed; audit generated override_get metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| json_bool_or_default(&entry.property, "virtual", false))
            .count(),
        1,
        "dev/defs virtual property count changed; audit generated virtual_ metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| json_bool_or_default(&entry.property, "pureVirtual", false))
            .count(),
        2,
        "dev/defs pureVirtual property count changed; audit generated pure_virtual metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| !json_effective_coop(&entry.property))
            .count(),
        8,
        "dev/defs coop/editorOnly property surface changed; audit generated coop metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| entry.property.get("description").is_some())
            .count(),
        438,
        "dev/defs description coverage changed; audit generated description metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| entry.property.get("journal").is_some())
            .count(),
        3,
        "dev/defs journal annotation count changed; audit generated journal metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| entry.property.get("records").is_some())
            .count(),
        1,
        "dev/defs records annotation count changed; audit generated records metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| entry.property.get("parentable").is_some())
            .count(),
        1,
        "dev/defs parentable annotation count changed; audit generated parentable metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| {
                json_bool_or_default(&entry.property, "exportsToRuntimeConditionally", false)
            })
            .count(),
        2,
        "dev/defs exportsToRuntimeConditionally property count changed; audit generated conditional export metadata"
    );
    assert_eq!(
        properties
            .iter()
            .filter(|entry| entry.property.get("initialValue").is_some())
            .count(),
        547,
        "dev/defs initialValue coverage changed; audit generated stored-field initializers"
    );

    let runtime_initial_values = properties
        .iter()
        .filter_map(|entry| {
            entry
                .property
                .get("initialValueRuntime")
                .and_then(Value::as_str)
        })
        .fold(BTreeMap::<String, usize>::new(), |mut counts, value| {
            *counts.entry(value.to_owned()).or_default() += 1;
            counts
        });
    let expected_runtime_initial_values = [("-1", 74usize), ("0", 9), ("false", 3), ("true", 1)]
        .into_iter()
        .map(|(value, count)| (value.to_owned(), count))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        runtime_initial_values, expected_runtime_initial_values,
        "dev/defs initialValueRuntime surface changed; audit runtime stored-field initializer parsing"
    );
}

#[test]
fn invalid_bitmask_passthrough_metadata_is_rejected_like_cpp_generator() {
    let cases = [
        (
            "missing_target",
            r#"
            {
              "name": "TestObject",
              "key": { "int": 5000, "string": "test" },
              "properties": {
                "field": {
                  "type": "bool",
                  "key": { "int": 5001, "string": "field" },
                  "passthroughForBitmask": "flags",
                  "passthroughBit": 0
                }
              }
            }
            "#,
            "target \"flags\" not found",
        ),
        (
            "target_not_uint",
            r#"
            {
              "name": "TestObject",
              "key": { "int": 5000, "string": "test" },
              "properties": {
                "flags": {
                  "type": "bool",
                  "key": { "int": 5001, "string": "flags" }
                },
                "field": {
                  "type": "bool",
                  "key": { "int": 5002, "string": "field" },
                  "passthroughForBitmask": "flags",
                  "passthroughBit": 0
                }
              }
            }
            "#,
            "target \"flags\" must be uint",
        ),
        (
            "bool_width",
            r#"
            {
              "name": "TestObject",
              "key": { "int": 5000, "string": "test" },
              "properties": {
                "flags": {
                  "type": "uint",
                  "key": { "int": 5001, "string": "flags" }
                },
                "field": {
                  "type": "bool",
                  "key": { "int": 5002, "string": "field" },
                  "passthroughForBitmask": "flags",
                  "passthroughBit": 0,
                  "passthroughBitWidth": 2
                }
              }
            }
            "#,
            "bool passthroughForBitmask must have width 1",
        ),
        (
            "uint_missing_width",
            r#"
            {
              "name": "TestObject",
              "key": { "int": 5000, "string": "test" },
              "properties": {
                "flags": {
                  "type": "uint",
                  "key": { "int": 5001, "string": "flags" }
                },
                "field": {
                  "type": "uint",
                  "key": { "int": 5002, "string": "field" },
                  "passthroughForBitmask": "flags",
                  "passthroughBit": 0
                }
              }
            }
            "#,
            "uint passthroughForBitmask requires passthroughBitWidth >= 1",
        ),
        (
            "overlap",
            r#"
            {
              "name": "TestObject",
              "key": { "int": 5000, "string": "test" },
              "properties": {
                "flags": {
                  "type": "uint",
                  "key": { "int": 5001, "string": "flags" }
                },
                "first": {
                  "type": "bool",
                  "key": { "int": 5002, "string": "first" },
                  "passthroughForBitmask": "flags",
                  "passthroughBit": 0
                },
                "second": {
                  "type": "bool",
                  "key": { "int": 5003, "string": "second" },
                  "passthroughForBitmask": "flags",
                  "passthroughBit": 0
                }
              }
            }
            "#,
            "overlapping passthrough bit ranges",
        ),
    ];

    for (name, def, expected_stderr) in cases {
        let defs_dir = write_codegen_defs_fixture(name, def);
        let out_path = defs_dir.join("schema.rs");
        let output = run_codegen(&defs_dir, &out_path);
        let _ = std::fs::remove_dir_all(&defs_dir);

        assert!(
            !output.status.success(),
            "nuxie-codegen unexpectedly accepted invalid bitmask fixture {name}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr),
            "expected stderr for {name} to contain {expected_stderr:?}\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn duplicate_and_reserved_keys_are_rejected_like_cpp_generator() {
    let cases = [
        (
            "duplicate_type_key",
            vec![
                (
                    "first.json",
                    r#"
                    {
                      "name": "First",
                      "key": { "int": 5000, "string": "first" }
                    }
                    "#,
                ),
                (
                    "second.json",
                    r#"
                    {
                      "name": "Second",
                      "key": { "int": 5000, "string": "second" }
                    }
                    "#,
                ),
            ],
            "duplicate definition type key 5000",
        ),
        (
            "reserved_property_key",
            vec![(
                "test.json",
                r#"
                {
                  "name": "TestObject",
                  "key": { "int": 5000, "string": "test" },
                  "properties": {
                    "field": {
                      "type": "uint",
                      "key": { "int": 2, "string": "field" }
                    }
                  }
                }
                "#,
            )],
            "ids less than 3 are reserved",
        ),
        (
            "duplicate_property_key",
            vec![
                (
                    "first.json",
                    r#"
                    {
                      "name": "First",
                      "key": { "int": 5000, "string": "first" },
                      "properties": {
                        "field": {
                          "type": "uint",
                          "key": { "int": 5002, "string": "field" }
                        }
                      }
                    }
                    "#,
                ),
                (
                    "second.json",
                    r#"
                    {
                      "name": "Second",
                      "key": { "int": 5001, "string": "second" },
                      "properties": {
                        "other": {
                          "type": "uint",
                          "key": { "int": 5002, "string": "other" }
                        }
                      }
                    }
                    "#,
                ),
            ],
            "duplicate property key 5002",
        ),
    ];

    for (name, files, expected_stderr) in cases {
        let defs_dir = write_codegen_defs_fixture_files(name, &files);
        let out_path = defs_dir.join("schema.rs");
        let output = run_codegen(&defs_dir, &out_path);
        let _ = std::fs::remove_dir_all(&defs_dir);

        assert!(
            !output.status.success(),
            "nuxie-codegen unexpectedly accepted invalid key fixture {name}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr),
            "expected stderr for {name} to contain {expected_stderr:?}\nstderr:\n{stderr}"
        );
    }
}

fn first_mismatch(actual: &str, expected: &str) -> String {
    for (index, (actual, expected)) in actual.lines().zip(expected.lines()).enumerate() {
        if actual != expected {
            return format!(
                "line {} actual {:?}, expected {:?}",
                index + 1,
                actual,
                expected
            );
        }
    }

    format!(
        "line counts differ: actual {}, expected {}",
        actual.lines().count(),
        expected.lines().count()
    )
}

fn write_codegen_defs_fixture(name: &str, def: &str) -> PathBuf {
    write_codegen_defs_fixture_files(name, &[("test.json", def)])
}

fn write_codegen_defs_fixture_files(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rive-rust-codegen-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", dir.display()));
    for (file, contents) in files {
        std::fs::write(dir.join(file), contents)
            .unwrap_or_else(|err| panic!("failed to write {file} in {}: {err}", dir.display()));
    }
    dir
}

fn run_codegen(defs_dir: &Path, out_path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nuxie-codegen"))
        .arg("--defs")
        .arg(defs_dir)
        .arg("--out")
        .arg(out_path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run nuxie-codegen: {err}"))
}

#[derive(Debug)]
struct RuntimeJsonProperty {
    file: String,
    name: String,
    key: u64,
    property: Value,
}

fn runtime_json_properties(defs_dir: &Path) -> Vec<RuntimeJsonProperty> {
    let mut files = Vec::new();
    collect_json_paths(defs_dir, &mut files);
    files.sort();

    let mut properties = Vec::new();
    for path in files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let def: Value = serde_json::from_str(&contents)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
        if !json_bool_or_default(&def, "runtime", true) || !json_key_fits_u16(&def) {
            continue;
        }

        let Some(raw_properties) = def.get("properties").and_then(Value::as_object) else {
            continue;
        };
        for (name, property) in raw_properties {
            if !json_bool_or_default(property, "runtime", true) || !json_key_fits_u16(property) {
                continue;
            }
            let key = property
                .get("key")
                .and_then(Value::as_object)
                .and_then(|key| key.get("int"))
                .and_then(Value::as_u64)
                .expect("runtime property key already validated");
            let file = path
                .strip_prefix(defs_dir)
                .unwrap_or_else(|err| panic!("failed to relativize {}: {err}", path.display()))
                .to_string_lossy()
                .replace('\\', "/");
            properties.push(RuntimeJsonProperty {
                file,
                name: name.clone(),
                key,
                property: property.clone(),
            });
        }
    }

    properties
}

fn collect_json_paths(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read directory {}: {err}", dir.display()));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths(&path, out);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            out.push(path);
        }
    }
}

fn json_bool_or_default(value: &Value, field: &str, default: bool) -> bool {
    value.get(field).and_then(Value::as_bool).unwrap_or(default)
}

fn json_effective_coop(value: &Value) -> bool {
    value
        .get("coop")
        .and_then(Value::as_bool)
        .unwrap_or(!json_bool_or_default(value, "editorOnly", false))
}

fn json_key_fits_u16(value: &Value) -> bool {
    value
        .get("key")
        .and_then(Value::as_object)
        .and_then(|key| key.get("int"))
        .and_then(Value::as_u64)
        .is_some_and(|key| u16::try_from(key).is_ok())
}
