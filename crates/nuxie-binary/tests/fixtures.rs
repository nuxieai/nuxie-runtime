use nuxie_binary::{
    FieldValue, HeaderFieldKind, RuntimeComponentDirt, RuntimeDataBindAddDirtEffect,
    RuntimeDataBindAddEffect, RuntimeDataBindBindEffect, RuntimeDataBindClearSourceEffect,
    RuntimeDataBindCollapseEffect, RuntimeDataBindContainerAddDirtyEffect,
    RuntimeDataBindContainerAdvanceEffect, RuntimeDataBindContainerBindContextEffect,
    RuntimeDataBindContainerUnbindEffect, RuntimeDataBindContainerUpdateEffect,
    RuntimeDataBindContainerUpdateReturnReason, RuntimeDataBindContainerUpdateStep,
    RuntimeDataBindContextBindBranch, RuntimeDataBindContextBindEffect,
    RuntimeDataBindInitializeEffect, RuntimeDataBindRelinkEffect, RuntimeDataBindRemoveEffect,
    RuntimeDataBindSourceEffect, RuntimeDataBindTargetEffect, RuntimeDataBindUnbindEffect,
    RuntimeDataBindUpdateEffect, RuntimeDataBindUpdateQueue,
    RuntimeDataConverterAddDirtyDataBindEffect, RuntimeDataConverterBindContextEffect,
    RuntimeDataConverterFormulaAddDirtEffect, RuntimeDataConverterMarkDirtyEffect,
    RuntimeDataConverterPropertyChangeEffect, RuntimeDataConverterResetEffect,
    RuntimeDataConverterUnbindEffect, RuntimeDataConverterUpdateEffect, RuntimeDataType,
    RuntimeFile, RuntimeImportDropReason, RuntimeImportStatus, RuntimeReadErrorKind,
    SUPPORTED_MAJOR_VERSION, SUPPORTED_MINOR_VERSION, SkipReason, SkippedBitmaskPassthrough,
    read_runtime_file, read_runtime_file_with_error_kind, read_runtime_file_with_scripting,
};
use nuxie_schema::definition_by_name;
use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures")
        .join(path)
}

fn read_fixture(path: &str) -> RuntimeFile {
    let path = fixture(path);
    let bytes = std::fs::read(&path).unwrap_or_else(|err| {
        panic!("failed to read fixture {}: {err}", path.display());
    });
    read_runtime_file(&bytes).unwrap_or_else(|err| {
        panic!("failed to import fixture {}: {err:#}", path.display());
    })
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

fn push_overwide_noncanonical_var_uint(bytes: &mut Vec<u8>, value: u8) {
    bytes.push(value | 0x80);
    bytes.extend_from_slice(&[0x80; 9]);
    bytes.push(0);
}

fn push_empty_object(bytes: &mut Vec<u8>, type_name: &str) {
    let type_key = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int;
    push_var_uint(bytes, u64::from(type_key));
    push_var_uint(bytes, 0);
}

fn property_key_for_name(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return property.key.int;
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)
            .unwrap_or_else(|| panic!("missing ancestor schema definition {ancestor}"));
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return property.key.int;
        }
    }

    panic!("missing property {type_name}.{property_name}");
}

fn push_object_with_properties(
    bytes: &mut Vec<u8>,
    type_name: &str,
    properties: impl FnOnce(&mut Vec<u8>),
) {
    let type_key = definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int;
    push_var_uint(bytes, u64::from(type_key));
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    let property_key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(property_key));
    push_var_uint(bytes, value);
}

fn push_string_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &str) {
    let property_key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(property_key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_bytes_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: &[u8]) {
    let property_key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(property_key));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn synthetic_runtime_file(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    synthetic_runtime_file_with_header(7, 0, file_id, &[], object_stream)
}

fn synthetic_runtime_file_with_header(
    major_version: u64,
    minor_version: u64,
    file_id: u64,
    property_toc: &[(u64, HeaderFieldKind)],
    object_stream: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, major_version);
    push_var_uint(&mut bytes, minor_version);
    push_var_uint(&mut bytes, file_id);

    for (property_key, _) in property_toc {
        push_var_uint(&mut bytes, *property_key);
    }
    push_var_uint(&mut bytes, 0);

    let mut current_int = 0u32;
    let mut current_bit = 0;
    for (index, (_, field)) in property_toc.iter().enumerate() {
        current_int |= u32::from(header_field_id(*field)) << current_bit;
        current_bit += 2;
        if current_bit == 8 || index == property_toc.len() - 1 {
            bytes.extend_from_slice(&current_int.to_le_bytes());
            current_int = 0;
            current_bit = 0;
        }
    }

    object_stream(&mut bytes);
    bytes
}

fn header_field_id(field: HeaderFieldKind) -> u8 {
    match field {
        HeaderFieldKind::Uint => 0,
        HeaderFieldKind::StringOrBytes => 1,
        HeaderFieldKind::Double => 2,
        HeaderFieldKind::Color => 3,
    }
}

#[test]
fn scripting_reader_gives_script_and_shader_contents_their_file_asset_importers() {
    let bytes = synthetic_runtime_file(4400, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ScriptAsset");
        push_empty_object(bytes, "FileAssetContents");
        push_empty_object(bytes, "ShaderAsset");
        push_empty_object(bytes, "FileAssetContents");
    });

    let conformance = read_runtime_file(&bytes).expect("read with non-scripting C++ profile");
    assert_eq!(
        conformance.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject,
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject,
            },
        ]
    );

    let scripting = read_runtime_file_with_scripting(&bytes)
        .expect("read with scripting-enabled FileAsset importers");
    assert_eq!(
        scripting.import_statuses,
        vec![RuntimeImportStatus::Imported; 5]
    );
}

#[test]
fn scripting_file_asset_catalog_uses_importer_ownership_not_adjacency() {
    let bytes = synthetic_runtime_file(4401, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ScriptAsset", |bytes| {
            push_string_property(bytes, "ScriptAsset", "name", "protocol");
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", b"protocol-bytes");
        });
        push_object_with_properties(bytes, "ScriptAsset", |bytes| {
            push_string_property(bytes, "ScriptAsset", "name", "module");
            push_uint_property(bytes, "ScriptAsset", "isModule", 1);
        });
        push_empty_object(bytes, "Node");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", b"module-bytes");
        });
    });
    let file = read_runtime_file_with_scripting(&bytes).expect("scripting import");
    let catalog = file.scripting_file_assets_with_contents();

    assert_eq!(catalog.len(), 2);
    assert_eq!(catalog[0].ordinal, 0);
    assert_eq!(catalog[0].asset.string_property("name"), Some("protocol"));
    assert_eq!(catalog[0].contents, Some(b"protocol-bytes".as_slice()));
    assert_eq!(catalog[1].ordinal, 1);
    assert_eq!(catalog[1].asset.string_property("name"), Some("module"));
    assert_eq!(catalog[1].contents, Some(b"module-bytes".as_slice()));
}

#[test]
fn scripting_file_asset_catalog_respects_manifest_importer_boundary() {
    let bytes = synthetic_runtime_file(4401, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ScriptAsset", |bytes| {
            push_string_property(bytes, "ScriptAsset", "name", "protocol");
        });
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", b"manifest-bytes");
        });
    });
    let file = read_runtime_file_with_scripting(&bytes).expect("scripting import");
    let catalog = file.scripting_file_assets_with_contents();

    assert_eq!(
        catalog.len(),
        1,
        "ManifestAsset is not in the dense catalog"
    );
    assert_eq!(catalog[0].asset.string_property("name"), Some("protocol"));
    assert_eq!(
        catalog[0].contents, None,
        "manifest bytes are not script bytes"
    );
}

#[test]
fn imports_entire_starter_fixture_corpus() {
    let cases = [
        ("minimal/long_name.riv", 9, 9),
        ("minimal/two_artboards.riv", 23, 20),
        ("graph/clipping_and_draw_order.riv", 99, 99),
        ("graph/dependency_test.riv", 19, 18),
        ("graph/draw_rule_cycle.riv", 49, 45),
        ("animation/smi_test.riv", 33, 33),
        ("animation/state_machine_transition.riv", 85, 85),
    ];

    for (path, object_count, known_object_count) in cases {
        let file = read_fixture(path);
        assert_eq!(file.header.major_version, 7, "{path}");
        assert_eq!(file.header.minor_version, 0, "{path}");
        assert_eq!(file.object_count(), object_count, "{path}");
        assert_eq!(file.known_object_count(), known_object_count, "{path}");
    }
}

#[test]
fn header_integer_semantics_match_cpp_runtime() {
    let mut noncanonical_major = Vec::new();
    noncanonical_major.extend_from_slice(b"RIVE");
    noncanonical_major.extend_from_slice(&[0x87, 0x00]);
    push_var_uint(&mut noncanonical_major, 0);
    push_var_uint(&mut noncanonical_major, 0);
    push_var_uint(&mut noncanonical_major, 0);
    let noncanonical_major =
        read_runtime_file(&noncanonical_major).expect("C++ accepts noncanonical LEB128 encodings");
    assert_eq!(noncanonical_major.header.major_version, 7);

    for minor_version in [
        0,
        SUPPORTED_MINOR_VERSION - 1,
        SUPPORTED_MINOR_VERSION,
        SUPPORTED_MINOR_VERSION + 1,
        999,
    ] {
        let file = read_runtime_file(&synthetic_runtime_file_with_header(
            SUPPORTED_MAJOR_VERSION,
            minor_version,
            0,
            &[],
            |_| {},
        ))
        .expect("C++ accepts every minor version with the same major version");
        assert_eq!(file.header.major_version, SUPPORTED_MAJOR_VERSION);
        assert_eq!(file.header.minor_version, minor_version);
    }

    assert!(
        read_runtime_file(&synthetic_runtime_file_with_header(8, 0, 0, &[], |_| {})).is_err(),
        "C++ rejects unsupported major versions"
    );
    assert!(
        read_runtime_file(&synthetic_runtime_file_with_header(
            7,
            i32::MAX as u64 + 1,
            0,
            &[],
            |_| {}
        ))
        .is_err(),
        "C++ treats minor version values outside signed int range as malformed"
    );
    assert!(
        read_runtime_file(&synthetic_runtime_file_with_header(
            7,
            0,
            i32::MAX as u64 + 1,
            &[],
            |_| {}
        ))
        .is_err(),
        "C++ treats file id values outside signed int range as malformed"
    );

    let high_toc_key = read_runtime_file(&synthetic_runtime_file_with_header(
        7,
        0,
        0,
        &[(70_000, HeaderFieldKind::Uint)],
        |_| {},
    ))
    .expect("C++ accepts header ToC property keys above u16 when they fit in signed int");
    assert_eq!(
        high_toc_key.header.property_field_ids.get(&70_000),
        Some(&HeaderFieldKind::Uint)
    );

    let max_toc_key = read_runtime_file(&synthetic_runtime_file_with_header(
        7,
        0,
        0,
        &[(i32::MAX as u64, HeaderFieldKind::Color)],
        |_| {},
    ))
    .expect("C++ accepts header ToC property keys at signed int max");
    assert_eq!(
        max_toc_key
            .header
            .property_field_ids
            .get(&(i32::MAX as u32)),
        Some(&HeaderFieldKind::Color)
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file_with_header(
            i32::MAX as u64 + 1,
            0,
            0,
            &[],
            |_| {}
        ))
        .is_err(),
        "C++ treats major version values outside signed int range as malformed"
    );
    assert!(
        read_runtime_file(&synthetic_runtime_file_with_header(
            7,
            0,
            0,
            &[(i32::MAX as u64 + 1, HeaderFieldKind::Uint)],
            |_| {}
        ))
        .is_err(),
        "C++ treats header ToC keys outside signed int range as malformed"
    );

    let mut truncated_toc_field_ids = Vec::new();
    truncated_toc_field_ids.extend_from_slice(b"RIVE");
    push_var_uint(&mut truncated_toc_field_ids, 7);
    push_var_uint(&mut truncated_toc_field_ids, 0);
    push_var_uint(&mut truncated_toc_field_ids, 0);
    push_var_uint(&mut truncated_toc_field_ids, 4);
    push_var_uint(&mut truncated_toc_field_ids, 0);
    truncated_toc_field_ids.extend_from_slice(&[0x00, 0x00, 0x00]);
    assert!(
        read_runtime_file(&truncated_toc_field_ids).is_err(),
        "C++ treats truncated packed header ToC field ids as malformed"
    );
}

#[test]
fn duplicate_header_toc_keys_overwrite_with_later_field_like_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file_with_header(
        7,
        0,
        4279,
        &[
            (65_000, HeaderFieldKind::Uint),
            (65_000, HeaderFieldKind::Double),
        ],
        |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 65_000);
            bytes.extend_from_slice(&12.5f32.to_le_bytes());
            push_var_uint(bytes, 0);
        },
    ))
    .expect("C++ lets later duplicate property ToC entries overwrite earlier field ids");

    assert_eq!(
        file.header.property_field_ids.get(&65_000),
        Some(&HeaderFieldKind::Double)
    );
    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());
}

#[test]
fn core_registry_fallback_takes_priority_over_conflicting_header_toc_like_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file_with_header(
        7,
        0,
        4281,
        &[(13, HeaderFieldKind::Uint)],
        |bytes| {
            push_var_uint(bytes, 70_000);
            // Node.x is globally known as CoreDoubleType. The header lies and
            // says uint; C++ still consumes the payload as a float32.
            push_var_uint(bytes, 13);
            bytes.extend_from_slice(&12.5f32.to_le_bytes());
            push_var_uint(bytes, 0);
        },
    ))
    .expect("C++ uses CoreRegistry::propertyFieldId before the file header ToC");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());
}

#[test]
fn known_object_alternate_key_uses_core_registry_before_conflicting_header_toc_like_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file_with_header(
        7,
        0,
        4282,
        &[(9, HeaderFieldKind::Uint)],
        |bytes| {
            push_var_uint(bytes, 2);
            // Node.xArtboard is an alternate key for Node.x. The generated C++
            // deserialize switch misses it, then CoreRegistry still wins over
            // the lying header ToC and consumes the payload as a float32.
            push_var_uint(bytes, 9);
            bytes.extend_from_slice(&12.5f32.to_le_bytes());
            push_var_uint(bytes, 0);
        },
    ))
    .expect("C++ uses CoreRegistry fallback before the file header ToC after deserialize misses");

    let node = file.object(0).expect("Node object");
    assert_eq!(node.double_property("x"), Some(0.0));
    assert_eq!(node.skipped_properties.len(), 1);
    assert_eq!(node.skipped_properties[0].key, 9);
    assert_eq!(
        node.skipped_properties[0].reason,
        SkipReason::UnknownProperty
    );
    assert_eq!(node.skipped_properties[0].field, Some("double"));
}

#[test]
fn malformed_runtime_headers_match_cpp_runtime() {
    let cases = [
        ("empty file", Vec::new()),
        ("truncated fingerprint", b"RIV".to_vec()),
        ("bad fingerprint", b"RIVX\x07\x00\x00\x00".to_vec()),
        ("missing major version", b"RIVE".to_vec()),
        ("missing minor version", b"RIVE\x07".to_vec()),
        ("missing file id", b"RIVE\x07\x00".to_vec()),
        (
            "missing property ToC terminator",
            b"RIVE\x07\x00\x00".to_vec(),
        ),
    ];

    for (name, bytes) in cases {
        assert!(
            read_runtime_file(&bytes).is_err(),
            "C++ treats {name} as a malformed runtime header"
        );
    }
}

#[test]
fn structured_import_errors_preserve_cpp_result_categories() {
    let unsupported = read_runtime_file_with_error_kind(&synthetic_runtime_file_with_header(
        8,
        0,
        0,
        &[],
        |_| {},
    ))
    .expect_err("C++ reports unsupportedVersion for unsupported major versions");
    assert_eq!(unsupported.kind(), RuntimeReadErrorKind::UnsupportedVersion);

    let malformed = read_runtime_file_with_error_kind(&synthetic_runtime_file_with_header(
        i32::MAX as u64 + 1,
        0,
        0,
        &[],
        |_| {},
    ))
    .expect_err("C++ reports malformed for version integers outside signed int range");
    assert_eq!(malformed.kind(), RuntimeReadErrorKind::Malformed);

    let malformed = read_runtime_file_with_error_kind(b"RIVE\x80")
        .expect_err("C++ reports malformed for truncated header varuints");
    assert_eq!(malformed.kind(), RuntimeReadErrorKind::Malformed);
}

#[test]
fn invalid_drawable_blend_mode_is_malformed_like_cpp_artboard_initialize() {
    let malformed = read_runtime_file_with_error_kind(&synthetic_runtime_file(4292, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");

        push_var_uint(bytes, 3);
        push_var_uint(bytes, 5);
        push_var_uint(bytes, 0);
        push_var_uint(bytes, 23);
        push_var_uint(bytes, 5);
        push_var_uint(bytes, 0);
    }))
    .expect_err("C++ rejects invalid Drawable.blendModeValue during artboard initialize");

    assert_eq!(malformed.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        malformed.message().contains("invalid blendModeValue"),
        "malformed message should identify the C++ drawable validation failure: {}",
        malformed.message()
    );
}

#[test]
fn invalid_mesh_triangle_indices_are_malformed_like_cpp_artboard_initialize() {
    let missing_indices =
        read_runtime_file_with_error_kind(&synthetic_mesh_runtime_file(4293, None))
            .expect_err("C++ rejects Mesh objects with no triangleIndexBytes buffer");
    assert_eq!(missing_indices.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        missing_indices.message().contains("triangleIndexBytes"),
        "malformed message should identify the missing C++ mesh index buffer: {}",
        missing_indices.message()
    );

    let out_of_range =
        read_runtime_file_with_error_kind(&synthetic_mesh_runtime_file(4294, Some(&[1])))
            .expect_err("C++ rejects Mesh triangle indices outside the MeshVertex list");
    assert_eq!(out_of_range.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        out_of_range.message().contains("triangle index"),
        "malformed message should identify the C++ mesh index validation failure: {}",
        out_of_range.message()
    );

    let valid = read_runtime_file(&synthetic_mesh_runtime_file(4295, Some(&[0])))
        .expect("C++ accepts Mesh triangle indices that reference existing MeshVertex entries");
    assert_eq!(valid.object_count(), 5);
    assert_eq!(valid.known_object_count(), 5);
    assert_eq!(valid.imported_object_count(), 5);

    let valid_contour_vertex = read_runtime_file(&synthetic_mesh_runtime_file_with_vertex(
        4296,
        Some(&[0]),
        "ContourMeshVertex",
    ))
    .expect("C++ counts MeshVertex subclasses when validating Mesh triangle indices");
    assert_eq!(valid_contour_vertex.object_count(), 5);
    assert_eq!(valid_contour_vertex.known_object_count(), 5);
    assert_eq!(valid_contour_vertex.imported_object_count(), 5);
}

#[test]
fn mesh_triangle_index_bytes_decode_to_cpp_uint16_indices() {
    let mut triangle_index_bytes = Vec::new();
    for index in [0, 1, 300, u16::MAX as u64, u16::MAX as u64 + 1, 5] {
        push_var_uint(&mut triangle_index_bytes, index);
    }

    let file = read_runtime_file(&synthetic_runtime_file(4297, |bytes| {
        push_object_with_properties(bytes, "Mesh", |bytes| {
            push_bytes_property(bytes, "Mesh", "triangleIndexBytes", &triangle_index_bytes);
        });
    }))
    .expect("mesh triangle index bytes decode before C++ import-context validation");

    let mesh = file.object(0).expect("Mesh object");
    assert_eq!(mesh.type_name, "Mesh");
    assert_eq!(
        mesh.bytes_property("triangleIndexBytes"),
        Some(triangle_index_bytes.as_slice())
    );
    assert_eq!(mesh.id_list_property("triangleIndexBytes"), None);
    assert_eq!(
        mesh.mesh_triangle_indices(),
        Some(vec![0, 1, 300, u16::MAX, 0])
    );
}

#[test]
fn runtime_artboard_skins_resolve_cpp_skinnable_tendons_and_bones() {
    let file = read_runtime_file(&synthetic_skin_runtime_file(4298))
        .expect("C++ accepts a Skin under a skinnable PointsPath with a Tendon to a RootBone");

    let skins = file.artboard_skins(0);
    assert_eq!(skins.len(), 1);

    let skin = &skins[0];
    assert_eq!(skin.local_id, 3);
    assert_eq!(skin.object.type_name, "Skin");
    assert_eq!(skin.skinnable_local_id, Some(2));
    assert_eq!(
        skin.skinnable.map(|object| object.type_name),
        Some("PointsPath")
    );
    assert_eq!(skin.tendons.len(), 1);

    let tendon = &skin.tendons[0];
    assert_eq!(tendon.local_id, 5);
    assert_eq!(tendon.object.type_name, "Tendon");
    assert_eq!(tendon.bone_local_id, Some(4));
    assert_eq!(tendon.bone.map(|object| object.type_name), Some("RootBone"));

    assert!(file.artboard_skins(1).is_empty());
    assert!(file.artboard_skin(0, 1).is_none());
}

#[test]
fn runtime_artboard_meshes_resolve_cpp_vertices_and_last_weight() {
    let file = read_runtime_file(&synthetic_weighted_mesh_runtime_file(4299))
        .expect("C++ accepts a MeshVertex with Weight children under a valid Mesh");

    let meshes = file.artboard_meshes(0);
    assert_eq!(meshes.len(), 1);

    let mesh = &meshes[0];
    assert_eq!(mesh.local_id, 2);
    assert_eq!(mesh.object.type_name, "Mesh");
    assert_eq!(mesh.vertices.len(), 1);

    let vertex = &mesh.vertices[0];
    assert_eq!(vertex.local_id, 3);
    assert_eq!(vertex.object.type_name, "MeshVertex");
    assert_eq!(vertex.weight_local_id, Some(5));
    let weight = vertex.weight.expect("resolved vertex weight");
    assert_eq!(weight.type_name, "Weight");
    assert_eq!(weight.uint_property("values"), Some(0x0403_0201));
    assert_eq!(weight.uint_property("indices"), Some(0x0807_0605));

    assert!(file.artboard_meshes(1).is_empty());
    assert!(file.artboard_mesh(0, 1).is_none());
}

#[test]
fn runtime_artboard_paths_resolve_cpp_vertices_and_weight_subclasses() {
    let file = read_runtime_file(&synthetic_weighted_path_runtime_file(4300))
        .expect("C++ accepts PathVertex children with Weight subclasses under a valid PointsPath");

    let paths = file.artboard_paths(0);
    assert_eq!(paths.len(), 1);

    let path = &paths[0];
    assert_eq!(path.local_id, 2);
    assert_eq!(path.object.type_name, "PointsPath");
    assert_eq!(path.vertices.len(), 2);

    let straight_vertex = &path.vertices[0];
    assert_eq!(straight_vertex.local_id, 3);
    assert_eq!(straight_vertex.object.type_name, "StraightVertex");
    assert_eq!(straight_vertex.weight_local_id, Some(4));
    let straight_weight = straight_vertex
        .weight
        .expect("resolved straight vertex weight");
    assert_eq!(straight_weight.type_name, "Weight");
    assert_eq!(straight_weight.uint_property("values"), Some(0x0d0c_0b0a));
    assert_eq!(straight_weight.uint_property("indices"), Some(0x1110_0f0e));

    let cubic_vertex = &path.vertices[1];
    assert_eq!(cubic_vertex.local_id, 5);
    assert_eq!(cubic_vertex.object.type_name, "CubicMirroredVertex");
    assert_eq!(cubic_vertex.weight_local_id, Some(6));
    let cubic_weight = cubic_vertex.weight.expect("resolved cubic vertex weight");
    assert_eq!(cubic_weight.type_name, "CubicWeight");
    assert_eq!(cubic_weight.uint_property("values"), Some(0x1514_1312));
    assert_eq!(cubic_weight.uint_property("indices"), Some(0x1918_1716));

    assert!(file.artboard_paths(1).is_empty());
    assert!(file.artboard_path(0, 1).is_none());
}

#[test]
fn runtime_artboard_shapes_resolve_cpp_paths_paints_mutators_and_gradient_stops() {
    let file = read_runtime_file(&synthetic_shape_paint_runtime_file(4301)).expect(
        "C++ accepts shape paths, paints, paint mutators, and gradient stops under a valid Shape",
    );

    let shapes = file.artboard_shapes(0);
    assert_eq!(shapes.len(), 1);

    let shape = &shapes[0];
    assert_eq!(shape.local_id, 1);
    assert_eq!(shape.object.type_name, "Shape");
    assert_eq!(
        shape
            .paths
            .iter()
            .map(|path| (path.local_id, path.object.type_name))
            .collect::<Vec<_>>(),
        vec![
            (2, "PointsPath"),
            (27, "Triangle"),
            (28, "Ellipse"),
            (29, "Rectangle")
        ]
    );
    assert_eq!(shape.paints.len(), 4);

    let fill = &shape.paints[0];
    assert_eq!(fill.local_id, 3);
    assert_eq!(fill.object.type_name, "Fill");
    assert_eq!(fill.mutator_local_id, Some(4));
    assert_eq!(
        fill.mutator.map(|object| object.type_name),
        Some("LinearGradient")
    );
    assert_eq!(
        fill.gradient_stops
            .iter()
            .map(|stop| (stop.local_id, stop.object.type_name))
            .collect::<Vec<_>>(),
        vec![(5, "GradientStop"), (6, "GradientStop")]
    );
    assert!(fill.feather.is_none());
    assert!(fill.effects.is_empty());

    let radial_fill = &shape.paints[1];
    assert_eq!(radial_fill.local_id, 7);
    assert_eq!(radial_fill.object.type_name, "Fill");
    assert_eq!(radial_fill.mutator_local_id, Some(8));
    assert_eq!(
        radial_fill.mutator.map(|object| object.type_name),
        Some("RadialGradient")
    );
    assert_eq!(
        radial_fill
            .gradient_stops
            .iter()
            .map(|stop| (stop.local_id, stop.object.type_name))
            .collect::<Vec<_>>(),
        vec![(9, "GradientStop"), (10, "GradientStop")]
    );
    assert!(radial_fill.feather.is_none());
    assert!(radial_fill.effects.is_empty());

    let stroke = &shape.paints[2];
    assert_eq!(stroke.local_id, 11);
    assert_eq!(stroke.object.type_name, "Stroke");
    assert_eq!(stroke.mutator_local_id, Some(12));
    assert_eq!(
        stroke.mutator.map(|object| object.type_name),
        Some("SolidColor")
    );
    assert!(stroke.gradient_stops.is_empty());
    assert_eq!(stroke.feather_local_id, Some(16));
    assert_eq!(
        stroke.feather.map(|object| object.type_name),
        Some("Feather")
    );
    assert_eq!(
        stroke
            .effects
            .iter()
            .map(|effect| (
                effect.local_id,
                effect.object.type_name,
                effect.target_group_effect_local_id,
                effect.target_group_effect.map(|object| object.type_name)
            ))
            .collect::<Vec<_>>(),
        vec![
            (13, "TrimPath", None, None),
            (14, "DashPath", None, None),
            (19, "TargetEffect", Some(17), Some("GroupEffect"))
        ]
    );

    let late_fill = &shape.paints[3];
    assert_eq!(late_fill.local_id, 31);
    assert_eq!(late_fill.object.type_name, "Fill");
    assert_eq!(late_fill.mutator_local_id, Some(30));
    assert_eq!(
        late_fill.mutator.map(|object| object.type_name),
        Some("SolidColor")
    );
    assert!(late_fill.gradient_stops.is_empty());
    assert!(late_fill.feather.is_none());
    assert!(late_fill.effects.is_empty());

    assert!(file.artboard_shapes(1).is_empty());
    assert!(file.artboard_shape(0, 1).is_none());
}

#[test]
fn runtime_artboard_shape_paint_containers_resolve_cpp_registered_paints() {
    let file = read_runtime_file(&synthetic_shape_paint_runtime_file(4302))
        .expect("C++ accepts shape paints registered on all ShapePaintContainer kinds");

    let containers = file.artboard_shape_paint_containers(0);
    assert_eq!(containers.len(), 3);
    assert_eq!(
        containers
            .iter()
            .map(|container| (container.local_id, container.object.type_name))
            .collect::<Vec<_>>(),
        vec![(0, "Artboard"), (1, "Shape"), (24, "TextStylePaint")]
    );
    assert_eq!(
        containers
            .iter()
            .map(|container| {
                container
                    .paints
                    .iter()
                    .map(|paint| {
                        (
                            paint.local_id,
                            paint.object.type_name,
                            paint.mutator_local_id,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec![(21, "Fill", Some(22))],
            vec![
                (3, "Fill", Some(4)),
                (7, "Fill", Some(8)),
                (11, "Stroke", Some(12)),
                (31, "Fill", Some(30)),
            ],
            vec![(25, "Fill", Some(26))]
        ]
    );

    assert!(file.artboard_shape_paint_containers(1).is_empty());
    assert!(file.artboard_shape_paint_container(0, 3).is_none());
}

#[test]
fn runtime_artboard_n_slicer_details_resolve_cpp_axes_and_tile_modes() {
    let file = read_runtime_file(&synthetic_n_slicer_runtime_file(4303, |bytes| {
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
        });
        push_object_with_properties(bytes, "NSlicer", |bytes| {
            push_uint_property(bytes, "NSlicer", "parentId", 1);
        });
        push_object_with_properties(bytes, "AxisX", |bytes| {
            push_uint_property(bytes, "AxisX", "parentId", 2);
        });
        push_object_with_properties(bytes, "AxisY", |bytes| {
            push_uint_property(bytes, "AxisY", "parentId", 2);
        });
        push_object_with_properties(bytes, "NSlicerTileMode", |bytes| {
            push_uint_property(bytes, "NSlicerTileMode", "parentId", 2);
            push_uint_property(bytes, "NSlicerTileMode", "patchIndex", 3);
            push_uint_property(bytes, "NSlicerTileMode", "style", 1);
        });
        push_object_with_properties(bytes, "NSlicerTileMode", |bytes| {
            push_uint_property(bytes, "NSlicerTileMode", "parentId", 2);
            push_uint_property(bytes, "NSlicerTileMode", "patchIndex", 3);
            push_uint_property(bytes, "NSlicerTileMode", "style", 2);
        });
        push_object_with_properties(bytes, "NSlicedNode", |bytes| {
            push_uint_property(bytes, "NSlicedNode", "parentId", 0);
        });
        push_object_with_properties(bytes, "AxisX", |bytes| {
            push_uint_property(bytes, "AxisX", "parentId", 7);
        });
        push_object_with_properties(bytes, "AxisY", |bytes| {
            push_uint_property(bytes, "AxisY", "parentId", 7);
        });
        push_object_with_properties(bytes, "NSlicerTileMode", |bytes| {
            push_uint_property(bytes, "NSlicerTileMode", "parentId", 7);
            push_uint_property(bytes, "NSlicerTileMode", "patchIndex", 4);
            push_uint_property(bytes, "NSlicerTileMode", "style", 5);
        });
    }))
    .expect("C++ accepts Axis and NSlicerTileMode children under NSlicerDetails objects");

    let details = file.artboard_n_slicer_details(0);
    assert_eq!(details.len(), 2);

    let n_slicer = &details[0];
    assert_eq!(n_slicer.local_id, 2);
    assert_eq!(n_slicer.object.type_name, "NSlicer");
    assert_eq!(
        n_slicer
            .x_axes
            .iter()
            .map(|axis| (axis.local_id, axis.object.type_name))
            .collect::<Vec<_>>(),
        vec![(3, "AxisX")]
    );
    assert_eq!(
        n_slicer
            .y_axes
            .iter()
            .map(|axis| (axis.local_id, axis.object.type_name))
            .collect::<Vec<_>>(),
        vec![(4, "AxisY")]
    );
    assert_eq!(
        n_slicer
            .tile_modes
            .iter()
            .map(|mode| {
                (
                    mode.local_id,
                    mode.object.type_name,
                    mode.patch_index,
                    mode.style,
                )
            })
            .collect::<Vec<_>>(),
        vec![(6, "NSlicerTileMode", 3, 2)],
        "C++ NSlicerDetails::addTileMode overwrites duplicate patch indexes"
    );

    let n_sliced_node = &details[1];
    assert_eq!(n_sliced_node.local_id, 7);
    assert_eq!(n_sliced_node.object.type_name, "NSlicedNode");
    assert_eq!(
        n_sliced_node
            .x_axes
            .iter()
            .map(|axis| axis.local_id)
            .collect::<Vec<_>>(),
        vec![8]
    );
    assert_eq!(
        n_sliced_node
            .y_axes
            .iter()
            .map(|axis| axis.local_id)
            .collect::<Vec<_>>(),
        vec![9]
    );
    assert_eq!(
        n_sliced_node
            .tile_modes
            .iter()
            .map(|mode| (mode.local_id, mode.patch_index, mode.style))
            .collect::<Vec<_>>(),
        vec![(10, 4, 5)]
    );

    assert_eq!(
        file.artboard_n_slicer_detail(0, 1)
            .map(|details| details.local_id),
        Some(7)
    );
    assert!(file.artboard_n_slicer_details(1).is_empty());
    assert!(file.artboard_n_slicer_detail(0, 2).is_none());
}

#[test]
fn invalid_n_slicer_parentage_keeps_slots_but_skips_cpp_registration() {
    let invalid_n_slicer_parent =
        read_runtime_file(&synthetic_n_slicer_runtime_file(4304, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "NSlicer", |bytes| {
                push_uint_property(bytes, "NSlicer", "parentId", 1);
            });
        }))
        .expect("C++ treats NSlicer wrong parents as recoverable onAddedDirty failures");
    assert_eq!(
        invalid_n_slicer_parent.import_status(3),
        Some(RuntimeImportStatus::Imported)
    );
    assert_eq!(
        invalid_n_slicer_parent
            .artboard_n_slicer_details(0)
            .iter()
            .map(|details| (details.local_id, details.object.type_name))
            .collect::<Vec<_>>(),
        vec![(2, "NSlicer")]
    );

    let invalid_axis_parent = read_runtime_file(&synthetic_n_slicer_runtime_file(4305, |bytes| {
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
        });
        push_object_with_properties(bytes, "NSlicer", |bytes| {
            push_uint_property(bytes, "NSlicer", "parentId", 1);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "AxisX", |bytes| {
            push_uint_property(bytes, "AxisX", "parentId", 3);
        });
    }))
    .expect("C++ treats Axis wrong parents as recoverable onAddedDirty failures");
    assert_eq!(
        invalid_axis_parent.import_status(5),
        Some(RuntimeImportStatus::Imported)
    );
    let details = invalid_axis_parent.artboard_n_slicer_details(0);
    assert_eq!(details.len(), 1);
    assert!(details[0].x_axes.is_empty());

    let invalid_tile_mode_parent =
        read_runtime_file(&synthetic_n_slicer_runtime_file(4306, |bytes| {
            push_object_with_properties(bytes, "Image", |bytes| {
                push_uint_property(bytes, "Image", "parentId", 0);
            });
            push_object_with_properties(bytes, "NSlicer", |bytes| {
                push_uint_property(bytes, "NSlicer", "parentId", 1);
            });
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "NSlicerTileMode", |bytes| {
                push_uint_property(bytes, "NSlicerTileMode", "parentId", 3);
            });
        }))
        .expect("C++ treats NSlicerTileMode wrong parents as recoverable onAddedDirty failures");
    assert_eq!(
        invalid_tile_mode_parent.import_status(5),
        Some(RuntimeImportStatus::Imported)
    );
    let details = invalid_tile_mode_parent.artboard_n_slicer_details(0);
    assert_eq!(details.len(), 1);
    assert!(details[0].tile_modes.is_empty());
}

fn synthetic_n_slicer_runtime_file(
    file_id: u64,
    artboard_objects: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        artboard_objects(bytes);
    })
}

fn synthetic_mesh_runtime_file(file_id: u64, triangle_index_bytes: Option<&[u8]>) -> Vec<u8> {
    synthetic_mesh_runtime_file_with_vertex(file_id, triangle_index_bytes, "MeshVertex")
}

fn synthetic_mesh_runtime_file_with_vertex(
    file_id: u64,
    triangle_index_bytes: Option<&[u8]>,
    vertex_type: &str,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
        });
        push_object_with_properties(bytes, "Mesh", |bytes| {
            push_uint_property(bytes, "Mesh", "parentId", 1);
            if let Some(triangle_index_bytes) = triangle_index_bytes {
                push_bytes_property(bytes, "Mesh", "triangleIndexBytes", triangle_index_bytes);
            }
        });
        push_object_with_properties(bytes, vertex_type, |bytes| {
            push_uint_property(bytes, vertex_type, "parentId", 2);
        });
    })
}

fn synthetic_skin_runtime_file(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 1);
        });
        push_object_with_properties(bytes, "Skin", |bytes| {
            push_uint_property(bytes, "Skin", "parentId", 2);
        });
        push_object_with_properties(bytes, "RootBone", |bytes| {
            push_uint_property(bytes, "RootBone", "parentId", 0);
        });
        push_object_with_properties(bytes, "Tendon", |bytes| {
            push_uint_property(bytes, "Tendon", "parentId", 3);
            push_uint_property(bytes, "Tendon", "boneId", 4);
        });
    })
}

fn synthetic_weighted_mesh_runtime_file(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
        });
        push_object_with_properties(bytes, "Mesh", |bytes| {
            push_uint_property(bytes, "Mesh", "parentId", 1);
            push_bytes_property(bytes, "Mesh", "triangleIndexBytes", &[0]);
        });
        push_object_with_properties(bytes, "MeshVertex", |bytes| {
            push_uint_property(bytes, "MeshVertex", "parentId", 2);
        });
        push_object_with_properties(bytes, "Weight", |bytes| {
            push_uint_property(bytes, "Weight", "parentId", 3);
            push_uint_property(bytes, "Weight", "values", 0x0101_0101);
            push_uint_property(bytes, "Weight", "indices", 0x0202_0202);
        });
        push_object_with_properties(bytes, "Weight", |bytes| {
            push_uint_property(bytes, "Weight", "parentId", 3);
            push_uint_property(bytes, "Weight", "values", 0x0403_0201);
            push_uint_property(bytes, "Weight", "indices", 0x0807_0605);
        });
    })
}

fn synthetic_weighted_path_runtime_file(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 1);
        });
        push_object_with_properties(bytes, "StraightVertex", |bytes| {
            push_uint_property(bytes, "StraightVertex", "parentId", 2);
        });
        push_object_with_properties(bytes, "Weight", |bytes| {
            push_uint_property(bytes, "Weight", "parentId", 3);
            push_uint_property(bytes, "Weight", "values", 0x0d0c_0b0a);
            push_uint_property(bytes, "Weight", "indices", 0x1110_0f0e);
        });
        push_object_with_properties(bytes, "CubicMirroredVertex", |bytes| {
            push_uint_property(bytes, "CubicMirroredVertex", "parentId", 2);
        });
        push_object_with_properties(bytes, "CubicWeight", |bytes| {
            push_uint_property(bytes, "CubicWeight", "parentId", 5);
            push_uint_property(bytes, "CubicWeight", "values", 0x1514_1312);
            push_uint_property(bytes, "CubicWeight", "indices", 0x1918_1716);
        });
    })
}

fn synthetic_shape_paint_runtime_file(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "PointsPath", |bytes| {
            push_uint_property(bytes, "PointsPath", "parentId", 1);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 1);
        });
        push_object_with_properties(bytes, "LinearGradient", |bytes| {
            push_uint_property(bytes, "LinearGradient", "parentId", 3);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "GradientStop", "parentId", 4);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "GradientStop", "parentId", 4);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 1);
        });
        push_object_with_properties(bytes, "RadialGradient", |bytes| {
            push_uint_property(bytes, "RadialGradient", "parentId", 7);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "GradientStop", "parentId", 8);
        });
        push_object_with_properties(bytes, "GradientStop", |bytes| {
            push_uint_property(bytes, "GradientStop", "parentId", 8);
        });
        push_object_with_properties(bytes, "Stroke", |bytes| {
            push_uint_property(bytes, "Stroke", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "SolidColor", "parentId", 11);
        });
        push_object_with_properties(bytes, "TrimPath", |bytes| {
            push_uint_property(bytes, "TrimPath", "parentId", 11);
            push_uint_property(bytes, "TrimPath", "modeValue", 1);
        });
        push_object_with_properties(bytes, "DashPath", |bytes| {
            push_uint_property(bytes, "DashPath", "parentId", 11);
        });
        push_object_with_properties(bytes, "Dash", |bytes| {
            push_uint_property(bytes, "Dash", "parentId", 14);
        });
        push_object_with_properties(bytes, "Feather", |bytes| {
            push_uint_property(bytes, "Feather", "parentId", 11);
        });
        push_object_with_properties(bytes, "GroupEffect", |bytes| {
            push_uint_property(bytes, "GroupEffect", "parentId", 11);
        });
        push_object_with_properties(bytes, "TrimPath", |bytes| {
            push_uint_property(bytes, "TrimPath", "parentId", 17);
            push_uint_property(bytes, "TrimPath", "modeValue", 1);
        });
        push_object_with_properties(bytes, "TargetEffect", |bytes| {
            push_uint_property(bytes, "TargetEffect", "parentId", 11);
            push_uint_property(bytes, "TargetEffect", "targetId", 17);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 1);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 0);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "SolidColor", "parentId", 21);
        });
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStylePaint", |bytes| {
            push_uint_property(bytes, "TextStylePaint", "parentId", 23);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 24);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "SolidColor", "parentId", 25);
        });
        push_object_with_properties(bytes, "Triangle", |bytes| {
            push_uint_property(bytes, "Triangle", "parentId", 1);
        });
        push_object_with_properties(bytes, "Ellipse", |bytes| {
            push_uint_property(bytes, "Ellipse", "parentId", 1);
        });
        push_object_with_properties(bytes, "Rectangle", |bytes| {
            push_uint_property(bytes, "Rectangle", "parentId", 1);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "SolidColor", "parentId", 31);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 1);
        });
    })
}

#[test]
fn invalid_constraint_parentage_is_malformed_like_cpp_artboard_initialize() {
    let valid = read_runtime_file(&synthetic_constraint_runtime_file(4320, |bytes| {
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "RotationConstraint", |bytes| {
            push_uint_property(bytes, "RotationConstraint", "parentId", 1);
            push_uint_property(bytes, "RotationConstraint", "targetId", 0);
        });
    }))
    .expect("C++ accepts constraints parented to TransformComponent objects");
    assert_eq!(valid.imported_object_count(), 4);

    let invalid_constraint_parent =
        read_runtime_file_with_error_kind(&synthetic_constraint_runtime_file(4321, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "Fill", |bytes| {
                push_uint_property(bytes, "Fill", "parentId", 1);
            });
            push_object_with_properties(bytes, "RotationConstraint", |bytes| {
                push_uint_property(bytes, "RotationConstraint", "parentId", 2);
                push_uint_property(bytes, "RotationConstraint", "targetId", 0);
            });
        }))
        .expect_err("C++ rejects constraints that are not parented to TransformComponent objects");
    assert_eq!(
        invalid_constraint_parent.kind(),
        RuntimeReadErrorKind::Malformed
    );
    assert!(
        invalid_constraint_parent
            .message()
            .contains("TransformComponent"),
        "malformed message should identify the TransformComponent parent requirement: {}",
        invalid_constraint_parent.message()
    );

    let invalid_ik_parent =
        read_runtime_file_with_error_kind(&synthetic_constraint_runtime_file(4322, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "IKConstraint", |bytes| {
                push_uint_property(bytes, "IKConstraint", "parentId", 1);
                push_uint_property(bytes, "IKConstraint", "targetId", 0);
            });
        }))
        .expect_err("C++ rejects IKConstraint objects outside Bone parents");
    assert_eq!(invalid_ik_parent.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        invalid_ik_parent.message().contains("Bone"),
        "malformed message should identify the Bone parent requirement: {}",
        invalid_ik_parent.message()
    );

    let pruned_invalid_target =
        read_runtime_file(&synthetic_constraint_runtime_file(4325, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "Fill", |bytes| {
                push_uint_property(bytes, "Fill", "parentId", 1);
            });
            push_object_with_properties(bytes, "RotationConstraint", |bytes| {
                push_uint_property(bytes, "RotationConstraint", "parentId", 2);
                push_uint_property(bytes, "RotationConstraint", "targetId", 2);
            });
        }))
        .expect("C++ validateObjects prunes invalid targeted constraints before onAddedDirty");
    assert_eq!(pruned_invalid_target.imported_object_count(), 5);
}

fn synthetic_constraint_runtime_file(
    file_id: u64,
    artboard_objects: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        artboard_objects(bytes);
    })
}

#[test]
fn joystick_handle_source_ids_import_without_binary_resolution_like_cpp_file_import() {
    let file = read_runtime_file(&synthetic_runtime_file(4333, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 0);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
            push_uint_property(bytes, "Joystick", "handleSourceId", 1);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
            push_uint_property(bytes, "Joystick", "handleSourceId", 99);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
            push_uint_property(bytes, "Joystick", "handleSourceId", 2);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
            push_uint_property(bytes, "Joystick", "handleSourceId", 3);
        });
    }))
    .expect("C++ File::import preserves Joystick handle ids without resolving them");

    assert_eq!(file.imported_object_count(), 9);
    assert_eq!(file.import_status(4), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(5), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(6), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(7), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(8), Some(RuntimeImportStatus::Imported));
    assert_eq!(
        file.object(4).unwrap().uint_property("handleSourceId"),
        Some(1)
    );
    assert_eq!(
        file.object(5).unwrap().uint_property("handleSourceId"),
        Some(u64::from(u32::MAX))
    );
    assert_eq!(
        file.object(6).unwrap().uint_property("handleSourceId"),
        Some(99)
    );
    assert_eq!(
        file.object(7).unwrap().uint_property("handleSourceId"),
        Some(2)
    );
    assert_eq!(
        file.object(8).unwrap().uint_property("handleSourceId"),
        Some(3)
    );

    assert_eq!(
        file.resolved_handle_source_for_joystick(4)
            .map(|object| object.id),
        Some(2),
        "C++ Joystick::onAddedDirty resolves handleSourceId to a TransformComponent"
    );
    assert_eq!(
        file.resolved_handle_source_for_joystick(5)
            .map(|object| object.id),
        None,
        "C++ Joystick::onAddedDirty treats Core::emptyId as no handle source"
    );
    assert_eq!(
        file.resolved_handle_source_for_joystick(6)
            .map(|object| object.id),
        None,
        "C++ Joystick::onAddedDirty leaves missing handle source ids unresolved"
    );
    assert_eq!(
        file.resolved_handle_source_for_joystick(7)
            .map(|object| object.id),
        None,
        "C++ Joystick::onAddedDirty rejects non-TransformComponent handle sources"
    );
    assert_eq!(
        file.resolved_handle_source_for_joystick(8)
            .map(|object| object.id),
        None,
        "C++ Joystick::onAddedDirty rejects non-TransformComponent controls as handle sources"
    );
    assert_eq!(
        file.resolved_handle_source_for_joystick(2)
            .map(|object| object.id),
        None,
        "handle-source resolution validates joystick objects"
    );
    assert_eq!(
        file.resolved_handle_source_for_joystick_object(file.object(4).unwrap())
            .map(|object| object.id),
        Some(2)
    );
}

#[test]
fn joystick_animation_ids_resolve_like_cpp_on_added_clean() {
    let file = read_runtime_file(&synthetic_runtime_file(4334, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_string_property(bytes, "LinearAnimation", "name", "x");
        });
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_string_property(bytes, "LinearAnimation", "name", "y");
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
            push_uint_property(bytes, "Joystick", "xId", 0);
            push_uint_property(bytes, "Joystick", "yId", 1);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
            push_uint_property(bytes, "Joystick", "xId", 1);
            push_uint_property(bytes, "Joystick", "yId", 99);
        });
        push_object_with_properties(bytes, "Joystick", |bytes| {
            push_uint_property(bytes, "Joystick", "parentId", 0);
        });
    }))
    .expect("C++ Joystick animation ids resolve during onAddedClean");

    assert_eq!(
        file.artboard_animations(0)
            .into_iter()
            .map(|animation| animation.string_property("name"))
            .collect::<Vec<_>>(),
        vec![Some("x"), Some("y")]
    );
    assert_eq!(
        file.resolved_x_animation_for_joystick(5)
            .map(|animation| animation.id),
        Some(2),
        "C++ Joystick::onAddedClean resolves xId through Artboard::animation"
    );
    assert_eq!(
        file.resolved_y_animation_for_joystick(5)
            .map(|animation| animation.id),
        Some(3),
        "C++ Joystick::onAddedClean resolves yId through Artboard::animation"
    );
    assert_eq!(
        file.resolved_x_animation_for_joystick(6)
            .map(|animation| animation.id),
        Some(3),
        "C++ Joystick::onAddedClean resolves axis ids independently"
    );
    assert_eq!(
        file.resolved_y_animation_for_joystick(6)
            .map(|animation| animation.id),
        None,
        "C++ Artboard::animation returns null for out-of-range joystick animation ids"
    );
    assert_eq!(
        file.resolved_x_animation_for_joystick(7)
            .map(|animation| animation.id),
        None,
        "C++ missing Joystick.xId defaults to Core::missingId and resolves to no animation"
    );
    assert_eq!(
        file.resolved_y_animation_for_joystick(7)
            .map(|animation| animation.id),
        None,
        "C++ missing Joystick.yId defaults to Core::missingId and resolves to no animation"
    );
    assert_eq!(
        file.resolved_x_animation_for_joystick(4)
            .map(|animation| animation.id),
        None,
        "joystick animation resolution validates joystick objects"
    );
    assert_eq!(
        file.resolved_y_animation_for_joystick_object(file.object(5).unwrap())
            .map(|animation| animation.id),
        Some(3)
    );
}

#[test]
fn invalid_paint_effect_parentage_is_malformed_like_cpp_artboard_initialize() {
    let valid = read_runtime_file(&synthetic_paint_effect_runtime_file(4312, |bytes| {
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 1);
        });
        push_object_with_properties(bytes, "DashPath", |bytes| {
            push_uint_property(bytes, "DashPath", "parentId", 2);
        });
        push_object_with_properties(bytes, "Dash", |bytes| {
            push_uint_property(bytes, "Dash", "parentId", 3);
        });
        push_object_with_properties(bytes, "SolidColor", |bytes| {
            push_uint_property(bytes, "SolidColor", "parentId", 2);
        });
    }))
    .expect("C++ accepts paint effects with matching parent containers");
    assert_eq!(valid.imported_object_count(), 7);

    let invalid_dash_parent =
        read_runtime_file_with_error_kind(&synthetic_paint_effect_runtime_file(4313, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "Fill", |bytes| {
                push_uint_property(bytes, "Fill", "parentId", 1);
            });
            push_object_with_properties(bytes, "Dash", |bytes| {
                push_uint_property(bytes, "Dash", "parentId", 2);
            });
        }))
        .expect_err("C++ rejects Dash objects that are not parented to DashPath");
    assert_eq!(invalid_dash_parent.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        invalid_dash_parent.message().contains("DashPath"),
        "malformed message should identify the DashPath parent requirement: {}",
        invalid_dash_parent.message()
    );

    let invalid_effect_parent =
        read_runtime_file_with_error_kind(&synthetic_paint_effect_runtime_file(4314, |bytes| {
            push_object_with_properties(bytes, "DashPath", |bytes| {
                push_uint_property(bytes, "DashPath", "parentId", 0);
            });
        }))
        .expect_err("C++ rejects stroke effects that are not parented to an effects container");
    assert_eq!(
        invalid_effect_parent.kind(),
        RuntimeReadErrorKind::Malformed
    );
    assert!(
        invalid_effect_parent
            .message()
            .contains("effects container"),
        "malformed message should identify the effects-container parent requirement: {}",
        invalid_effect_parent.message()
    );

    let duplicate_mutator =
        read_runtime_file_with_error_kind(&synthetic_paint_effect_runtime_file(4315, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "Fill", |bytes| {
                push_uint_property(bytes, "Fill", "parentId", 1);
            });
            push_object_with_properties(bytes, "SolidColor", |bytes| {
                push_uint_property(bytes, "SolidColor", "parentId", 2);
            });
            push_object_with_properties(bytes, "LinearGradient", |bytes| {
                push_uint_property(bytes, "LinearGradient", "parentId", 2);
            });
        }))
        .expect_err("C++ rejects multiple paint mutators under the same ShapePaint");
    assert_eq!(duplicate_mutator.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        duplicate_mutator.message().contains("duplicate mutator"),
        "malformed message should identify duplicate ShapePaint mutators: {}",
        duplicate_mutator.message()
    );
}

fn synthetic_paint_effect_runtime_file(
    file_id: u64,
    artboard_objects: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        artboard_objects(bytes);
    })
}

#[test]
fn invalid_trim_path_mode_is_malformed_like_cpp_artboard_initialize() {
    let valid = read_runtime_file(&synthetic_paint_effect_runtime_file(4323, |bytes| {
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "Fill", |bytes| {
            push_uint_property(bytes, "Fill", "parentId", 1);
        });
        push_object_with_properties(bytes, "TrimPath", |bytes| {
            push_uint_property(bytes, "TrimPath", "parentId", 2);
            push_uint_property(bytes, "TrimPath", "modeValue", 1);
        });
    }))
    .expect("C++ accepts TrimPath with sequential mode");
    assert_eq!(valid.imported_object_count(), 5);

    let invalid_mode =
        read_runtime_file_with_error_kind(&synthetic_paint_effect_runtime_file(4324, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "Fill", |bytes| {
                push_uint_property(bytes, "Fill", "parentId", 1);
            });
            push_object_with_properties(bytes, "TrimPath", |bytes| {
                push_uint_property(bytes, "TrimPath", "parentId", 2);
                push_uint_property(bytes, "TrimPath", "modeValue", 0);
            });
        }))
        .expect_err("C++ rejects TrimPath mode values outside the runtime enum");
    assert_eq!(invalid_mode.kind(), RuntimeReadErrorKind::Malformed);
    assert!(
        invalid_mode.message().contains("modeValue"),
        "malformed message should identify the invalid TrimPath mode: {}",
        invalid_mode.message()
    );
}

#[test]
fn invalid_text_parentage_is_malformed_like_cpp_artboard_initialize() {
    let valid = read_runtime_file(&synthetic_text_runtime_file(4316, |bytes| {
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStyle", |bytes| {
            push_uint_property(bytes, "TextStyle", "parentId", 1);
        });
        push_object_with_properties(bytes, "TextStyleFeature", |bytes| {
            push_uint_property(bytes, "TextStyleFeature", "parentId", 2);
        });
        push_object_with_properties(bytes, "TextStyleAxis", |bytes| {
            push_uint_property(bytes, "TextStyleAxis", "parentId", 2);
        });
        push_object_with_properties(bytes, "TextInput", |bytes| {
            push_uint_property(bytes, "TextInput", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextInputText", |bytes| {
            push_uint_property(bytes, "TextInputText", "parentId", 5);
        });
    }))
    .expect("C++ accepts text style and text input drawable children with matching parents");
    assert_eq!(valid.imported_object_count(), 8);

    let invalid_style_feature_parent =
        read_runtime_file_with_error_kind(&synthetic_text_runtime_file(4317, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "TextStyleFeature", |bytes| {
                push_uint_property(bytes, "TextStyleFeature", "parentId", 1);
            });
        }))
        .expect_err("C++ rejects TextStyleFeature objects outside TextStyle parents");
    assert_eq!(
        invalid_style_feature_parent.kind(),
        RuntimeReadErrorKind::Malformed
    );
    assert!(
        invalid_style_feature_parent.message().contains("TextStyle"),
        "malformed message should identify the TextStyle parent requirement: {}",
        invalid_style_feature_parent.message()
    );

    let invalid_style_axis_parent =
        read_runtime_file_with_error_kind(&synthetic_text_runtime_file(4318, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "TextStyleAxis", |bytes| {
                push_uint_property(bytes, "TextStyleAxis", "parentId", 1);
            });
        }))
        .expect_err("C++ rejects TextStyleAxis objects outside TextStyle parents");
    assert_eq!(
        invalid_style_axis_parent.kind(),
        RuntimeReadErrorKind::Malformed
    );
    assert!(
        invalid_style_axis_parent.message().contains("TextStyle"),
        "malformed message should identify the TextStyle parent requirement: {}",
        invalid_style_axis_parent.message()
    );

    let invalid_input_drawable_parent =
        read_runtime_file_with_error_kind(&synthetic_text_runtime_file(4319, |bytes| {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "TextInputText", |bytes| {
                push_uint_property(bytes, "TextInputText", "parentId", 1);
            });
        }))
        .expect_err("C++ rejects TextInputDrawable objects outside TextInput parents");
    assert_eq!(
        invalid_input_drawable_parent.kind(),
        RuntimeReadErrorKind::Malformed
    );
    assert!(
        invalid_input_drawable_parent
            .message()
            .contains("TextInput"),
        "malformed message should identify the TextInput parent requirement: {}",
        invalid_input_drawable_parent.message()
    );
}

#[test]
fn text_value_run_resolution_is_permissive_like_cpp_import() {
    let valid = read_runtime_file(&synthetic_text_runtime_file(4326, |bytes| {
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStylePaint", |bytes| {
            push_uint_property(bytes, "TextStylePaint", "parentId", 1);
        });
        push_object_with_properties(bytes, "TextValueRun", |bytes| {
            push_uint_property(bytes, "TextValueRun", "parentId", 1);
            push_uint_property(bytes, "TextValueRun", "styleId", 2);
        });
    }))
    .expect("C++ accepts TextValueRun under Text with a TextStylePaint style");
    assert_eq!(valid.imported_object_count(), 5);

    let non_text_run_parent = read_runtime_file(&synthetic_text_runtime_file(4327, |bytes| {
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStylePaint", |bytes| {
            push_uint_property(bytes, "TextStylePaint", "parentId", 1);
        });
        push_object_with_properties(bytes, "TextValueRun", |bytes| {
            push_uint_property(bytes, "TextValueRun", "parentId", 2);
            push_uint_property(bytes, "TextValueRun", "styleId", 3);
        });
    }))
    .expect("C++ imports TextValueRun with a non-Text parent when styleId resolves");
    assert_eq!(non_text_run_parent.imported_object_count(), 6);

    let non_paint_style_id = read_runtime_file(&synthetic_text_runtime_file(4328, |bytes| {
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStyle", |bytes| {
            push_uint_property(bytes, "TextStyle", "parentId", 1);
        });
        push_object_with_properties(bytes, "TextValueRun", |bytes| {
            push_uint_property(bytes, "TextValueRun", "parentId", 1);
            push_uint_property(bytes, "TextValueRun", "styleId", 2);
        });
    }))
    .expect("C++ imports TextValueRun even when styleId resolves to plain TextStyle");
    assert_eq!(non_paint_style_id.imported_object_count(), 5);
}

fn synthetic_text_runtime_file(
    file_id: u64,
    artboard_objects: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        artboard_objects(bytes);
    })
}

#[test]
fn truncated_varuints_are_malformed_like_cpp_runtime() {
    assert!(
        read_runtime_file(b"RIVE\x80").is_err(),
        "C++ treats a truncated header varuint as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4261, |bytes| {
            bytes.push(0x80);
        }))
        .is_err(),
        "C++ treats a truncated object type varuint as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4262, |bytes| {
            push_var_uint(bytes, 2);
            bytes.push(0x80);
        }))
        .is_err(),
        "C++ treats a truncated property-key varuint as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4263, |bytes| {
            push_var_uint(bytes, 2);
            push_var_uint(bytes, 4);
            bytes.push(0x80);
        }))
        .is_err(),
        "C++ treats a truncated string-length varuint as malformed"
    );
}

#[test]
fn overwide_noncanonical_varuints_decode_like_cpp_runtime() {
    let mut overwide_header = Vec::new();
    overwide_header.extend_from_slice(b"RIVE");
    push_overwide_noncanonical_var_uint(&mut overwide_header, 7);
    push_var_uint(&mut overwide_header, 0);
    push_var_uint(&mut overwide_header, 0);
    push_overwide_noncanonical_var_uint(&mut overwide_header, 0);

    let header =
        read_runtime_file(&overwide_header).expect("C++ accepts overwide LEB128 header values");
    assert_eq!(header.header.major_version, 7);

    let file = read_runtime_file(&synthetic_runtime_file(4280, |bytes| {
        push_overwide_noncanonical_var_uint(bytes, 2);
        push_overwide_noncanonical_var_uint(bytes, 4);
        push_overwide_noncanonical_var_uint(bytes, 2);
        bytes.extend_from_slice(b"ok");
        push_overwide_noncanonical_var_uint(bytes, 0);
    }))
    .expect("C++ accepts overwide LEB128 object-stream values");

    let node = file.object(0).expect("Node object");
    assert_eq!(node.type_name, "Node");
    assert_eq!(node.string_property("name"), Some("ok"));
}

#[test]
fn truncated_fixed_width_primitive_payloads_are_malformed_like_cpp_runtime() {
    assert!(
        read_runtime_file(&synthetic_runtime_file(4269, |bytes| {
            push_var_uint(bytes, 2);
            push_var_uint(bytes, 13);
            bytes.extend_from_slice(&[0x00, 0x00]);
        }))
        .is_err(),
        "C++ treats truncated known float32/double fields as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4270, |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 13);
            bytes.extend_from_slice(&[0x00, 0x00]);
        }))
        .is_err(),
        "C++ treats truncated global-fallback float32/double fields as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file_with_header(
            7,
            0,
            4271,
            &[(65_000, HeaderFieldKind::Double)],
            |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 65_000);
                bytes.extend_from_slice(&[0x00, 0x00]);
            },
        ))
        .is_err(),
        "C++ treats truncated header-ToC float32/double fields as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4272, |bytes| {
            push_var_uint(bytes, 18);
            push_var_uint(bytes, 37);
            bytes.extend_from_slice(&[0x00, 0x00]);
        }))
        .is_err(),
        "C++ treats truncated known color fields as malformed"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4273, |bytes| {
            push_var_uint(bytes, 129);
            push_var_uint(bytes, 245);
        }))
        .is_err(),
        "C++ treats truncated known bool fields as malformed"
    );
}

#[test]
fn noncanonical_bool_payloads_decode_like_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file(4277, |bytes| {
        push_var_uint(bytes, 129);
        push_var_uint(bytes, 245);
        bytes.push(2);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ imports bool payload bytes other than 1 as false");

    let property = file.object(0).expect("CustomPropertyBoolean object");
    assert_eq!(property.type_name, "CustomPropertyBoolean");
    assert_eq!(property.bool_property("propertyValue"), Some(false));
}

#[test]
fn unknown_object_type_keys_match_cpp_integer_semantics() {
    let max_unknown_object = read_runtime_file(&synthetic_runtime_file(4266, |bytes| {
        push_var_uint(bytes, i32::MAX as u64);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ accepts unknown object type keys at signed int max");
    assert_eq!(max_unknown_object.object_count(), 1);
    assert_eq!(max_unknown_object.known_object_count(), 0);
    assert!(max_unknown_object.objects[0].is_none());

    let unknown_object = read_runtime_file(&synthetic_runtime_file(4243, |bytes| {
        push_var_uint(bytes, 70_000);
        push_var_uint(bytes, 4);
        push_var_uint(bytes, 7);
        bytes.extend_from_slice(b"mystery");
        push_var_uint(bytes, 0);
    }))
    .expect("C++ accepts unknown object type keys above u16 when they fit in signed int");
    assert_eq!(unknown_object.object_count(), 1);
    assert_eq!(unknown_object.known_object_count(), 0);
    assert!(unknown_object.objects[0].is_none());

    assert!(
        read_runtime_file(&synthetic_runtime_file(4244, |bytes| {
            push_var_uint(bytes, i32::MAX as u64 + 1);
            push_var_uint(bytes, 0);
        }))
        .is_err(),
        "C++ treats object type keys outside signed int range as malformed"
    );
}

#[test]
fn property_keys_match_cpp_uint16_range() {
    let max_property_key_missing_toc = read_runtime_file(&synthetic_runtime_file(4267, |bytes| {
        push_var_uint(bytes, 70_000);
        push_var_uint(bytes, u16::MAX as u64);
    }))
    .expect("C++ accepts object property keys at uint16_t max");
    assert_eq!(max_property_key_missing_toc.object_count(), 1);
    assert_eq!(max_property_key_missing_toc.known_object_count(), 0);
    assert!(max_property_key_missing_toc.objects[0].is_none());

    let max_property_key_with_toc = read_runtime_file(&synthetic_runtime_file_with_header(
        7,
        0,
        4268,
        &[(u16::MAX as u64, HeaderFieldKind::Uint)],
        |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, u16::MAX as u64);
            push_var_uint(bytes, u32::MAX as u64);
            push_var_uint(bytes, 0);
        },
    ))
    .expect("C++ skips object property keys at uint16_t max when the header ToC supplies a field");
    assert_eq!(max_property_key_with_toc.object_count(), 1);
    assert_eq!(max_property_key_with_toc.known_object_count(), 0);
    assert!(max_property_key_with_toc.objects[0].is_none());

    assert!(
        read_runtime_file(&synthetic_runtime_file(4258, |bytes| {
            push_var_uint(bytes, 2);
            push_var_uint(bytes, u64::from(u16::MAX) + 1);
        }))
        .is_err(),
        "C++ treats object property keys outside uint16_t range as malformed"
    );
}

#[test]
fn alternate_property_keys_are_skipped_like_cpp_deserialize_switches() {
    let file = read_runtime_file(&synthetic_runtime_file(4259, |bytes| {
        push_var_uint(bytes, 2);
        // Node.xArtboard is globally known as a double, but NodeBase::deserialize
        // only handles xPropertyKey. C++ skips this value instead of setting x.
        push_var_uint(bytes, 9);
        bytes.extend_from_slice(&12.5f32.to_le_bytes());
        push_var_uint(bytes, 0);
    }))
    .expect("alternate-key payload imports by falling back to the global field table");

    let node = file.object(0).expect("Node object");
    assert_eq!(node.double_property("x"), Some(0.0));
    assert_eq!(node.skipped_properties.len(), 1);
    assert_eq!(node.skipped_properties[0].key, 9);
    assert_eq!(
        node.skipped_properties[0].reason,
        SkipReason::UnknownProperty
    );
    assert_eq!(node.skipped_properties[0].field, Some("double"));
}

#[test]
fn duplicate_stored_properties_overwrite_like_cpp_deserialize_members() {
    let file = read_runtime_file(&synthetic_runtime_file(4274, |bytes| {
        push_var_uint(bytes, 2);
        push_var_uint(bytes, 13);
        bytes.extend_from_slice(&1.25f32.to_le_bytes());
        push_var_uint(bytes, 13);
        bytes.extend_from_slice(&2.5f32.to_le_bytes());
        push_var_uint(bytes, 0);
    }))
    .expect("C++ imports duplicate stored properties by assigning into the same member");

    let node = file.object(0).expect("Node object");
    assert_eq!(node.double_property("x"), Some(2.5));
    assert_eq!(
        node.properties
            .iter()
            .filter(|property| property.name == "x")
            .count(),
        1,
        "the arena should expose the final stored member state, not stale serialized writes"
    );
}

#[test]
fn absent_stored_properties_expose_cpp_member_defaults() {
    let file = read_runtime_file(&synthetic_runtime_file(4275, |bytes| {
        push_var_uint(bytes, 2);
        push_var_uint(bytes, 0);
        push_var_uint(bytes, 18);
        push_var_uint(bytes, 0);
        push_var_uint(bytes, 653);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ imports objects with omitted stored properties using generated member defaults");

    let node = file.object(0).expect("Node object");
    assert_eq!(node.string_property("name"), Some(""));
    assert_eq!(node.string_property_bytes("name"), Some(&[][..]));
    assert_eq!(node.uint_property("parentId"), Some(0));
    assert_eq!(node.double_property("x"), Some(0.0));
    assert_eq!(node.double_property("scaleX"), Some(1.0));
    assert!(
        node.property("name").is_none(),
        "serialized property list stays sparse even when typed helpers expose defaults"
    );

    let solid_color = file.object(1).expect("SolidColor object");
    assert_eq!(solid_color.color_property("colorValue"), Some(0xFF747474));

    let focus_data = file.object(2).expect("FocusData object");
    assert_eq!(focus_data.bool_property("canFocus"), Some(true));
    assert_eq!(focus_data.bool_property("canTouch"), Some(true));
    assert_eq!(focus_data.bool_property("canTraverse"), Some(true));
    assert_eq!(focus_data.uint_property("edgeBehaviorValue"), Some(0));

    let artboard = read_runtime_file(&synthetic_runtime_file(4284, |bytes| {
        push_var_uint(bytes, 1);
        push_var_uint(bytes, 0);
    }))
    .expect("empty Artboard imports with C++ constructor defaults");
    let artboard = artboard.object(0).expect("Artboard object");
    assert_eq!(artboard.bool_property("clip"), Some(true));
}

#[test]
fn focus_bool_aliases_read_the_serialized_focus_flags_bitmask() {
    let file = read_runtime_file(&synthetic_runtime_file(4287, |bytes| {
        push_var_uint(bytes, 653);
        push_var_uint(bytes, 1033);
        push_var_uint(bytes, 0b010);
        push_var_uint(bytes, 0);
    }))
    .expect("FocusData imports its packed focus flags");

    let focus_data = file.object(0).expect("FocusData object");
    assert_eq!(focus_data.uint_property("focusFlags"), Some(0b010));
    assert_eq!(focus_data.bool_property("canFocus"), Some(false));
    assert_eq!(focus_data.bool_property("canTouch"), Some(true));
    assert_eq!(focus_data.bool_property("canTraverse"), Some(false));
}

#[test]
fn duplicate_file_asset_ids_are_normalized_like_cpp_importer() {
    let file = read_runtime_file(&synthetic_runtime_file(4285, |bytes| {
        // Backboard.
        push_var_uint(bytes, 23);
        push_var_uint(bytes, 0);

        // ImageAsset inherits FileAsset.assetId.
        push_var_uint(bytes, 105);
        push_var_uint(bytes, 204);
        push_var_uint(bytes, 10);
        push_var_uint(bytes, 0);

        push_var_uint(bytes, 105);
        push_var_uint(bytes, 204);
        push_var_uint(bytes, 10);
        push_var_uint(bytes, 0);
    }))
    .expect("synthetic duplicate file asset ids import");

    let asset_ids = file
        .objects
        .iter()
        .filter_map(|object| object.as_ref())
        .filter(|object| object.type_name == "ImageAsset")
        .map(|object| object.uint_property("assetId"))
        .collect::<Vec<_>>();

    assert_eq!(
        asset_ids,
        vec![Some(10), Some(11)],
        "C++ BackboardImporter rewrites duplicate FileAsset.assetId values during import"
    );
}

#[test]
fn runtime_file_assets_match_cpp_file_assets_collection() {
    let file = read_runtime_file(&synthetic_runtime_file(4329, |bytes| {
        push_empty_image_asset_with_id(bytes, 7);
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ImageAsset");
        push_empty_object(bytes, "FontAsset");
        push_empty_object(bytes, "ScriptAsset");
        push_empty_object(bytes, "ManifestAsset");
    }))
    .expect("synthetic file assets import");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );

    let asset_type_names = file
        .file_assets()
        .iter()
        .map(|asset| asset.type_name)
        .collect::<Vec<_>>();
    assert_eq!(
        asset_type_names,
        vec!["ImageAsset", "FontAsset", "ScriptAsset"],
        "C++ File::assets() contains imported FileAssets that add to the Backboard and excludes ManifestAsset"
    );
    assert_eq!(
        file.file_asset(0).map(|asset| asset.type_name),
        Some("ImageAsset")
    );
    assert_eq!(
        file.file_asset(2).map(|asset| asset.type_name),
        Some("ScriptAsset")
    );
    assert!(file.file_asset(3).is_none());
}

#[test]
fn runtime_artboards_match_cpp_file_artboard_collection() {
    let raw_second_name = [b'S', b'e', b'c', 0xff, b'n', b'd'];
    let file = read_runtime_file(&synthetic_runtime_file(4333, |bytes| {
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "Dropped");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "First");
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_bytes_property(bytes, "Artboard", "name", &raw_second_name);
        });
    }))
    .expect("synthetic artboard collection imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );
    assert_eq!(file.object(0).unwrap().type_name, "Artboard");

    let artboard_names = file
        .artboards()
        .iter()
        .map(|artboard| artboard.string_property_bytes("name").unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(
        artboard_names,
        vec![b"First".as_slice(), raw_second_name.as_slice()],
        "C++ File::artboards contains only successfully imported Artboard objects in import order"
    );
    assert_eq!(file.default_artboard().map(|artboard| artboard.id), Some(2));
    assert_eq!(file.artboard(0).map(|artboard| artboard.id), Some(2));
    assert_eq!(file.artboard(1).map(|artboard| artboard.id), Some(3));
    assert!(file.artboard(2).is_none());
    assert_eq!(
        file.artboard_named("First").map(|artboard| artboard.id),
        Some(2)
    );
    assert!(file.artboard_named("Dropped").is_none());
    assert!(file.artboard_named("Sec").is_none());
    assert_eq!(
        file.artboard_named_bytes(&raw_second_name)
            .map(|artboard| artboard.id),
        Some(3)
    );
}

#[test]
fn runtime_artboard_referencers_resolve_cpp_backboard_artboard_ids() {
    let file = read_runtime_file(&synthetic_runtime_file(4344, |bytes| {
        push_object_with_properties(bytes, "NestedArtboard", |bytes| {
            push_uint_property(bytes, "NestedArtboard", "parentId", 0);
            push_uint_property(bytes, "NestedArtboard", "artboardId", 0);
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "First");
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "Second");
        });
        push_object_with_properties(bytes, "NestedArtboard", |bytes| {
            push_uint_property(bytes, "NestedArtboard", "parentId", 0);
            push_uint_property(bytes, "NestedArtboard", "artboardId", 0);
        });
        push_object_with_properties(bytes, "NestedArtboardLeaf", |bytes| {
            push_uint_property(bytes, "NestedArtboardLeaf", "parentId", 0);
            push_uint_property(bytes, "NestedArtboardLeaf", "artboardId", 1);
        });
        push_object_with_properties(bytes, "NestedArtboardLayout", |bytes| {
            push_uint_property(bytes, "NestedArtboardLayout", "parentId", 0);
            push_uint_property(bytes, "NestedArtboardLayout", "artboardId", 2);
        });
    }))
    .expect("synthetic artboard referencer stream imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        }),
        "NestedArtboard requires both BackboardImporter and ArtboardImporter contexts"
    );
    assert_eq!(
        file.resolved_artboard_for_referencer(0)
            .map(|artboard| artboard.string_property("name")),
        None
    );
    assert_eq!(
        file.resolved_artboard_for_referencer(4)
            .and_then(|artboard| artboard.string_property("name")),
        Some("First"),
        "C++ BackboardImporter resolves NestedArtboard.artboardId through File::artboard(index), not file object id"
    );
    assert_eq!(
        file.resolved_artboard_for_referencer(5)
            .and_then(|artboard| artboard.string_property("name")),
        Some("Second")
    );
    assert!(
        file.resolved_artboard_for_referencer(6).is_none(),
        "out-of-range C++ artboard referencer ids stay unresolved"
    );
    assert!(
        file.resolved_artboard_for_referencer(2).is_none(),
        "plain Artboard objects are not ArtboardReferencers"
    );
}

#[test]
fn runtime_artboard_authored_animation_and_state_machine_collections_match_cpp() {
    let raw_animation_name = [b'J', 0xff, b'm', b'p'];
    let file = read_runtime_file(&synthetic_runtime_file(4336, |bytes| {
        push_empty_object(bytes, "LinearAnimation");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "First");
        });
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_string_property(bytes, "LinearAnimation", "name", "walk");
            push_uint_property(bytes, "LinearAnimation", "fps", 24);
        });
        push_object_with_properties(bytes, "StateMachine", |bytes| {
            push_string_property(bytes, "StateMachine", "name", "controls");
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "Second");
        });
        push_object_with_properties(bytes, "StateMachine", |bytes| {
            push_string_property(bytes, "StateMachine", "name", "second-machine");
        });
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_bytes_property(bytes, "LinearAnimation", "name", &raw_animation_name);
        });
    }))
    .expect("synthetic artboard animation/state-machine collections import");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );
    assert_eq!(
        file.import_status(1),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );

    assert_eq!(
        file.artboard_animations(0)
            .iter()
            .map(|animation| animation.id)
            .collect::<Vec<_>>(),
        vec![4]
    );
    assert_eq!(
        file.artboard_animations(1)
            .iter()
            .map(|animation| animation.id)
            .collect::<Vec<_>>(),
        vec![8]
    );
    assert!(file.artboard_animations(2).is_empty());
    assert_eq!(
        file.artboard_animation(0, 0)
            .and_then(|animation| animation.string_property("name")),
        Some("walk")
    );
    assert_eq!(
        file.artboard_animation_named(0, "walk")
            .and_then(|animation| animation.uint_property("fps")),
        Some(24)
    );
    assert!(file.artboard_animation_named(0, "Jump").is_none());
    assert_eq!(
        file.artboard_animation_named_bytes(1, &raw_animation_name)
            .and_then(|animation| animation.string_property_bytes("name")),
        Some(raw_animation_name.as_slice())
    );
    assert!(file.artboard_animation(1, 1).is_none());

    assert_eq!(
        file.artboard_state_machines(0)
            .iter()
            .map(|state_machine| state_machine.id)
            .collect::<Vec<_>>(),
        vec![5]
    );
    assert_eq!(
        file.artboard_state_machines(1)
            .iter()
            .map(|state_machine| state_machine.id)
            .collect::<Vec<_>>(),
        vec![7]
    );
    assert!(file.artboard_state_machines(2).is_empty());
    assert_eq!(
        file.artboard_state_machine(0, 0)
            .and_then(|state_machine| state_machine.string_property("name")),
        Some("controls")
    );
    assert_eq!(
        file.artboard_state_machine_named(1, "second-machine")
            .map(|state_machine| state_machine.type_name),
        Some("StateMachine")
    );
    assert!(
        file.artboard_state_machine_named(1, "missing-machine")
            .is_none()
    );
    assert!(file.artboard_state_machine(1, 1).is_none());
}

#[test]
fn runtime_linear_animation_keyed_objects_match_cpp_pruning() {
    let node_x_key = property_key_for_name("Node", "x");
    let artboard_width_key = property_key_for_name("Artboard", "width");
    let file = read_runtime_file(&synthetic_runtime_file(4337, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_string_property(bytes, "LinearAnimation", "name", "move");
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(bytes, "KeyedProperty", "propertyKey", u64::from(node_x_key));
        });
        push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
            push_uint_property(bytes, "KeyFrameDouble", "frame", 10);
        });
        push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
            push_uint_property(bytes, "KeyFrameDouble", "frame", 20);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(artboard_width_key),
            );
        });
        push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
            push_uint_property(bytes, "KeyFrameDouble", "frame", 30);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 99);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(bytes, "KeyedProperty", "propertyKey", u64::from(node_x_key));
        });
        push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
            push_uint_property(bytes, "KeyFrameDouble", "frame", 40);
        });
        push_empty_object(bytes, "LinearAnimation");
    }))
    .expect("synthetic keyed animation collection imports");

    let animations = file.artboard_linear_animations(0);
    assert_eq!(animations.len(), 2);
    assert_eq!(animations[0].object.id, 3);
    assert_eq!(animations[1].object.id, 13);
    assert!(animations[1].keyed_objects.is_empty());

    assert_eq!(
        animations[0]
            .keyed_objects
            .iter()
            .map(|keyed_object| keyed_object.object.id)
            .collect::<Vec<_>>(),
        vec![4],
        "C++ removes KeyedObject entries whose objectId cannot resolve through Artboard::objects()"
    );
    assert_eq!(
        animations[0].keyed_objects[0]
            .object
            .uint_property("objectId"),
        Some(1)
    );
    assert_eq!(
        animations[0].keyed_objects[0]
            .keyed_properties
            .iter()
            .map(|keyed_property| keyed_property.object.id)
            .collect::<Vec<_>>(),
        vec![5],
        "C++ removes keyed properties unsupported by the resolved keyed target"
    );
    assert_eq!(
        animations[0].keyed_objects[0].keyed_properties[0]
            .object
            .uint_property("propertyKey"),
        Some(u64::from(node_x_key))
    );
    assert_eq!(
        animations[0].keyed_objects[0].keyed_properties[0]
            .first_key_frame
            .and_then(|key_frame| key_frame.uint_property("frame")),
        Some(10)
    );

    assert_eq!(
        file.artboard_linear_animation(0, 0)
            .map(|animation| animation.object.id),
        Some(3)
    );
    assert!(file.artboard_linear_animation(0, 2).is_none());
    assert!(file.artboard_linear_animations(1).is_empty());
}

#[test]
fn runtime_state_machine_graphs_match_cpp_child_grouping() {
    let file = read_runtime_file(&synthetic_runtime_file(4338, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "StateMachine", |bytes| {
            push_string_property(bytes, "StateMachine", "name", "controls");
        });
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
            push_uint_property(bytes, "ListenerBoolChange", "inputId", 0);
        });
        push_object_with_properties(bytes, "ListenerFireEvent", |bytes| {
            push_uint_property(bytes, "ListenerFireEvent", "flags", 2);
        });
        push_empty_object(bytes, "ListenerInputTypeKeyboard");
        push_empty_object(bytes, "BindablePropertyNumber");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", 123);
            push_uint_property(bytes, "DataBind", "flags", 5);
            push_uint_property(bytes, "DataBind", "converterId", 9);
        });
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", 777);
        });
        push_object_with_properties(bytes, "StateMachine", |bytes| {
            push_string_property(bytes, "StateMachine", "name", "empty");
        });
        push_empty_object(bytes, "StateMachineTrigger");
    }))
    .expect("synthetic state-machine graph stream imports");

    assert_eq!(file.import_status(11), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(16), Some(RuntimeImportStatus::Imported));

    let state_machines = file.artboard_state_machine_graphs(0);
    assert_eq!(state_machines.len(), 2);
    assert_eq!(state_machines[0].object.id, 2);
    assert_eq!(
        state_machines[0].object.string_property("name"),
        Some("controls")
    );
    assert_eq!(state_machines[0].layers.len(), 1);
    assert_eq!(state_machines[0].layers[0].object.id, 3);
    assert_eq!(state_machines[0].layers[0].state_count, 3);
    assert_eq!(
        state_machines[0]
            .inputs
            .iter()
            .map(|input| input.id)
            .collect::<Vec<_>>(),
        vec![7, 8]
    );
    assert_eq!(state_machines[0].listeners.len(), 1);
    assert_eq!(state_machines[0].listeners[0].object.id, 9);
    assert_eq!(
        state_machines[0].listeners[0]
            .actions
            .iter()
            .map(|action| action.object.id)
            .collect::<Vec<_>>(),
        vec![10],
        "listener actions flagged for layer ownership import but do not attach to the listener"
    );
    assert_eq!(
        state_machines[0].listeners[0]
            .listener_input_types
            .iter()
            .map(|input_type| input_type.id)
            .collect::<Vec<_>>(),
        vec![12]
    );
    assert_eq!(
        state_machines[0]
            .data_binds
            .iter()
            .map(|data_bind| data_bind.id)
            .collect::<Vec<_>>(),
        vec![14],
        "only DataBind objects following C++ state-machine-owned targets attach to the state machine"
    );
    assert_eq!(
        state_machines[0].data_binds[0].uint_property("flags"),
        Some(5)
    );
    assert_eq!(
        state_machines[0].data_binds[0].uint_property("converterId"),
        Some(9)
    );

    assert_eq!(state_machines[1].object.id, 17);
    assert!(state_machines[1].layers.is_empty());
    assert_eq!(
        state_machines[1]
            .inputs
            .iter()
            .map(|input| input.id)
            .collect::<Vec<_>>(),
        vec![18]
    );
    assert!(state_machines[1].listeners.is_empty());
    assert!(state_machines[1].data_binds.is_empty());

    assert_eq!(
        file.artboard_state_machine_graph(0, 0)
            .map(|state_machine| state_machine.object.id),
        Some(2)
    );
    assert!(file.artboard_state_machine_graph(0, 2).is_none());
    assert!(file.artboard_state_machine_graphs(1).is_empty());
}

#[test]
fn runtime_data_binds_resolve_cpp_data_converter_ids() {
    let file = read_runtime_file(&synthetic_runtime_file(4340, |bytes| {
        push_empty_object(bytes, "DataConverterRounder");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 0);
        });
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "DataConverterRounder");
        push_empty_object(bytes, "DataConverterStringTrim");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 1);
        });
        push_empty_object(bytes, "DataBind");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 7);
        });
    }))
    .expect("synthetic data-converter stream imports");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ only adds successfully imported converters and data binds to the backboard resolution path"
    );

    let converters = file.data_converters();
    assert_eq!(
        converters
            .iter()
            .map(|converter| converter.type_name)
            .collect::<Vec<_>>(),
        vec!["DataConverterRounder", "DataConverterStringTrim"]
    );
    assert_eq!(
        file.data_converter(1).map(|converter| converter.type_name),
        Some("DataConverterStringTrim")
    );
    assert!(file.data_converter(2).is_none());

    assert_eq!(
        file.resolved_data_converter_for_data_bind(5)
            .map(|converter| converter.id),
        Some(3),
        "converterId 0 resolves to the first imported DataConverter, not the dropped pre-backboard converter"
    );
    assert_eq!(
        file.resolved_data_converter_for_data_bind(6)
            .map(|converter| converter.id),
        Some(4)
    );
    assert!(file.resolved_data_converter_for_data_bind(7).is_none());
    assert!(file.resolved_data_converter_for_data_bind(8).is_none());
}

#[test]
fn runtime_data_converter_group_items_resolve_cpp_converter_ids() {
    let file = read_runtime_file(&synthetic_runtime_file(4341, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_empty_object(bytes, "DataConverterRounder");
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_empty_object(bytes, "DataConverterStringTrim");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 2);
        });
        push_empty_object(bytes, "DataConverterGroupItem");
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
    }))
    .expect("synthetic data-converter group stream imports");

    assert_eq!(
        file.import_status(1),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        }),
        "C++ requires a latest DataConverterGroupImporter before a group item imports"
    );
    assert_eq!(
        file.data_converters()
            .iter()
            .map(|converter| converter.type_name)
            .collect::<Vec<_>>(),
        vec![
            "DataConverterRounder",
            "DataConverterGroup",
            "DataConverterStringTrim",
            "DataConverterGroup"
        ]
    );

    let first_group_items = file.data_converter_group_items(1);
    assert_eq!(
        first_group_items
            .iter()
            .map(|item| item.object.id)
            .collect::<Vec<_>>(),
        vec![4, 6, 7],
        "C++ attaches group items to the latest DataConverterGroupImporter"
    );
    assert_eq!(
        first_group_items
            .iter()
            .map(|item| item.converter.map(|converter| converter.id))
            .collect::<Vec<_>>(),
        vec![Some(2), Some(5), None]
    );

    let second_group_items = file.data_converter_group_items(3);
    assert_eq!(second_group_items.len(), 1);
    assert_eq!(second_group_items[0].object.id, 9);
    assert_eq!(
        second_group_items[0]
            .converter
            .map(|converter| converter.id),
        Some(2)
    );
    assert!(file.data_converter_group_items(0).is_empty());
    assert!(file.data_converter_group_item(1, 3).is_none());
}

#[test]
fn runtime_data_converter_bind_from_context_effect_matches_cpp_overrides() {
    let node_x_key = u64::from(property_key_for_name("Node", "x"));
    let file = read_runtime_file(&synthetic_runtime_file(4341, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "propertyKey", node_x_key);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
        });
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            let mut source_path = Vec::new();
            push_var_uint(&mut source_path, 7);
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &source_path,
            );
        });
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "DataConverterToString");
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 99);
        });
    }))
    .expect("synthetic data-converter bind-from-context stream imports");

    assert_eq!(
        file.data_converter_bind_context_effect_for_object(
            file.object(3).expect("DataBindContext object"),
            &[3],
            true,
            true,
            true,
            None,
        ),
        None,
        "DataConverter::bindFromContext effect only applies to DataConverter objects"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(2, &[2], true, true, false, None),
        None,
        "converter-owned bind context modeling validates owned DataBind ids"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(4, &[], true, true, false, None),
        None
    );

    assert_eq!(
        file.data_converter_bind_context_effect(2, &[3, 4], true, true, false, None),
        Some(RuntimeDataConverterBindContextEffect {
            stores_parent_data_bind: true,
            owned_data_bind_context_effect: RuntimeDataBindContainerBindContextEffect {
                bind_from_context_data_bind_ids: vec![3],
                stores_data_context: true,
                clears_data_context: false,
            },
            group_child_bind_from_context_converter_ids: Vec::new(),
            operation_view_model_uses_source_path_lookup: false,
            operation_view_model_sets_number_source: false,
            operation_view_model_adds_data_bind_dependent: false,
            formula_checks_parent_data_bind_source: false,
            formula_sets_source_from_parent: false,
            formula_adds_source_dependent: false,
        }),
        "C++ DataConverter::bindFromContext stores the parent DataBind and binds owned DataBindContext children"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(2, &[3, 4], false, false, false, None),
        Some(RuntimeDataConverterBindContextEffect {
            stores_parent_data_bind: true,
            owned_data_bind_context_effect: RuntimeDataBindContainerBindContextEffect {
                bind_from_context_data_bind_ids: vec![3],
                stores_data_context: false,
                clears_data_context: true,
            },
            group_child_bind_from_context_converter_ids: Vec::new(),
            operation_view_model_uses_source_path_lookup: false,
            operation_view_model_sets_number_source: false,
            operation_view_model_adds_data_bind_dependent: false,
            formula_checks_parent_data_bind_source: false,
            formula_sets_source_from_parent: false,
            formula_adds_source_dependent: false,
        }),
        "C++ DataConverter::bindFromContext delegates null data contexts through DataBindContainer::bindDataBindsFromContext"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(
            0,
            &[3],
            true,
            true,
            false,
            Some(RuntimeDataType::Number),
        ),
        Some(RuntimeDataConverterBindContextEffect {
            stores_parent_data_bind: true,
            owned_data_bind_context_effect: RuntimeDataBindContainerBindContextEffect {
                bind_from_context_data_bind_ids: vec![3],
                stores_data_context: true,
                clears_data_context: false,
            },
            group_child_bind_from_context_converter_ids: Vec::new(),
            operation_view_model_uses_source_path_lookup: true,
            operation_view_model_sets_number_source: true,
            operation_view_model_adds_data_bind_dependent: true,
            formula_checks_parent_data_bind_source: false,
            formula_sets_source_from_parent: false,
            formula_adds_source_dependent: false,
        }),
        "C++ DataConverterOperationViewModel::bindFromContext stores number lookup results and adds the parent bind as a dependent"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(
            0,
            &[3],
            true,
            true,
            false,
            Some(RuntimeDataType::String),
        )
        .map(|effect| {
            (
                effect.operation_view_model_uses_source_path_lookup,
                effect.operation_view_model_sets_number_source,
                effect.operation_view_model_adds_data_bind_dependent,
            )
        }),
        Some((true, false, false)),
        "C++ DataConverterOperationViewModel::bindFromContext ignores non-number lookup results"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(1, &[], true, true, true, None),
        Some(RuntimeDataConverterBindContextEffect {
            stores_parent_data_bind: true,
            owned_data_bind_context_effect: RuntimeDataBindContainerBindContextEffect {
                bind_from_context_data_bind_ids: Vec::new(),
                stores_data_context: true,
                clears_data_context: false,
            },
            group_child_bind_from_context_converter_ids: Vec::new(),
            operation_view_model_uses_source_path_lookup: false,
            operation_view_model_sets_number_source: false,
            operation_view_model_adds_data_bind_dependent: false,
            formula_checks_parent_data_bind_source: true,
            formula_sets_source_from_parent: true,
            formula_adds_source_dependent: true,
        }),
        "C++ DataConverterFormula::bindFromContext binds to the parent DataBind source when one exists"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(1, &[], true, true, false, None)
            .map(|effect| {
                (
                    effect.formula_checks_parent_data_bind_source,
                    effect.formula_sets_source_from_parent,
                    effect.formula_adds_source_dependent,
                )
            }),
        Some((true, false, false)),
        "C++ DataConverterFormula::bindFromContext skips source/dependent work when the parent bind has no source"
    );
    assert_eq!(
        file.data_converter_bind_context_effect(3, &[3], true, true, true, None),
        Some(RuntimeDataConverterBindContextEffect {
            stores_parent_data_bind: true,
            owned_data_bind_context_effect: RuntimeDataBindContainerBindContextEffect {
                bind_from_context_data_bind_ids: vec![3],
                stores_data_context: true,
                clears_data_context: false,
            },
            group_child_bind_from_context_converter_ids: vec![5, 6],
            operation_view_model_uses_source_path_lookup: false,
            operation_view_model_sets_number_source: false,
            operation_view_model_adds_data_bind_dependent: false,
            formula_checks_parent_data_bind_source: false,
            formula_sets_source_from_parent: false,
            formula_adds_source_dependent: false,
        }),
        "C++ DataConverterGroup::bindFromContext calls bindFromContext on each non-null item converter after the base binding"
    );
}

#[test]
fn runtime_data_converter_lifecycle_effects_match_cpp_overrides() {
    let node_x_key = u64::from(property_key_for_name("Node", "x"));
    let file = read_runtime_file(&synthetic_runtime_file(4341, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "propertyKey", node_x_key);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
        });
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            let mut source_path = Vec::new();
            push_var_uint(&mut source_path, 7);
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &source_path,
            );
        });
        push_object_with_properties(bytes, "DataConverterFormula", |bytes| {
            push_uint_property(bytes, "DataConverterFormula", "randomModeValue", 2);
        });
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "DataConverterToString");
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 99);
        });
        push_empty_object(bytes, "DataConverterInterpolator");
        push_empty_object(bytes, "DataConverterRangeMapper");
        push_empty_object(bytes, "DataConverterStringPad");
        push_empty_object(bytes, "DataConverterStringTrim");
        push_empty_object(bytes, "DataConverterToString");
        push_empty_object(bytes, "DataConverterNumberToList");
        push_empty_object(bytes, "DataConverterOperationValue");
        push_empty_object(bytes, "DataConverterRounder");
    }))
    .expect("synthetic data-converter lifecycle stream imports");

    assert_eq!(
        file.data_converter_unbind_effect_for_object(
            file.object(3).expect("DataBindContext object"),
            &[],
            false
        ),
        None,
        "DataConverter::unbind effect only applies to DataConverter objects"
    );
    assert_eq!(
        file.data_converter_unbind_effect(3, &[3, 5], false),
        Some(RuntimeDataConverterUnbindEffect {
            owned_data_bind_unbind_effect: Some(RuntimeDataBindContainerUnbindEffect {
                unbinds_data_bind_ids: vec![3, 5],
                clears_data_context: true,
            }),
            group_child_unbind_converter_ids: Vec::new(),
            formula_removes_source_dependent: false,
            formula_clears_source: false,
        }),
        "C++ base DataConverter::unbind delegates to DataBindContainer::unbindDataBinds"
    );
    assert_eq!(
        file.data_converter_unbind_effect(1, &[3], true),
        Some(RuntimeDataConverterUnbindEffect {
            owned_data_bind_unbind_effect: None,
            group_child_unbind_converter_ids: Vec::new(),
            formula_removes_source_dependent: true,
            formula_clears_source: true,
        }),
        "C++ DataConverterFormula::unbind removes its bound source dependent but does not call base unbindDataBinds"
    );
    assert_eq!(
        file.data_converter_unbind_effect(1, &[3], false)
            .map(|effect| {
                (
                    effect.owned_data_bind_unbind_effect,
                    effect.formula_removes_source_dependent,
                    effect.formula_clears_source,
                )
            }),
        Some((None, false, false)),
        "C++ DataConverterFormula::unbind is a no-op when it has no bound source"
    );
    assert_eq!(
        file.data_converter_unbind_effect(4, &[3], false),
        Some(RuntimeDataConverterUnbindEffect {
            owned_data_bind_unbind_effect: None,
            group_child_unbind_converter_ids: vec![6, 7],
            formula_removes_source_dependent: false,
            formula_clears_source: false,
        }),
        "C++ DataConverterGroup::unbind forwards to non-null child converters without calling base unbindDataBinds"
    );

    assert_eq!(
        file.data_converter_update_effect(
            3,
            &[4],
            &[false],
            &[3],
            &[5],
            &[3],
            &[5],
            &[4],
            &[5],
            false,
        ),
        Some(RuntimeDataConverterUpdateEffect {
            owned_data_bind_update_effect: Some(RuntimeDataBindContainerUpdateEffect {
                return_reason: None,
                enters_processing: true,
                update_steps: vec![
                    RuntimeDataBindContainerUpdateStep {
                        data_bind_id: 4,
                        queue: RuntimeDataBindUpdateQueue::Persisting,
                        apply_target_to_source: false,
                        clears_dirty_list_flag: false,
                    },
                    RuntimeDataBindContainerUpdateStep {
                        data_bind_id: 3,
                        queue: RuntimeDataBindUpdateQueue::DirtyToSource,
                        apply_target_to_source: false,
                        clears_dirty_list_flag: true,
                    },
                    RuntimeDataBindContainerUpdateStep {
                        data_bind_id: 5,
                        queue: RuntimeDataBindUpdateQueue::DirtyToTarget,
                        apply_target_to_source: false,
                        clears_dirty_list_flag: true,
                    },
                ],
                skipped_persisting_data_bind_ids: Vec::new(),
                clears_dirty_to_source_queue: true,
                clears_dirty_to_target_queue: true,
                clears_dirty_list_flag_data_bind_ids: vec![3, 5],
                next_dirty_to_source_data_bind_ids: vec![3],
                next_dirty_to_target_data_bind_ids: vec![5],
                flushes_pending_addition_ids: vec![4],
                flushes_pending_removal_ids: vec![5],
                exits_processing: true,
            }),
            group_child_update_converter_ids: Vec::new(),
        }),
        "C++ DataConverter::update calls updateDataBinds(false), so target-to-source application stays disabled"
    );
    assert_eq!(
        file.data_converter_update_effect(4, &[], &[], &[], &[], &[], &[], &[], &[], false),
        Some(RuntimeDataConverterUpdateEffect {
            owned_data_bind_update_effect: None,
            group_child_update_converter_ids: vec![6, 7],
        }),
        "C++ DataConverterGroup::update forwards to non-null child converters without calling base updateDataBinds"
    );
    assert_eq!(
        file.data_converter_update_effect(3, &[4], &[], &[], &[], &[], &[], &[], &[], false),
        None,
        "Rust still requires one canSkip result per persisting data bind when base updateDataBinds runs"
    );
    assert_eq!(
        file.data_converter_reset_effect_for_object(
            file.object(3).expect("DataBindContext object")
        ),
        None,
        "DataConverter::reset effect only applies to DataConverter objects"
    );
    assert_eq!(
        file.data_converter_reset_effect(3),
        Some(RuntimeDataConverterResetEffect {
            group_child_reset_converter_ids: Vec::new(),
            resets_interpolator_advance_count: false,
            disposes_interpolator_advancer_values: false,
            clears_interpolator_smoothing_animation: false,
            clears_interpolator_initialized: false,
        }),
        "C++ base DataConverter::reset is an empty virtual method"
    );
    assert_eq!(
        file.data_converter_reset_effect(1),
        Some(RuntimeDataConverterResetEffect {
            group_child_reset_converter_ids: Vec::new(),
            resets_interpolator_advance_count: false,
            disposes_interpolator_advancer_values: false,
            clears_interpolator_smoothing_animation: false,
            clears_interpolator_initialized: false,
        }),
        "C++ DataConverterFormula inherits the empty base reset"
    );
    assert_eq!(
        file.data_converter_reset_effect(4),
        Some(RuntimeDataConverterResetEffect {
            group_child_reset_converter_ids: vec![6, 7],
            resets_interpolator_advance_count: false,
            disposes_interpolator_advancer_values: false,
            clears_interpolator_smoothing_animation: false,
            clears_interpolator_initialized: false,
        }),
        "C++ DataConverterGroup::reset forwards to each non-null child converter"
    );
    assert_eq!(
        file.data_converter_reset_effect(5),
        Some(RuntimeDataConverterResetEffect {
            group_child_reset_converter_ids: Vec::new(),
            resets_interpolator_advance_count: true,
            disposes_interpolator_advancer_values: true,
            clears_interpolator_smoothing_animation: true,
            clears_interpolator_initialized: true,
        }),
        "C++ DataConverterInterpolator::reset clears advance count and resets the advancer"
    );

    assert_eq!(
        file.data_bind_container_add_dirty_effect(4, true, false, false),
        Some(RuntimeDataBindContainerAddDirtyEffect {
            skips_persisting_to_source: true,
            skips_already_dirty: false,
            queue: None,
            queues_pending: false,
            sets_dirty_list_flag: false,
        }),
        "C++ DataBindContainer::addDirtyDataBind skips to-source binds already covered by the persisting list"
    );
    assert_eq!(
        file.data_bind_container_add_dirty_effect(4, false, false, true),
        Some(RuntimeDataBindContainerAddDirtyEffect {
            skips_persisting_to_source: false,
            skips_already_dirty: false,
            queue: Some(RuntimeDataBindUpdateQueue::DirtyToSource),
            queues_pending: true,
            sets_dirty_list_flag: true,
        }),
        "C++ DataBindContainer::addDirtyDataBind queues to-source dirty binds into the pending to-source list while processing"
    );
    assert_eq!(
        file.data_bind_container_add_dirty_effect(5, false, false, false),
        Some(RuntimeDataBindContainerAddDirtyEffect {
            skips_persisting_to_source: false,
            skips_already_dirty: false,
            queue: Some(RuntimeDataBindUpdateQueue::DirtyToTarget),
            queues_pending: false,
            sets_dirty_list_flag: true,
        }),
        "C++ DataBindContainer::addDirtyDataBind queues ordinary dirty binds into the active to-target list outside processing"
    );
    assert_eq!(
        file.data_bind_container_add_dirty_effect(5, false, true, false),
        Some(RuntimeDataBindContainerAddDirtyEffect {
            skips_persisting_to_source: false,
            skips_already_dirty: true,
            queue: None,
            queues_pending: false,
            sets_dirty_list_flag: false,
        }),
        "C++ DataBindContainer::addDirtyDataBind does not enqueue duplicates"
    );

    assert_eq!(
        file.data_converter_mark_dirty_effect(
            3,
            None,
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        Some(RuntimeDataConverterMarkDirtyEffect {
            parent_add_dirt_effect: None,
        }),
        "C++ DataConverter::markConverterDirty is a no-op without a parent DataBind"
    );
    assert_eq!(
        file.data_converter_mark_dirty_effect(
            3,
            Some(4),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        Some(RuntimeDataConverterMarkDirtyEffect {
            parent_add_dirt_effect: Some(RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
                target_origin: false,
                changed: true,
                invalidates_context_value: true,
                requests_dirty_update: true,
            }),
        }),
        "C++ DataConverter::markConverterDirty preserves a source-originated parent direction"
    );
    assert_eq!(
        file.data_converter_mark_dirty_effect_with_origin(
            3,
            Some(4),
            RuntimeComponentDirt::NONE,
            true,
            false,
            false,
            true,
            true,
        ),
        Some(RuntimeDataConverterMarkDirtyEffect {
            parent_add_dirt_effect: Some(RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS_TARGET,
                target_origin: true,
                changed: true,
                invalidates_context_value: true,
                requests_dirty_update: true,
            }),
        }),
        "C++ DataConverter::markConverterDirty preserves a target-originated parent direction"
    );
    assert_eq!(
        file.data_converter_property_change_effect(
            6,
            "minInput",
            Some(4),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        Some(RuntimeDataConverterPropertyChangeEffect {
            clears_number_to_list_items: false,
            mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect {
                parent_add_dirt_effect: Some(RuntimeDataBindAddDirtEffect {
                    dirt: RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
                    target_origin: false,
                    changed: true,
                    invalidates_context_value: true,
                    requests_dirty_update: true,
                }),
            },
        }),
        "C++ DataConverterRangeMapper::minInputChanged marks the parent converter bind dirty"
    );
    assert_eq!(
        file.data_converter_property_change_effect(
            10,
            "viewModelId",
            Some(4),
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            false,
            true,
        ),
        Some(RuntimeDataConverterPropertyChangeEffect {
            clears_number_to_list_items: true,
            mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect {
                parent_add_dirt_effect: Some(RuntimeDataBindAddDirtEffect {
                    dirt: RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
                    target_origin: false,
                    changed: true,
                    invalidates_context_value: false,
                    requests_dirty_update: true,
                }),
            },
        }),
        "C++ DataConverterNumberToList::viewModelIdChanged clears cached items before dirtying the parent bind"
    );
    assert_eq!(
        file.data_converter_property_change_effect(
            11,
            "operationValue",
            None,
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        Some(RuntimeDataConverterPropertyChangeEffect {
            clears_number_to_list_items: false,
            mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect {
                parent_add_dirt_effect: None,
            },
        }),
        "C++ property-change hooks still call markConverterDirty even when the converter has no parent bind"
    );
    assert_eq!(
        file.data_converter_property_change_effect(
            6,
            "flags",
            Some(4),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        None,
        "DataConverterRangeMapper::flags has no C++ dirty callback"
    );
    assert_eq!(
        file.data_converter_property_change_effect(
            12,
            "decimals",
            Some(4),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        None,
        "DataConverterRounder::decimals does not currently mark the converter dirty in C++"
    );
    assert_eq!(
        file.data_converter_property_change_effect_for_object(
            file.object(3).expect("DataBindContext object"),
            "minInput",
            Some(4),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        None,
        "converter property-change modeling validates converter objects"
    );
    assert_eq!(
        file.data_converter_mark_dirty_effect(
            3,
            Some(2),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
        ),
        None,
        "converter parent dirty modeling validates live parent DataBind ids"
    );
    assert_eq!(
        file.data_converter_add_dirty_data_bind_effect(
            3,
            5,
            Some(4),
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true,
            false,
            false,
            false,
        ),
        Some(RuntimeDataConverterAddDirtyDataBindEffect {
            mark_converter_dirty_effect: RuntimeDataConverterMarkDirtyEffect {
                parent_add_dirt_effect: Some(RuntimeDataBindAddDirtEffect {
                    dirt: RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
                    target_origin: false,
                    changed: true,
                    invalidates_context_value: true,
                    requests_dirty_update: true,
                }),
            },
            container_add_dirty_effect: RuntimeDataBindContainerAddDirtyEffect {
                skips_persisting_to_source: false,
                skips_already_dirty: false,
                queue: Some(RuntimeDataBindUpdateQueue::DirtyToTarget),
                queues_pending: false,
                sets_dirty_list_flag: true,
            },
        }),
        "C++ DataConverter::addDirtyDataBind first dirties the parent converter bind, then queues the owned bind"
    );
    assert_eq!(
        file.data_converter_formula_add_dirt_effect(1),
        Some(RuntimeDataConverterFormulaAddDirtEffect {
            clears_randoms: true,
        }),
        "C++ DataConverterFormula::addDirt clears cached randoms only for RandomMode::sourceChange"
    );
    assert_eq!(
        file.data_converter_formula_add_dirt_effect(2),
        Some(RuntimeDataConverterFormulaAddDirtEffect {
            clears_randoms: false,
        })
    );
    assert_eq!(file.data_converter_formula_add_dirt_effect(3), None);
}

#[test]
fn runtime_data_converter_formula_tokens_follow_cpp_latest_formula_importer() {
    let file = read_runtime_file(&synthetic_runtime_file(4351, |bytes| {
        push_empty_object(bytes, "FormulaTokenInput");
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "FormulaTokenInput");
        push_empty_object(bytes, "FormulaTokenOperation");
        push_empty_object(bytes, "DataConverterRounder");
        push_empty_object(bytes, "FormulaTokenValue");
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "FormulaTokenInput");
    }))
    .expect("synthetic formula-token stream imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        }),
        "C++ requires a latest DataConverterFormulaImporter before formula tokens import"
    );
    assert_eq!(
        file.data_converters()
            .iter()
            .map(|converter| converter.type_name)
            .collect::<Vec<_>>(),
        vec![
            "DataConverterFormula",
            "DataConverterRounder",
            "DataConverterFormula",
        ]
    );
    assert_eq!(
        file.data_converter_formula_tokens(0)
            .iter()
            .map(|token| token.type_name)
            .collect::<Vec<_>>(),
        vec![
            "FormulaTokenInput",
            "FormulaTokenValue",
            "FormulaTokenOperation",
        ],
        "C++ keeps the latest DataConverterFormulaImporter active across intervening non-formula converters and resolves formula output order"
    );
    assert!(
        file.data_converter_formula_tokens(1).is_empty(),
        "non-formula converters do not own formula tokens"
    );
    assert_eq!(
        file.data_converter_formula_tokens(2)
            .iter()
            .map(|token| token.type_name)
            .collect::<Vec<_>>(),
        vec!["FormulaTokenInput"]
    );
}

#[test]
fn runtime_data_converters_resolve_cpp_key_frame_interpolator_ids() {
    let file = read_runtime_file(&synthetic_runtime_file(4342, |bytes| {
        push_empty_object(bytes, "CubicEaseInterpolator");
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "CubicEaseInterpolator");
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "interpolatorId", 0);
        });
        push_object_with_properties(bytes, "DataConverterInterpolator", |bytes| {
            push_uint_property(bytes, "DataConverterInterpolator", "interpolatorId", 0);
        });
        push_empty_object(bytes, "DataConverterRangeMapper");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "CubicEaseInterpolator");
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "interpolatorId", 1);
        });
    }))
    .expect("synthetic data-converter interpolator stream imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        }),
        "C++ requires either a backboard or artboard importer for KeyFrameInterpolator objects"
    );

    assert_eq!(
        file.data_converter_interpolators()
            .iter()
            .map(|interpolator| interpolator.id)
            .collect::<Vec<_>>(),
        vec![2],
        "C++ BackboardImporter only collects keyframe interpolators imported before the latest ArtboardImporter exists"
    );
    assert_eq!(
        file.data_converter_interpolator(0)
            .map(|interpolator| interpolator.type_name),
        Some("CubicEaseInterpolator")
    );
    assert!(file.data_converter_interpolator(1).is_none());

    assert_eq!(
        file.resolved_interpolator_for_data_converter(3)
            .map(|interpolator| interpolator.id),
        Some(2)
    );
    assert_eq!(
        file.resolved_interpolator_for_data_converter(4)
            .map(|interpolator| interpolator.id),
        Some(2)
    );
    assert!(
        file.resolved_interpolator_for_data_converter(5).is_none(),
        "Core.missingId defaults do not resolve"
    );
    assert!(
        file.resolved_interpolator_for_data_converter(8).is_none(),
        "artboard-local interpolators are not part of the backboard converter-interpolator list"
    );
}

#[test]
fn runtime_scroll_physics_match_cpp_backboard_collection_and_constraint_resolution() {
    let file = read_runtime_file(&synthetic_runtime_file(4363, |bytes| {
        push_empty_object(bytes, "ElasticScrollPhysics");
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ElasticScrollPhysics", |bytes| {
            push_uint_property(bytes, "ElasticScrollPhysics", "constraintId", 100);
        });
        push_empty_object(bytes, "ClampedScrollPhysics");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "ElasticScrollPhysics", |bytes| {
            push_uint_property(bytes, "ElasticScrollPhysics", "constraintId", 200);
        });
        push_object_with_properties(bytes, "ScrollConstraint", |bytes| {
            push_uint_property(bytes, "ScrollConstraint", "parentId", 0);
            push_uint_property(bytes, "ScrollConstraint", "physicsId", 0);
        });
        push_object_with_properties(bytes, "ScrollConstraint", |bytes| {
            push_uint_property(bytes, "ScrollConstraint", "parentId", 0);
            push_uint_property(bytes, "ScrollConstraint", "physicsId", 1);
        });
        push_object_with_properties(bytes, "ScrollConstraint", |bytes| {
            push_uint_property(bytes, "ScrollConstraint", "parentId", 0);
            push_uint_property(bytes, "ScrollConstraint", "physicsId", 2);
        });
        push_object_with_properties(bytes, "ScrollConstraint", |bytes| {
            push_uint_property(bytes, "ScrollConstraint", "parentId", 0);
            push_uint_property(bytes, "ScrollConstraint", "physicsId", 3);
        });
        push_object_with_properties(bytes, "ScrollConstraint", |bytes| {
            push_uint_property(bytes, "ScrollConstraint", "parentId", 0);
        });
    }))
    .expect("synthetic scroll physics stream imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );
    assert_eq!(
        file.scroll_physics()
            .iter()
            .map(|physics| (physics.id, physics.type_name))
            .collect::<Vec<_>>(),
        vec![
            (2, "ElasticScrollPhysics"),
            (3, "ClampedScrollPhysics"),
            (5, "ElasticScrollPhysics")
        ],
        "C++ BackboardImporter collects imported ScrollPhysics objects in file order, even after an Artboard importer exists"
    );
    assert_eq!(
        file.scroll_physics_object(0)
            .and_then(|physics| physics.uint_property("constraintId")),
        Some(100)
    );
    assert_eq!(
        file.scroll_physics_object(2)
            .and_then(|physics| physics.uint_property("constraintId")),
        Some(200)
    );
    assert!(file.scroll_physics_object(3).is_none());

    assert_eq!(
        file.resolved_scroll_physics_for_constraint(6)
            .map(|physics| physics.id),
        Some(2)
    );
    assert_eq!(
        file.resolved_scroll_physics_for_constraint(7)
            .map(|physics| physics.id),
        Some(3)
    );
    assert_eq!(
        file.resolved_scroll_physics_for_constraint(8)
            .map(|physics| physics.id),
        Some(5)
    );
    assert!(file.resolved_scroll_physics_for_constraint(9).is_none());
    assert!(
        file.resolved_scroll_physics_for_constraint(10).is_none(),
        "Core.missingId defaults do not resolve"
    );
    assert!(
        file.resolved_scroll_physics_for_constraint(5).is_none(),
        "ScrollPhysics objects are not ScrollConstraint referencers"
    );
}

#[test]
fn runtime_artboard_data_binds_match_cpp_target_routing() {
    let file = read_runtime_file(&synthetic_runtime_file(4339, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", 11);
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Node", |bytes| {
            push_uint_property(bytes, "Node", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", 22);
        });
        push_empty_object(bytes, "DataEnumValue");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", 33);
        });
        push_empty_object(bytes, "DataBindPath");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", 44);
        });
    }))
    .expect("synthetic artboard data-bind stream imports");

    assert_eq!(
        file.import_status(7),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );

    let first_artboard_binds = file.artboard_data_binds(0);
    assert_eq!(first_artboard_binds.len(), 1);
    assert_eq!(first_artboard_binds[0].object.id, 3);
    assert_eq!(
        first_artboard_binds[0].target.map(|target| target.id),
        Some(2)
    );
    assert_eq!(first_artboard_binds[0].target_local_id, Some(1));
    assert_eq!(
        first_artboard_binds[0].object.uint_property("propertyKey"),
        Some(11)
    );

    let second_artboard_binds = file.artboard_data_binds(1);
    assert_eq!(
        second_artboard_binds
            .iter()
            .map(|data_bind| data_bind.object.id)
            .collect::<Vec<_>>(),
        vec![6, 10],
        "C++ clears lastBindableObject after a failed non-DataBind import, so object 8 has no artboard owner"
    );
    assert_eq!(
        second_artboard_binds
            .iter()
            .map(|data_bind| data_bind.target.map(|target| target.id))
            .collect::<Vec<_>>(),
        vec![Some(5), Some(9)]
    );
    assert_eq!(
        second_artboard_binds
            .iter()
            .map(|data_bind| data_bind.target_local_id)
            .collect::<Vec<_>>(),
        vec![Some(1), None],
        "non-component fallback targets attach to the latest artboard without an artboard-local target id"
    );

    assert_eq!(
        file.artboard_data_bind(1, 1)
            .map(|data_bind| data_bind.object.id),
        Some(10)
    );
    assert!(file.artboard_data_bind(1, 2).is_none());
    assert!(file.artboard_data_binds(2).is_empty());
}

#[test]
fn runtime_data_bind_lifecycle_flags_match_cpp_polling_rules() {
    let node_x_key = u64::from(property_key_for_name("Node", "x"));
    let node_computed_world_x_key = u64::from(property_key_for_name("Node", "computedWorldX"));
    let shape_length_key = u64::from(property_key_for_name("Shape", "length"));
    let solo_active_component_key = u64::from(property_key_for_name("Solo", "activeComponentId"));
    let bindable_asset_value_key = u64::from(property_key_for_name(
        "BindablePropertyAsset",
        "propertyValue",
    ));
    let bindable_number_value_key = u64::from(property_key_for_name(
        "BindablePropertyNumber",
        "propertyValue",
    ));
    let layout_display_value_key = u64::from(property_key_for_name(
        "LayoutComponentStyle",
        "displayValue",
    ));

    let file = read_runtime_file(&synthetic_runtime_file(4341, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_computed_world_x_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", shape_length_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", shape_length_key);
        });
        push_object_with_properties(bytes, "Solo", |bytes| {
            push_uint_property(bytes, "Solo", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", solo_active_component_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "BindablePropertyAsset");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", bindable_asset_value_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_empty_object(bytes, "BindablePropertyNumber");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", bindable_number_value_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", bindable_number_value_key);
            push_uint_property(bytes, "DataBind", "flags", 2);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", bindable_number_value_key);
            push_uint_property(bytes, "DataBind", "flags", 1 | 4 | 8 | 16);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", layout_display_value_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "propertyKey", node_x_key);
        });
        push_object_with_properties(bytes, "ArtboardComponentList", |bytes| {
            push_uint_property(bytes, "ArtboardComponentList", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
        });
        push_empty_object(bytes, "DataConverterToString");
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
            push_uint_property(bytes, "DataBind", "flags", 1);
            push_uint_property(bytes, "DataBind", "converterId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            let mut source_path = Vec::new();
            push_var_uint(&mut source_path, 42);
            push_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path);
            push_uint_property(bytes, "DataBindContext", "propertyKey", node_x_key);
            push_uint_property(bytes, "DataBindContext", "flags", 16);
            push_uint_property(bytes, "DataBindContext", "converterId", 0);
        });
    }))
    .expect("synthetic data-bind lifecycle stream imports");

    assert_eq!(file.data_bind_can_skip(2, true), None);
    assert_eq!(
        file.data_bind_collapse_effect(2, false, true, true, true),
        None
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            2,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            true,
            true
        ),
        None
    );
    assert_eq!(file.data_bind_add_effect(2, false, true), None);
    assert_eq!(file.data_bind_remove_effect(2, false, true, true), None);
    assert_eq!(
        file.data_bind_source_effect(2, RuntimeDataType::Number),
        None
    );
    assert_eq!(file.data_bind_clear_source_effect(2, true), None);
    assert_eq!(
        file.data_bind_bind_effect(
            2,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        None
    );
    assert_eq!(
        file.data_bind_target_effect(2, Some(2), false, true, false),
        None
    );
    assert_eq!(
        file.data_bind_unbind_effect(2, true, true, true, true),
        None
    );
    assert_eq!(
        file.data_bind_initialize_effect(2, false, false, true, true, true),
        None
    );
    assert_eq!(file.data_bind_relink_effect(2, true, true), None);
    assert_eq!(
        file.data_bind_context_bind_effect(
            2,
            false,
            true,
            true,
            false,
            false,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        None
    );
    assert_eq!(
        file.data_bind_context_bind_effect(
            3,
            false,
            true,
            true,
            false,
            false,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        None,
        "DataBindContext::bindFromContext effect only applies to DataBindContext objects"
    );
    assert_eq!(
        file.data_bind_container_bind_context_effect(&[2, 3], true),
        None
    );
    assert_eq!(file.data_bind_container_unbind_effect(&[2, 3]), None);
    assert_eq!(
        file.data_bind_container_advance_effect(&[2, 3], &[false, true]),
        None
    );
    assert_eq!(
        file.data_bind_container_update_effect(
            &[2],
            &[false],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            false,
            true
        ),
        None
    );
    assert_eq!(
        file.data_bind_container_update_effect(
            &[3],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            false,
            true
        ),
        None,
        "Rust requires one canSkip result per persisting data bind"
    );
    assert_eq!(file.data_bind_to_source(3), Some(true));
    assert_eq!(file.data_bind_to_target(3), Some(false));
    assert_eq!(file.data_bind_is_main_to_source(3), Some(true));
    assert_eq!(file.data_bind_binds_once(3), Some(false));
    assert_eq!(file.data_bind_source_to_target_runs_first(3), Some(false));
    assert_eq!(file.data_bind_is_name_based(3), Some(false));
    assert_eq!(file.data_bind_can_skip(3, false), Some(false));
    assert_eq!(file.data_bind_can_skip(3, true), Some(true));
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::NONE,
            false,
            false,
            true,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::NONE,
            target_origin: false,
            changed: false,
            invalidates_context_value: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::addDirt treats ComponentDirt::None as already marked"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::BINDINGS,
            true,
            false,
            true,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::NONE,
            target_origin: false,
            changed: false,
            invalidates_context_value: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::addDirt exits while Flag::SuppressDirt is set"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::BINDINGS,
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            true,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::BINDINGS,
            target_origin: false,
            changed: false,
            invalidates_context_value: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::addDirt exits when all added dirt bits are already marked"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::DEPENDENTS,
            false,
            false,
            true,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::DEPENDENTS,
            target_origin: false,
            changed: true,
            invalidates_context_value: true,
            requests_dirty_update: true,
        }),
        "C++ DataBind::addDirt invalidates context values when Dependents dirt is present"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::DEPENDENTS,
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            true,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
            target_origin: false,
            changed: true,
            invalidates_context_value: true,
            requests_dirty_update: true,
        }),
        "C++ DataBind::addDirt checks Dependents after OR-ing the new dirt"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::DEPENDENTS,
            false,
            false,
            false,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::DEPENDENTS,
            target_origin: false,
            changed: true,
            invalidates_context_value: false,
            requests_dirty_update: true,
        }),
        "C++ DataBind::addDirt only invalidates when m_ContextValue exists"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::BINDINGS,
            false,
            true,
            true,
            true
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::BINDINGS,
            target_origin: false,
            changed: true,
            invalidates_context_value: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::addDirt stores dirt while collapsed but does not ask the container to queue"
    );
    assert_eq!(
        file.data_bind_add_dirt_effect(
            3,
            RuntimeComponentDirt::NONE,
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            true,
            false
        ),
        Some(RuntimeDataBindAddDirtEffect {
            dirt: RuntimeComponentDirt::BINDINGS,
            target_origin: false,
            changed: true,
            invalidates_context_value: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::addDirt cannot queue without a container"
    );
    assert_eq!(
        file.data_bind_add_effect(3, true, true),
        Some(RuntimeDataBindAddEffect {
            queues_pending_addition: true,
            appends_to_data_binds: false,
            appends_to_persisting_list: false,
            sets_persisting_list_flag: false,
            sets_container: false,
            binds_from_data_context: false,
            runs_initial_update: false,
            initial_update_applies_target_to_source: false,
        }),
        "C++ DataBindContainer::addDataBind defers all immediate work while updateDataBinds is processing"
    );
    assert_eq!(
        file.data_bind_add_effect(3, false, false),
        Some(RuntimeDataBindAddEffect {
            queues_pending_addition: false,
            appends_to_data_binds: true,
            appends_to_persisting_list: false,
            sets_persisting_list_flag: false,
            sets_container: true,
            binds_from_data_context: false,
            runs_initial_update: false,
            initial_update_applies_target_to_source: false,
        }),
        "C++ DataBindContainer::addDataBind immediately appends push-capable data binds and sets their container"
    );
    assert_eq!(
        file.data_bind_add_effect(3, false, true),
        Some(RuntimeDataBindAddEffect {
            queues_pending_addition: false,
            appends_to_data_binds: true,
            appends_to_persisting_list: false,
            sets_persisting_list_flag: false,
            sets_container: true,
            binds_from_data_context: false,
            runs_initial_update: false,
            initial_update_applies_target_to_source: false,
        }),
        "C++ DataBindContainer::addDataBind only binds existing data contexts for DataBindContext objects"
    );
    assert_eq!(
        file.data_bind_add_effect(5, false, false),
        Some(RuntimeDataBindAddEffect {
            queues_pending_addition: false,
            appends_to_data_binds: true,
            appends_to_persisting_list: true,
            sets_persisting_list_flag: true,
            sets_container: true,
            binds_from_data_context: false,
            runs_initial_update: false,
            initial_update_applies_target_to_source: false,
        }),
        "C++ DataBindContainer::addDataBind puts to-source binds without push-capable targets on the persisting list"
    );
    assert_eq!(
        file.data_bind_source_effect(3, RuntimeDataType::Number),
        Some(RuntimeDataBindSourceEffect {
            adds_source_dependent: true,
            sets_source: true,
            updates_artboard_component_list_reset: false,
            artboard_component_list_should_reset_instances: false,
        }),
        "C++ DataBind::source adds a dependent for non-once data binds and stores the source"
    );
    assert_eq!(
        file.data_bind_clear_source_effect(3, true),
        Some(RuntimeDataBindClearSourceEffect {
            removes_source_dependent: true,
            clears_source: true,
        }),
        "C++ DataBind::clearSource removes the dependent for non-once data binds"
    );
    assert_eq!(
        file.data_bind_clear_source_effect(3, false),
        Some(RuntimeDataBindClearSourceEffect {
            removes_source_dependent: false,
            clears_source: false,
        }),
        "C++ DataBind::clearSource is a no-op when m_Source is null"
    );
    assert_eq!(
        file.data_bind_bind_effect(
            3,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindBindEffect {
            clears_existing_context_value: false,
            context_value_type: Some(RuntimeDataType::Number),
            resets_converter: false,
            removes_existing_target_observer: false,
            adds_target_observer: true,
            observing_after: true,
            add_dirt_effect: RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS_TARGET,
                target_origin: true,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            },
        }),
        "C++ DataBind::bind creates a source-typed context value, subscribes push-capable toSource targets, and marks target-direction reconcile dirt"
    );
    assert_eq!(
        file.data_bind_bind_effect(
            3,
            RuntimeDataType::String,
            true,
            true,
            true,
            RuntimeComponentDirt::BINDINGS,
            false,
            true
        ),
        Some(RuntimeDataBindBindEffect {
            clears_existing_context_value: true,
            context_value_type: Some(RuntimeDataType::String),
            resets_converter: false,
            removes_existing_target_observer: true,
            adds_target_observer: true,
            observing_after: true,
            add_dirt_effect: RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS | RuntimeComponentDirt::BINDINGS_TARGET,
                target_origin: true,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            },
        }),
        "C++ DataBind::bind deletes the old context value and re-subscribes an existing observer before re-marking dirt"
    );
    assert_eq!(
        file.data_bind_bind_effect(
            3,
            RuntimeDataType::None,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindBindEffect {
            clears_existing_context_value: false,
            context_value_type: None,
            resets_converter: false,
            removes_existing_target_observer: false,
            adds_target_observer: true,
            observing_after: true,
            add_dirt_effect: RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS_TARGET,
                target_origin: true,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            },
        }),
        "C++ DataBind::bind leaves m_ContextValue null when outputType() is none but still subscribes and dirties"
    );
    assert_eq!(
        file.data_bind_target_effect(3, Some(2), true, true, true),
        Some(RuntimeDataBindTargetEffect {
            unchanged: true,
            removes_existing_target_observer: false,
            sets_target: false,
            adds_target_observer: false,
            observing_after: true,
        }),
        "C++ DataBind::target returns immediately when the target pointer is unchanged"
    );
    assert_eq!(
        file.data_bind_target_effect(3, None, false, true, true),
        Some(RuntimeDataBindTargetEffect {
            unchanged: false,
            removes_existing_target_observer: true,
            sets_target: true,
            adds_target_observer: false,
            observing_after: false,
        }),
        "C++ DataBind::target removes the old observer before assigning a null target"
    );
    assert_eq!(
        file.data_bind_unbind_effect(3, true, true, true, true),
        Some(RuntimeDataBindUnbindEffect {
            clear_source_effect: RuntimeDataBindClearSourceEffect {
                removes_source_dependent: true,
                clears_source: true,
            },
            removes_target_observer: true,
            observing_after: false,
            unbinds_converter: false,
            clears_context_value: true,
        }),
        "C++ DataBind::unbind clears source, observer, and context value state"
    );
    assert_eq!(
        file.data_bind_initialize_effect(3, false, false, true, true, true),
        Some(RuntimeDataBindInitializeEffect {
            target_is_component: true,
            adds_component_collapsable: true,
            runs_collapse: true,
            collapse_effect: Some(RuntimeDataBindCollapseEffect {
                is_collapsed: true,
                changed: true,
                requests_dirty_update: false,
            }),
        }),
        "C++ DataBind::initialize adds component targets as collapsables and immediately mirrors the component collapsed state"
    );
    assert_eq!(
        file.data_bind_initialize_effect(3, true, false, true, true, true),
        Some(RuntimeDataBindInitializeEffect {
            target_is_component: true,
            adds_component_collapsable: false,
            runs_collapse: false,
            collapse_effect: None,
        }),
        "C++ Component::addCollapsable uses pushUnique semantics and skips collapse on duplicate registration"
    );
    assert_eq!(
        file.data_bind_initialize_effect(3, false, true, false, true, true),
        Some(RuntimeDataBindInitializeEffect {
            target_is_component: true,
            adds_component_collapsable: true,
            runs_collapse: true,
            collapse_effect: Some(RuntimeDataBindCollapseEffect {
                is_collapsed: false,
                changed: true,
                requests_dirty_update: true,
            }),
        }),
        "C++ Component::addCollapsable can uncollapse a dirty contained bind and queue it through DataBind::collapse"
    );
    assert_eq!(
        file.data_bind_relink_effect(3, true, true),
        Some(RuntimeDataBindRelinkEffect {
            calls_container_rebuild: true,
            rebuild_binds_from_context: false,
            rebuild_has_data_context: false,
        }),
        "C++ DataBind::relinkDataBind asks the container to rebuild ordinary DataBind objects without binding a DataBindContext"
    );
    assert_eq!(
        file.data_bind_remove_effect(3, true, true, true),
        Some(RuntimeDataBindRemoveEffect {
            queues_pending_removal: true,
            removes_from_data_binds: false,
            scans_persisting_list: false,
            clears_persisting_list_flag: false,
            scans_dirty_lists: false,
            clears_dirty_list_flag: false,
            clears_container: false,
        }),
        "C++ DataBindContainer::removeDataBind defers all cleanup while updateDataBinds is processing"
    );
    assert_eq!(
        file.data_bind_remove_effect(3, false, false, false),
        Some(RuntimeDataBindRemoveEffect {
            queues_pending_removal: false,
            removes_from_data_binds: true,
            scans_persisting_list: false,
            clears_persisting_list_flag: false,
            scans_dirty_lists: false,
            clears_dirty_list_flag: false,
            clears_container: true,
        }),
        "C++ DataBindContainer::removeDataBind immediately erases the primary bind list and clears the container"
    );
    assert_eq!(
        file.data_bind_remove_effect(3, false, true, false),
        Some(RuntimeDataBindRemoveEffect {
            queues_pending_removal: false,
            removes_from_data_binds: true,
            scans_persisting_list: true,
            clears_persisting_list_flag: true,
            scans_dirty_lists: false,
            clears_dirty_list_flag: false,
            clears_container: true,
        }),
        "C++ DataBindContainer::removeDataBind cleans the persisting list only when the bind flag is set"
    );
    assert_eq!(
        file.data_bind_remove_effect(3, false, false, true),
        Some(RuntimeDataBindRemoveEffect {
            queues_pending_removal: false,
            removes_from_data_binds: true,
            scans_persisting_list: false,
            clears_persisting_list_flag: false,
            scans_dirty_lists: true,
            clears_dirty_list_flag: true,
            clears_container: true,
        }),
        "C++ DataBindContainer::removeDataBind scans dirty queues only when the bind flag is set"
    );
    assert_eq!(
        file.data_bind_remove_effect(3, false, true, true),
        Some(RuntimeDataBindRemoveEffect {
            queues_pending_removal: false,
            removes_from_data_binds: true,
            scans_persisting_list: true,
            clears_persisting_list_flag: true,
            scans_dirty_lists: true,
            clears_dirty_list_flag: true,
            clears_container: true,
        }),
        "C++ DataBindContainer::removeDataBind can clean both persisting and dirty membership"
    );
    assert_eq!(
        file.data_bind_update_effect(2, RuntimeComponentDirt::BINDINGS, true, true, true, true),
        None
    );
    assert_eq!(
        file.data_bind_update_effect(3, RuntimeComponentDirt::NONE, true, true, true, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: false,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "push-driven C++ binds do not run target-to-source without target-origin dirt"
    );
    assert_eq!(
        file.data_bind_update_effect(
            3,
            RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
            true,
            true,
            true,
            true
        ),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: true,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "source-originated converter dirt updates dependents without reopening target-to-source"
    );
    assert_eq!(
        file.data_bind_update_effect(3, RuntimeComponentDirt::BINDINGS, false, true, true, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "C++ DataBindContainer::updateDataBind skips target-to-source when applyTargetToSource is false"
    );
    assert_eq!(
        file.data_bind_update_effect(3, RuntimeComponentDirt::BINDINGS, true, true, false, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "C++ DataBind::updateSourceBinding and update both require m_ContextValue"
    );
    assert_eq!(
        file.data_bind_update_effect(3, RuntimeComponentDirt::BINDINGS, true, true, true, false),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "C++ DataBind::updateSourceBinding requires m_target"
    );
    assert_eq!(
        file.data_bind_collapse_effect(3, false, false, true, true),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: false,
            changed: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::collapse is a no-op when the requested collapsed state already matches"
    );
    assert_eq!(
        file.data_bind_collapse_effect(3, false, true, true, true),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: true,
            changed: true,
            requests_dirty_update: false,
        }),
        "C++ DataBind::collapse sets Flag::Collapsed when targetSupportsPush is true"
    );
    assert_eq!(
        file.data_bind_collapse_effect(3, true, false, true, true),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: false,
            changed: true,
            requests_dirty_update: true,
        }),
        "C++ DataBind::collapse queues a dirty update when uncollapsing a dirty contained bind"
    );
    assert_eq!(
        file.data_bind_collapse_effect(3, true, false, false, true),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: false,
            changed: true,
            requests_dirty_update: false,
        }),
        "C++ DataBind::collapse does not dirty-queue when m_Dirt is none"
    );
    assert_eq!(
        file.data_bind_collapse_effect(3, true, false, true, false),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: false,
            changed: true,
            requests_dirty_update: false,
        }),
        "C++ DataBind::collapse does not dirty-queue without a container"
    );
    assert_eq!(file.data_bind_target_supports_push(3), Some(true));
    assert_eq!(file.data_bind_uses_persisting_list(3), Some(false));
    assert_eq!(
        file.data_bind_update_queue(3),
        Some(RuntimeDataBindUpdateQueue::DirtyToSource)
    );
    assert_eq!(file.data_bind_target_supports_push(5), Some(false));
    assert_eq!(
        file.data_bind_initialize_effect(5, false, false, true, true, true),
        Some(RuntimeDataBindInitializeEffect {
            target_is_component: true,
            adds_component_collapsable: true,
            runs_collapse: true,
            collapse_effect: Some(RuntimeDataBindCollapseEffect {
                is_collapsed: false,
                changed: false,
                requests_dirty_update: false,
            }),
        }),
        "C++ DataBind::initialize registers computed-property binds as collapsables even though DataBind::collapse ignores non-push targets"
    );
    assert_eq!(
        file.data_bind_bind_effect(
            5,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindBindEffect {
            clears_existing_context_value: false,
            context_value_type: Some(RuntimeDataType::Number),
            resets_converter: false,
            removes_existing_target_observer: false,
            adds_target_observer: false,
            observing_after: false,
            add_dirt_effect: RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS_TARGET,
                target_origin: true,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            },
        }),
        "C++ DataBind::bind does not subscribe toSource binds whose targetSupportsPush() is false"
    );
    assert_eq!(
        file.data_bind_target_effect(5, Some(4), false, true, true),
        Some(RuntimeDataBindTargetEffect {
            unchanged: false,
            removes_existing_target_observer: true,
            sets_target: true,
            adds_target_observer: false,
            observing_after: false,
        }),
        "C++ DataBind::target removes an old observer but does not add one for computed polling-only properties"
    );
    assert_eq!(
        file.data_bind_collapse_effect(5, false, true, true, true),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: false,
            changed: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::collapse ignores binds whose targets cannot push changes"
    );
    assert_eq!(file.data_bind_uses_persisting_list(5), Some(true));
    assert_eq!(
        file.data_bind_update_queue(5),
        Some(RuntimeDataBindUpdateQueue::Persisting)
    );
    assert_eq!(file.data_bind_target_supports_push(7), Some(false));
    assert_eq!(file.data_bind_uses_persisting_list(7), Some(true));
    assert_eq!(
        file.data_bind_update_queue(7),
        Some(RuntimeDataBindUpdateQueue::Persisting)
    );
    assert_eq!(
        file.data_bind_uses_persisting_list(8),
        Some(false),
        "C++ only puts toSource/twoWay binds on the persisting polling path"
    );
    assert_eq!(
        file.data_bind_bind_effect(
            8,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindBindEffect {
            clears_existing_context_value: false,
            context_value_type: Some(RuntimeDataType::Number),
            resets_converter: false,
            removes_existing_target_observer: false,
            adds_target_observer: false,
            observing_after: false,
            add_dirt_effect: RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS,
                target_origin: false,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            },
        }),
        "C++ DataBind::bind does not add property observers for toTarget-only binds"
    );
    assert_eq!(file.data_bind_to_source(8), Some(false));
    assert_eq!(file.data_bind_to_target(8), Some(true));
    assert_eq!(file.data_bind_is_main_to_source(8), Some(false));
    assert_eq!(file.data_bind_binds_once(8), Some(false));
    assert_eq!(file.data_bind_source_to_target_runs_first(8), Some(false));
    assert_eq!(file.data_bind_is_name_based(8), Some(false));
    assert_eq!(
        file.data_bind_update_effect(8, RuntimeComponentDirt::BINDINGS, true, true, true, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: true,
            source_to_target_is_main_direction: true,
            suppresses_dirt_while_applying_source_to_target: true,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "toTarget binds apply source-to-target during DataBind::update with dirt suppression"
    );
    assert_eq!(
        file.data_bind_update_effect(8, RuntimeComponentDirt::BINDINGS, true, false, true, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "C++ DataBind::update requires m_Source before source-to-target apply"
    );
    assert_eq!(
        file.data_bind_update_queue(8),
        Some(RuntimeDataBindUpdateQueue::DirtyToTarget),
        "toTarget binds use the ordinary dirtyDataBinds queue"
    );
    assert_eq!(file.data_bind_target_supports_push(10), Some(false));
    assert_eq!(file.data_bind_uses_persisting_list(10), Some(true));
    assert_eq!(
        file.data_bind_update_queue(10),
        Some(RuntimeDataBindUpdateQueue::Persisting)
    );
    assert_eq!(file.data_bind_target_supports_push(13), Some(false));
    assert_eq!(file.data_bind_uses_persisting_list(13), Some(true));
    assert_eq!(
        file.data_bind_initialize_effect(13, false, false, true, true, true),
        Some(RuntimeDataBindInitializeEffect {
            target_is_component: false,
            adds_component_collapsable: false,
            runs_collapse: false,
            collapse_effect: None,
        }),
        "C++ DataBind::initialize only registers Component targets as collapsables"
    );
    assert_eq!(
        file.data_bind_update_queue(13),
        Some(RuntimeDataBindUpdateQueue::Persisting)
    );
    assert_eq!(file.data_bind_target_supports_push(15), Some(true));
    assert_eq!(file.data_bind_uses_persisting_list(15), Some(false));
    assert_eq!(
        file.data_bind_update_queue(15),
        Some(RuntimeDataBindUpdateQueue::DirtyToSource)
    );
    assert_eq!(file.data_bind_target_supports_push(16), Some(true));
    assert_eq!(file.data_bind_uses_persisting_list(16), Some(false));
    assert_eq!(file.data_bind_to_source(16), Some(true));
    assert_eq!(file.data_bind_to_target(16), Some(true));
    assert_eq!(file.data_bind_is_main_to_source(16), Some(false));
    assert_eq!(file.data_bind_binds_once(16), Some(false));
    assert_eq!(file.data_bind_source_to_target_runs_first(16), Some(false));
    assert_eq!(file.data_bind_is_name_based(16), Some(false));
    assert_eq!(
        file.data_bind_update_effect(16, RuntimeComponentDirt::BINDINGS, true, true, true, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: true,
            source_to_target_is_main_direction: true,
            suppresses_dirt_while_applying_source_to_target: true,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "source-originated TwoWay dirt applies only source-to-target"
    );
    assert_eq!(
        file.data_bind_update_queue(16),
        Some(RuntimeDataBindUpdateQueue::DirtyToSource),
        "twoWay binds are toSource for C++ dirty queue ordering"
    );
    assert_eq!(file.data_bind_target_supports_push(17), Some(true));
    assert_eq!(file.data_bind_uses_persisting_list(17), Some(false));
    assert_eq!(file.data_bind_to_source(17), Some(true));
    assert_eq!(file.data_bind_to_target(17), Some(false));
    assert_eq!(file.data_bind_is_main_to_source(17), Some(true));
    assert_eq!(file.data_bind_binds_once(17), Some(true));
    assert_eq!(file.data_bind_source_to_target_runs_first(17), Some(true));
    assert_eq!(file.data_bind_is_name_based(17), Some(true));
    assert_eq!(
        file.data_bind_update_effect(17, RuntimeComponentDirt::BINDINGS, true, true, true, true),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: false,
            updates_converter_dependents: false,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: false,
            source_to_target_is_main_direction: false,
            suppresses_dirt_while_applying_source_to_target: false,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "source-originated dirt does not run target-to-source even when that direction is ordered last"
    );
    assert_eq!(
        file.data_bind_update_queue(17),
        Some(RuntimeDataBindUpdateQueue::DirtyToSource)
    );
    assert_eq!(
        file.data_bind_source_effect(17, RuntimeDataType::String),
        Some(RuntimeDataBindSourceEffect {
            adds_source_dependent: false,
            sets_source: true,
            updates_artboard_component_list_reset: false,
            artboard_component_list_should_reset_instances: false,
        }),
        "C++ DataBind::source skips addDependent for once-only data binds"
    );
    assert_eq!(
        file.data_bind_clear_source_effect(17, true),
        Some(RuntimeDataBindClearSourceEffect {
            removes_source_dependent: false,
            clears_source: true,
        }),
        "C++ DataBind::clearSource skips removeDependent for once-only data binds"
    );
    assert_eq!(
        file.data_bind_unbind_effect(17, true, true, true, true),
        Some(RuntimeDataBindUnbindEffect {
            clear_source_effect: RuntimeDataBindClearSourceEffect {
                removes_source_dependent: false,
                clears_source: true,
            },
            removes_target_observer: true,
            observing_after: false,
            unbinds_converter: false,
            clears_context_value: true,
        }),
        "C++ DataBind::unbind delegates once-only source handling to clearSource but still removes target observers"
    );
    assert_eq!(
        file.data_bind_can_skip(19, true),
        Some(false),
        "C++ keeps LayoutComponentStyle.displayValue binds live even for collapsed component targets"
    );
    assert_eq!(
        file.data_bind_collapse_effect(19, false, true, true, true),
        Some(RuntimeDataBindCollapseEffect {
            is_collapsed: false,
            changed: false,
            requests_dirty_update: false,
        }),
        "C++ DataBind::collapse ignores LayoutComponentStyle.displayValue binds"
    );
    assert_eq!(
        file.data_bind_initialize_effect(19, false, false, true, true, true),
        Some(RuntimeDataBindInitializeEffect {
            target_is_component: true,
            adds_component_collapsable: true,
            runs_collapse: true,
            collapse_effect: Some(RuntimeDataBindCollapseEffect {
                is_collapsed: false,
                changed: false,
                requests_dirty_update: false,
            }),
        }),
        "C++ DataBind::initialize still registers displayValue binds but DataBind::collapse keeps them uncollapsed"
    );
    assert_eq!(
        file.data_bind_add_effect(21, false, true),
        Some(RuntimeDataBindAddEffect {
            queues_pending_addition: false,
            appends_to_data_binds: true,
            appends_to_persisting_list: false,
            sets_persisting_list_flag: false,
            sets_container: true,
            binds_from_data_context: true,
            runs_initial_update: true,
            initial_update_applies_target_to_source: true,
        }),
        "C++ DataBindContainer::addDataBind binds an existing data context and runs updateDataBind(dataBind, true) for DataBindContext objects"
    );
    assert_eq!(
        file.data_bind_add_effect(21, true, true),
        Some(RuntimeDataBindAddEffect {
            queues_pending_addition: true,
            appends_to_data_binds: false,
            appends_to_persisting_list: false,
            sets_persisting_list_flag: false,
            sets_container: false,
            binds_from_data_context: false,
            runs_initial_update: false,
            initial_update_applies_target_to_source: false,
        }),
        "C++ DataBindContainer::addDataBind defers DataBindContext binding too when processing"
    );
    assert_eq!(
        file.data_bind_relink_effect(21, true, true),
        Some(RuntimeDataBindRelinkEffect {
            calls_container_rebuild: true,
            rebuild_binds_from_context: true,
            rebuild_has_data_context: true,
        }),
        "C++ Artboard/StateMachine rebuildDataBind rebinds DataBindContext children from the current data context"
    );
    assert_eq!(
        file.data_bind_relink_effect(21, true, false),
        Some(RuntimeDataBindRelinkEffect {
            calls_container_rebuild: true,
            rebuild_binds_from_context: true,
            rebuild_has_data_context: false,
        }),
        "C++ rebuildDataBind still calls DataBindContext::bindFromContext with a null data context pointer"
    );
    assert_eq!(
        file.data_bind_relink_effect(21, false, true),
        Some(RuntimeDataBindRelinkEffect {
            calls_container_rebuild: false,
            rebuild_binds_from_context: false,
            rebuild_has_data_context: false,
        }),
        "Rust keeps DataBind::relinkDataBind's required live container explicit instead of hiding the C++ null-deref hazard"
    );
    assert_eq!(
        file.data_bind_context_bind_effect(
            21,
            false,
            false,
            true,
            false,
            false,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindContextBindEffect {
            branch: RuntimeDataBindContextBindBranch::NoDataContext,
            resolves_path: false,
            marks_path_resolved: false,
            updates_source_path_ids: false,
            uses_relative_view_model_property_lookup: false,
            uses_view_model_property_lookup: false,
            clear_source_effect: None,
            source_effect: None,
            bind_effect: None,
            unbind_effect: None,
            add_dirt_effect: None,
            binds_converter_from_context: false,
        }),
        "C++ DataBindContext::bindFromContext is a no-op when dataContext is null"
    );
    assert_eq!(
        file.data_bind_context_bind_effect(
            21,
            false,
            true,
            true,
            false,
            false,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindContextBindEffect {
            branch: RuntimeDataBindContextBindBranch::BindSource,
            resolves_path: false,
            marks_path_resolved: false,
            updates_source_path_ids: false,
            uses_relative_view_model_property_lookup: false,
            uses_view_model_property_lookup: true,
            clear_source_effect: Some(RuntimeDataBindClearSourceEffect {
                removes_source_dependent: false,
                clears_source: false,
            }),
            source_effect: Some(RuntimeDataBindSourceEffect {
                adds_source_dependent: true,
                sets_source: true,
                updates_artboard_component_list_reset: false,
                artboard_component_list_should_reset_instances: false,
            }),
            bind_effect: Some(RuntimeDataBindBindEffect {
                clears_existing_context_value: false,
                context_value_type: Some(RuntimeDataType::Number),
                resets_converter: false,
                removes_existing_target_observer: false,
                adds_target_observer: false,
                observing_after: false,
                add_dirt_effect: RuntimeDataBindAddDirtEffect {
                    dirt: RuntimeComponentDirt::BINDINGS,
                    target_origin: false,
                    changed: true,
                    invalidates_context_value: false,
                    requests_dirty_update: true,
                },
            }),
            unbind_effect: None,
            add_dirt_effect: None,
            binds_converter_from_context: false,
        }),
        "C++ DataBindContext::bindFromContext clears the old source, stores the lookup result, and calls bind() when the source changes"
    );
    assert_eq!(
        file.data_bind_context_bind_effect(
            21,
            false,
            true,
            true,
            true,
            true,
            RuntimeDataType::Number,
            true,
            false,
            true,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindContextBindEffect {
            branch: RuntimeDataBindContextBindBranch::AddDirtExistingSource,
            resolves_path: false,
            marks_path_resolved: false,
            updates_source_path_ids: false,
            uses_relative_view_model_property_lookup: false,
            uses_view_model_property_lookup: true,
            clear_source_effect: None,
            source_effect: None,
            bind_effect: None,
            unbind_effect: None,
            add_dirt_effect: Some(RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS,
                target_origin: false,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            }),
            binds_converter_from_context: false,
        }),
        "C++ DataBindContext::bindFromContext marks Bindings dirt when the looked-up source already matches"
    );
    assert_eq!(
        file.data_bind_context_bind_effect(
            21,
            false,
            true,
            false,
            false,
            true,
            RuntimeDataType::Number,
            true,
            true,
            true,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindContextBindEffect {
            branch: RuntimeDataBindContextBindBranch::UnbindMissingSource,
            resolves_path: false,
            marks_path_resolved: false,
            updates_source_path_ids: false,
            uses_relative_view_model_property_lookup: false,
            uses_view_model_property_lookup: true,
            clear_source_effect: None,
            source_effect: None,
            bind_effect: None,
            unbind_effect: Some(RuntimeDataBindUnbindEffect {
                clear_source_effect: RuntimeDataBindClearSourceEffect {
                    removes_source_dependent: true,
                    clears_source: true,
                },
                removes_target_observer: true,
                observing_after: false,
                unbinds_converter: false,
                clears_context_value: true,
            }),
            add_dirt_effect: None,
            binds_converter_from_context: false,
        }),
        "C++ DataBindContext::bindFromContext unbinds when the data context lookup misses"
    );
    assert_eq!(
        file.data_bind_container_bind_context_effect(&[3, 5, 21], true),
        Some(RuntimeDataBindContainerBindContextEffect {
            bind_from_context_data_bind_ids: vec![21],
            stores_data_context: true,
            clears_data_context: false,
        }),
        "C++ DataBindContainer::bindDataBindsFromContext calls bindFromContext for DataBindContext children in container order and stores the context pointer"
    );
    assert_eq!(
        file.data_bind_container_bind_context_effect(&[21, 3, 5], false),
        Some(RuntimeDataBindContainerBindContextEffect {
            bind_from_context_data_bind_ids: vec![21],
            stores_data_context: false,
            clears_data_context: true,
        }),
        "C++ DataBindContainer::bindDataBindsFromContext still visits DataBindContext children before assigning a null context"
    );
    assert_eq!(
        file.data_bind_container_unbind_effect(&[3, 5, 21]),
        Some(RuntimeDataBindContainerUnbindEffect {
            unbinds_data_bind_ids: vec![3, 5, 21],
            clears_data_context: true,
        }),
        "C++ DataBindContainer::unbindDataBinds calls unbind on every data bind in container order and clears the context"
    );
    assert_eq!(
        file.data_bind_container_advance_effect(&[], &[]),
        Some(RuntimeDataBindContainerAdvanceEffect {
            advances_data_bind_ids: Vec::new(),
            did_update: false,
        }),
        "C++ DataBindContainer::advanceDataBinds exits false when the container has no data binds"
    );
    assert_eq!(
        file.data_bind_container_advance_effect(&[3, 5, 21], &[false, true, false]),
        Some(RuntimeDataBindContainerAdvanceEffect {
            advances_data_bind_ids: vec![3, 5, 21],
            did_update: true,
        }),
        "C++ DataBindContainer::advanceDataBinds visits every data bind and returns true if any child advances"
    );
    assert_eq!(
        file.data_bind_container_advance_effect(&[3, 5, 21], &[false, false, false]),
        Some(RuntimeDataBindContainerAdvanceEffect {
            advances_data_bind_ids: vec![3, 5, 21],
            did_update: false,
        }),
        "C++ DataBindContainer::advanceDataBinds returns false when no child advances"
    );
    assert_eq!(
        file.data_bind_container_advance_effect(&[3, 5, 21], &[false, true]),
        None,
        "Rust requires one supplied advance result per validated data bind"
    );
    assert_eq!(
        file.data_bind_container_update_effect(
            &[3],
            &[false],
            &[16],
            &[8],
            &[21],
            &[19],
            &[10],
            &[5],
            true,
            true
        ),
        Some(RuntimeDataBindContainerUpdateEffect {
            return_reason: Some(RuntimeDataBindContainerUpdateReturnReason::AlreadyProcessing),
            enters_processing: false,
            update_steps: Vec::new(),
            skipped_persisting_data_bind_ids: Vec::new(),
            clears_dirty_to_source_queue: false,
            clears_dirty_to_target_queue: false,
            clears_dirty_list_flag_data_bind_ids: Vec::new(),
            next_dirty_to_source_data_bind_ids: Vec::new(),
            next_dirty_to_target_data_bind_ids: Vec::new(),
            flushes_pending_addition_ids: Vec::new(),
            flushes_pending_removal_ids: Vec::new(),
            exits_processing: false,
        }),
        "C++ DataBindContainer::updateDataBinds returns immediately when already processing"
    );
    assert_eq!(
        file.data_bind_container_update_effect(
            &[],
            &[],
            &[],
            &[],
            &[21],
            &[19],
            &[10],
            &[5],
            false,
            true
        ),
        Some(RuntimeDataBindContainerUpdateEffect {
            return_reason: Some(RuntimeDataBindContainerUpdateReturnReason::NoActiveQueues),
            enters_processing: false,
            update_steps: Vec::new(),
            skipped_persisting_data_bind_ids: Vec::new(),
            clears_dirty_to_source_queue: false,
            clears_dirty_to_target_queue: false,
            clears_dirty_list_flag_data_bind_ids: Vec::new(),
            next_dirty_to_source_data_bind_ids: Vec::new(),
            next_dirty_to_target_data_bind_ids: Vec::new(),
            flushes_pending_addition_ids: Vec::new(),
            flushes_pending_removal_ids: Vec::new(),
            exits_processing: false,
        }),
        "C++ DataBindContainer::updateDataBinds ignores pending queues when there are no active queues to process"
    );
    assert_eq!(
        file.data_bind_container_update_effect(
            &[3, 5, 7],
            &[false, true, false],
            &[16, 17],
            &[8],
            &[21],
            &[19],
            &[10],
            &[5],
            false,
            false
        ),
        Some(RuntimeDataBindContainerUpdateEffect {
            return_reason: None,
            enters_processing: true,
            update_steps: vec![
                RuntimeDataBindContainerUpdateStep {
                    data_bind_id: 3,
                    queue: RuntimeDataBindUpdateQueue::Persisting,
                    apply_target_to_source: false,
                    clears_dirty_list_flag: false,
                },
                RuntimeDataBindContainerUpdateStep {
                    data_bind_id: 7,
                    queue: RuntimeDataBindUpdateQueue::Persisting,
                    apply_target_to_source: false,
                    clears_dirty_list_flag: false,
                },
                RuntimeDataBindContainerUpdateStep {
                    data_bind_id: 16,
                    queue: RuntimeDataBindUpdateQueue::DirtyToSource,
                    apply_target_to_source: false,
                    clears_dirty_list_flag: true,
                },
                RuntimeDataBindContainerUpdateStep {
                    data_bind_id: 17,
                    queue: RuntimeDataBindUpdateQueue::DirtyToSource,
                    apply_target_to_source: false,
                    clears_dirty_list_flag: true,
                },
                RuntimeDataBindContainerUpdateStep {
                    data_bind_id: 8,
                    queue: RuntimeDataBindUpdateQueue::DirtyToTarget,
                    apply_target_to_source: false,
                    clears_dirty_list_flag: true,
                },
            ],
            skipped_persisting_data_bind_ids: vec![5],
            clears_dirty_to_source_queue: true,
            clears_dirty_to_target_queue: true,
            clears_dirty_list_flag_data_bind_ids: vec![16, 17, 8],
            next_dirty_to_source_data_bind_ids: vec![21],
            next_dirty_to_target_data_bind_ids: vec![19],
            flushes_pending_addition_ids: vec![10],
            flushes_pending_removal_ids: vec![5],
            exits_processing: true,
        }),
        "C++ DataBindContainer::updateDataBinds drains persisting, dirty-to-source, then dirty-to-target queues before swapping pending dirty queues and flushing additions before removals"
    );
    assert_eq!(
        file.data_bind_source_effect(23, RuntimeDataType::Number),
        Some(RuntimeDataBindSourceEffect {
            adds_source_dependent: true,
            sets_source: true,
            updates_artboard_component_list_reset: true,
            artboard_component_list_should_reset_instances: true,
        }),
        "C++ DataBind::source asks ArtboardComponentList targets to reset instances for number sources"
    );
    assert_eq!(
        file.data_bind_source_effect(23, RuntimeDataType::String),
        Some(RuntimeDataBindSourceEffect {
            adds_source_dependent: true,
            sets_source: true,
            updates_artboard_component_list_reset: true,
            artboard_component_list_should_reset_instances: false,
        }),
        "C++ DataBind::source clears ArtboardComponentList reset for non-number sources"
    );
    assert_eq!(
        file.data_bind_output_type(25),
        Some(RuntimeDataType::String),
        "C++ DataBind::outputType prefers a non-input/non-none converter output type over the live source type"
    );
    assert_eq!(
        file.data_bind_bind_effect(
            25,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindBindEffect {
            clears_existing_context_value: false,
            context_value_type: Some(RuntimeDataType::String),
            resets_converter: true,
            removes_existing_target_observer: false,
            adds_target_observer: true,
            observing_after: true,
            add_dirt_effect: RuntimeDataBindAddDirtEffect {
                dirt: RuntimeComponentDirt::BINDINGS_TARGET,
                target_origin: true,
                changed: true,
                invalidates_context_value: false,
                requests_dirty_update: true,
            },
        }),
        "C++ DataBind::bind resets converters and creates the context value from converter outputType()"
    );
    assert_eq!(
        file.data_bind_unbind_effect(25, true, true, true, true),
        Some(RuntimeDataBindUnbindEffect {
            clear_source_effect: RuntimeDataBindClearSourceEffect {
                removes_source_dependent: true,
                clears_source: true,
            },
            removes_target_observer: true,
            observing_after: false,
            unbinds_converter: true,
            clears_context_value: true,
        }),
        "C++ DataBind::unbind delegates to the resolved converter's unbind() when present"
    );
    assert_eq!(file.data_bind_context_source_path_ids(26), Some(vec![42]));
    assert_eq!(
        file.data_bind_context_bind_effect(
            26,
            false,
            true,
            true,
            false,
            false,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindContextBindEffect {
            branch: RuntimeDataBindContextBindBranch::BindSource,
            resolves_path: true,
            marks_path_resolved: true,
            updates_source_path_ids: false,
            uses_relative_view_model_property_lookup: true,
            uses_view_model_property_lookup: false,
            clear_source_effect: Some(RuntimeDataBindClearSourceEffect {
                removes_source_dependent: false,
                clears_source: false,
            }),
            source_effect: Some(RuntimeDataBindSourceEffect {
                adds_source_dependent: true,
                sets_source: true,
                updates_artboard_component_list_reset: false,
                artboard_component_list_should_reset_instances: false,
            }),
            bind_effect: Some(RuntimeDataBindBindEffect {
                clears_existing_context_value: false,
                context_value_type: Some(RuntimeDataType::String),
                resets_converter: true,
                removes_existing_target_observer: false,
                adds_target_observer: false,
                observing_after: false,
                add_dirt_effect: RuntimeDataBindAddDirtEffect {
                    dirt: RuntimeComponentDirt::BINDINGS,
                    target_origin: false,
                    changed: true,
                    invalidates_context_value: false,
                    requests_dirty_update: true,
                },
            }),
            unbind_effect: None,
            add_dirt_effect: None,
            binds_converter_from_context: true,
        }),
        "C++ name-based DataBindContext::bindFromContext resolves its path, uses relative lookup, binds the new source, resets the converter in bind(), then binds the converter from context"
    );
    assert_eq!(
        file.data_bind_context_bind_effect(
            26,
            true,
            true,
            true,
            false,
            false,
            RuntimeDataType::Number,
            true,
            false,
            false,
            RuntimeComponentDirt::NONE,
            false,
            true
        ),
        Some(RuntimeDataBindContextBindEffect {
            branch: RuntimeDataBindContextBindBranch::BindSource,
            resolves_path: false,
            marks_path_resolved: false,
            updates_source_path_ids: false,
            uses_relative_view_model_property_lookup: true,
            uses_view_model_property_lookup: false,
            clear_source_effect: Some(RuntimeDataBindClearSourceEffect {
                removes_source_dependent: false,
                clears_source: false,
            }),
            source_effect: Some(RuntimeDataBindSourceEffect {
                adds_source_dependent: true,
                sets_source: true,
                updates_artboard_component_list_reset: false,
                artboard_component_list_should_reset_instances: false,
            }),
            bind_effect: Some(RuntimeDataBindBindEffect {
                clears_existing_context_value: false,
                context_value_type: Some(RuntimeDataType::String),
                resets_converter: true,
                removes_existing_target_observer: false,
                adds_target_observer: false,
                observing_after: false,
                add_dirt_effect: RuntimeDataBindAddDirtEffect {
                    dirt: RuntimeComponentDirt::BINDINGS,
                    target_origin: false,
                    changed: true,
                    invalidates_context_value: false,
                    requests_dirty_update: true,
                },
            }),
            unbind_effect: None,
            add_dirt_effect: None,
            binds_converter_from_context: true,
        }),
        "C++ DataBindContext::resolvePath is skipped once m_isPathResolved is already true"
    );

    assert_eq!(
        file.artboard_data_binds(0)
            .iter()
            .map(|data_bind| data_bind.object.id)
            .collect::<Vec<_>>(),
        vec![3, 5, 7, 8, 10, 19, 21, 23]
    );
    assert_eq!(file.sorted_data_bind_ids(&[2, 3]), None);
    assert_eq!(
        file.sorted_data_bind_ids(&[3, 5, 7, 8, 10, 19]),
        Some(vec![3, 5, 7, 10, 19, 8]),
        "C++ DataBindContainer::sortDataBinds partitions toSource binds to the front"
    );
    assert_eq!(
        file.sorted_data_bind_ids(&[8, 3, 19, 5, 7, 10]),
        Some(vec![3, 19, 5, 7, 10, 8]),
        "C++ sortDataBinds uses swaps, preserving toSource order while moving displaced toTarget binds to the tail"
    );
    assert_eq!(
        file.artboard_state_machine_graph(0, 0)
            .map(|state_machine| {
                state_machine
                    .data_binds
                    .iter()
                    .map(|data_bind| data_bind.id)
                    .collect::<Vec<_>>()
            }),
        Some(vec![13, 15, 16, 17])
    );
}

#[test]
fn runtime_data_bind_reconcile_dirt_covers_each_supported_direction() {
    let node_x_key = u64::from(property_key_for_name("Node", "x"));
    let file = read_runtime_file(&synthetic_runtime_file(4342, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        for flags in [2, 2 | 8, 0, 1] {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "DataBind", |bytes| {
                push_uint_property(bytes, "DataBind", "propertyKey", node_x_key);
                push_uint_property(bytes, "DataBind", "flags", flags);
            });
        }
    }))
    .expect("synthetic directional data-bind stream imports");

    assert_eq!(
        file.data_bind_reconcile_dirt(3),
        Some(RuntimeComponentDirt::BINDINGS | RuntimeComponentDirt::BINDINGS_TARGET)
    );
    assert_eq!(
        file.data_bind_reconcile_dirt(5),
        Some(RuntimeComponentDirt::BINDINGS | RuntimeComponentDirt::BINDINGS_TARGET),
        "precedence changes order, not the directions participating in reconcile"
    );
    assert_eq!(
        file.data_bind_reconcile_dirt(7),
        Some(RuntimeComponentDirt::BINDINGS)
    );
    assert_eq!(
        file.data_bind_reconcile_dirt(9),
        Some(RuntimeComponentDirt::BINDINGS_TARGET)
    );

    let source_origin = file
        .data_bind_add_dirt_effect_with_origin(
            3,
            RuntimeComponentDirt::NONE,
            true,
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            true,
            true,
        )
        .expect("source-origin dirt effect");
    assert!(!source_origin.target_origin);

    let target_origin = file
        .data_bind_add_dirt_effect_with_origin(
            3,
            RuntimeComponentDirt::NONE,
            false,
            RuntimeComponentDirt::BINDINGS_TARGET,
            false,
            false,
            true,
            true,
        )
        .expect("target-origin dirt effect");
    assert!(target_origin.target_origin);

    let suppressed = file
        .data_bind_add_dirt_effect_with_origin(
            3,
            RuntimeComponentDirt::NONE,
            true,
            RuntimeComponentDirt::BINDINGS,
            true,
            false,
            true,
            true,
        )
        .expect("suppressed dirt effect");
    assert!(suppressed.target_origin);
    assert!(!suppressed.changed);

    let already_marked = file
        .data_bind_add_dirt_effect_with_origin(
            3,
            RuntimeComponentDirt::BINDINGS,
            true,
            RuntimeComponentDirt::BINDINGS,
            false,
            false,
            true,
            true,
        )
        .expect("already-marked dirt effect");
    assert!(already_marked.target_origin);
    assert!(!already_marked.changed);

    let target_first_reconcile = file
        .data_bind_add_dirt_effect_with_origin(
            3,
            RuntimeComponentDirt::NONE,
            false,
            file.data_bind_reconcile_dirt(3)
                .expect("target-first reconcile dirt"),
            false,
            false,
            true,
            true,
        )
        .expect("target-first reconcile effect");
    assert!(target_first_reconcile.target_origin);

    let source_first_reconcile = file
        .data_bind_add_dirt_effect_with_origin(
            5,
            RuntimeComponentDirt::NONE,
            true,
            file.data_bind_reconcile_dirt(5)
                .expect("source-first reconcile dirt"),
            false,
            false,
            true,
            true,
        )
        .expect("source-first reconcile effect");
    assert!(!source_first_reconcile.target_origin);

    let target_first_update = file
        .data_bind_update_effect_with_persisting_state(
            3,
            RuntimeComponentDirt::BINDINGS | RuntimeComponentDirt::BINDINGS_TARGET,
            true,
            false,
            true,
            true,
            true,
        )
        .expect("target-first reconcile update");
    assert!(target_first_update.applies_target_to_source_before_update);
    assert!(target_first_update.applies_source_to_target);
    assert!(!target_first_update.applies_target_to_source_after_update);

    let source_first_update = file
        .data_bind_update_effect_with_persisting_state(
            5,
            RuntimeComponentDirt::BINDINGS | RuntimeComponentDirt::BINDINGS_TARGET,
            true,
            false,
            true,
            true,
            true,
        )
        .expect("source-first reconcile update");
    assert!(!source_first_update.applies_target_to_source_before_update);
    assert!(source_first_update.applies_source_to_target);
    assert!(source_first_update.applies_target_to_source_after_update);

    let source_only_update = file
        .data_bind_update_effect_with_persisting_state(
            3,
            RuntimeComponentDirt::BINDINGS,
            true,
            false,
            true,
            true,
            true,
        )
        .expect("source-only update");
    assert!(!source_only_update.applies_target_to_source_before_update);
    assert!(source_only_update.applies_source_to_target);

    let polled_update = file
        .data_bind_update_effect_with_persisting_state(
            9,
            RuntimeComponentDirt::NONE,
            true,
            true,
            true,
            true,
            true,
        )
        .expect("persisting target-to-source update");
    assert!(polled_update.applies_target_to_source_before_update);
}

#[test]
fn runtime_data_bind_path_referencers_follow_cpp_claiming() {
    fn encoded_ids(ids: &[u64]) -> Vec<u8> {
        let mut encoded = Vec::new();
        for id in ids {
            push_var_uint(&mut encoded, *id);
        }
        encoded
    }

    let file = read_runtime_file(&synthetic_runtime_file(4361, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_ids(&[1]));
        });
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_ids(&[2]));
        });
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "StateMachineListenerSingle", |bytes| {
            push_bytes_property(
                bytes,
                "StateMachineListenerSingle",
                "viewModelPathIds",
                &encoded_ids(&[3]),
            );
        });
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_ids(&[4]));
        });
        push_object_with_properties(bytes, "ListenerInputTypeViewModel", |bytes| {
            push_bytes_property(
                bytes,
                "ListenerInputTypeViewModel",
                "viewModelPathIds",
                &encoded_ids(&[5]),
            );
        });
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_ids(&[6]));
        });
        push_object_with_properties(bytes, "NestedArtboard", |bytes| {
            push_uint_property(bytes, "NestedArtboard", "artboardId", 0);
            push_bytes_property(
                bytes,
                "NestedArtboard",
                "dataBindPathIds",
                &encoded_ids(&[7]),
            );
        });
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_ids(&[8]));
        });
        push_empty_object(bytes, "ScriptedDrawable");
        push_object_with_properties(bytes, "ScriptInputViewModelProperty", |bytes| {
            push_bytes_property(
                bytes,
                "ScriptInputViewModelProperty",
                "dataBindPathIds",
                &encoded_ids(&[9]),
            );
        });
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_ids(&[10]));
        });
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
        push_object_with_properties(bytes, "StateMachineFireTrigger", |bytes| {
            push_bytes_property(
                bytes,
                "StateMachineFireTrigger",
                "viewModelPathIds",
                &encoded_ids(&[11]),
            );
        });
    }))
    .expect("synthetic data-bind path referencer stream imports");

    let first_claim = file
        .data_bind_path_for_referencer(5)
        .expect("first listener claims latest DataBindPath");
    assert_eq!(first_claim.object.map(|object| object.id), Some(2));
    assert_eq!(first_claim.property_name, "path");
    assert_eq!(first_claim.path_ids, vec![2]);

    assert!(
        file.data_bind_path_for_referencer(6).is_none(),
        "the claimed DataBindPath is single-use"
    );

    let inline_listener = file
        .data_bind_path_for_referencer(7)
        .expect("listener falls back to inline viewModelPathIds");
    assert!(inline_listener.object.is_none());
    assert_eq!(inline_listener.property_name, "viewModelPathIds");
    assert_eq!(inline_listener.path_ids, vec![3]);

    let input_type = file
        .data_bind_path_for_referencer(9)
        .expect("ListenerInputTypeViewModel uses inline bytes");
    assert!(input_type.object.is_none());
    assert_eq!(input_type.property_name, "viewModelPathIds");
    assert_eq!(input_type.path_ids, vec![5]);

    let second_claim = file
        .data_bind_path_for_referencer(10)
        .expect("listener claims path left pending by non-claiming input type");
    assert_eq!(second_claim.object.map(|object| object.id), Some(8));
    assert_eq!(second_claim.path_ids, vec![4]);

    let nested_claim = file
        .data_bind_path_for_referencer(12)
        .expect("NestedArtboard claims latest DataBindPath");
    assert_eq!(nested_claim.object.map(|object| object.id), Some(11));
    assert_eq!(
        nested_claim.path_ids,
        vec![6],
        "claimed path takes precedence over inline dataBindPathIds like C++ importDataBindPath"
    );

    let script_input_claim = file
        .data_bind_path_for_referencer(15)
        .expect("ScriptInputViewModelProperty claims latest DataBindPath");
    assert_eq!(script_input_claim.object.map(|object| object.id), Some(13));
    assert_eq!(script_input_claim.path_ids, vec![8]);

    let fire_trigger_claim = file
        .data_bind_path_for_referencer(21)
        .expect("StateMachineFireTrigger claims latest DataBindPath");
    assert_eq!(fire_trigger_claim.object.map(|object| object.id), Some(16));
    assert_eq!(fire_trigger_claim.path_ids, vec![10]);

    assert!(
        file.data_bind_path_for_referencer(1).is_none(),
        "DataBindPath objects are not themselves referencers"
    );
}

#[test]
fn runtime_data_bind_path_dropped_claimers_still_consume_latest_path() {
    let mut encoded_path = Vec::new();
    push_var_uint(&mut encoded_path, 42);

    let file = read_runtime_file(&synthetic_runtime_file(4362, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_path);
        });
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineListenerSingle");
    }))
    .expect("synthetic dropped data-bind path claimer stream imports");

    assert_eq!(
        file.import_status(2),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );
    assert!(
        file.data_bind_path_for_referencer(5).is_none(),
        "C++ importDataBindPath runs before the contextless listener fails, so the later valid listener cannot claim the path"
    );
}

#[test]
fn runtime_data_enums_match_cpp_file_enums_collection() {
    let raw_custom_name = [b'C', b'u', 0xff, b't'];
    let file = read_runtime_file(&synthetic_runtime_file(4334, |bytes| {
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "dropped");
        });
        push_empty_object(bytes, "DataEnumSystem");
        push_object_with_properties(bytes, "DataEnumCustom", |bytes| {
            push_bytes_property(bytes, "DataEnumCustom", "name", &raw_custom_name);
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "a");
            push_string_property(bytes, "DataEnumValue", "value", "A");
        });
        push_empty_object(bytes, "DataEnum");
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "b");
        });
        push_empty_object(bytes, "DataEnumSystem");
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "c");
            push_string_property(bytes, "DataEnumValue", "value", "C");
        });
        push_empty_object(bytes, "DataEnumCustom");
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "next");
        });
    }))
    .expect("synthetic data enum collection imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );

    let enums = file.data_enums();
    assert_eq!(
        enums
            .iter()
            .map(|item| item.object.type_name)
            .collect::<Vec<_>>(),
        vec!["DataEnumCustom", "DataEnum", "DataEnumCustom"],
        "C++ File::enums contains exact DataEnum/DataEnumCustom objects, not DataEnumSystem"
    );
    assert_eq!(
        enums[0].object.string_property_bytes("name"),
        Some(raw_custom_name.as_slice())
    );
    assert_eq!(
        enums[0]
            .values
            .iter()
            .map(|value| value.string_property("key").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["a", "b", "c"],
        "DataEnumValue objects attach to the latest DataEnumCustom importer, even across DataEnum/DataEnumSystem records"
    );
    assert_eq!(enums[1].values.len(), 0);
    assert_eq!(
        enums[2]
            .values
            .iter()
            .map(|value| value.string_property("key").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["next"]
    );

    assert_eq!(
        file.data_enum(0)
            .and_then(|item| item.object.string_property_bytes("name")),
        Some(raw_custom_name.as_slice())
    );
    assert_eq!(
        file.data_enum(1).map(|item| item.object.type_name),
        Some("DataEnum")
    );
    assert!(file.data_enum(3).is_none());
}

#[test]
fn runtime_view_models_match_cpp_file_view_model_collection() {
    let raw_second_name = [b'S', b'e', 0xff, b'o', b'n', b'd'];
    let file = read_runtime_file(&synthetic_runtime_file(4335, |bytes| {
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "dropped");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "First");
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "no-backboard");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 0);
            push_uint_property(bytes, "ViewModelInstanceBoolean", "propertyValue", 1);
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "first-instance");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "future-instance");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_bytes_property(bytes, "ViewModel", "name", &raw_second_name);
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "second-instance");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "missing-view-model");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 99);
        });
    }))
    .expect("synthetic view model collection imports");

    assert_eq!(
        file.import_status(0),
        Some(RuntimeImportStatus::Dropped {
            reason: RuntimeImportDropReason::MissingObject
        })
    );
    assert_eq!(
        file.import_status(3),
        Some(RuntimeImportStatus::Imported),
        "publisher-era files attach instances to the latest ViewModel before a Backboard exists"
    );
    assert_eq!(
        file.import_status(4),
        Some(RuntimeImportStatus::Imported),
        "values following a pre-Backboard instance import with that instance"
    );

    let view_models = file.view_models();
    assert_eq!(view_models.len(), 2);
    assert_eq!(view_models[0].object.string_property("name"), Some("First"));
    assert_eq!(
        view_models[0]
            .properties
            .iter()
            .map(|property| property.string_property("name").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["enabled"]
    );
    assert_eq!(
        view_models[0]
            .instances
            .iter()
            .map(|instance| { instance.object.string_property("name").unwrap_or_default() })
            .collect::<Vec<_>>(),
        vec!["no-backboard", "first-instance"],
        "pre-Backboard instances attach to the latest ViewModel while later instances resolve by viewModelId"
    );
    assert_eq!(view_models[0].instances[0].values.len(), 1);
    assert_eq!(
        view_models[0].instances[0].values[0].object.type_name,
        "ViewModelInstanceBoolean"
    );
    let publisher_value = file
        .data_context_view_model_property_for_instance(view_models[0].instances[0].object, &[0, 0])
        .expect("publisher-era instance value resolves through the data-context path");
    assert_eq!(
        file.view_model_instance_boolean_value_for_object(publisher_value),
        Some(true)
    );

    assert_eq!(
        view_models[1].object.string_property_bytes("name"),
        Some(raw_second_name.as_slice())
    );
    assert_eq!(
        view_models[1]
            .properties
            .iter()
            .map(|property| property.string_property("name").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["amount"]
    );
    assert_eq!(
        view_models[1]
            .instances
            .iter()
            .map(|instance| { instance.object.string_property("name").unwrap_or_default() })
            .collect::<Vec<_>>(),
        vec!["second-instance"]
    );

    assert_eq!(
        file.view_model(0)
            .and_then(|item| item.object.string_property("name")),
        Some("First")
    );
    assert_eq!(
        file.view_model_named("First")
            .map(|item| item.object.type_name),
        Some("ViewModel")
    );
    assert!(file.view_model_named("Second").is_none());
    assert_eq!(
        file.view_model_named_bytes(&raw_second_name)
            .and_then(|item| item.object.string_property_bytes("name")),
        Some(raw_second_name.as_slice())
    );
    assert!(file.view_model(2).is_none());
}

#[test]
fn runtime_view_model_instances_group_cpp_values_and_list_items() {
    let file = read_runtime_file(&synthetic_runtime_file(4343, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Item");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceList", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceList", "viewModelPropertyId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceListItem", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelId", 0);
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "item");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 2);
        });
        push_object_with_properties(bytes, "ViewModelInstanceListItem", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelId", 0);
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "missing");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 99);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 3);
        });
    }))
    .expect("synthetic view-model instance values import");

    let view_models = file.view_models();
    assert_eq!(view_models.len(), 2);
    assert!(view_models[0].properties.is_empty());
    assert!(view_models[1].properties.is_empty());

    assert_eq!(view_models[0].instances.len(), 1);
    assert_eq!(
        view_models[0].instances[0].object.string_property("name"),
        Some("item")
    );
    assert_eq!(
        view_models[0].instances[0]
            .values
            .iter()
            .map(|value| value.object.type_name)
            .collect::<Vec<_>>(),
        vec!["ViewModelInstanceString"],
        "values after the item instance attach to the latest C++ ViewModelInstanceImporter"
    );

    assert_eq!(view_models[1].instances.len(), 1);
    let root_instance = &view_models[1].instances[0];
    assert_eq!(root_instance.object.string_property("name"), Some("root"));
    assert_eq!(
        root_instance
            .values
            .iter()
            .map(|value| value.object.type_name)
            .collect::<Vec<_>>(),
        vec!["ViewModelInstanceNumber", "ViewModelInstanceList"]
    );
    assert_eq!(
        root_instance.values[1]
            .list_items
            .iter()
            .map(|item| item.uint_property("viewModelInstanceId"))
            .collect::<Vec<_>>(),
        vec![Some(0), Some(1)],
        "C++ list items attach to the latest ViewModelInstanceListImporter, even after another instance importer becomes latest"
    );
    assert_eq!(
        root_instance.values[1]
            .list_items
            .iter()
            .map(|item| item.type_name)
            .collect::<Vec<_>>(),
        vec!["ViewModelInstanceListItem", "ViewModelInstanceListItem"]
    );
}

#[test]
fn runtime_view_model_instance_view_model_references_follow_cpp_import_resolution() {
    let file = read_runtime_file(&synthetic_runtime_file(4344, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Item");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
            push_string_property(bytes, "ViewModelPropertyViewModel", "name", "child");
            push_uint_property(
                bytes,
                "ViewModelPropertyViewModel",
                "viewModelReferenceId",
                0,
            );
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "item0");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "item1");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "pre-artboard-root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceViewModel", "propertyValue", 1);
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "post-artboard-root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceViewModel", "propertyValue", 1);
        });
    }))
    .expect("synthetic view-model reference stream imports");

    let view_models = file.view_models();
    assert_eq!(view_models.len(), 2);
    assert_eq!(
        view_models[0]
            .instances
            .iter()
            .map(|instance| instance.object.string_property("name").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["item0", "item1"]
    );
    assert_eq!(
        view_models[1]
            .instances
            .iter()
            .map(|instance| instance.object.string_property("name").unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["pre-artboard-root", "post-artboard-root"]
    );

    let pre_artboard_value = view_models[1].instances[0].values[0].object;
    assert!(
        file.referenced_view_model_instance_for_value_object(pre_artboard_value)
            .is_none(),
        "C++ only resolves ViewModelInstanceViewModel references once an ArtboardImporter is latest"
    );

    let post_artboard_value = view_models[1].instances[1].values[0].object;
    let reference = file
        .referenced_view_model_instance_for_value_object(post_artboard_value)
        .expect("post-artboard value resolves to the referenced view-model instance");
    assert_eq!(reference.view_model_index, 0);
    assert_eq!(reference.instance_index, 1);
    assert_eq!(reference.object.string_property("name"), Some("item1"));
    assert_eq!(
        file.referenced_view_model_instance_for_value(post_artboard_value.id as usize)
            .map(|reference| reference.object.id),
        Some(reference.object.id)
    );
}

#[test]
fn runtime_view_model_instance_values_resolve_linked_properties_like_cpp_completion() {
    let file = read_runtime_file(&synthetic_runtime_file(4345, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Item");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 99);
        });
    }))
    .expect("synthetic view-model property link stream imports");

    let view_models = file.view_models();
    assert_eq!(view_models[1].properties.len(), 2);
    let root_values = &view_models[1].instances[0].values;
    let amount_property = file
        .view_model_property_for_instance_value_object(root_values[0].object)
        .expect("number value resolves to amount property");
    assert_eq!(amount_property.type_name, "ViewModelPropertyNumber");
    assert_eq!(amount_property.string_property("name"), Some("amount"));
    assert_eq!(
        file.view_model_instance_value_name_for_object(root_values[0].object),
        Some("amount")
    );
    assert_eq!(
        file.view_model_property_for_instance_value(root_values[0].object.id as usize)
            .map(|property| property.id),
        Some(amount_property.id)
    );

    let label_property = file
        .view_model_property_for_instance_value_object(root_values[1].object)
        .expect("string value resolves to label property");
    assert_eq!(label_property.type_name, "ViewModelPropertyString");
    assert_eq!(label_property.string_property("name"), Some("label"));
    assert_eq!(
        file.view_model_instance_value_name(root_values[1].object.id as usize),
        Some("label")
    );

    assert!(
        file.view_model_property_for_instance_value_object(root_values[2].object)
            .is_none(),
        "out-of-range viewModelPropertyId leaves the value without a linked property"
    );
    assert!(
        file.view_model_instance_value_name_for_object(root_values[2].object)
            .is_none()
    );
    assert!(
        file.view_model_property_for_instance_value_object(view_models[1].object)
            .is_none(),
        "non-value objects are not treated as view-model instance values"
    );
}

#[test]
fn runtime_file_asset_referencers_resolve_like_cpp_backboard_importer() {
    let file = read_runtime_file(&synthetic_runtime_file(4330, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ImageAsset");
        push_empty_object(bytes, "FontAsset");
        push_empty_object(bytes, "AudioAsset");
        push_empty_object(bytes, "ScriptAsset");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
            push_uint_property(bytes, "Image", "assetId", 0);
        });
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
            push_uint_property(bytes, "Image", "assetId", 1);
        });
        push_object_with_properties(bytes, "AudioEvent", |bytes| {
            push_uint_property(bytes, "AudioEvent", "parentId", 0);
            push_uint_property(bytes, "AudioEvent", "assetId", 2);
        });
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStyle", |bytes| {
            push_uint_property(bytes, "TextStyle", "parentId", 4);
            push_uint_property(bytes, "TextStyle", "fontAssetId", 1);
        });
        push_object_with_properties(bytes, "ScriptedDrawable", |bytes| {
            push_uint_property(bytes, "ScriptedDrawable", "parentId", 0);
            push_uint_property(bytes, "ScriptedDrawable", "scriptAssetId", 3);
        });
    }))
    .expect("synthetic file asset referencers import");

    assert_eq!(
        file.resolved_file_asset_for_object(6)
            .map(|asset| asset.type_name),
        Some("ImageAsset")
    );
    assert!(
        file.resolved_file_asset_for_object(7).is_none(),
        "C++ Image::setAsset ignores non-ImageAsset entries"
    );
    assert_eq!(
        file.resolved_file_asset_for_object(8)
            .map(|asset| asset.type_name),
        Some("AudioAsset")
    );
    assert_eq!(
        file.resolved_file_asset_for_object(10)
            .map(|asset| asset.type_name),
        Some("FontAsset")
    );
    assert_eq!(
        file.resolved_file_asset_for_object(11)
            .map(|asset| asset.type_name),
        Some("ScriptAsset")
    );
    assert!(file.resolved_file_asset_for_object(42).is_none());
}

#[test]
fn file_asset_cdn_uuid_bytes_format_like_cpp_runtime() {
    let valid_cdn_uuid = (0u8..16).collect::<Vec<_>>();
    let invalid_cdn_uuid = [0xaa, 0xbb, 0xcc];

    let file = read_runtime_file(&synthetic_runtime_file(4298, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ImageAsset", |bytes| {
            push_bytes_property(bytes, "ImageAsset", "cdnUuid", &valid_cdn_uuid);
        });
        push_object_with_properties(bytes, "ImageAsset", |bytes| {
            push_bytes_property(bytes, "ImageAsset", "cdnUuid", &invalid_cdn_uuid);
        });
        push_empty_object(bytes, "ImageAsset");
    }))
    .expect("FileAsset CDN UUID bytes decode with a Backboard importer like C++");

    assert_eq!(file.imported_object_count(), 4);
    assert_eq!(file.object(0).unwrap().file_asset_cdn_uuid_string(), None);

    let valid_asset = file.object(1).expect("valid ImageAsset object");
    assert_eq!(
        valid_asset.bytes_property("cdnUuid"),
        Some(valid_cdn_uuid.as_slice())
    );
    assert_eq!(
        valid_asset.file_asset_cdn_uuid_string(),
        Some("03020100-0504-0706-0908-0f0e0d0c0b0a".to_owned())
    );

    assert_eq!(
        file.object(2).unwrap().file_asset_cdn_uuid_string(),
        Some(String::new())
    );
    assert_eq!(
        file.object(3).unwrap().file_asset_cdn_uuid_string(),
        Some(String::new())
    );
}

#[test]
fn file_asset_unique_names_and_filenames_match_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file(4331, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_file_asset_with_name_and_id(bytes, "ImageAsset", "photo.large.png", 2);
        push_file_asset_with_name_and_id(bytes, "FontAsset", "Inter", 3);
        push_file_asset_with_name_and_id(bytes, "AudioAsset", "hit.", 4);
        push_file_asset_with_name_and_id(bytes, "BlobAsset", ".hidden", 5);
        push_file_asset_with_name_and_id(bytes, "ScriptAsset", "behavior.lua", 6);
        push_file_asset_with_name_and_id(bytes, "ShaderAsset", "fill.rstb", 7);
        push_file_asset_with_name_and_id(bytes, "ManifestAsset", "manifest.man", 8);
    }))
    .expect("synthetic named file assets import");

    let expected = [
        (
            1,
            Some("png"),
            Some("photo.large-2"),
            b"photo.large-2".as_slice(),
            Some("photo.large-2.png"),
            b"photo.large-2.png".as_slice(),
        ),
        (
            2,
            Some("ttf"),
            Some("Inter-3"),
            b"Inter-3".as_slice(),
            Some("Inter-3.ttf"),
            b"Inter-3.ttf".as_slice(),
        ),
        (
            3,
            Some("wav"),
            Some("hit-4"),
            b"hit-4".as_slice(),
            Some("hit-4.wav"),
            b"hit-4.wav".as_slice(),
        ),
        (
            4,
            Some("blob"),
            Some("-5"),
            b"-5".as_slice(),
            Some("-5.blob"),
            b"-5.blob".as_slice(),
        ),
        (
            5,
            Some("lua"),
            Some("behavior-6"),
            b"behavior-6".as_slice(),
            Some("behavior-6.lua"),
            b"behavior-6.lua".as_slice(),
        ),
        (
            6,
            Some("rstb"),
            Some("fill-7"),
            b"fill-7".as_slice(),
            Some("fill-7.rstb"),
            b"fill-7.rstb".as_slice(),
        ),
        (
            7,
            Some("man"),
            Some("manifest-8"),
            b"manifest-8".as_slice(),
            Some("manifest-8.man"),
            b"manifest-8.man".as_slice(),
        ),
    ];

    for (
        object_id,
        extension,
        unique_name,
        unique_name_bytes,
        unique_filename,
        unique_filename_bytes,
    ) in expected
    {
        let asset = file.object(object_id).expect("file asset object");
        assert_eq!(asset.file_asset_extension(), extension);
        assert_eq!(asset.file_asset_unique_name().as_deref(), unique_name);
        assert_eq!(
            asset.file_asset_unique_name_bytes().as_deref(),
            Some(unique_name_bytes)
        );
        assert_eq!(
            asset.file_asset_unique_filename().as_deref(),
            unique_filename
        );
        assert_eq!(
            asset.file_asset_unique_filename_bytes().as_deref(),
            Some(unique_filename_bytes)
        );
    }

    assert_eq!(file.object(0).unwrap().file_asset_extension(), None);
    assert_eq!(file.object(0).unwrap().file_asset_unique_name(), None);
    assert_eq!(file.object(0).unwrap().file_asset_unique_name_bytes(), None);
    assert_eq!(file.object(0).unwrap().file_asset_unique_filename(), None);
    assert_eq!(
        file.object(0).unwrap().file_asset_unique_filename_bytes(),
        None
    );

    let raw_name_file = read_runtime_file(&synthetic_runtime_file(4332, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ImageAsset", |bytes| {
            push_bytes_property(bytes, "ImageAsset", "name", &[b'r', b'a', 0xff, b'.', b'x']);
            push_uint_property(bytes, "ImageAsset", "assetId", 9);
        });
    }))
    .expect("synthetic raw named file asset imports");
    let raw_name_asset = raw_name_file.object(1).expect("raw named ImageAsset");
    assert_eq!(raw_name_asset.string_property("name"), None);
    assert_eq!(
        raw_name_asset.file_asset_unique_name_bytes().as_deref(),
        Some(&[b'r', b'a', 0xff, b'-', b'9'][..])
    );
    assert_eq!(raw_name_asset.file_asset_unique_name(), None);
    assert_eq!(
        raw_name_asset.file_asset_unique_filename_bytes().as_deref(),
        Some(&[b'r', b'a', 0xff, b'-', b'9', b'.', b'p', b'n', b'g'][..])
    );
    assert_eq!(raw_name_asset.file_asset_unique_filename(), None);
}

fn push_file_asset_with_name_and_id(
    bytes: &mut Vec<u8>,
    type_name: &str,
    name: &str,
    asset_id: u64,
) {
    push_object_with_properties(bytes, type_name, |bytes| {
        push_string_property(bytes, type_name, "name", name);
        push_uint_property(bytes, type_name, "assetId", asset_id);
    });
}

#[test]
fn manifest_asset_contents_decode_names_and_paths_like_cpp_runtime() {
    fn push_manifest_string(bytes: &mut Vec<u8>, value: &[u8]) {
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    let mut names = Vec::new();
    push_var_uint(&mut names, 3);
    push_var_uint(&mut names, 7);
    push_manifest_string(&mut names, b"position");
    push_var_uint(&mut names, 9);
    push_manifest_string(&mut names, b"opacity");
    push_var_uint(&mut names, u32::MAX as u64);
    push_manifest_string(&mut names, b"wrapped");

    let mut paths = Vec::new();
    push_var_uint(&mut paths, 2);
    push_var_uint(&mut paths, 3);
    push_var_uint(&mut paths, 3);
    for path_id in [7, 9, 11] {
        push_var_uint(&mut paths, path_id);
    }
    push_var_uint(&mut paths, u32::MAX as u64);
    push_var_uint(&mut paths, 2);
    for path_id in [13, 21] {
        push_var_uint(&mut paths, path_id);
    }

    let mut manifest_bytes = Vec::new();
    push_var_uint(&mut manifest_bytes, 0);
    push_var_uint(&mut manifest_bytes, names.len() as u64);
    manifest_bytes.extend_from_slice(&names);
    push_var_uint(&mut manifest_bytes, 99);
    push_var_uint(&mut manifest_bytes, 3);
    manifest_bytes.extend_from_slice(&[0xaa, 0xbb, 0xcc]);
    push_var_uint(&mut manifest_bytes, 1);
    push_var_uint(&mut manifest_bytes, paths.len() as u64);
    manifest_bytes.extend_from_slice(&paths);

    let file = read_runtime_file(&synthetic_runtime_file(4299, |bytes| {
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
    }))
    .expect("ManifestAsset resolves in-band FileAssetContents like C++");

    assert_eq!(file.imported_object_count(), 2);

    let manifest = file.manifest().expect("decoded manifest data");
    assert_eq!(manifest.resolve_name(7), Some("position"));
    assert_eq!(manifest.resolve_name_bytes(9), Some(b"opacity".as_slice()));
    assert_eq!(manifest.resolve_name(u32::MAX), Some("wrapped"));
    assert!(manifest.names.contains_key(&-1));
    assert_eq!(manifest.resolve_name(404), None);
    assert_eq!(manifest.resolve_path(3), Some(&[7, 9, 11][..]));
    assert_eq!(manifest.resolve_path(u32::MAX), Some(&[13, 21][..]));
    assert!(manifest.paths.contains_key(&-1));
    assert_eq!(manifest.resolve_path(404), None);
}

#[test]
fn malformed_manifest_asset_contents_keep_cpp_partial_decode_state() {
    fn push_manifest_string(bytes: &mut Vec<u8>, value: &[u8]) {
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    let mut names = Vec::new();
    push_var_uint(&mut names, 1);
    push_var_uint(&mut names, 7);
    push_manifest_string(&mut names, b"position");

    let mut paths = Vec::new();
    push_var_uint(&mut paths, 1);
    push_var_uint(&mut paths, 3);
    push_var_uint(&mut paths, 2);
    push_var_uint(&mut paths, 7);
    paths.push(0x80);

    let mut manifest_bytes = Vec::new();
    push_var_uint(&mut manifest_bytes, 0);
    push_var_uint(&mut manifest_bytes, names.len() as u64);
    manifest_bytes.extend_from_slice(&names);
    push_var_uint(&mut manifest_bytes, 1);
    push_var_uint(&mut manifest_bytes, paths.len() as u64);
    manifest_bytes.extend_from_slice(&paths);

    let file = read_runtime_file(&synthetic_runtime_file(4300, |bytes| {
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
    }))
    .expect("C++ ignores ManifestAsset::decode failure during file asset resolution");

    assert_eq!(file.imported_object_count(), 2);

    let manifest = file
        .manifest()
        .expect("C++ keeps the manifest resolver after decode failure");
    assert_eq!(manifest.resolve_name(7), Some("position"));
    assert_eq!(manifest.resolve_path(3), Some(&[7, 0][..]));
}

#[test]
fn runtime_import_status_tracks_cpp_missing_context_without_hiding_decode() {
    let file = read_runtime_file(&synthetic_runtime_file(4286, |bytes| {
        push_empty_object(bytes, "Node");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "ImageAsset");
        push_empty_object(bytes, "FileAssetContents");
        push_empty_object(bytes, "LinearAnimation");
        push_empty_object(bytes, "StateMachine");
    }))
    .expect("contextless known objects still decode");

    assert_eq!(file.object_count(), 6);
    assert_eq!(file.known_object_count(), 6);
    assert_eq!(file.imported_object_count(), 0);

    for index in 0..file.object_count() {
        assert!(
            file.object(index).is_some(),
            "raw decoded object {index} should remain inspectable"
        );
        assert_eq!(
            file.import_status(index),
            Some(RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            }),
            "object {index} should match C++ failed object->import deletion"
        );
    }
}

#[test]
fn runtime_import_status_tracks_valid_cpp_import_stack_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4287, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "Node");
        push_empty_object(bytes, "ImageAsset");
        push_empty_object(bytes, "FileAssetContents");
        push_empty_object(bytes, "LinearAnimation");
        push_empty_object(bytes, "KeyedObject");
        push_empty_object(bytes, "KeyedProperty");
        push_empty_object(bytes, "KeyFrameDouble");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
        push_empty_object(bytes, "StateMachineBool");
    }))
    .expect("objects with required import contexts decode");

    assert_eq!(file.object_count(), 15);
    assert_eq!(file.known_object_count(), 15);
    assert_eq!(file.imported_object_count(), 15);
    for index in 0..file.object_count() {
        assert_eq!(
            file.import_status(index),
            Some(RuntimeImportStatus::Imported),
            "object {index} should import through the C++ stack model"
        );
    }
}

#[test]
fn runtime_import_status_tracks_view_model_and_enum_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4288, |bytes| {
        push_empty_object(bytes, "ViewModelPropertyBoolean");
        push_empty_object(bytes, "ViewModel");
        push_empty_object(bytes, "ViewModelPropertyBoolean");
        push_empty_object(bytes, "ViewModelInstance");
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ViewModelInstance");
        push_empty_object(bytes, "ViewModelInstanceBoolean");
        push_empty_object(bytes, "DataEnumValue");
        push_empty_object(bytes, "DataEnumCustom");
        push_empty_object(bytes, "DataEnumValue");
    }))
    .expect("view-model and enum context objects decode");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ]
    );
}

#[test]
fn runtime_import_status_tracks_asset_content_importer_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4290, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ScriptAsset");
        push_empty_object(bytes, "FileAssetContents");
        push_empty_object(bytes, "ImageAsset");
        push_empty_object(bytes, "FileAssetContents");
    }))
    .expect("asset content context stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "in the current non-scripting C++ probe build, ScriptAsset does not create a FileAssetImporter but ImageAsset does"
    );
}

#[test]
fn runtime_import_status_tracks_listener_input_type_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4297, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "KeyboardInput");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_empty_object(bytes, "KeyboardInput");
        push_empty_object(bytes, "ListenerInputTypeKeyboard");
        push_empty_object(bytes, "KeyboardInput");
        push_empty_object(bytes, "ListenerInputTypeSemantic");
        push_empty_object(bytes, "SemanticInput");
        push_empty_object(bytes, "ListenerInputTypeGamepad");
        push_empty_object(bytes, "GamepadInput");
    }))
    .expect("listener input type context stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ requires a matching ListenerInputType importer before concrete user inputs import"
    );
}

#[test]
fn runtime_import_status_tracks_scripted_object_input_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4304, |bytes| {
        push_empty_object(bytes, "ScriptInputBoolean");
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ScriptedDataConverter");
        push_empty_object(bytes, "ScriptInputString");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "ScriptedDrawable");
        push_empty_object(bytes, "ScriptInputNumber");
        push_object_with_properties(bytes, "ScriptInputArtboard", |bytes| {
            push_uint_property(bytes, "ScriptInputArtboard", "artboardId", 0);
        });
    }))
    .expect("scripted-object input context stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "script inputs require the latest C++ ScriptedObjectImporter; ScriptInputArtboard also requires the backboard importer"
    );
}

#[test]
fn runtime_import_status_tracks_transition_importer_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4298, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 0);
        });
        push_empty_object(bytes, "TransitionValueNumberComparator");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_empty_object(bytes, "ListenerViewModelChange");
        push_empty_object(bytes, "BindablePropertyNumber");
        push_empty_object(bytes, "TransitionPropertyViewModelComparator");
    }))
    .expect("transition importer context stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
        ],
        "C++ requires StateTransition, TransitionViewModelCondition, and BindableProperty importer contexts for transition children"
    );
}

#[test]
fn runtime_import_status_validates_transition_input_condition_types() {
    let valid = read_runtime_file(&synthetic_runtime_file(4299, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 0);
        });
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("valid transition input condition stream decodes");

    assert!(
        valid
            .import_statuses
            .iter()
            .all(|status| *status == RuntimeImportStatus::Imported),
        "matching bool inputs should import through the C++ transition condition validator"
    );

    let invalid = read_runtime_file(&synthetic_runtime_file(4300, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 0);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 1);
        });
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("invalid transition input condition stream still decodes");

    assert_eq!(
        invalid.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ drops transition input conditions whose input id is out of range or points at the wrong input type"
    );
}

#[test]
fn runtime_import_status_validates_listener_input_change_types() {
    let valid = read_runtime_file(&synthetic_runtime_file(4307, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
            push_uint_property(bytes, "ListenerBoolChange", "inputId", 0);
        });
        push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
            push_uint_property(bytes, "ListenerBoolChange", "inputId", 1);
        });
    }))
    .expect("valid listener input changes decode");

    assert!(
        valid
            .import_statuses
            .iter()
            .all(|status| *status == RuntimeImportStatus::Imported),
        "C++ listener input changes allow matching inputs and out-of-range/null inputs"
    );

    let invalid = read_runtime_file(&synthetic_runtime_file(4308, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
            push_uint_property(bytes, "ListenerBoolChange", "inputId", 0);
        });
        push_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
            push_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
        });
        push_object_with_properties(bytes, "ListenerTriggerChange", |bytes| {
            push_uint_property(bytes, "ListenerTriggerChange", "inputId", 0);
        });
    }))
    .expect("invalid listener input changes still decode");

    assert_eq!(
        invalid.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
        ],
        "C++ drops listener input changes only when a resolved state-machine input has the wrong kind"
    );

    let nested_valid = read_runtime_file(&synthetic_runtime_file(4310, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "NestedBool", |bytes| {
            push_uint_property(bytes, "NestedBool", "parentId", 0);
        });
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
            push_uint_property(bytes, "ListenerBoolChange", "inputId", 0);
            push_uint_property(bytes, "ListenerBoolChange", "nestedInputId", 1);
        });
    }))
    .expect("matching nested listener input changes decode");

    assert!(
        nested_valid
            .import_statuses
            .iter()
            .all(|status| *status == RuntimeImportStatus::Imported),
        "C++ validates listener input changes against resolved nested inputs before state-machine inputs"
    );

    let nested_invalid = read_runtime_file(&synthetic_runtime_file(4311, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "NestedNumber", |bytes| {
            push_uint_property(bytes, "NestedNumber", "parentId", 0);
        });
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "ListenerBoolChange", |bytes| {
            push_uint_property(bytes, "ListenerBoolChange", "inputId", 0);
            push_uint_property(bytes, "ListenerBoolChange", "nestedInputId", 1);
        });
        push_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
            push_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
            push_uint_property(bytes, "ListenerNumberChange", "nestedInputId", 1);
        });
    }))
    .expect("wrong-kind nested listener input changes still decode");

    assert_eq!(
        nested_invalid.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
        ],
        "C++ drops listener input changes whose resolved nested input has the wrong kind"
    );
}

#[test]
fn runtime_import_status_counts_state_machine_null_input_placeholders() {
    // The C++ importer adds the placeholder before later finalization paths
    // trip over null inputs, so this pins the importer-side context effect.
    let file = read_runtime_file(&synthetic_runtime_file(4304, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineInput");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 1);
        });
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("state-machine null input placeholder stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::NullObject,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ StateMachineImporter::readNullObject inserts a null input placeholder, so later input ids keep their serialized indexes"
    );
}

#[test]
fn runtime_import_status_routes_null_objects_to_latest_cpp_importer() {
    let file = read_runtime_file(&synthetic_runtime_file(4312, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "StateMachineInput");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 1);
        });
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("latest importer consumes null object before older state-machine importer");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::NullObject,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ ImportStack::readNullObject routes the null slot to the latest StateMachineLayerImporter, not the older StateMachineImporter"
    );
}

#[test]
fn runtime_import_status_skips_non_consuming_latest_importers_for_null_objects() {
    let file = read_runtime_file(&synthetic_runtime_file(4313, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "LinearAnimation");
        push_empty_object(bytes, "StateMachineInput");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 1);
        });
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("non-consuming latest importer should not block null-object routing");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::NullObject,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ ImportStack::readNullObject skips latest importers whose readNullObject returns false before an older importer consumes the null"
    );
}

#[test]
fn runtime_import_status_validates_blend_state_input_contexts() {
    let valid = read_runtime_file(&synthetic_runtime_file(4305, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineLayer");
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
        });
        push_empty_object(bytes, "BlendAnimation1D");
        push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
            push_uint_property(bytes, "BlendAnimationDirect", "inputId", 0);
        });
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("valid blend input stream decodes");

    assert!(
        valid
            .import_statuses
            .iter()
            .all(|status| *status == RuntimeImportStatus::Imported),
        "number-backed blend states and direct blend animations should import"
    );

    let invalid = read_runtime_file(&synthetic_runtime_file(4306, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineLayer");
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 0);
        });
        push_object_with_properties(bytes, "BlendState1DInput", |bytes| {
            push_uint_property(bytes, "BlendState1DInput", "inputId", 1);
        });
        push_empty_object(bytes, "BlendStateDirect");
        push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
            push_uint_property(bytes, "BlendAnimationDirect", "inputId", 0);
        });
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "BlendAnimation1D");
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("invalid blend input stream still decodes");

    assert_eq!(
        invalid.import_statuses,
        vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::InvalidObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ rejects blend states/direct blend animations with missing or non-number inputs and rejects blend animations under non-blend layer states"
    );
}

#[test]
fn runtime_import_resolution_validates_state_transition_state_to_ids() {
    let state_transition_file = |file_id: u64, include_null_state: bool, state_to_id: u64| {
        synthetic_runtime_file(file_id, |bytes| {
            push_empty_object(bytes, "Backboard");
            push_empty_object(bytes, "Artboard");
            push_empty_object(bytes, "StateMachine");
            push_empty_object(bytes, "StateMachineLayer");
            push_empty_object(bytes, "AnyState");
            push_object_with_properties(bytes, "StateTransition", |bytes| {
                push_uint_property(bytes, "StateTransition", "stateToId", state_to_id);
            });
            if include_null_state {
                push_empty_object(bytes, "LayerState");
            }
            push_empty_object(bytes, "EntryState");
            push_empty_object(bytes, "ExitState");
        })
    };

    let invalid_without_null =
        read_runtime_file_with_error_kind(&state_transition_file(4301, false, 3))
            .expect_err("C++ rejects transitions targeting states outside the layer state list");
    assert_eq!(invalid_without_null.kind(), RuntimeReadErrorKind::Malformed);

    let valid_with_null = read_runtime_file(&state_transition_file(4302, true, 3))
        .expect("C++ counts a null LayerState slot as a placeholder state");
    assert_eq!(
        valid_with_null.import_status(6),
        Some(RuntimeImportStatus::NullObject),
        "abstract LayerState remains a null serialized slot while contributing to C++ layer state count"
    );

    let invalid_with_null =
        read_runtime_file_with_error_kind(&state_transition_file(4303, true, 4))
            .expect_err("C++ still rejects targets beyond the placeholder-extended state list");
    assert_eq!(invalid_with_null.kind(), RuntimeReadErrorKind::Malformed);
}

#[test]
fn runtime_state_transition_interpolator_ids_follow_cpp_optional_resolution() {
    let state_transition_file = |file_id: u64, interpolator_id: u64| {
        synthetic_runtime_file(file_id, |bytes| {
            push_empty_object(bytes, "Backboard");
            push_empty_object(bytes, "Artboard");
            push_empty_object(bytes, "CubicEaseInterpolator");
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_empty_object(bytes, "StateMachine");
            push_empty_object(bytes, "StateMachineLayer");
            push_empty_object(bytes, "AnyState");
            push_object_with_properties(bytes, "StateTransition", |bytes| {
                push_uint_property(bytes, "StateTransition", "stateToId", 2);
                push_uint_property(bytes, "StateTransition", "interpolatorId", interpolator_id);
            });
            push_empty_object(bytes, "EntryState");
            push_empty_object(bytes, "ExitState");
        })
    };

    let valid = read_runtime_file(&state_transition_file(4348, 1))
        .expect("C++ accepts transitions whose interpolatorId resolves to KeyFrameInterpolator");
    let transition =
        &valid.artboard_state_machine_graph(0, 0).unwrap().layers[0].states[0].transitions[0];
    assert_eq!(
        transition.interpolator.map(|object| object.type_name),
        Some("CubicEaseInterpolator")
    );

    let wrong_kind = read_runtime_file(&state_transition_file(4349, 2))
        .expect("C++ accepts wrong-kind transition interpolator ids as unresolved");
    let transition = &wrong_kind
        .artboard_state_machine_graph(0, 0)
        .unwrap()
        .layers[0]
        .states[0]
        .transitions[0];
    assert!(
        transition.interpolator.is_none(),
        "wrong-kind interpolatorId imports but resolves to no interpolator"
    );

    let missing = read_runtime_file(&state_transition_file(4350, 99))
        .expect("C++ accepts out-of-range transition interpolator ids as unresolved");
    let transition =
        &missing.artboard_state_machine_graph(0, 0).unwrap().layers[0].states[0].transitions[0];
    assert!(
        transition.interpolator.is_none(),
        "missing interpolatorId imports but resolves to no interpolator"
    );
}

#[test]
fn runtime_state_machine_layers_expose_cpp_states_transitions_and_conditions() {
    let file = read_runtime_file(&synthetic_runtime_file(4346, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "CubicEaseInterpolator");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "OpenUrlEvent", |bytes| {
            push_uint_property(bytes, "OpenUrlEvent", "parentId", 2);
        });
        push_empty_object(bytes, "LinearAnimation");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineBool");
        push_empty_object(bytes, "StateMachineNumber");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_empty_object(bytes, "ScriptedListenerAction");
        push_object_with_properties(bytes, "ScriptInputNumber", |bytes| {
            push_string_property(bytes, "ScriptInputNumber", "name", "listener-input");
        });
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 5);
            push_uint_property(bytes, "StateTransition", "interpolatorId", 1);
        });
        push_object_with_properties(bytes, "StateMachineFireEvent", |bytes| {
            push_uint_property(bytes, "StateMachineFireEvent", "eventId", 3);
            push_uint_property(bytes, "StateMachineFireEvent", "occursValue", 1);
        });
        push_object_with_properties(bytes, "ListenerFireEvent", |bytes| {
            push_uint_property(bytes, "ListenerFireEvent", "eventId", 3);
            push_uint_property(bytes, "ListenerFireEvent", "flags", 2);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 0);
        });
        push_empty_object(bytes, "ScriptedTransitionCondition");
        push_object_with_properties(bytes, "ScriptInputBoolean", |bytes| {
            push_string_property(bytes, "ScriptInputBoolean", "name", "condition-input");
        });
        push_empty_object(bytes, "EntryState");
        push_object_with_properties(bytes, "StateMachineFireEvent", |bytes| {
            push_uint_property(bytes, "StateMachineFireEvent", "eventId", 2);
            push_uint_property(bytes, "StateMachineFireEvent", "occursValue", 2);
        });
        push_object_with_properties(bytes, "ListenerFireEvent", |bytes| {
            push_uint_property(bytes, "ListenerFireEvent", "eventId", 2);
            push_uint_property(bytes, "ListenerFireEvent", "flags", 4);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 0);
        });
        push_object_with_properties(bytes, "AnimationState", |bytes| {
            push_uint_property(bytes, "AnimationState", "animationId", 99);
        });
        push_empty_object(bytes, "BlendStateDirect");
        push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
            push_uint_property(bytes, "BlendAnimationDirect", "animationId", 0);
            push_uint_property(bytes, "BlendAnimationDirect", "inputId", 1);
        });
        push_object_with_properties(bytes, "BlendAnimationDirect", |bytes| {
            push_uint_property(bytes, "BlendAnimationDirect", "animationId", 99);
            push_uint_property(bytes, "BlendAnimationDirect", "inputId", 1);
        });
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 5);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 5);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 1);
        });
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 5);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 99);
        });
        push_empty_object(bytes, "ExitState");
    }))
    .expect("synthetic state-machine transition graph imports");

    let state_machine = file
        .artboard_state_machine_graph(0, 0)
        .expect("state machine graph exists");
    assert_eq!(
        state_machine
            .scripted_objects
            .iter()
            .map(|scripted_object| scripted_object.object.type_name)
            .collect::<Vec<_>>(),
        vec!["ScriptedListenerAction", "ScriptedTransitionCondition"]
    );
    assert_eq!(
        state_machine.scripted_objects[0]
            .inputs
            .iter()
            .map(|input| (input.type_name, input.string_property("name")))
            .collect::<Vec<_>>(),
        vec![("ScriptInputNumber", Some("listener-input"))]
    );
    assert_eq!(
        state_machine.scripted_objects[1]
            .inputs
            .iter()
            .map(|input| (input.type_name, input.string_property("name")))
            .collect::<Vec<_>>(),
        vec![("ScriptInputBoolean", Some("condition-input"))]
    );
    assert_eq!(state_machine.listeners.len(), 1);
    assert_eq!(
        state_machine.listeners[0].actions[0].object.type_name,
        "ScriptedListenerAction"
    );
    let layer = &state_machine.layers[0];
    assert_eq!(layer.state_count, 6);
    assert_eq!(layer.states.len(), 6);
    assert_eq!(layer.states[0].object.unwrap().type_name, "AnyState");
    assert_eq!(layer.states[1].object.unwrap().type_name, "EntryState");
    assert_eq!(layer.states[1].fire_actions.len(), 1);
    assert_eq!(
        layer.states[1].fire_actions[0].object.type_name,
        "StateMachineFireEvent"
    );
    assert_eq!(layer.states[1].fire_actions[0].event_local_index, None);
    assert!(layer.states[1].fire_actions[0].event.is_none());
    assert_eq!(layer.states[1].listener_actions.len(), 1);
    assert_eq!(
        layer.states[1].listener_actions[0].object.type_name,
        "ListenerFireEvent"
    );
    assert_eq!(layer.states[1].listener_actions[0].event_local_index, None);
    assert!(layer.states[1].listener_actions[0].event.is_none());
    assert_eq!(layer.states[2].object.unwrap().type_name, "AnimationState");
    assert_eq!(
        layer.states[2]
            .animation
            .map(|animation| animation.type_name),
        Some("LinearAnimation")
    );
    assert_eq!(layer.states[3].object.unwrap().type_name, "AnimationState");
    assert!(
        layer.states[3].animation.is_none(),
        "out-of-range C++ AnimationState.animationId imports but resolves to no animation"
    );
    assert_eq!(
        layer.states[4].object.unwrap().type_name,
        "BlendStateDirect"
    );
    assert_eq!(layer.states[4].blend_animations.len(), 2);
    assert_eq!(
        layer.states[4].blend_animations[0].object.type_name,
        "BlendAnimationDirect"
    );
    assert_eq!(layer.states[4].blend_animations[0].animation_index, Some(0));
    assert_eq!(
        layer.states[4].blend_animations[0]
            .animation
            .map(|animation| animation.type_name),
        Some("LinearAnimation")
    );
    assert_eq!(
        layer.states[4].blend_animations[1].object.type_name,
        "BlendAnimationDirect"
    );
    assert_eq!(layer.states[4].blend_animations[1].animation_index, None);
    assert!(
        layer.states[4].blend_animations[1].animation.is_none(),
        "out-of-range C++ BlendAnimation.animationId imports but resolves to no animation"
    );
    assert_eq!(layer.states[4].transitions.len(), 3);
    assert_eq!(
        layer.states[4].transitions[0].object.type_name,
        "BlendStateTransition"
    );
    assert_eq!(
        layer.states[4].transitions[0].exit_blend_animation_index,
        Some(0)
    );
    assert_eq!(
        layer.states[4].transitions[0]
            .exit_blend_animation
            .map(|animation| animation.type_name),
        Some("BlendAnimationDirect")
    );
    assert_eq!(layer.states[4].transitions[0].exit_animation_index, Some(0));
    assert_eq!(
        layer.states[4].transitions[0]
            .exit_animation
            .map(|animation| animation.type_name),
        Some("LinearAnimation")
    );
    assert_eq!(
        layer.states[4].transitions[1].exit_blend_animation_index,
        Some(1)
    );
    assert!(
        layer.states[4].transitions[1].exit_animation.is_none(),
        "valid exit blend animation can still point at an unresolved authored animation"
    );
    assert_eq!(
        layer.states[4].transitions[2].exit_blend_animation_index,
        None
    );
    assert!(
        layer.states[4].transitions[2]
            .exit_blend_animation
            .is_none()
    );
    assert!(layer.states[4].transitions[2].exit_animation.is_none());
    assert_eq!(layer.states[5].object.unwrap().type_name, "ExitState");

    let transition = &layer.states[0].transitions[0];
    assert_eq!(transition.object.type_name, "StateTransition");
    assert_eq!(transition.state_to_index, Some(5));
    assert_eq!(transition.state_to.unwrap().type_name, "ExitState");
    assert_eq!(transition.fire_actions.len(), 1);
    assert_eq!(
        transition.fire_actions[0].object.type_name,
        "StateMachineFireEvent"
    );
    assert_eq!(transition.fire_actions[0].event_local_index, Some(3));
    assert_eq!(
        transition.fire_actions[0]
            .event
            .map(|event| event.type_name),
        Some("OpenUrlEvent")
    );
    assert_eq!(transition.listener_actions.len(), 1);
    assert_eq!(
        transition.listener_actions[0].object.type_name,
        "ListenerFireEvent"
    );
    assert_eq!(transition.listener_actions[0].event_local_index, Some(3));
    assert_eq!(
        transition.listener_actions[0]
            .event
            .map(|event| event.type_name),
        Some("OpenUrlEvent")
    );
    assert_eq!(
        transition.interpolator.unwrap().type_name,
        "CubicEaseInterpolator"
    );
    assert_eq!(
        transition
            .conditions
            .iter()
            .map(|condition| condition.type_name)
            .collect::<Vec<_>>(),
        vec!["TransitionBoolCondition", "ScriptedTransitionCondition"]
    );

    let placeholder_file = read_runtime_file(&synthetic_runtime_file(4347, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_empty_object(bytes, "LayerState");
        push_empty_object(bytes, "EntryState");
        push_empty_object(bytes, "ExitState");
    }))
    .expect("synthetic state-machine placeholder state graph imports");

    let placeholder_layer = &placeholder_file
        .artboard_state_machine_graph(0, 0)
        .expect("placeholder state machine graph exists")
        .layers[0];
    assert_eq!(placeholder_layer.state_count, 4);
    assert!(placeholder_layer.states[1].object.is_none());
    let placeholder_transition = &placeholder_layer.states[0].transitions[0];
    assert_eq!(placeholder_transition.state_to_index, Some(1));
    assert!(
        placeholder_transition.state_to.is_none(),
        "C++ placeholder states preserve transition indexes without a serialized Rust object"
    );
}

#[test]
fn runtime_import_resolution_requires_state_machine_layer_scaffold_states() {
    let missing_exit = read_runtime_file_with_error_kind(&synthetic_runtime_file(4309, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineLayer");
        push_empty_object(bytes, "AnyState");
        push_empty_object(bytes, "EntryState");
    }))
    .expect_err("C++ rejects state-machine layers missing AnyState, EntryState, or ExitState");

    assert_eq!(missing_exit.kind(), RuntimeReadErrorKind::Malformed);
}

#[test]
fn runtime_import_status_tracks_formula_and_scroll_physics_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4291, |bytes| {
        push_empty_object(bytes, "FormulaTokenParenthesisOpen");
        push_empty_object(bytes, "ElasticScrollPhysics");
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ElasticScrollPhysics");
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "FormulaTokenParenthesisOpen");
    }))
    .expect("formula and scroll-physics context stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ]
    );
}

#[test]
fn duplicate_file_asset_ids_are_not_normalized_for_dropped_assets() {
    let file = read_runtime_file(&synthetic_runtime_file(4289, |bytes| {
        // Without a BackboardImporter, C++ deletes these ImageAsset objects
        // before BackboardImporter::addFileAsset can normalize duplicate ids.
        push_empty_image_asset_with_id(bytes, 10);
        push_empty_image_asset_with_id(bytes, 10);
    }))
    .expect("contextless image assets still decode");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
        ]
    );

    let asset_ids = file
        .objects
        .iter()
        .filter_map(|object| object.as_ref())
        .map(|object| object.uint_property("assetId"))
        .collect::<Vec<_>>();
    assert_eq!(asset_ids, vec![Some(10), Some(10)]);
}

#[test]
fn runtime_import_status_tracks_file_asset_referencer_contexts() {
    let file = read_runtime_file(&synthetic_runtime_file(4364, |bytes| {
        push_empty_object(bytes, "Image");
        push_empty_object(bytes, "AudioEvent");
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStyle", |bytes| {
            push_uint_property(bytes, "TextStyle", "parentId", 2);
        });
        push_empty_object(bytes, "ScriptedDataConverter");
        push_empty_object(bytes, "ScriptedInterpolator");
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Image", |bytes| {
            push_uint_property(bytes, "Image", "parentId", 0);
        });
        push_empty_object(bytes, "AudioEvent");
        push_object_with_properties(bytes, "Text", |bytes| {
            push_uint_property(bytes, "Text", "parentId", 0);
        });
        push_object_with_properties(bytes, "TextStyle", |bytes| {
            push_uint_property(bytes, "TextStyle", "parentId", 10);
        });
        push_empty_object(bytes, "ScriptedDataConverter");
        push_empty_object(bytes, "ScriptedInterpolator");
    }))
    .expect("file asset referencer context stream decodes");

    assert_eq!(
        file.import_statuses,
        vec![
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Dropped {
                reason: RuntimeImportDropReason::MissingObject
            },
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
        "C++ file-asset referencers need their owning Backboard/Artboard importers before they survive import"
    );
}

fn push_empty_image_asset_with_id(bytes: &mut Vec<u8>, asset_id: u64) {
    push_var_uint(bytes, 105);
    push_var_uint(bytes, 204);
    push_var_uint(bytes, asset_id);
    push_var_uint(bytes, 0);
}

#[test]
fn abstract_object_type_keys_import_as_null_slots_like_cpp_registry() {
    let file = read_runtime_file(&synthetic_runtime_file(4260, |bytes| {
        // Component is a known generated type key, but CoreRegistry has no
        // makeCoreInstance case for it because it is abstract.
        push_var_uint(bytes, 10);
        push_var_uint(bytes, 4);
        push_var_uint(bytes, 8);
        bytes.extend_from_slice(b"abstract");
        push_var_uint(bytes, 0);
    }))
    .expect("abstract object with skippable properties imports as a null slot");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());
}

#[test]
fn unknown_object_bool_skip_matches_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file(4245, |bytes| {
        push_var_uint(bytes, 70_000);
        // CustomPropertyBoolean.propertyValue is globally known as a bool.
        // C++ accepts this stream without an explicit object terminator.
        push_var_uint(bytes, 245);
        bytes.push(1);
    }))
    .expect("C++ accepts unknown-object bool fallback at end of stream");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());

    assert!(
        read_runtime_file(&synthetic_runtime_file(4283, |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 245);
            bytes.push(1);
            push_var_uint(bytes, 0);
        }))
        .is_err(),
        "C++ does not consume bool fallback payloads, so the byte before the terminator is re-read"
    );
}

#[test]
fn missing_toc_property_at_eof_collapses_object_to_null_like_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file(4246, |bytes| {
        push_var_uint(bytes, 70_000);
        push_var_uint(bytes, 65_000);
    }))
    .expect("C++ treats a missing-ToC property at EOF as a null object slot");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());

    assert!(
        read_runtime_file(&synthetic_runtime_file(4247, |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 65_000);
            push_var_uint(bytes, 0);
        }))
        .is_err(),
        "C++ leaves the explicit terminator in the stream and later reports malformed"
    );
}

#[test]
fn callback_fallback_matches_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file(4248, |bytes| {
        push_var_uint(bytes, 70_000);
        // NestedTrigger.fire is a callback key. C++ does not use
        // CoreRegistry::isCallback() in the unknown-property skip path.
        push_var_uint(bytes, 401);
    }))
    .expect("C++ treats a callback fallback at EOF as a null object slot");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());

    assert!(
        read_runtime_file(&synthetic_runtime_file(4249, |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 401);
            push_var_uint(bytes, 0);
        }))
        .is_err(),
        "C++ leaves the explicit terminator in the stream and later reports malformed"
    );
}

#[test]
fn bitmask_passthrough_fallback_matches_cpp_core_registry_table() {
    let file = read_runtime_file(&synthetic_runtime_file(4255, |bytes| {
        push_var_uint(bytes, 70_000);
        // Text.verticalTrimTopValue is object-local bitmask passthrough
        // metadata, but C++ does not include it in CoreRegistry::propertyFieldId.
        push_var_uint(bytes, 1027);
    }))
    .expect("C++ treats bitmask passthrough fallback at EOF as a null object slot");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());

    assert!(
        read_runtime_file(&synthetic_runtime_file(4256, |bytes| {
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 1027);
            push_var_uint(bytes, 0);
        }))
        .is_err(),
        "C++ leaves the explicit terminator in the stream and later reports malformed"
    );
}

#[test]
fn known_object_bitmask_passthrough_without_toc_collapses_to_null_like_cpp() {
    let file = read_runtime_file(&synthetic_runtime_file(4264, |bytes| {
        push_var_uint(bytes, 134);
        // Text.verticalTrimTopValue is a known schema property, but it is not
        // in TextBase::deserialize or CoreRegistry::propertyFieldId.
        push_var_uint(bytes, 1027);
    }))
    .expect("C++ treats a known-object bitmask passthrough fallback at EOF as a null slot");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());
}

#[test]
fn known_object_callback_without_toc_collapses_to_null_like_cpp() {
    let file = read_runtime_file(&synthetic_runtime_file(4265, |bytes| {
        push_var_uint(bytes, 122);
        // NestedTrigger.fire is a known schema callback, but it is not in
        // NestedTriggerBase::deserialize or CoreRegistry::propertyFieldId.
        push_var_uint(bytes, 401);
    }))
    .expect("C++ treats a known-object callback fallback at EOF as a null slot");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());
}

#[test]
fn uint_values_match_cpp_unsigned_int_range() {
    let known_uint = read_runtime_file(&synthetic_runtime_file(4250, |bytes| {
        push_var_uint(bytes, 2);
        push_var_uint(bytes, 5);
        push_var_uint(bytes, u32::MAX as u64);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ accepts known uint values at u32::MAX");
    assert_eq!(
        known_uint.object(0).unwrap().uint_property("parentId"),
        Some(u64::from(u32::MAX))
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4251, |bytes| {
            push_var_uint(bytes, 2);
            push_var_uint(bytes, 5);
            push_var_uint(bytes, u64::from(u32::MAX) + 1);
            push_var_uint(bytes, 0);
        }))
        .is_err(),
        "C++ treats known uint values outside unsigned int range as malformed"
    );

    read_runtime_file(&synthetic_runtime_file(4252, |bytes| {
        push_var_uint(bytes, 70_000);
        push_var_uint(bytes, 5);
        push_var_uint(bytes, u32::MAX as u64);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ accepts uint values at u32::MAX");

    let unknown_wide_uint = read_runtime_file(&synthetic_runtime_file(4253, |bytes| {
        push_var_uint(bytes, 70_000);
        push_var_uint(bytes, 5);
        push_var_uint(bytes, u64::MAX);
        push_var_uint(bytes, 0);

        // Prove the full varuint64 was consumed and the next object stayed
        // aligned after skipping the unknown object's uint field.
        push_var_uint(bytes, 2);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ skips unknown uint fields through the full varuint64 range");
    assert_eq!(unknown_wide_uint.object_count(), 2);
    assert!(unknown_wide_uint.objects[0].is_none());
    assert_eq!(unknown_wide_uint.object(1).unwrap().type_name, "Node");
}

#[test]
fn string_fields_preserve_raw_bytes_without_forcing_utf8() {
    let file = read_runtime_file(&synthetic_runtime_file(4254, |bytes| {
        push_var_uint(bytes, 2);
        push_var_uint(bytes, 4);
        push_var_uint(bytes, 2);
        bytes.extend_from_slice(&[0xff, 0xfe]);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ accepts raw std::string bytes that are not valid UTF-8");

    let node = file.object(0).expect("node object");
    assert_eq!(node.string_property("name"), None);
    assert_eq!(node.string_property_bytes("name"), Some(&[0xff, 0xfe][..]));
}

#[test]
fn length_prefixed_payloads_accept_noncanonical_leb128_lengths_like_cpp_runtime() {
    let file = read_runtime_file(&synthetic_runtime_file(4278, |bytes| {
        push_var_uint(bytes, 2);
        push_var_uint(bytes, 4);
        bytes.extend_from_slice(&[0x82, 0x00]);
        bytes.extend_from_slice(b"ok");
        push_var_uint(bytes, 0);

        push_var_uint(bytes, 70_000);
        push_var_uint(bytes, 212);
        bytes.extend_from_slice(&[0x84, 0x00]);
        bytes.extend_from_slice(&[0x00, 0xff, 0x10, 0x7f]);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ accepts noncanonical LEB128 lengths for string/bytes payloads");

    let node = file.object(0).expect("Node object");
    assert_eq!(node.string_property("name"), Some("ok"));
    assert_eq!(node.string_property_bytes("name"), Some(b"ok".as_slice()));

    assert_eq!(file.object_count(), 2);
    assert!(file.objects[1].is_none());
}

#[test]
fn bytes_fields_preserve_raw_payloads_for_future_decoders() {
    let file = read_runtime_file(&synthetic_runtime_file(4257, |bytes| {
        // FileAssetContents.bytes is a known CoreBytesType field.
        push_var_uint(bytes, 106);
        push_var_uint(bytes, 212);
        push_var_uint(bytes, 4);
        bytes.extend_from_slice(&[0x00, 0xff, 0x10, 0x7f]);
        push_var_uint(bytes, 0);
    }))
    .expect("known bytes payload imports");

    let contents = file.object(0).expect("FileAssetContents object");
    assert_eq!(contents.type_name, "FileAssetContents");
    assert_eq!(
        contents.bytes_property("bytes"),
        Some(&[0x00, 0xff, 0x10, 0x7f][..])
    );
    assert_eq!(contents.id_list_property("bytes"), None);
}

#[test]
fn encoded_list_id_bytes_decode_to_cpp_uint32_buffers() {
    let mut encoded_path = Vec::new();
    for path_id in [0, 1, 127, 128, 300, u32::MAX as u64] {
        push_var_uint(&mut encoded_path, path_id);
    }

    let mut encoded_source_path = Vec::new();
    for path_id in [42, 16_384] {
        push_var_uint(&mut encoded_source_path, path_id);
    }

    let file = read_runtime_file(&synthetic_runtime_file(4260, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_path);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_bytes_property(
                bytes,
                "DataBindContext",
                "sourcePathIds",
                &encoded_source_path,
            );
        });
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &encoded_source_path,
            );
        });
    }))
    .expect("encoded List<Id> bytes import when a Backboard importer is latest like C++");

    assert_eq!(file.import_status(0), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(1), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(2), Some(RuntimeImportStatus::Imported));
    assert_eq!(file.import_status(3), Some(RuntimeImportStatus::Imported));

    let data_bind_path = file.object(1).expect("DataBindPath object");
    assert_eq!(data_bind_path.type_name, "DataBindPath");
    assert_eq!(
        data_bind_path.bytes_property("path"),
        Some(encoded_path.as_slice())
    );
    assert_eq!(
        data_bind_path.id_list_property("path"),
        Some(vec![0, 1, 127, 128, 300, u32::MAX])
    );
    assert_eq!(
        data_bind_path.data_bind_path_ids_property("path"),
        data_bind_path.id_list_property("path")
    );

    let data_bind_context = file.object(2).expect("DataBindContext object");
    assert_eq!(
        data_bind_context.bytes_property("sourcePathIds"),
        Some(encoded_source_path.as_slice())
    );
    assert_eq!(
        data_bind_context.id_list_property("sourcePathIds"),
        Some(vec![42, 16_384])
    );

    let converter = file
        .object(3)
        .expect("DataConverterOperationViewModel object");
    assert_eq!(
        converter.id_list_property("sourcePathIds"),
        Some(vec![42, 16_384])
    );
}

#[test]
fn unknown_object_bytes_fallback_matches_cpp_core_registry_string_slot() {
    let file = read_runtime_file(&synthetic_runtime_file(4276, |bytes| {
        push_var_uint(bytes, 70_000);
        // FileAssetContents.bytes is a CoreBytesType key. C++ uses the shared
        // CoreStringType/CoreBytesType field id to consume the length-prefixed
        // payload in the unknown-property fallback path.
        push_var_uint(bytes, 212);
        push_var_uint(bytes, 4);
        bytes.extend_from_slice(&[0x00, 0xff, 0x10, 0x7f]);
        push_var_uint(bytes, 0);
    }))
    .expect("C++ skips bytes fallback payloads through the shared string/bytes field id");

    assert_eq!(file.object_count(), 1);
    assert_eq!(file.known_object_count(), 0);
    assert!(file.objects[0].is_none());
}

#[test]
fn truncated_bytes_payloads_are_malformed_like_cpp_runtime() {
    assert!(
        read_runtime_file(&synthetic_runtime_file(4258, |bytes| {
            // Mesh.triangleIndexBytes is a generated known bytes field.
            push_var_uint(bytes, 109);
            push_var_uint(bytes, 223);
            push_var_uint(bytes, 4);
            bytes.extend_from_slice(&[1, 2]);
        }))
        .is_err(),
        "C++ fails import when a known bytes payload is shorter than its declared length"
    );

    assert!(
        read_runtime_file(&synthetic_runtime_file(4259, |bytes| {
            // FileAssetContents.bytes identifies the fallback CoreBytesType path.
            push_var_uint(bytes, 70_000);
            push_var_uint(bytes, 212);
            push_var_uint(bytes, 4);
            bytes.extend_from_slice(&[1, 2]);
        }))
        .is_err(),
        "C++ fails import when a fallback bytes payload is shorter than its declared length"
    );
}

#[test]
fn dependency_test_preserves_stable_object_slots_and_inherited_properties() {
    let file = read_fixture("graph/dependency_test.riv");
    assert_eq!(file.header.file_id, 2007);
    assert_eq!(file.object_count(), 19);
    assert_eq!(file.known_object_count(), 18);
    assert!(file.objects[11].is_none(), "abstract BindableProperty slot");

    let artboard = file.object(1).expect("Artboard at global slot 1");
    assert_eq!(artboard.type_name, "Artboard");
    assert_eq!(artboard.string_property("name"), Some("Blue"));
    assert_eq!(artboard.double_property("width"), Some(800.0));
    assert_eq!(artboard.double_property("height"), Some(800.0));

    let a = file.object(2).expect("A node");
    assert_eq!(a.type_name, "Node");
    assert_eq!(a.string_property("name"), Some("A"));
    assert_eq!(a.uint_property("parentId"), Some(0));

    let b = file.object(3).expect("B node");
    assert_eq!(b.string_property("name"), Some("B"));
    assert_eq!(b.uint_property("parentId"), Some(1));

    let rectangle = file.object(5).expect("Rectangle shape");
    assert_eq!(rectangle.type_name, "Shape");
    assert_eq!(rectangle.string_property("name"), Some("Rectangle"));
    assert_eq!(rectangle.uint_property("parentId"), Some(2));

    let path = file.object(6).expect("Rectangle Path");
    assert_eq!(path.type_name, "PointsPath");
    assert_eq!(path.string_property("name"), Some("Rectangle Path"));
    assert_eq!(path.uint_property("parentId"), Some(4));
}

#[test]
fn artboard_local_object_slots_batch_matches_point_queries_and_preserves_nulls() {
    let file = read_fixture("graph/dependency_test.riv");

    let slots = file
        .artboard_local_object_slots(0)
        .expect("dependency_test has an artboard");

    assert!(
        slots.iter().any(Option::is_none),
        "the abstract BindableProperty must remain an indexed null slot"
    );
    for (local_index, slot) in slots.iter().enumerate() {
        assert_eq!(
            slot.map(|object| object.id),
            file.artboard_local_object(0, local_index)
                .map(|object| object.id),
            "batch and point lookup disagree at local slot {local_index}"
        );
    }
}

#[test]
fn two_artboards_preserves_named_artboard_slots() {
    let file = read_fixture("minimal/two_artboards.riv");

    let first = file.object(1).expect("first artboard");
    assert_eq!(first.type_name, "Artboard");
    assert_eq!(first.string_property("name"), Some("Two"));

    let second = file.object(9).expect("second artboard");
    assert_eq!(second.type_name, "Artboard");
    assert_eq!(second.string_property("name"), Some("One"));
}

#[test]
fn state_machine_input_fixture_decodes_nested_input_ids() {
    let file = read_fixture("animation/smi_test.riv");

    let nested_artboard = file.object(2).expect("NestedArtboard");
    assert_eq!(nested_artboard.type_name, "NestedArtboard");
    assert_eq!(
        nested_artboard.string_property("name"),
        Some("artboard to nest component")
    );
    assert_eq!(nested_artboard.double_property("x"), Some(100.0));
    assert_eq!(nested_artboard.double_property("y"), Some(100.0));
    assert_eq!(nested_artboard.uint_property("artboardId"), Some(1));

    let nested_state_machine = file.object(3).expect("NestedStateMachine");
    assert_eq!(nested_state_machine.type_name, "NestedStateMachine");
    assert_eq!(nested_state_machine.uint_property("animationId"), Some(0));

    let nested_trigger = file.object(4).expect("NestedTrigger");
    assert_eq!(nested_trigger.uint_property("inputId"), Some(0));

    let nested_bool = file.object(5).expect("NestedBool");
    assert_eq!(nested_bool.uint_property("inputId"), Some(1));

    let nested_number = file.object(6).expect("NestedNumber");
    assert_eq!(nested_number.uint_property("inputId"), Some(2));
}

#[test]
fn known_non_stored_properties_preserve_skip_metadata_and_decode_value() {
    let bytes =
        synthetic_runtime_file_with_header(7, 0, 4242, &[(1027, HeaderFieldKind::Uint)], |bytes| {
            // Node with passthrough computedWorldX.
            push_var_uint(bytes, 2);
            push_var_uint(bytes, 808);
            bytes.extend_from_slice(&12.5f32.to_le_bytes());
            push_var_uint(bytes, 0);

            // Text with stored verticalTrimValue and bitmask passthrough top value.
            push_var_uint(bytes, 134);
            push_var_uint(bytes, 1026);
            push_var_uint(bytes, 0);
            push_var_uint(bytes, 1027);
            push_var_uint(bytes, 7);
            push_var_uint(bytes, 0);
        });

    let file = read_runtime_file(&bytes).expect("synthetic runtime file imports");

    let node = file.object(0).expect("node object");
    assert!(node.property("computedWorldX").is_none());
    let computed_world_x = node
        .skipped_property("computedWorldX")
        .expect("computedWorldX skipped metadata");
    assert_eq!(computed_world_x.owner, Some("Node"));
    assert_eq!(computed_world_x.reason, SkipReason::PassthroughProperty);
    assert_eq!(computed_world_x.field, Some("double"));
    assert_eq!(computed_world_x.value, Some(FieldValue::Double(12.5)));
    assert_eq!(computed_world_x.bitmask_passthrough, None);

    let text = file.object(1).expect("text object");
    assert_eq!(text.uint_property("verticalTrimValue"), Some(0));
    let vertical_trim_top = text
        .skipped_property("verticalTrimTopValue")
        .expect("verticalTrimTopValue skipped metadata");
    assert_eq!(vertical_trim_top.owner, Some("Text"));
    assert_eq!(
        vertical_trim_top.reason,
        SkipReason::BitmaskPassthroughProperty
    );
    assert_eq!(vertical_trim_top.field, Some("uint"));
    assert_eq!(vertical_trim_top.value, Some(FieldValue::Uint(7)));
    assert_eq!(
        vertical_trim_top.bitmask_passthrough,
        Some(SkippedBitmaskPassthrough {
            target: "verticalTrimValue",
            bit: 0,
            width: 8,
        })
    );
}
