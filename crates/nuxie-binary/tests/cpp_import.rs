use nuxie_binary::{
    FieldValue, RuntimeComponentDirt, RuntimeConvertedDataValue, RuntimeDataBindUpdateEffect,
    RuntimeDataConverterInterpolatorState, RuntimeDataConverterState, RuntimeDataType,
    RuntimeDataValue, RuntimeFile, RuntimeHeader, RuntimeImportStatus, RuntimeManifest,
    RuntimeObject, RuntimeProperty, RuntimeReadErrorKind, RuntimeViewModelInstance,
    read_runtime_file, read_runtime_file_with_error_kind,
};
use nuxie_schema::{
    Definition, FieldKind, core_registry_getter_field_kind_by_property_key, definition_by_name,
    definition_by_type_key, generated::DEFINITIONS,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

const RIVE_CPP_CORPUS_ENV: &str = "RIVE_CPP_CORPUS";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference_runtime_dir() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn default_probe_path() -> PathBuf {
    let os = match std::env::consts::OS {
        "macos" => "macosx",
        other => other,
    };

    repo_root()
        .join("tools/cpp-probe/build")
        .join(os)
        .join("bin/debug/rive_cpp_probe")
}

fn probe_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("RIVE_CPP_PROBE") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Some(path);
        }
        return Some(repo_root().join(path));
    }

    let path = default_probe_path();
    path.exists().then_some(path)
}

fn reference_unit_fixture_dir() -> PathBuf {
    reference_runtime_dir().join("tests/unit_tests/assets")
}

fn fixture(path: &str) -> PathBuf {
    repo_root().join("fixtures").join(path)
}

fn reference_fixture(path: &str) -> PathBuf {
    reference_unit_fixture_dir().join(path)
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

fn push_bool_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: bool) {
    let property_key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(property_key));
    bytes.push(u8::from(value));
}

fn push_color_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
    let property_key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(property_key));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_float_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
    let property_key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(property_key));
    bytes.extend_from_slice(&value.to_le_bytes());
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

fn push_formula_function_converter(bytes: &mut Vec<u8>, function_type: u64, values: &[f32]) {
    push_empty_object(bytes, "DataConverterFormula");
    push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
        push_uint_property(bytes, "FormulaTokenFunction", "functionType", function_type);
    });
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            push_empty_object(bytes, "FormulaTokenArgumentSeparator");
        }
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", *value);
        });
    }
    push_empty_object(bytes, "FormulaTokenParenthesisClose");
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
    })
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

fn synthetic_runtime_file(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, 0);
    object_stream(&mut bytes);
    bytes
}

fn synthetic_runtime_header(major: u64, minor: u64, file_id: u64, toc_keys: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, major);
    push_var_uint(&mut bytes, minor);
    push_var_uint(&mut bytes, file_id);
    for key in toc_keys {
        push_var_uint(&mut bytes, *key);
    }
    push_var_uint(&mut bytes, 0);
    for _ in 0..((toc_keys.len() + 3) / 4) {
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }
    bytes
}

fn synthetic_runtime_file_with_single_toc_field(
    file_id: u64,
    property_key: u64,
    field_id: u8,
    object_stream: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, property_key);
    push_var_uint(&mut bytes, 0);
    bytes.extend_from_slice(&u32::from(field_id).to_le_bytes());
    object_stream(&mut bytes);
    bytes
}

fn synthetic_runtime_file_with_toc_fields(
    file_id: u64,
    toc_fields: &[(u64, u8)],
    object_stream: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    for (property_key, _) in toc_fields {
        push_var_uint(&mut bytes, *property_key);
    }
    push_var_uint(&mut bytes, 0);

    let mut current_int = 0u32;
    let mut current_bit = 0;
    for (index, (_, field_id)) in toc_fields.iter().enumerate() {
        current_int |= u32::from(*field_id) << current_bit;
        current_bit += 2;
        if current_bit == 8 || index == toc_fields.len() - 1 {
            bytes.extend_from_slice(&current_int.to_le_bytes());
            current_int = 0;
            current_bit = 0;
        }
    }

    object_stream(&mut bytes);
    bytes
}

fn runtime_header_prefix(file_id: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, 0);
    bytes
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeImportResult {
    Success,
    UnsupportedVersion,
    Malformed,
}

fn cpp_import_result(probe: &Path, name: &str, bytes: &[u8]) -> ProbeImportResult {
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-import-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));

    let output = Command::new(probe)
        .arg("--no-advance")
        .arg("--file")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    let _ = std::fs::remove_file(&path);

    if output.status.success() {
        return ProbeImportResult::Success;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("result=unsupportedVersion") {
        ProbeImportResult::UnsupportedVersion
    } else if stderr.contains("result=malformed") {
        ProbeImportResult::Malformed
    } else {
        panic!(
            "failed to parse C++ import result for {name}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
    }
}

fn rust_import_result(bytes: &[u8]) -> ProbeImportResult {
    match read_runtime_file_with_error_kind(bytes) {
        Ok(_) => ProbeImportResult::Success,
        Err(error) => match error.kind() {
            RuntimeReadErrorKind::UnsupportedVersion => ProbeImportResult::UnsupportedVersion,
            RuntimeReadErrorKind::Malformed => ProbeImportResult::Malformed,
        },
    }
}

fn cpp_import_file_result(probe: &Path, path: &Path) -> Option<ProbeImportResult> {
    let output = Command::new(probe)
        .arg("--file")
        .arg(path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    if output.status.success() {
        return Some(ProbeImportResult::Success);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("result=unsupportedVersion") {
        Some(ProbeImportResult::UnsupportedVersion)
    } else if stderr.contains("result=malformed") {
        Some(ProbeImportResult::Malformed)
    } else {
        None
    }
}

fn cpp_failed_import_type_keys(probe: &Path, name: &str, bytes: &[u8]) -> Vec<u16> {
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-import-failures-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));

    let output = Command::new(probe)
        .arg("--no-advance")
        .arg("--file")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "C++ probe failed for {name}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    failed_import_type_keys_from_stderr(&output.stderr)
}

fn cpp_failed_import_file_type_keys(probe: &Path, path: &Path, label: &str) -> Vec<u16> {
    let output = Command::new(probe)
        .arg("--no-advance")
        .arg("--file")
        .arg(path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    assert!(
        output.status.success(),
        "C++ probe failed for {label}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    failed_import_type_keys_from_stderr(&output.stderr)
}

fn failed_import_type_keys_from_stderr(stderr: &[u8]) -> Vec<u16> {
    String::from_utf8_lossy(stderr)
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("Failed to import object of type ")?;
            let digits = rest
                .chars()
                .take_while(|character| character.is_ascii_digit())
                .collect::<String>();
            Some(
                digits
                    .parse()
                    .unwrap_or_else(|err| panic!("failed to parse C++ import type key: {err}")),
            )
        })
        .collect()
}

fn rust_dropped_import_type_keys(file: &RuntimeFile) -> Vec<u16> {
    file.objects
        .iter()
        .zip(&file.import_statuses)
        .filter_map(|(object, status)| match status {
            RuntimeImportStatus::Dropped { .. } => Some(
                object
                    .as_ref()
                    .expect("dropped runtime import status must have an object")
                    .type_key,
            ),
            RuntimeImportStatus::Imported | RuntimeImportStatus::NullObject => None,
        })
        .collect()
}

fn format_type_keys(type_keys: &[u16]) -> String {
    let formatted = type_keys
        .iter()
        .map(|type_key| {
            let name = definition_by_type_key(*type_key)
                .map(|definition| definition.name)
                .unwrap_or("<unknown>");
            format!("{type_key}({name})")
        })
        .collect::<Vec<_>>();
    format!("[{}]", formatted.join(", "))
}

#[test]
fn cpp_probe_matches_rust_artboard_metadata_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ artboard metadata comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    for fixture_path in [
        "minimal/long_name.riv",
        "minimal/two_artboards.riv",
        "graph/clipping_and_draw_order.riv",
        "graph/dependency_test.riv",
        "graph/draw_rule_cycle.riv",
        "animation/smi_test.riv",
        "animation/state_machine_transition.riv",
    ] {
        compare_artboard_metadata(&probe, fixture_path);
    }
}

fn compare_artboard_metadata(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_file(probe, fixture_path);
    let rust = read_rust_runtime_file(fixture_path);
    let rust_artboards = runtime_artboards(&rust);

    assert_eq!(
        cpp.artboards.len(),
        rust_artboards.len(),
        "artboard count mismatch for {fixture_path}"
    );

    for (index, (cpp_artboard, rust_artboard)) in
        cpp.artboards.iter().zip(&rust_artboards).enumerate()
    {
        assert_eq!(
            cpp_artboard.name,
            rust_artboard.string_property("name").unwrap_or_default(),
            "artboard {index} name mismatch for {fixture_path}"
        );
        assert_float_eq(
            cpp_artboard.width,
            rust_artboard.double_property("width"),
            &format!("artboard {index} width for {fixture_path}"),
        );
        assert_float_eq(
            cpp_artboard.height,
            rust_artboard.double_property("height"),
            &format!("artboard {index} height for {fixture_path}"),
        );
        compare_artboard_view_model_result(&rust, cpp_artboard, rust_artboard, index, fixture_path)
            .unwrap_or_else(|err| panic!("{err}"));
    }
}

#[test]
fn cpp_probe_matches_rust_artboard_object_slots_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ artboard object comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    for fixture_path in [
        "minimal/long_name.riv",
        "minimal/two_artboards.riv",
        "graph/clipping_and_draw_order.riv",
        "graph/dependency_test.riv",
        "graph/draw_rule_cycle.riv",
        "animation/smi_test.riv",
        "animation/state_machine_transition.riv",
    ] {
        compare_artboard_object_slots(&probe, fixture_path);
    }
}

fn compare_artboard_object_slots(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_file(probe, fixture_path);
    let rust = read_rust_runtime_file(fixture_path);
    compare_artboard_object_slots_from_imports(&cpp, &rust, fixture_path);
}

fn compare_artboard_object_slots_from_imports(cpp: &CppProbeFile, rust: &RuntimeFile, label: &str) {
    let ranges = runtime_artboard_ranges(&rust);

    assert_eq!(
        cpp.artboards.len(),
        ranges.len(),
        "artboard count mismatch for {label}"
    );

    for (artboard_index, (cpp_artboard, range)) in cpp.artboards.iter().zip(&ranges).enumerate() {
        let rust_objects = runtime_artboard_local_objects(&rust, *range);

        assert_eq!(
            cpp_artboard.object_count,
            rust_objects.len(),
            "artboard {artboard_index} local object count mismatch for {label}"
        );
        assert_eq!(
            cpp_artboard.objects.len(),
            rust_objects.len(),
            "artboard {artboard_index} C++ object list length mismatch for {label}"
        );

        for (local_id, (cpp_object, rust_object)) in
            cpp_artboard.objects.iter().zip(&rust_objects).enumerate()
        {
            match (cpp_object, rust_object) {
                (None, None) => {}
                (Some(cpp_object), Some(rust_object)) => {
                    assert_eq!(
                        cpp_object.local_id, local_id,
                        "C++ local id mismatch at object {local_id} in artboard {artboard_index} for {label}"
                    );
                    assert_eq!(
                        cpp_object.core_type, rust_object.type_key,
                        "object {local_id} core type mismatch in artboard {artboard_index} for {label}"
                    );
                    assert_eq!(
                        cpp_object.is_component,
                        runtime_object_is_component(rust_object),
                        "object {local_id} component flag mismatch in artboard {artboard_index} for {label}"
                    );
                    assert_eq!(
                        cpp_object.source_artboard_index,
                        resolved_artboard_index_for_referencer(rust, rust_object),
                        "object {local_id} resolved nested artboard mismatch in artboard {artboard_index} for {label}"
                    );
                    assert_eq!(
                        cpp_object.data_bind_path_ids,
                        data_bind_path_ids_for_referencer(rust, rust_object),
                        "object {local_id} data-bind path mismatch in artboard {artboard_index} for {label}"
                    );
                    assert_eq!(
                        cpp_object.resolved_data_bind_path_ids,
                        resolved_data_bind_path_ids_for_referencer(rust, rust_object),
                        "object {local_id} resolved data-bind path mismatch in artboard {artboard_index} for {label}"
                    );
                    assert_eq!(
                        cpp_object.scroll_physics_core_type,
                        scroll_physics_core_type_for_constraint(rust, rust_object),
                        "object {local_id} scroll physics core type mismatch in artboard {artboard_index} for {label}"
                    );
                }
                (None, Some(rust_object)) => panic!(
                    "C++ has null object but Rust has {} at local id {local_id} in artboard {artboard_index} for {label}",
                    rust_object.type_name
                ),
                (Some(cpp_object), None) => panic!(
                    "Rust has null object but C++ has core type {} at local id {local_id} in artboard {artboard_index} for {label}",
                    cpp_object.core_type
                ),
            }
        }

        let rust_components = rust_objects
            .iter()
            .enumerate()
            .filter_map(|(local_id, object)| {
                let object = object.as_ref()?;
                runtime_object_is_component(object).then_some((local_id, *object))
            })
            .collect::<BTreeMap<_, _>>();

        assert_eq!(
            cpp_artboard.components.len(),
            rust_components.len(),
            "artboard {artboard_index} component count mismatch for {label}"
        );

        for cpp_component in &cpp_artboard.components {
            let rust_component = rust_components
                .get(&cpp_component.local_id)
                .unwrap_or_else(|| {
                    panic!(
                        "missing Rust component local id {} in artboard {artboard_index} for {label}",
                        cpp_component.local_id
                    )
                });
            let rust_parent_id = rust_component.uint_property("parentId").unwrap_or(0);
            let rust_parent_local = if rust_component.type_name == "Artboard" {
                None
            } else {
                Some(rust_parent_id as usize)
            };

            assert_eq!(
                cpp_component.core_type, rust_component.type_key,
                "component {} core type mismatch in artboard {artboard_index} for {label}",
                cpp_component.local_id
            );
            assert_eq!(
                cpp_component.name,
                rust_component.string_property("name").unwrap_or_default(),
                "component {} name mismatch in artboard {artboard_index} for {label}",
                cpp_component.local_id
            );
            assert_eq!(
                cpp_component.parent_id, rust_parent_id,
                "component {} parent id mismatch in artboard {artboard_index} for {label}",
                cpp_component.local_id
            );
            assert_eq!(
                cpp_component.parent_local, rust_parent_local,
                "component {} resolved parent mismatch in artboard {artboard_index} for {label}",
                cpp_component.local_id
            );
        }
    }
}

#[test]
fn cpp_probe_matches_rust_validate_objects_null_slots_for_synthetic_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ validateObjects slot comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "validate_objects_null_slots";
    let bytes = synthetic_runtime_file(6071, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
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
        push_object_with_properties(bytes, "TextStyle", |bytes| {
            push_uint_property(bytes, "TextStyle", "parentId", 1);
        });
        push_object_with_properties(bytes, "TextStylePaint", |bytes| {
            push_uint_property(bytes, "TextStylePaint", "parentId", 1);
        });
        push_object_with_properties(bytes, "NestedLinearAnimation", |bytes| {
            push_uint_property(bytes, "NestedLinearAnimation", "parentId", 1);
        });
        push_object_with_properties(bytes, "ScrollBarConstraint", |bytes| {
            push_uint_property(bytes, "ScrollBarConstraint", "parentId", 1);
            push_uint_property(bytes, "ScrollBarConstraint", "scrollConstraintId", 99);
        });
        push_object_with_properties(bytes, "Feather", |bytes| {
            push_uint_property(bytes, "Feather", "parentId", 1);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-validate-objects-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic validateObjects stream");

    compare_artboard_object_slots_from_imports(&cpp, &rust, name);
}

#[test]
fn cpp_probe_matches_rust_resolved_data_bind_paths_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ resolved data-bind path comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let mut manifest_paths = Vec::new();
    push_var_uint(&mut manifest_paths, 2);
    push_var_uint(&mut manifest_paths, 77);
    push_var_uint(&mut manifest_paths, 3);
    for path_id in [4, 512, 9] {
        push_var_uint(&mut manifest_paths, path_id);
    }
    push_var_uint(&mut manifest_paths, u32::MAX as u64);
    push_var_uint(&mut manifest_paths, 2);
    for path_id in [13, 21] {
        push_var_uint(&mut manifest_paths, path_id);
    }

    let mut manifest_bytes = Vec::new();
    push_var_uint(&mut manifest_bytes, 1);
    push_var_uint(&mut manifest_bytes, manifest_paths.len() as u64);
    manifest_bytes.extend_from_slice(&manifest_paths);

    let mut encoded_claimed_path = Vec::new();
    push_var_uint(&mut encoded_claimed_path, 77);

    let mut encoded_missing_path = Vec::new();
    push_var_uint(&mut encoded_missing_path, 404);

    let mut encoded_multi_id_path = Vec::new();
    push_var_uint(&mut encoded_multi_id_path, 77);
    push_var_uint(&mut encoded_multi_id_path, 9);

    let mut encoded_wrapped_path = Vec::new();
    push_var_uint(&mut encoded_wrapped_path, u32::MAX as u64);

    let name = "resolved_data_bind_paths";
    let bytes = synthetic_runtime_file(6090, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_claimed_path);
        });
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_missing_path);
        });
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_multi_id_path);
        });
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "DataBindPath", |bytes| {
            push_bytes_property(bytes, "DataBindPath", "path", &encoded_wrapped_path);
        });
        push_empty_object(bytes, "StateMachineListenerSingle");
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-resolved-data-bind-paths-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic resolved path stream");

    assert_eq!(cpp.artboards.len(), 1);
    assert_eq!(cpp.artboards[0].state_machines.len(), 1);
    let rust_state_machines = rust.artboard_state_machine_graphs(0);
    assert_eq!(rust_state_machines.len(), 1);
    compare_state_machine_children(
        &rust,
        &cpp.artboards[0].state_machines[0],
        &rust_state_machines[0],
        name,
    );

    let listener = rust.object(6).expect("StateMachineListenerSingle object");
    let path = rust
        .data_bind_path_for_referencer_object(listener)
        .expect("listener claims authored DataBindPath");
    assert_eq!(path.path_ids, vec![77]);
    assert_eq!(path.resolved_path_ids, vec![4, 512, 9]);
    assert_eq!(
        rust.resolved_data_bind_path_ids_for_referencer_object(listener),
        Some(vec![4, 512, 9])
    );

    let missing_listener = rust
        .object(8)
        .expect("StateMachineListenerSingle with missing manifest path");
    let missing_path = rust
        .data_bind_path_for_referencer_object(missing_listener)
        .expect("listener claims missing manifest path");
    assert_eq!(missing_path.path_ids, vec![404]);
    assert_eq!(missing_path.resolved_path_ids, Vec::<u32>::new());
    assert_eq!(
        rust.resolved_data_bind_path_ids_for_referencer_object(missing_listener),
        Some(Vec::<u32>::new())
    );

    let multi_listener = rust
        .object(10)
        .expect("StateMachineListenerSingle with multi-id path");
    let multi_path = rust
        .data_bind_path_for_referencer_object(multi_listener)
        .expect("listener claims multi-id authored path");
    assert_eq!(multi_path.path_ids, vec![77, 9]);
    assert_eq!(
        multi_path.resolved_path_ids,
        vec![77, 9],
        "C++ only expands single-id paths through the manifest data resolver"
    );
    assert_eq!(
        rust.resolved_data_bind_path_ids_for_referencer_object(multi_listener),
        Some(vec![77, 9])
    );

    let wrapped_listener = rust
        .object(12)
        .expect("StateMachineListenerSingle with wrapped manifest path id");
    let wrapped_path = rust
        .data_bind_path_for_referencer_object(wrapped_listener)
        .expect("listener claims wrapped manifest path");
    assert_eq!(wrapped_path.path_ids, vec![u32::MAX]);
    assert_eq!(
        wrapped_path.resolved_path_ids,
        vec![13, 21],
        "C++ resolves u32::MAX through ManifestAsset's signed int key -1"
    );
    assert_eq!(
        rust.resolved_data_bind_path_ids_for_referencer_object(wrapped_listener),
        Some(vec![13, 21])
    );
}

#[test]
fn cpp_probe_matches_rust_listener_input_type_view_model_path_ids_buffer_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ listener input type view-model path buffer comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let mut encoded_path = Vec::new();
    for path_id in [11, 222, 5] {
        push_var_uint(&mut encoded_path, path_id);
    }

    let name = "listener_input_type_view_model_path_ids_buffer";
    let bytes = synthetic_runtime_file(6091, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "StateMachine");
        push_empty_object(bytes, "StateMachineListenerSingle");
        push_object_with_properties(bytes, "ListenerInputTypeViewModel", |bytes| {
            push_bytes_property(
                bytes,
                "ListenerInputTypeViewModel",
                "viewModelPathIds",
                &encoded_path,
            );
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-listener-input-type-view-model-path-ids-buffer-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic listener input type path stream");

    assert_eq!(cpp.artboards.len(), 1);
    assert_eq!(cpp.artboards[0].state_machines.len(), 1);
    let rust_state_machines = rust.artboard_state_machine_graphs(0);
    assert_eq!(rust_state_machines.len(), 1);
    compare_state_machine_children(
        &rust,
        &cpp.artboards[0].state_machines[0],
        &rust_state_machines[0],
        name,
    );

    let input_type = rust.object(4).expect("ListenerInputTypeViewModel object");
    assert_eq!(
        rust.listener_input_type_view_model_path_ids_buffer_for_object(input_type),
        Some(vec![11, 222, 5])
    );
}

#[test]
fn cpp_probe_matches_rust_file_assets_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ file asset comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    for fixture_path in [
        "align_target.riv",
        "bad_skin.riv",
        "hosted_image_file.riv",
        "library_with_text_and_image.riv",
    ] {
        compare_file_assets(&probe, fixture_path);
    }
}

fn compare_file_assets(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_reference_file(probe, fixture_path);
    let rust = read_rust_reference_runtime_file(fixture_path);
    let rust_assets = runtime_file_assets(&rust);

    assert_eq!(
        cpp.assets.len(),
        rust_assets.len(),
        "file asset count mismatch for reference fixture {fixture_path}"
    );

    for (asset_index, (cpp_asset, rust_asset)) in cpp.assets.iter().zip(&rust_assets).enumerate() {
        assert_eq!(
            cpp_asset.index, asset_index,
            "C++ asset index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_asset.core_type, rust_asset.type_key,
            "asset {asset_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_asset.name,
            rust_asset.string_property("name").unwrap_or_default(),
            "asset {asset_index} name mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            rust_asset.uint_property("assetId"),
            Some(cpp_asset.asset_id),
            "asset {asset_index} asset id mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_asset.cdn_base_url,
            rust_asset.string_property("cdnBaseUrl").unwrap_or_default(),
            "asset {asset_index} cdn base url mismatch for reference fixture {fixture_path}"
        );
    }
}

#[test]
fn cpp_probe_matches_rust_skinning_relationships_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ skinning relationship comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "skinning_relationships";
    let bytes = synthetic_skin_runtime_file(6094);
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-skinning-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic skinning stream");

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_skins(&cpp.artboards[0].skins, &rust.artboard_skins(0), name);
}

#[test]
fn cpp_probe_matches_rust_mesh_vertex_weights_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ mesh vertex weight comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "mesh_vertex_weights";
    let bytes = synthetic_weighted_mesh_runtime_file(6095);
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-mesh-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic weighted mesh stream");

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_meshes(&cpp.artboards[0].meshes, &rust.artboard_meshes(0), name);
}

#[test]
fn cpp_probe_matches_rust_path_vertex_weights_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ path vertex weight comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "path_vertex_weights";
    let bytes = synthetic_weighted_path_runtime_file(6096);
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-path-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic weighted path stream");

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_paths(&cpp.artboards[0].paths, &rust.artboard_paths(0), name);
}

#[test]
fn cpp_probe_matches_rust_n_slicer_details_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ NSlicer comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    let name = "n_slicer_details";
    let bytes = synthetic_n_slicer_runtime_file(6110, |bytes| {
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
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "NSlicer", |bytes| {
            push_uint_property(bytes, "NSlicer", "parentId", 9);
        });
        push_object_with_properties(bytes, "AxisX", |bytes| {
            push_uint_property(bytes, "AxisX", "parentId", 9);
        });
        push_object_with_properties(bytes, "NSlicerTileMode", |bytes| {
            push_uint_property(bytes, "NSlicerTileMode", "parentId", 9);
            push_uint_property(bytes, "NSlicerTileMode", "patchIndex", 4);
            push_uint_property(bytes, "NSlicerTileMode", "style", 1);
        });
    });
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-n-slicer-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_with_options(
        &probe,
        &path,
        name,
        CppProbePropertyValueMode::None,
        true,
        false,
    );
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic NSlicer stream");

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_n_slicer_details(
        &cpp.artboards[0].n_slicer_details,
        &rust.artboard_n_slicer_details(0),
        name,
    );
}

#[test]
fn cpp_probe_matches_rust_shape_paint_relationships_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ shape paint comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

    let name = "shape_paint_relationships";
    let bytes = synthetic_shape_paint_runtime_file(6097);
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-shape-paint-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_with_options(
        &probe,
        &path,
        name,
        CppProbePropertyValueMode::None,
        true,
        false,
    );
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic shape paint stream");

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_shapes(&cpp.artboards[0].shapes, &rust.artboard_shapes(0), name);
}

#[test]
fn cpp_probe_matches_rust_shape_paint_containers_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ shape paint container comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "shape_paint_containers";
    let bytes = synthetic_shape_paint_runtime_file(6098);
    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-shape-paint-containers-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_with_options(
        &probe,
        &path,
        name,
        CppProbePropertyValueMode::None,
        true,
        false,
    );
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic shape paint stream");

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_shape_paint_containers(
        &cpp.artboards[0].shape_paint_containers,
        &rust.artboard_shape_paint_containers(0),
        name,
    );
}

#[test]
fn local_bounds_shape_relationships_match_cpp_probe_regression() {
    let fixture_path = reference_runtime_dir()
        .join("tests")
        .join("unit_tests")
        .join("assets")
        .join("local_bounds.riv");
    if !fixture_path.exists() {
        eprintln!(
            "skipping local_bounds shape relationship regression; fixture not found at {}",
            fixture_path.display()
        );
        return;
    }

    let bytes = std::fs::read(&fixture_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display()));
    let rust = read_runtime_file(&bytes).expect("Rust imports local_bounds.riv");

    assert_eq!(
        rust.artboard_paths(0)
            .iter()
            .map(|path| (path.local_id, path.object.type_name))
            .collect::<Vec<_>>(),
        vec![
            (3, "PointsPath"),
            (14, "PointsPath"),
            (37, "Triangle"),
            (41, "Ellipse"),
            (44, "Rectangle"),
        ],
        "C++ Path::onAddedClean registers concrete Path objects without text runs"
    );

    assert_eq!(
        rust.artboard_shapes(0)
            .iter()
            .map(|shape| {
                (
                    shape.local_id,
                    shape
                        .paths
                        .iter()
                        .map(|path| path.local_id)
                        .collect::<Vec<_>>(),
                    shape
                        .paints
                        .iter()
                        .map(|paint| (paint.local_id, paint.mutator_local_id))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (2, vec![3], vec![(59, Some(8))]),
            (13, vec![14], vec![(60, Some(19))]),
            (36, vec![37], vec![(63, Some(38))]),
            (40, vec![41], vec![(64, Some(42))]),
            (43, vec![44], vec![(65, Some(45))]),
        ],
        "C++ ShapePaint::onAddedClean registers fills whose mutators resolved earlier"
    );
}

#[test]
fn cpp_probe_matches_rust_file_view_models_and_enums_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ file view model comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    for fixture_path in [
        "car_widgets_v01.riv",
        "databind_solo_to_enum.riv",
        "viewmodel_runtime_file.riv",
    ] {
        compare_file_view_models_and_enums(&probe, fixture_path);
    }
}

#[test]
fn cpp_probe_matches_rust_data_enum_lookups_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data-enum lookup comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "data_enum_lookups";
    let bytes = synthetic_runtime_file(6093, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataEnumCustom", |bytes| {
            push_string_property(bytes, "DataEnumCustom", "name", "Choice");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "First Label");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "empty");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "Duplicate Label");
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-data-enum-lookups-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic data-enum stream");

    assert_eq!(cpp.enums.len(), 1);
    assert_eq!(rust.data_enums().len(), 1);
    assert_eq!(cpp.enums[0].values.len(), 3);
    compare_data_enum_lookup_result(&rust, &cpp.enums[0], 0, name)
        .unwrap_or_else(|err| panic!("{err}"));

    assert_eq!(
        rust.data_enum_value_for_key(0, "first"),
        Some("First Label")
    );
    assert_eq!(rust.data_enum_value_index_for_key(0, "first"), Some(0));
    assert_eq!(rust.data_enum_value_for_key(0, "empty"), Some("empty"));
    assert_eq!(rust.data_enum_value_for_index(0, 1), Some(&b"empty"[..]));
    assert_eq!(
        rust.data_enum_value_for_index(0, 2),
        Some(&b"Duplicate Label"[..])
    );
    assert_eq!(rust.data_enum_value_index_for_index(0, 2), Some(2));
    assert_eq!(rust.data_enum_value_for_key(0, "__missing__"), None);
    assert_eq!(rust.data_enum_value_index_for_key(0, "__missing__"), None);
    assert_eq!(rust.data_enum_value_for_index(0, 99), None);
    assert_eq!(rust.data_enum_value_index_for_index(0, 99), None);
}

#[test]
fn cpp_probe_matches_rust_view_model_property_enum_lookups_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ view-model enum property lookup comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "view_model_property_enum_lookups";
    let bytes = synthetic_runtime_file(6094, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyEnumCustom", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumCustom", "name", "early");
            push_uint_property(bytes, "ViewModelPropertyEnumCustom", "enumId", 0);
        });
        push_object_with_properties(bytes, "DataEnumCustom", |bytes| {
            push_string_property(bytes, "DataEnumCustom", "name", "Choice");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "First Label");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "empty");
        });
        push_object_with_properties(bytes, "ViewModelPropertyEnumCustom", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumCustom", "name", "late");
            push_uint_property(bytes, "ViewModelPropertyEnumCustom", "enumId", 0);
        });
        push_object_with_properties(bytes, "ViewModelPropertyEnumCustom", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumCustom", "name", "missing");
            push_uint_property(bytes, "ViewModelPropertyEnumCustom", "enumId", 99);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-view-model-property-enum-lookups-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic enum-property stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert_eq!(cpp.view_models.len(), 1);
    assert_eq!(rust_view_models.len(), 1);
    assert_eq!(cpp.enums.len(), 1);
    assert_eq!(rust.data_enums().len(), 1);
    assert_eq!(cpp.view_models[0].properties.len(), 3);
    for (property_index, (cpp_property, rust_property)) in cpp.view_models[0]
        .properties
        .iter()
        .zip(&rust_view_models[0].properties)
        .enumerate()
    {
        compare_view_model_property_enum_lookup_result(
            &rust,
            cpp_property,
            rust_property,
            &format!("{name}: property {property_index}"),
        )
        .unwrap_or_else(|err| panic!("{err}"));
    }

    let early = rust.view_model_property_named(0, "early").unwrap();
    let late = rust.view_model_property_named(0, "late").unwrap();
    let missing = rust.view_model_property_named(0, "missing").unwrap();

    assert!(
        rust.data_enum_for_view_model_property_object(early)
            .is_none()
    );
    assert_eq!(
        rust.view_model_property_enum_value_for_key_bytes_object(early, b"first"),
        None,
        "C++ does not backfill enum properties that appear before their enum"
    );
    assert_eq!(
        rust.view_model_property_enum_value_for_key_object(late, "first"),
        Some("First Label")
    );
    assert_eq!(
        rust.view_model_property_enum_value_index_for_key_bytes_object(late, b"first"),
        Some(0)
    );
    assert_eq!(
        rust.view_model_property_enum_value_for_key_object(late, "empty"),
        Some("empty")
    );
    assert_eq!(
        rust.view_model_property_enum_value_for_index_object(late, 1),
        Some(&b"empty"[..])
    );
    assert_eq!(
        rust.view_model_property_enum_value_index_for_index_object(late, 1),
        Some(1)
    );
    assert!(
        rust.data_enum_for_view_model_property_object(missing)
            .is_none()
    );
    assert_eq!(
        rust.view_model_property_enum_value_for_key_object(missing, "first"),
        None
    );
}

#[test]
fn cpp_probe_matches_rust_view_model_instance_enum_runtime_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ view-model enum instance runtime comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "view_model_instance_enum_runtime";
    let bytes = synthetic_runtime_file(6095, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "DataEnumCustom", |bytes| {
            push_string_property(bytes, "DataEnumCustom", "name", "Choice");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "First Label");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "second");
            push_string_property(bytes, "DataEnumValue", "value", "Second Label");
        });
        push_object_with_properties(bytes, "ViewModelPropertyEnumCustom", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumCustom", "name", "choice");
            push_uint_property(bytes, "ViewModelPropertyEnumCustom", "enumId", 0);
        });
        push_object_with_properties(bytes, "ViewModelPropertyEnumSystem", |bytes| {
            push_string_property(bytes, "ViewModelPropertyEnumSystem", "name", "system");
            push_uint_property(bytes, "ViewModelPropertyEnumSystem", "enumType", 7);
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 0);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 0);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 99);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 99);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceEnum", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceEnum", "viewModelPropertyId", 1);
            push_uint_property(bytes, "ViewModelInstanceEnum", "propertyValue", 99);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-view-model-instance-enum-runtime-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_with_options(
        &probe,
        &path,
        name,
        CppProbePropertyValueMode::None,
        false,
        true,
    );
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic enum instance stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert_eq!(cpp.view_models.len(), 1);
    assert_eq!(rust_view_models.len(), 1);
    assert_eq!(cpp.view_models[0].instances.len(), 1);
    assert_eq!(rust_view_models[0].instances.len(), 1);

    let cpp_values = &cpp.view_models[0].instances[0].values;
    let rust_values = &rust_view_models[0].instances[0].values;
    assert_eq!(cpp_values.len(), 4);
    assert_eq!(rust_values.len(), 4);
    for (value_index, (cpp_value, rust_value)) in cpp_values.iter().zip(rust_values).enumerate() {
        compare_view_model_instance_enum_runtime_result(
            &rust,
            cpp_value,
            rust_value.object,
            &format!("{name}: value {value_index}"),
        )
        .unwrap_or_else(|err| panic!("{err}"));
    }

    assert_eq!(
        rust.view_model_instance_enum_value_key_string_for_object(rust_values[0].object),
        Some("second")
    );
    assert_eq!(
        rust.view_model_instance_enum_value_index_for_object(rust_values[0].object),
        Some(1)
    );
    assert_eq!(
        rust.view_model_instance_enum_value_keys_for_object(rust_values[0].object),
        Some(vec![&b"first"[..], &b"second"[..]])
    );
    assert_eq!(
        rust.view_model_instance_enum_type_for_object(rust_values[0].object),
        Some(&b"Choice"[..])
    );
    assert_eq!(
        rust.view_model_instance_enum_value_key_for_object(rust_values[1].object),
        Some(&b""[..])
    );
    assert_eq!(
        rust.view_model_instance_enum_value_index_for_object(rust_values[1].object),
        Some(0)
    );
    assert!(
        rust.data_enum_for_view_model_instance_enum_value_object(rust_values[2].object)
            .is_none()
    );
    assert!(
        rust.data_enum_for_view_model_instance_enum_value_object(rust_values[3].object)
            .is_none(),
        "C++ system enum properties use a static empty DataEnum, not a file enum"
    );
    assert_eq!(
        rust.view_model_instance_enum_value_key_for_object(rust_values[3].object),
        Some(&b""[..])
    );
    assert_eq!(
        rust.view_model_instance_enum_value_index_for_object(rust_values[3].object),
        Some(0)
    );
    assert_eq!(
        rust.view_model_instance_enum_value_keys_for_object(rust_values[3].object),
        Some(Vec::new())
    );
    assert_eq!(
        rust.view_model_instance_enum_type_for_object(rust_values[3].object),
        Some(&b""[..])
    );
}

#[test]
fn cpp_probe_matches_rust_view_model_instance_values_for_synthetic_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ view-model instance value comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "view_model_instance_values";
    let bytes = synthetic_runtime_file(6072, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Item");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "ViewModelPropertyList", |bytes| {
            push_string_property(bytes, "ViewModelPropertyList", "name", "items");
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
        });
        push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
            push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
        });
        push_object_with_properties(bytes, "ViewModelPropertyTrigger", |bytes| {
            push_string_property(bytes, "ViewModelPropertyTrigger", "name", "fire");
        });
        push_object_with_properties(bytes, "ViewModelPropertySymbolListIndex", |bytes| {
            push_string_property(bytes, "ViewModelPropertySymbolListIndex", "name", "symbol");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_float_property(bytes, "ViewModelInstanceNumber", "propertyValue", 12.5);
        });
        push_object_with_properties(bytes, "ViewModelInstanceList", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceList", "viewModelPropertyId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceListItem", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelId", 0);
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 2);
            push_bool_property(bytes, "ViewModelInstanceBoolean", "propertyValue", true);
        });
        push_object_with_properties(bytes, "ViewModelInstanceColor", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceColor", "viewModelPropertyId", 3);
            push_color_property(
                bytes,
                "ViewModelInstanceColor",
                "propertyValue",
                0x1122_3344,
            );
        });
        push_object_with_properties(bytes, "ViewModelInstanceTrigger", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceTrigger", "viewModelPropertyId", 4);
            push_uint_property(bytes, "ViewModelInstanceTrigger", "propertyValue", 3);
        });
        push_object_with_properties(bytes, "ViewModelInstanceSymbolListIndex", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "viewModelPropertyId",
                5,
            );
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "propertyValue",
                4,
            );
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "item");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
            push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "hello");
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
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-view-model-instance-values-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic view-model instance stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert_eq!(cpp.view_models.len(), rust_view_models.len());
    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        compare_view_model_instances(
            &rust,
            &cpp_view_model.instances,
            &rust_view_model.instances,
            view_model_index,
            name,
        );
    }
}

#[test]
fn cpp_probe_matches_rust_artboard_component_list_map_rules_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ artboard component list map-rule comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "artboard_component_list_map_rules";
    let bytes = synthetic_runtime_file(6092, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "ExplicitItem");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "FallbackItem");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "explicit");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "fallback");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 2);
        });
        push_object_with_properties(bytes, "ViewModelInstanceList", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceList", "viewModelPropertyId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceListItem", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelId", 0);
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceListItem", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelId", 1);
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelInstanceId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceListItem", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceListItem", "viewModelId", 1);
            push_uint_property(
                bytes,
                "ViewModelInstanceListItem",
                "viewModelInstanceId",
                99,
            );
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "FallbackArtboard");
            push_uint_property(bytes, "Artboard", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ArtboardComponentList", |bytes| {
            push_uint_property(bytes, "ArtboardComponentList", "parentId", 0);
        });
        push_object_with_properties(bytes, "ArtboardListMapRule", |bytes| {
            push_uint_property(bytes, "ArtboardListMapRule", "parentId", 1);
            push_uint_property(bytes, "ArtboardListMapRule", "viewModelId", 0);
            push_uint_property(bytes, "ArtboardListMapRule", "artboardId", 1);
        });
        push_object_with_properties(bytes, "Artboard", |bytes| {
            push_string_property(bytes, "Artboard", "name", "ExplicitArtboard");
            push_uint_property(bytes, "Artboard", "viewModelId", 99);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-artboard-component-list-map-rules-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes)
        .expect("Rust imports synthetic artboard component list map-rule stream");

    assert_eq!(cpp.artboards.len(), 2);
    assert_eq!(cpp.artboards[0].artboard_component_lists.len(), 1);
    let cpp_list = &cpp.artboards[0].artboard_component_lists[0];
    let range =
        runtime_artboard_range_by_index(&rust, 0).expect("first artboard range should exist");
    let rust_local_objects = runtime_artboard_local_objects(&rust, range);
    let rust_list = rust_local_objects
        .get(cpp_list.local_id)
        .and_then(|object| *object)
        .expect("Rust artboard component list local object should exist");
    compare_artboard_component_list_result(&rust, cpp_list, rust_list, name)
        .unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn cpp_probe_matches_rust_view_model_instance_value_property_links_when_completed() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ view-model instance value property comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "view_model_instance_value_property_links";
    let bytes = synthetic_runtime_file(6081, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Item");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
            push_uint_property(bytes, "ViewModelPropertyNumber", "symbolTypeValue", 16);
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
            push_uint_property(bytes, "ViewModelPropertyString", "symbolTypeValue", 16);
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "label");
        });
        push_object_with_properties(bytes, "ViewModelPropertySymbolListIndex", |bytes| {
            push_string_property(
                bytes,
                "ViewModelPropertySymbolListIndex",
                "name",
                "item-index",
            );
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 2);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceSymbolListIndex", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceSymbolListIndex",
                "viewModelPropertyId",
                3,
            );
        });
        push_object_with_properties(bytes, "ViewModelInstanceBoolean", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceBoolean", "viewModelPropertyId", 99);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-view-model-instance-value-property-links-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_with_options(
        &probe,
        &path,
        name,
        CppProbePropertyValueMode::None,
        false,
        true,
    );
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic view-model property link stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert_eq!(cpp.view_models.len(), rust_view_models.len());
    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        compare_view_model_instances(
            &rust,
            &cpp_view_model.instances,
            &rust_view_model.instances,
            view_model_index,
            name,
        );
        compare_view_model_name_lookups(
            &rust,
            cpp_view_model,
            rust_view_model,
            view_model_index,
            name,
        );
        compare_view_model_symbol_lookups(
            &rust,
            cpp_view_model,
            rust_view_model,
            view_model_index,
            name,
        );
        compare_view_model_default_instance(
            &rust,
            cpp_view_model,
            rust_view_model,
            view_model_index,
            name,
        );
    }

    let cpp_values = &cpp.view_models[1].instances[0].values;
    let rust_values = &rust_view_models[1].instances[0].values;
    assert_view_model_instance_value_property_link(
        &rust,
        &cpp_values[0],
        &rust_values[0],
        "number property link",
    );
    assert_view_model_instance_value_property_link(
        &rust,
        &cpp_values[1],
        &rust_values[1],
        "duplicate-label boolean property link",
    );
    assert_view_model_instance_value_property_link(
        &rust,
        &cpp_values[2],
        &rust_values[2],
        "duplicate-label string property link",
    );
    assert_view_model_instance_value_property_link(
        &rust,
        &cpp_values[3],
        &rust_values[3],
        "duplicate-id number property link",
    );
    assert_view_model_instance_value_property_link(
        &rust,
        &cpp_values[4],
        &rust_values[4],
        "symbol-list-index property link",
    );
    assert_view_model_instance_value_property_link(
        &rust,
        &cpp_values[5],
        &rust_values[5],
        "missing property link",
    );
}

#[test]
fn cpp_probe_matches_rust_state_machine_layer_transition_graph_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ state-machine layer transition graph comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "state_machine_layer_transition_graph";
    let bytes = synthetic_runtime_file(6082, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "Artboard");
        push_empty_object(bytes, "CubicEaseInterpolator");
        push_empty_object(bytes, "LinearAnimation");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "OpenUrlEvent", |bytes| {
            push_uint_property(bytes, "OpenUrlEvent", "parentId", 2);
        });
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
            push_uint_property(bytes, "StateTransition", "stateToId", 1);
        });
        push_object_with_properties(bytes, "StateMachineFireEvent", |bytes| {
            push_uint_property(bytes, "StateMachineFireEvent", "eventId", 3);
            push_uint_property(bytes, "StateMachineFireEvent", "occursValue", 1);
        });
        push_object_with_properties(bytes, "ListenerFireEvent", |bytes| {
            push_uint_property(bytes, "ListenerFireEvent", "eventId", 3);
            push_uint_property(bytes, "ListenerFireEvent", "flags", 2);
        });
        push_empty_object(bytes, "LayerState");
        push_empty_object(bytes, "EntryState");
        push_object_with_properties(bytes, "StateMachineFireEvent", |bytes| {
            push_uint_property(bytes, "StateMachineFireEvent", "eventId", 2);
            push_uint_property(bytes, "StateMachineFireEvent", "occursValue", 2);
        });
        push_object_with_properties(bytes, "ListenerFireEvent", |bytes| {
            push_uint_property(bytes, "ListenerFireEvent", "eventId", 2);
            push_uint_property(bytes, "ListenerFireEvent", "flags", 4);
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 6);
            push_uint_property(bytes, "StateTransition", "interpolatorId", 1);
        });
        push_object_with_properties(bytes, "TransitionBoolCondition", |bytes| {
            push_uint_property(bytes, "TransitionBoolCondition", "inputId", 0);
        });
        push_empty_object(bytes, "ScriptedTransitionCondition");
        push_object_with_properties(bytes, "ScriptInputBoolean", |bytes| {
            push_string_property(bytes, "ScriptInputBoolean", "name", "condition-input");
        });
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", 6);
            push_uint_property(bytes, "StateTransition", "interpolatorId", 2);
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
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 6);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 0);
        });
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 6);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 1);
        });
        push_object_with_properties(bytes, "BlendStateTransition", |bytes| {
            push_uint_property(bytes, "BlendStateTransition", "stateToId", 6);
            push_uint_property(bytes, "BlendStateTransition", "exitBlendAnimationId", 99);
        });
        push_empty_object(bytes, "ExitState");
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-state-machine-layer-transition-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic state-machine transition graph");
    let rust_state_machines = rust.artboard_state_machine_graphs(0);

    assert_eq!(cpp.artboards.len(), 1);
    assert_eq!(cpp.artboards[0].state_machines.len(), 1);
    assert_eq!(rust_state_machines.len(), 1);
    compare_corpus_state_machine(
        &rust,
        &cpp.artboards[0].state_machines[0],
        &rust_state_machines[0],
        0,
        0,
        name,
    )
    .unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn cpp_probe_matches_rust_data_converter_formula_tokens_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data converter formula token comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "data_converter_formula_tokens";
    let bytes = synthetic_runtime_file(6086, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "FormulaTokenValue");
        push_empty_object(bytes, "FormulaTokenOperation");
        push_empty_object(bytes, "FormulaTokenValue");
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 0);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-data-converter-formula-tokens-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic formula converter tokens");
    let rust_data_binds = rust.artboard_data_binds(0);

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_data_binds(&rust, &cpp.artboards[0].data_binds, &rust_data_binds, name);
}

#[test]
fn cpp_probe_matches_rust_data_converter_output_types_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data converter output-type comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let direct_converters = [
        ("DataConverterRounder", RuntimeDataType::Number),
        ("DataConverterToString", RuntimeDataType::String),
        ("DataConverterOperationValue", RuntimeDataType::Number),
        ("DataConverterSystemDegsToRads", RuntimeDataType::Number),
        ("DataConverterSystemNormalizer", RuntimeDataType::Number),
        ("DataConverterOperation", RuntimeDataType::Number),
        ("DataConverterOperationViewModel", RuntimeDataType::Number),
        ("DataConverterRangeMapper", RuntimeDataType::Number),
        ("DataConverterStringPad", RuntimeDataType::String),
        ("DataConverterStringRemoveZeros", RuntimeDataType::String),
        ("DataConverterStringTrim", RuntimeDataType::String),
        ("DataConverterInterpolator", RuntimeDataType::Input),
        ("DataConverterBooleanNegate", RuntimeDataType::Boolean),
        ("DataConverterFormula", RuntimeDataType::Number),
        ("DataConverterNumberToList", RuntimeDataType::List),
        ("DataConverterListToLength", RuntimeDataType::Number),
        ("DataConverterToNumber", RuntimeDataType::Number),
        ("DataConverterTrigger", RuntimeDataType::Trigger),
        ("ScriptedDataConverter", RuntimeDataType::Any),
    ];
    let group_last_string_index = direct_converters.len();
    let group_all_input_index = group_last_string_index + 1;
    let group_skips_trailing_input_index = group_all_input_index + 1;
    let expected_output_types = direct_converters
        .iter()
        .map(|(_, output_type)| *output_type)
        .chain([
            RuntimeDataType::String,
            RuntimeDataType::None,
            RuntimeDataType::String,
        ])
        .collect::<Vec<_>>();

    let name = "data_converter_output_types";
    let bytes = synthetic_runtime_file(6094, |bytes| {
        push_empty_object(bytes, "Backboard");
        for (converter_name, _) in direct_converters {
            if converter_name == "DataConverterOperationViewModel" {
                push_object_with_properties(bytes, converter_name, |bytes| {
                    push_bytes_property(
                        bytes,
                        "DataConverterOperationViewModel",
                        "sourcePathIds",
                        &[],
                    );
                });
            } else {
                push_empty_object(bytes, converter_name);
            }
        }

        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 11);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });

        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 11);
        });

        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 1);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 11);
        });

        push_empty_object(bytes, "Artboard");
        for converter_index in 0..expected_output_types.len() {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "DataBind", |bytes| {
                push_uint_property(bytes, "DataBind", "converterId", converter_index as u64);
            });
        }
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_empty_object(bytes, "DataBind");
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-data-converter-output-types-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic converter output types");
    let rust_data_binds = rust.artboard_data_binds(0);
    let no_converter_data_bind_index = expected_output_types.len();

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_data_binds(&rust, &cpp.artboards[0].data_binds, &rust_data_binds, name);

    let converters = rust.data_converters();
    assert_eq!(converters.len(), expected_output_types.len());
    assert_eq!(rust_data_binds.len(), expected_output_types.len() + 1);
    assert_eq!(
        rust.data_converter_output_type(group_last_string_index),
        Some(RuntimeDataType::String),
        "group output type uses the last non-input converter"
    );
    assert_eq!(
        rust.data_converter_output_type(group_all_input_index),
        Some(RuntimeDataType::None),
        "group output type falls back to none when all items are input"
    );
    assert_eq!(
        rust.data_converter_output_type(group_skips_trailing_input_index),
        Some(RuntimeDataType::String),
        "group output type skips trailing input converters"
    );
    assert_eq!(
        rust.data_bind_source_output_type_for_object(rust_data_binds[0].object),
        Some(RuntimeDataType::None),
        "imported/unbound data binds have no source output type"
    );
    assert_eq!(
        rust.data_bind_output_type_for_object(rust_data_binds[1].object),
        Some(RuntimeDataType::String),
        "concrete converter output overrides the unbound source type"
    );
    assert_eq!(
        rust.data_bind_output_type_for_object(rust_data_binds[11].object),
        Some(RuntimeDataType::None),
        "input converter output falls through to the unbound source type"
    );
    assert_eq!(
        rust.data_bind_output_type_for_object(rust_data_binds[group_last_string_index].object),
        Some(RuntimeDataType::String),
        "group converter output drives the data bind output type"
    );
    assert_eq!(
        rust.data_bind_output_type_for_object(rust_data_binds[group_all_input_index].object),
        Some(RuntimeDataType::None),
        "all-input group output falls through to the unbound source type"
    );
    assert_eq!(
        rust.data_bind_output_type_for_object(rust_data_binds[no_converter_data_bind_index].object),
        Some(RuntimeDataType::None),
        "data binds without a converter fall through to the unbound source type"
    );
    for (converter_index, expected_output_type) in expected_output_types.into_iter().enumerate() {
        assert_eq!(
            rust.data_converter_output_type(converter_index),
            Some(expected_output_type),
            "converter {converter_index} output type mismatch"
        );
    }
}

#[test]
fn cpp_probe_matches_rust_data_converter_value_conversions_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data converter value-conversion comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let cpp = read_cpp_converter_samples(&probe);
    let mut operation_view_model_source_path = Vec::new();
    push_var_uint(&mut operation_view_model_source_path, 0);
    push_var_uint(&mut operation_view_model_source_path, 0);
    let mut missing_operation_view_model_source_path = Vec::new();
    push_var_uint(&mut missing_operation_view_model_source_path, 0);
    push_var_uint(&mut missing_operation_view_model_source_path, 99);
    let bytes = synthetic_runtime_file(6121, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "OperationContext");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "operand");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "CubicEaseInterpolator", |bytes| {
            push_float_property(bytes, "CubicEaseInterpolator", "x1", 0.25);
            push_float_property(bytes, "CubicEaseInterpolator", "y1", 0.1);
            push_float_property(bytes, "CubicEaseInterpolator", "x2", 0.25);
            push_float_property(bytes, "CubicEaseInterpolator", "y2", 1.0);
        });
        push_object_with_properties(bytes, "ElasticInterpolator", |bytes| {
            push_uint_property(bytes, "ElasticInterpolator", "easingValue", 1);
            push_float_property(bytes, "ElasticInterpolator", "amplitude", 1.2);
            push_float_property(bytes, "ElasticInterpolator", "period", 0.4);
        });
        push_empty_object(bytes, "DataConverterBooleanNegate");
        push_empty_object(bytes, "DataConverterTrigger");
        push_empty_object(bytes, "DataConverterToNumber");
        push_object_with_properties(bytes, "DataConverterToString", |bytes| {
            push_uint_property(bytes, "DataConverterToString", "flags", 1 | 2 | 4);
            push_uint_property(bytes, "DataConverterToString", "decimals", 2);
        });
        push_object_with_properties(bytes, "DataConverterRounder", |bytes| {
            push_uint_property(bytes, "DataConverterRounder", "decimals", 2);
        });
        push_empty_object(bytes, "DataConverterStringRemoveZeros");
        push_object_with_properties(bytes, "DataConverterStringTrim", |bytes| {
            push_uint_property(bytes, "DataConverterStringTrim", "trimType", 3);
        });
        push_object_with_properties(bytes, "DataConverterStringPad", |bytes| {
            push_uint_property(bytes, "DataConverterStringPad", "length", 6);
            push_string_property(bytes, "DataConverterStringPad", "text", "ab");
            push_uint_property(bytes, "DataConverterStringPad", "padType", 0);
        });
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
            push_float_property(bytes, "DataConverterOperationValue", "operationValue", 3.0);
        });
        push_object_with_properties(bytes, "DataConverterSystemDegsToRads", |bytes| {
            push_uint_property(bytes, "DataConverterSystemDegsToRads", "operationType", 2);
            push_float_property(
                bytes,
                "DataConverterSystemDegsToRads",
                "operationValue",
                std::f32::consts::PI / 180.0,
            );
        });
        push_object_with_properties(bytes, "DataConverterSystemNormalizer", |bytes| {
            push_uint_property(bytes, "DataConverterSystemNormalizer", "operationType", 3);
            push_float_property(
                bytes,
                "DataConverterSystemNormalizer",
                "operationValue",
                100.0,
            );
        });
        push_empty_object(bytes, "DataConverterListToLength");
        push_empty_object(bytes, "DataConverterNumberToList");
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_float_property(bytes, "DataConverterRangeMapper", "minInput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxInput", 10.0);
            push_float_property(bytes, "DataConverterRangeMapper", "minOutput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxOutput", 100.0);
        });
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "flags", 4);
            push_float_property(bytes, "DataConverterRangeMapper", "minInput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxInput", 10.0);
            push_float_property(bytes, "DataConverterRangeMapper", "minOutput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxOutput", 100.0);
        });
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "flags", 8);
            push_float_property(bytes, "DataConverterRangeMapper", "minInput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxInput", 10.0);
            push_float_property(bytes, "DataConverterRangeMapper", "minOutput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxOutput", 100.0);
        });
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "interpolationType", 0);
            push_float_property(bytes, "DataConverterRangeMapper", "minInput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxInput", 1.0);
            push_float_property(bytes, "DataConverterRangeMapper", "minOutput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxOutput", 100.0);
        });
        push_object_with_properties(bytes, "DataConverterStringTrim", |bytes| {
            push_uint_property(bytes, "DataConverterStringTrim", "trimType", 3);
        });
        push_object_with_properties(bytes, "DataConverterStringPad", |bytes| {
            push_uint_property(bytes, "DataConverterStringPad", "length", 4);
            push_string_property(bytes, "DataConverterStringPad", "text", "!");
            push_uint_property(bytes, "DataConverterStringPad", "padType", 1);
        });
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 17);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 18);
        });
        push_empty_object(bytes, "DataConverterFormula");
        push_empty_object(bytes, "FormulaTokenInput");
        push_object_with_properties(bytes, "FormulaTokenOperation", |bytes| {
            push_uint_property(bytes, "FormulaTokenOperation", "operationType", 0);
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 3.0);
        });
        push_object_with_properties(bytes, "FormulaTokenOperation", |bytes| {
            push_uint_property(bytes, "FormulaTokenOperation", "operationType", 2);
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 2.0);
        });
        push_empty_object(bytes, "DataConverterFormula");
        push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
            push_uint_property(bytes, "FormulaTokenFunction", "functionType", 1);
        });
        push_empty_object(bytes, "FormulaTokenInput");
        push_empty_object(bytes, "FormulaTokenArgumentSeparator");
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 3.0);
        });
        push_empty_object(bytes, "FormulaTokenArgumentSeparator");
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 8.0);
        });
        push_empty_object(bytes, "FormulaTokenParenthesisClose");
        push_empty_object(bytes, "DataConverterFormula");
        push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
            push_uint_property(bytes, "FormulaTokenFunction", "functionType", 6);
        });
        push_empty_object(bytes, "FormulaTokenInput");
        push_empty_object(bytes, "FormulaTokenArgumentSeparator");
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 2.0);
        });
        push_empty_object(bytes, "FormulaTokenParenthesisClose");
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            push_uint_property(bytes, "DataConverterOperationViewModel", "operationType", 0);
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &operation_view_model_source_path,
            );
        });
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            push_uint_property(bytes, "DataConverterOperationViewModel", "operationType", 0);
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &missing_operation_view_model_source_path,
            );
        });
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            push_uint_property(bytes, "DataConverterOperationViewModel", "operationType", 0);
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &operation_view_model_source_path,
            );
        });
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "interpolatorId", 0);
            push_float_property(bytes, "DataConverterRangeMapper", "minInput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxInput", 1.0);
            push_float_property(bytes, "DataConverterRangeMapper", "minOutput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxOutput", 100.0);
        });
        push_object_with_properties(bytes, "DataConverterRangeMapper", |bytes| {
            push_uint_property(bytes, "DataConverterRangeMapper", "interpolatorId", 1);
            push_float_property(bytes, "DataConverterRangeMapper", "minInput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxInput", 1.0);
            push_float_property(bytes, "DataConverterRangeMapper", "minOutput", 0.0);
            push_float_property(bytes, "DataConverterRangeMapper", "maxOutput", 100.0);
        });
        push_empty_object(bytes, "DataConverterInterpolator");
        push_empty_object(bytes, "DataConverterInterpolator");
        push_empty_object(bytes, "DataConverterInterpolator");
        for (function_type, values) in [
            (0, vec![9.0, -2.0]),
            (2, vec![2.6]),
            (3, vec![2.1]),
            (4, vec![2.9]),
            (5, vec![9.0]),
            (7, vec![1.0]),
            (8, vec![2.7182817]),
            (9, vec![0.0]),
            (10, vec![1.5707964]),
            (11, vec![0.7853982]),
            (12, vec![0.5]),
            (13, vec![0.5]),
            (14, vec![1.0]),
            (15, vec![1.0, 1.0]),
        ] {
            push_formula_function_converter(bytes, function_type, &values);
        }
        push_object_with_properties(bytes, "DataConverterToString", |bytes| {
            push_string_property(
                bytes,
                "DataConverterToString",
                "colorFormat",
                r"rgba(%r,%g,%b,%a)|#%R%G%B%A|hsl(%h,%s,%l)|%%|\%|%x|%",
            );
        });
        push_formula_function_converter(bytes, 16, &[2.0, 6.0]);
        push_object_with_properties(bytes, "DataConverterFormula", |bytes| {
            push_uint_property(bytes, "DataConverterFormula", "randomModeValue", 1);
        });
        push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
            push_uint_property(bytes, "FormulaTokenFunction", "functionType", 16);
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 2.0);
        });
        push_empty_object(bytes, "FormulaTokenArgumentSeparator");
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 6.0);
        });
        push_empty_object(bytes, "FormulaTokenParenthesisClose");
        push_object_with_properties(bytes, "FormulaTokenOperation", |bytes| {
            push_uint_property(bytes, "FormulaTokenOperation", "operationType", 0);
        });
        push_object_with_properties(bytes, "FormulaTokenFunction", |bytes| {
            push_uint_property(bytes, "FormulaTokenFunction", "functionType", 16);
        });
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 10.0);
        });
        push_empty_object(bytes, "FormulaTokenArgumentSeparator");
        push_object_with_properties(bytes, "FormulaTokenValue", |bytes| {
            push_float_property(bytes, "FormulaTokenValue", "operationValue", 20.0);
        });
        push_empty_object(bytes, "FormulaTokenParenthesisClose");
        push_object_with_properties(bytes, "DataConverterInterpolator", |bytes| {
            push_float_property(bytes, "DataConverterInterpolator", "duration", 1.0);
        });
        push_object_with_properties(bytes, "DataConverterInterpolator", |bytes| {
            push_uint_property(bytes, "DataConverterInterpolator", "interpolatorId", 0);
            push_float_property(bytes, "DataConverterInterpolator", "duration", 1.0);
        });
        push_object_with_properties(bytes, "DataConverterOperationValue", |bytes| {
            push_uint_property(bytes, "DataConverterOperationValue", "operationType", 2);
            push_float_property(bytes, "DataConverterOperationValue", "operationValue", 2.0);
        });
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 48);
        });
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 50);
        });
        push_empty_object(bytes, "DataConverterOperation");
        push_empty_object(bytes, "ScriptedDataConverter");
        push_empty_object(bytes, "DataEnumCustom");
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "first");
            push_string_property(bytes, "DataEnumValue", "value", "");
        });
        push_object_with_properties(bytes, "DataEnumValue", |bytes| {
            push_string_property(bytes, "DataEnumValue", "key", "second");
            push_string_property(bytes, "DataEnumValue", "value", "Second Label");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "operation-context");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_float_property(bytes, "ViewModelInstanceNumber", "propertyValue", 2.5);
        });
        push_empty_object(bytes, "Artboard");
        for (converter_id, flags) in [(9, 0), (9, 1), (10, 0), (10, 1)] {
            push_object_with_properties(bytes, "Shape", |bytes| {
                push_uint_property(bytes, "Shape", "parentId", 0);
            });
            push_object_with_properties(bytes, "DataBind", |bytes| {
                push_uint_property(bytes, "DataBind", "converterId", converter_id);
                push_uint_property(bytes, "DataBind", "flags", flags);
            });
        }
    });
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic converter conversion file");
    let rust_data_binds = rust.artboard_data_binds(0);
    let operation_context = rust
        .view_model_default_instance(0)
        .expect("operation view-model context instance imports");
    let data_enum = rust.data_enum(0).expect("synthetic data enum imports");
    let list_items = vec![
        rust.data_converter(0).expect("dummy list item 0"),
        rust.data_converter(1).expect("dummy list item 1"),
        rust.data_converter(2).expect("dummy list item 2"),
    ];

    let cases = vec![
        (
            "boolean_negate_true",
            0usize,
            RuntimeDataValue::Boolean(true),
            None,
            false,
        ),
        (
            "trigger_increment",
            1,
            RuntimeDataValue::Trigger(41),
            None,
            false,
        ),
        (
            "trigger_symbol_list_index_default",
            1,
            RuntimeDataValue::SymbolListIndex(5),
            None,
            false,
        ),
        (
            "to_number_string_prefix",
            2,
            RuntimeDataValue::String(b"123.5suffix"),
            None,
            false,
        ),
        (
            "to_number_hex_integer_prefix",
            2,
            RuntimeDataValue::String(b"0x10tail"),
            None,
            false,
        ),
        (
            "to_number_hex_float_prefix",
            2,
            RuntimeDataValue::String(b"-0x1.8p+2tail"),
            None,
            false,
        ),
        (
            "to_number_invalid_utf8_suffix",
            2,
            RuntimeDataValue::String(b"12.5\xfftail"),
            None,
            false,
        ),
        (
            "to_number_color_signed",
            2,
            RuntimeDataValue::Color(0xff000000),
            None,
            false,
        ),
        (
            "to_string_number_flags",
            3,
            RuntimeDataValue::Number(12345.5),
            None,
            false,
        ),
        (
            "to_string_enum_value",
            3,
            RuntimeDataValue::Enum {
                value: 1,
                data_enum: Some(data_enum.clone()),
            },
            None,
            false,
        ),
        (
            "to_string_color_format",
            45,
            RuntimeDataValue::Color(0xcc336699),
            None,
            false,
        ),
        (
            "rounder_two_decimals",
            4,
            RuntimeDataValue::Number(12.345),
            None,
            false,
        ),
        (
            "string_remove_zeros",
            5,
            RuntimeDataValue::String(b"42.5000"),
            None,
            false,
        ),
        (
            "string_trim_all",
            6,
            RuntimeDataValue::String(b" \ttrim me \n"),
            None,
            false,
        ),
        (
            "string_pad_start",
            7,
            RuntimeDataValue::String(b"xy"),
            None,
            false,
        ),
        (
            "operation_value_multiply",
            8,
            RuntimeDataValue::Number(4.0),
            None,
            false,
        ),
        (
            "operation_viewmodel_bound_add",
            23,
            RuntimeDataValue::Number(4.0),
            None,
            true,
        ),
        (
            "operation_viewmodel_missing_source",
            24,
            RuntimeDataValue::Number(4.0),
            None,
            true,
        ),
        (
            "operation_viewmodel_unbound_default",
            25,
            RuntimeDataValue::Number(4.0),
            None,
            false,
        ),
        (
            "system_degs_to_rads_to_target",
            9,
            RuntimeDataValue::Number(180.0),
            Some(0),
            false,
        ),
        (
            "system_degs_to_rads_to_source",
            9,
            RuntimeDataValue::Number(std::f32::consts::PI),
            Some(1),
            false,
        ),
        (
            "system_normalizer_to_target",
            10,
            RuntimeDataValue::Number(25.0),
            Some(2),
            false,
        ),
        (
            "system_normalizer_to_source",
            10,
            RuntimeDataValue::Number(0.25),
            Some(3),
            false,
        ),
        (
            "list_to_length_three",
            11,
            RuntimeDataValue::List(list_items.clone()),
            None,
            false,
        ),
        (
            "number_to_list_passthrough",
            12,
            RuntimeDataValue::List(list_items.clone()),
            None,
            false,
        ),
        (
            "number_to_list_no_file_empty",
            12,
            RuntimeDataValue::Number(3.8),
            None,
            false,
        ),
        (
            "range_mapper_linear",
            13,
            RuntimeDataValue::Number(5.0),
            None,
            false,
        ),
        (
            "range_mapper_cubic_ease",
            26,
            RuntimeDataValue::Number(0.25),
            None,
            false,
        ),
        (
            "range_mapper_elastic_out",
            27,
            RuntimeDataValue::Number(0.25),
            None,
            false,
        ),
        (
            "range_mapper_modulo",
            14,
            RuntimeDataValue::Number(12.0),
            None,
            false,
        ),
        (
            "range_mapper_reverse",
            15,
            RuntimeDataValue::Number(2.0),
            None,
            false,
        ),
        (
            "range_mapper_hold",
            16,
            RuntimeDataValue::Number(0.2),
            None,
            false,
        ),
        (
            "interpolator_number_first_run",
            28,
            RuntimeDataValue::Number(7.25),
            None,
            false,
        ),
        (
            "interpolator_color_first_run",
            29,
            RuntimeDataValue::Color(0xff336699),
            None,
            false,
        ),
        (
            "interpolator_string_passthrough",
            30,
            RuntimeDataValue::String(b"steady"),
            None,
            false,
        ),
        (
            "interpolator_number_after_half_duration",
            48,
            RuntimeDataValue::Number(10.0),
            None,
            false,
        ),
        (
            "interpolator_number_midflight_retarget",
            48,
            RuntimeDataValue::Number(20.0),
            None,
            false,
        ),
        (
            "interpolator_color_after_half_duration",
            48,
            RuntimeDataValue::Color(0xffffffff),
            None,
            false,
        ),
        (
            "interpolator_cubic_after_quarter_duration",
            49,
            RuntimeDataValue::Number(100.0),
            None,
            false,
        ),
        (
            "group_interpolator_then_multiply_after_half_duration",
            51,
            RuntimeDataValue::Number(10.0),
            None,
            false,
        ),
        (
            "group_trim_then_pad",
            19,
            RuntimeDataValue::String(b" x "),
            None,
            false,
        ),
        (
            "formula_arithmetic_precedence",
            20,
            RuntimeDataValue::Number(5.0),
            None,
            false,
        ),
        (
            "formula_non_number_default",
            20,
            RuntimeDataValue::String(b"not a number"),
            None,
            false,
        ),
        (
            "formula_function_max",
            21,
            RuntimeDataValue::Number(4.0),
            None,
            false,
        ),
        (
            "formula_symbol_list_pow",
            22,
            RuntimeDataValue::SymbolListIndex(4),
            None,
            false,
        ),
        (
            "formula_function_min",
            31,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_round",
            32,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_ceil",
            33,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_floor",
            34,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_sqrt",
            35,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_exp",
            36,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_log",
            37,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_cosine",
            38,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_sine",
            39,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_tangent",
            40,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_acosine",
            41,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_asine",
            42,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_atangent",
            43,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_atangent2",
            44,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_random_range",
            46,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "formula_function_random_always_pair",
            47,
            RuntimeDataValue::Number(0.0),
            None,
            false,
        ),
        (
            "operation_base_passthrough",
            52,
            RuntimeDataValue::Number(42.25),
            None,
            false,
        ),
        (
            "scripted_data_converter_non_scripting_passthrough",
            53,
            RuntimeDataValue::String(b"scripted"),
            None,
            false,
        ),
    ];

    assert_eq!(cpp.samples.len(), cases.len());
    assert_eq!(rust_data_binds.len(), 4);
    assert!(
        rust.data_converter_convert(46, &RuntimeDataValue::Number(0.0))
            .is_none(),
        "random formula conversion requires an explicit random source"
    );
    assert!(
        rust.data_converter_convert(47, &RuntimeDataValue::Number(0.0))
            .is_none(),
        "RandomMode::always formula conversion requires an explicit random source"
    );
    for (cpp_sample, (label, converter_index, input, data_bind_index, use_context)) in
        cpp.samples.iter().zip(cases)
    {
        assert_eq!(cpp_sample.label, label);
        let converter = if let Some(data_bind_index) = data_bind_index {
            rust_data_binds[data_bind_index]
                .converter
                .unwrap_or_else(|| panic!("{label} missing Rust data-bind converter"))
        } else {
            rust.data_converter(converter_index)
                .unwrap_or_else(|| panic!("{label} missing Rust converter {converter_index}"))
        };
        assert_eq!(
            cpp_sample.converter_core_type, converter.type_key,
            "{label} converter core type mismatch"
        );
        if matches!(
            label,
            "interpolator_number_after_half_duration"
                | "interpolator_number_midflight_retarget"
                | "interpolator_color_after_half_duration"
                | "interpolator_cubic_after_quarter_duration"
        ) {
            compare_interpolator_state_sample(&rust, cpp_sample, label, converter_index);
            continue;
        }
        if label == "group_interpolator_then_multiply_after_half_duration" {
            compare_stateful_group_sample(&rust, cpp_sample, label, converter_index);
            continue;
        }
        let formula_randoms = if !cpp_sample.random_values.is_empty() {
            Some(cpp_sample.random_values.clone())
        } else if label == "formula_function_random_range" {
            let cpp_value = cpp_sample
                .output
                .number_value
                .unwrap_or_else(|| panic!("{label} missing C++ random formula number output"));
            Some(vec![(cpp_value - 2.0) / 4.0])
        } else {
            None
        };
        let rust_output = if let Some(data_bind_index) = data_bind_index {
            rust.data_bind_convert_for_object(rust_data_binds[data_bind_index].object, &input)
        } else if use_context {
            rust.data_converter_convert_with_context_for_object(
                converter,
                &input,
                &[operation_context.object],
            )
        } else if let Some(randoms) = formula_randoms.as_deref() {
            rust.data_converter_convert_with_formula_randoms(converter_index, &input, randoms)
        } else {
            rust.data_converter_convert(converter_index, &input)
        }
        .unwrap_or_else(|| panic!("{label} Rust conversion returned None"));
        compare_converter_output(&cpp_sample.output, &rust_output, label);
        if label == "to_number_invalid_utf8_suffix" {
            continue;
        }
        let reverse_formula_randoms = if !cpp_sample.reverse_random_values.is_empty() {
            Some(cpp_sample.reverse_random_values.clone())
        } else {
            formula_randoms.clone()
        };
        let rust_reverse_output = if let Some(data_bind_index) = data_bind_index {
            rust.data_bind_reverse_convert_for_object(
                rust_data_binds[data_bind_index].object,
                &input,
            )
        } else if use_context {
            rust.data_converter_reverse_convert_with_context_for_object(
                converter,
                &input,
                &[operation_context.object],
            )
        } else if let Some(randoms) = reverse_formula_randoms.as_deref() {
            rust.data_converter_reverse_convert_with_formula_randoms(
                converter_index,
                &input,
                randoms,
            )
        } else {
            rust.data_converter_reverse_convert(converter_index, &input)
        }
        .unwrap_or_else(|| panic!("{label} Rust reverse conversion returned None"));
        compare_converter_output(
            &cpp_sample.reverse_output,
            &rust_reverse_output,
            &format!("{label} reverse"),
        );
    }
}

#[test]
fn rust_models_cpp_base_data_converter_passthrough_dispatch() {
    let definition =
        definition_by_name("DataConverter").expect("schema includes the DataConverter base class");
    assert!(
        definition.abstract_,
        "binary import should still treat plain DataConverter as abstract"
    );

    let rust = RuntimeFile {
        header: RuntimeHeader {
            major_version: 7,
            minor_version: 0,
            file_id: 0,
            property_field_ids: BTreeMap::new(),
        },
        objects: vec![Some(RuntimeObject {
            id: 0,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties: Vec::new(),
            skipped_properties: Vec::new(),
        })],
        import_statuses: vec![RuntimeImportStatus::Imported],
    };
    let input = RuntimeDataValue::String(b"base");

    assert_eq!(rust.data_converters().len(), 1);
    assert_eq!(
        rust.data_converter_output_type(0),
        Some(RuntimeDataType::None),
        "C++ DataConverter::outputType() base implementation returns none"
    );

    for (label, output) in [
        (
            "forward",
            rust.data_converter_convert(0, &input)
                .expect("base DataConverter::convert passes through"),
        ),
        (
            "reverse",
            rust.data_converter_reverse_convert(0, &input)
                .expect("base DataConverter::reverseConvert passes through"),
        ),
        (
            "stateful forward",
            rust.data_converter_stateful_convert(0, &mut RuntimeDataConverterState::new(), &input)
                .expect("stateful forward delegates to base convert"),
        ),
        (
            "stateful reverse",
            rust.data_converter_stateful_reverse_convert(
                0,
                &mut RuntimeDataConverterState::new(),
                &input,
            )
            .expect("stateful reverse delegates to base reverseConvert"),
        ),
    ] {
        match output {
            RuntimeConvertedDataValue::String(value) => assert_eq!(
                value, b"base",
                "{label} conversion should preserve the input string bytes"
            ),
            other => panic!("{label} conversion returned unexpected value {other:?}"),
        }
    }

    assert_eq!(
        rust.data_converter_stateful_advance(0, &mut RuntimeDataConverterState::new(), 0.25),
        Some(false),
        "C++ DataConverter::advance() base implementation returns false"
    );
}

#[test]
fn rust_models_cpp_data_bind_advance_guards_and_converter_delegation() {
    fn runtime_object(
        id: u32,
        type_name: &'static str,
        properties: Vec<RuntimeProperty>,
    ) -> RuntimeObject {
        let definition =
            definition_by_name(type_name).unwrap_or_else(|| panic!("missing {type_name} schema"));
        RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties,
            skipped_properties: Vec::new(),
        }
    }

    fn uint_property(owner: &'static str, name: &'static str, value: u64) -> RuntimeProperty {
        RuntimeProperty {
            key: property_key_for_name(owner, name),
            name,
            owner,
            value: FieldValue::Uint(value),
        }
    }

    let rust = RuntimeFile {
        header: RuntimeHeader {
            major_version: 7,
            minor_version: 0,
            file_id: 0,
            property_field_ids: BTreeMap::new(),
        },
        objects: vec![
            Some(runtime_object(0, "DataConverter", Vec::new())),
            Some(runtime_object(1, "DataConverterInterpolator", Vec::new())),
            Some(runtime_object(
                2,
                "DataBind",
                vec![uint_property("DataBind", "converterId", 0)],
            )),
            Some(runtime_object(
                3,
                "DataBind",
                vec![uint_property("DataBind", "converterId", 1)],
            )),
            Some(runtime_object(4, "DataBind", Vec::new())),
        ],
        import_statuses: vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
    };

    assert_eq!(
        rust.data_bind_update_effect(
            2,
            RuntimeComponentDirt::DEPENDENTS | RuntimeComponentDirt::BINDINGS,
            true,
            true,
            true,
            true
        ),
        Some(RuntimeDataBindUpdateEffect {
            calls_update_dependents: true,
            updates_converter_dependents: true,
            applies_target_to_source_before_update: false,
            clears_dirt: true,
            applies_source_to_target: true,
            source_to_target_is_main_direction: true,
            suppresses_dirt_while_applying_source_to_target: true,
            applies_target_to_source_after_update: false,
            target_to_source_is_main_direction: false,
        }),
        "C++ DataBind::updateDependents calls converter->update() when a converter is resolved"
    );
    assert_eq!(
        rust.data_bind_update_effect(4, RuntimeComponentDirt::DEPENDENTS, true, true, true, true),
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
        "C++ DataBind::updateDependents is a no-op when converter() is null"
    );
    assert_eq!(
        rust.data_bind_stateful_advance(
            0,
            &mut RuntimeDataConverterState::new(),
            0.25,
            true,
            false
        ),
        None,
        "non-DataBind objects do not expose DataBind::advance"
    );
    assert_eq!(
        rust.data_bind_stateful_advance(
            4,
            &mut RuntimeDataConverterState::new(),
            0.25,
            true,
            false
        ),
        Some(false),
        "C++ DataBind::advance returns false when converter() is null"
    );
    assert_eq!(
        rust.data_bind_stateful_advance(
            2,
            &mut RuntimeDataConverterState::new(),
            0.25,
            false,
            false
        ),
        Some(false),
        "C++ DataBind::advance returns false when m_Source is null"
    );
    assert_eq!(
        rust.data_bind_stateful_advance(2, &mut RuntimeDataConverterState::new(), 0.25, true, true),
        Some(false),
        "C++ DataBind::advance returns false while the bind has Flag::Collapsed"
    );
    assert_eq!(
        rust.data_bind_stateful_advance(
            2,
            &mut RuntimeDataConverterState::new(),
            0.25,
            true,
            false
        ),
        Some(false),
        "C++ base DataConverter::advance returns false after DataBind delegates"
    );
    assert_eq!(
        rust.data_bind_stateful_advance(
            3,
            &mut RuntimeDataConverterState::new(),
            0.25,
            true,
            false
        ),
        Some(true),
        "DataBind::advance delegates to a resolved converter when converter/source/collapse guards pass"
    );
}

#[test]
fn rust_models_cpp_data_bind_sort_order_swap_partition() {
    fn runtime_data_bind(id: u32, flags: u64) -> RuntimeObject {
        let definition =
            definition_by_name("DataBind").unwrap_or_else(|| panic!("missing DataBind schema"));
        RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties: vec![RuntimeProperty {
                key: property_key_for_name("DataBind", "flags"),
                name: "flags",
                owner: "DataBind",
                value: FieldValue::Uint(flags),
            }],
            skipped_properties: Vec::new(),
        }
    }

    let rust = RuntimeFile {
        header: RuntimeHeader {
            major_version: 7,
            minor_version: 0,
            file_id: 0,
            property_field_ids: BTreeMap::new(),
        },
        objects: vec![
            Some(runtime_data_bind(0, 0)),
            Some(runtime_data_bind(1, 0)),
            Some(runtime_data_bind(2, 1)),
            Some(runtime_data_bind(3, 0)),
            Some(runtime_data_bind(4, 2)),
        ],
        import_statuses: vec![
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
            RuntimeImportStatus::Imported,
        ],
    };

    assert_eq!(
        rust.sorted_data_bind_ids(&[0, 1, 2, 3, 4]),
        Some(vec![2, 4, 0, 3, 1]),
        "C++ DataBindContainer::sortDataBinds uses iter_swap partitioning, not stable_partition"
    );
}

#[test]
fn cpp_probe_matches_rust_number_to_list_generated_items_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ number-to-list generated-item comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let bytes = synthetic_runtime_file(6121, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "DefaultedItem");
        });
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "EmptyItem");
        });
        push_object_with_properties(bytes, "ViewModelPropertyBoolean", |bytes| {
            push_string_property(bytes, "ViewModelPropertyBoolean", "name", "enabled");
        });
        push_object_with_properties(bytes, "ViewModelPropertyColor", |bytes| {
            push_string_property(bytes, "ViewModelPropertyColor", "name", "tint");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataConverterNumberToList", |bytes| {
            push_uint_property(bytes, "DataConverterNumberToList", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "DataConverterNumberToList", |bytes| {
            push_uint_property(bytes, "DataConverterNumberToList", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "DataConverterNumberToList", |bytes| {
            push_uint_property(bytes, "DataConverterNumberToList", "viewModelId", 99);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "default");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_float_property(bytes, "ViewModelInstanceNumber", "propertyValue", 42.0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 1);
            push_string_property(bytes, "ViewModelInstanceString", "propertyValue", "seed");
        });
        push_empty_object(bytes, "Artboard");
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-number-to-list-generated-items-{}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_number_to_list_samples(&probe, &path);
    let _ = std::fs::remove_file(&path);

    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic number-to-list file");
    let type_key = |name: &str| {
        definition_by_name(name)
            .unwrap_or_else(|| panic!("missing schema definition {name}"))
            .type_key
            .int
    };
    let cases = [
        (
            0usize,
            vec![
                type_key("ViewModelInstanceNumber"),
                type_key("ViewModelInstanceString"),
            ],
        ),
        (
            1usize,
            vec![
                type_key("ViewModelInstanceBoolean"),
                type_key("ViewModelInstanceColor"),
            ],
        ),
        (2usize, Vec::<u16>::new()),
    ];

    assert_eq!(cpp.samples.len(), cases.len());
    for (sample, (converter_index, expected_value_core_types)) in cpp.samples.iter().zip(cases) {
        assert_eq!(sample.converter_index, converter_index);
        let converter = rust
            .data_converter(converter_index)
            .unwrap_or_else(|| panic!("missing Rust converter {converter_index}"));
        assert_eq!(sample.converter_core_type, converter.type_key);

        let rust_output = rust
            .data_converter_convert(converter_index, &RuntimeDataValue::Number(2.8))
            .unwrap_or_else(|| panic!("converter {converter_index} returned None"));
        compare_converter_output(
            &sample.output,
            &rust_output,
            "number_to_list generated item",
        );

        let RuntimeConvertedDataValue::GeneratedList(items) = rust_output else {
            panic!("converter {converter_index} did not produce a generated list");
        };

        assert_eq!(sample.output.list_items.len(), items.len());
        if expected_value_core_types.is_empty() {
            assert!(items.is_empty());
            continue;
        }

        assert_eq!(items.len(), 2);
        for (item_index, (cpp_item, rust_item)) in sample
            .output
            .list_items
            .iter()
            .zip(items.iter())
            .enumerate()
        {
            let cpp_item = cpp_item.as_ref().unwrap_or_else(|| {
                panic!("converter {converter_index} C++ item {item_index} was null")
            });
            assert_eq!(
                cpp_item.view_model_id, rust_item.view_model_id,
                "converter {converter_index} item {item_index} view-model id mismatch"
            );
            assert_eq!(
                cpp_item.value_core_types, rust_item.value_core_types,
                "converter {converter_index} item {item_index} value core types mismatch"
            );
            assert_eq!(
                expected_value_core_types, rust_item.value_core_types,
                "converter {converter_index} item {item_index} expected core types mismatch"
            );
        }
    }
}

#[test]
fn cpp_probe_matches_rust_data_converter_operation_view_model_source_paths_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data converter operation view-model source-path comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let mut encoded_source_path = Vec::new();
    for path_id in [7, 300] {
        push_var_uint(&mut encoded_source_path, path_id);
    }

    let name = "data_converter_operation_view_model_source_paths";
    let bytes = synthetic_runtime_file(6087, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataConverterOperationViewModel", |bytes| {
            push_bytes_property(
                bytes,
                "DataConverterOperationViewModel",
                "sourcePathIds",
                &encoded_source_path,
            );
        });
        push_empty_object(bytes, "DataConverterGroup");
        push_object_with_properties(bytes, "DataConverterGroupItem", |bytes| {
            push_uint_property(bytes, "DataConverterGroupItem", "converterId", 0);
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 1);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-data-converter-operation-view-model-source-paths-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes)
        .expect("Rust imports synthetic operation view-model converter source paths");
    let rust_data_binds = rust.artboard_data_binds(0);

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_data_binds(&rust, &cpp.artboards[0].data_binds, &rust_data_binds, name);
}

#[test]
fn cpp_probe_matches_rust_number_to_list_converter_view_model_resolution_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ number-to-list converter view-model comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "number_to_list_converter_view_model_resolution";
    let bytes = synthetic_runtime_file(6090, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Other");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "ListItem");
        });
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "DataConverterNumberToList", |bytes| {
            push_uint_property(bytes, "DataConverterNumberToList", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "DataConverterNumberToList", |bytes| {
            push_uint_property(bytes, "DataConverterNumberToList", "viewModelId", 99);
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 0);
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBind", |bytes| {
            push_uint_property(bytes, "DataBind", "converterId", 1);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-number-to-list-converter-view-model-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic number-to-list converter stream");
    let rust_data_binds = rust.artboard_data_binds(0);

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_data_binds(&rust, &cpp.artboards[0].data_binds, &rust_data_binds, name);

    let converters = rust.data_converters();
    assert_eq!(converters.len(), 2);
    let resolved = rust
        .resolved_view_model_for_number_to_list_converter_object(converters[0])
        .expect("valid DataConverterNumberToList viewModelId resolves through File::viewModel");
    assert_eq!(resolved.object.string_property("name"), Some("ListItem"));
    assert!(
        rust.resolved_view_model_for_number_to_list_converter_object(converters[1])
            .is_none(),
        "out-of-range DataConverterNumberToList viewModelId matches C++ File::viewModel null"
    );
}

#[test]
fn cpp_probe_matches_rust_data_bind_context_source_paths_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data bind context source-path comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let mut manifest_paths = Vec::new();
    push_var_uint(&mut manifest_paths, 2);
    push_var_uint(&mut manifest_paths, 77);
    push_var_uint(&mut manifest_paths, 3);
    for path_id in [4, 512, 9] {
        push_var_uint(&mut manifest_paths, path_id);
    }
    push_var_uint(&mut manifest_paths, u32::MAX as u64);
    push_var_uint(&mut manifest_paths, 2);
    for path_id in [13, 21] {
        push_var_uint(&mut manifest_paths, path_id);
    }

    let mut manifest_bytes = Vec::new();
    push_var_uint(&mut manifest_bytes, 1);
    push_var_uint(&mut manifest_bytes, manifest_paths.len() as u64);
    manifest_bytes.extend_from_slice(&manifest_paths);

    let mut encoded_resolved_path = Vec::new();
    push_var_uint(&mut encoded_resolved_path, 77);

    let mut encoded_missing_path = Vec::new();
    push_var_uint(&mut encoded_missing_path, 404);

    let mut encoded_multi_id_path = Vec::new();
    push_var_uint(&mut encoded_multi_id_path, 77);
    push_var_uint(&mut encoded_multi_id_path, 9);

    let mut encoded_wrapped_path = Vec::new();
    push_var_uint(&mut encoded_wrapped_path, u32::MAX as u64);

    let name = "data_bind_context_source_paths";
    let bytes = synthetic_runtime_file(6089, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "flags", 1 << 4);
            push_bytes_property(
                bytes,
                "DataBindContext",
                "sourcePathIds",
                &encoded_resolved_path,
            );
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "flags", 1 << 4);
            push_bytes_property(
                bytes,
                "DataBindContext",
                "sourcePathIds",
                &encoded_missing_path,
            );
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "flags", 1 << 4);
            push_bytes_property(
                bytes,
                "DataBindContext",
                "sourcePathIds",
                &encoded_multi_id_path,
            );
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_bytes_property(
                bytes,
                "DataBindContext",
                "sourcePathIds",
                &encoded_resolved_path,
            );
        });
        push_object_with_properties(bytes, "Shape", |bytes| {
            push_uint_property(bytes, "Shape", "parentId", 0);
        });
        push_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_uint_property(bytes, "DataBindContext", "flags", 1 << 4);
            push_bytes_property(
                bytes,
                "DataBindContext",
                "sourcePathIds",
                &encoded_wrapped_path,
            );
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-data-bind-context-source-paths-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic data-bind context stream");
    let rust_data_binds = rust.artboard_data_binds(0);

    assert_eq!(cpp.artboards.len(), 1);
    compare_artboard_data_binds(&rust, &cpp.artboards[0].data_binds, &rust_data_binds, name);
    assert_eq!(rust_data_binds.len(), 5);
    assert_eq!(
        rust.data_bind_context_source_path_ids_for_object(rust_data_binds[0].object),
        Some(vec![77])
    );
    assert_eq!(
        rust.data_bind_context_resolved_source_path_ids_for_object(rust_data_binds[0].object),
        Some(vec![4, 512, 9])
    );
    assert_eq!(
        rust.data_bind_context_source_path_ids_for_object(rust_data_binds[1].object),
        Some(vec![404])
    );
    assert_eq!(
        rust.data_bind_context_resolved_source_path_ids_for_object(rust_data_binds[1].object),
        Some(vec![404]),
        "C++ keeps the raw name-based context path when manifest resolution returns empty"
    );
    assert_eq!(
        rust.data_bind_context_source_path_ids_for_object(rust_data_binds[2].object),
        Some(vec![77, 9])
    );
    assert_eq!(
        rust.data_bind_context_resolved_source_path_ids_for_object(rust_data_binds[2].object),
        Some(vec![4, 512, 9]),
        "C++ DataBindContext expands the first id of a name-based multi-id source path"
    );
    assert_eq!(
        rust.data_bind_context_source_path_ids_for_object(rust_data_binds[3].object),
        Some(vec![77])
    );
    assert_eq!(
        rust.data_bind_context_resolved_source_path_ids_for_object(rust_data_binds[3].object),
        Some(vec![77]),
        "C++ only resolves DataBindContext source paths when the NameBased flag is set"
    );
    assert_eq!(
        rust.data_bind_context_source_path_ids_for_object(rust_data_binds[4].object),
        Some(vec![u32::MAX])
    );
    assert_eq!(
        rust.data_bind_context_resolved_source_path_ids_for_object(rust_data_binds[4].object),
        Some(vec![13, 21])
    );
}

#[test]
fn rust_imports_view_model_font_assets_as_a_distinct_runtime_type() {
    let bytes = synthetic_runtime_file(6153, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "FontAsset", |bytes| {
            push_string_property(bytes, "FontAsset", "name", "authored.ttf");
            push_uint_property(bytes, "FontAsset", "assetId", 42);
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyAssetFont", |bytes| {
            push_string_property(bytes, "ViewModelPropertyAssetFont", "name", "font");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceAssetFont", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceAssetFont",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceAssetFont", "propertyValue", 0);
        });
    });

    let file = read_runtime_file(&bytes).expect("font view-model fixture imports");
    let view_model = file.view_model(0).expect("root view model");
    assert_eq!(
        view_model.properties[0].type_name,
        "ViewModelPropertyAssetFont"
    );
    let value = view_model.instances[0].values[0].object;
    assert_eq!(value.type_name, "ViewModelInstanceAssetFont");
    assert_eq!(
        file.view_model_instance_value_data_type_for_object(value),
        Some(RuntimeDataType::AssetFont)
    );
    assert_eq!(RuntimeDataType::AssetFont.as_cpp_u32(), 13);
    assert_eq!(
        file.view_model_instance_font_asset_index_for_object(value),
        Some(0)
    );
    assert_eq!(
        file.view_model_instance_asset_index_for_object(value),
        None,
        "font identities must not leak through the image-asset accessor"
    );
    assert!(matches!(
        file.view_model_instance_source_data_value_for_object(value),
        Some(RuntimeDataValue::AssetFont(0))
    ));
    assert_eq!(
        file.resolved_file_asset_for_view_model_instance_asset_object(value)
            .map(|asset| asset.type_name),
        Some("FontAsset")
    );
}

#[test]
fn cpp_probe_matches_rust_view_model_instance_asset_snapshots_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ view-model asset snapshot comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "view_model_instance_asset_snapshots";
    let bytes = synthetic_runtime_file(6088, |bytes| {
        push_empty_object(bytes, "Backboard");
        push_object_with_properties(bytes, "ImageAsset", |bytes| {
            push_string_property(bytes, "ImageAsset", "name", "first.png");
            push_uint_property(bytes, "ImageAsset", "assetId", 5);
            push_string_property(bytes, "ImageAsset", "cdnBaseUrl", "https://cdn.example/");
            push_bytes_property(
                bytes,
                "ImageAsset",
                "cdnUuid",
                &(0u8..16).collect::<Vec<_>>(),
            );
        });
        push_object_with_properties(bytes, "ManifestAsset", |bytes| {
            push_string_property(bytes, "ManifestAsset", "name", "manifest.json");
            push_uint_property(bytes, "ManifestAsset", "assetId", 6);
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Root");
        });
        push_object_with_properties(bytes, "ViewModelPropertyAssetImage", |bytes| {
            push_string_property(bytes, "ViewModelPropertyAssetImage", "name", "image");
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceAssetImage", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceAssetImage",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceAssetImage", "propertyValue", 0);
        });
        push_object_with_properties(bytes, "AudioAsset", |bytes| {
            push_string_property(bytes, "AudioAsset", "name", "later.wav");
            push_uint_property(bytes, "AudioAsset", "assetId", 7);
        });
        push_object_with_properties(bytes, "ViewModelInstanceAssetImage", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceAssetImage",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceAssetImage", "propertyValue", 1);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-view-model-instance-asset-snapshots-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic view-model asset snapshot stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert_eq!(cpp.view_models.len(), rust_view_models.len());
    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        compare_view_model_instances(
            &rust,
            &cpp_view_model.instances,
            &rust_view_model.instances,
            view_model_index,
            name,
        );
    }

    let values = &rust_view_models[0].instances[0].values;
    let first_snapshot = rust.view_model_instance_asset_file_assets_for_object(values[0].object);
    assert_eq!(first_snapshot.len(), 1);
    assert_eq!(first_snapshot[0].string_property("name"), Some("first.png"));
    assert_eq!(
        rust.resolved_file_asset_for_view_model_instance_asset_object(values[0].object)
            .and_then(|asset| asset.string_property("name")),
        Some("first.png")
    );

    let second_snapshot = rust.view_model_instance_asset_file_assets_for_object(values[1].object);
    assert_eq!(second_snapshot.len(), 2);
    assert_eq!(
        second_snapshot[0].string_property("name"),
        Some("first.png")
    );
    assert_eq!(
        second_snapshot[1].string_property("name"),
        Some("later.wav")
    );
    assert_eq!(
        rust.resolved_file_asset_for_view_model_instance_asset_object(values[1].object)
            .and_then(|asset| asset.string_property("name")),
        Some("later.wav")
    );
}

#[test]
fn cpp_probe_matches_rust_manifest_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ manifest comparison; set RIVE_CPP_PROBE or run make cpp-probe");
        return;
    };

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
    push_var_uint(&mut paths, 3);
    push_var_uint(&mut paths, 3);
    push_var_uint(&mut paths, 3);
    for path_id in [7, 9, 11] {
        push_var_uint(&mut paths, path_id);
    }
    push_var_uint(&mut paths, 4);
    push_var_uint(&mut paths, 1);
    push_var_uint(&mut paths, 9);
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

    let name = "manifest_names_and_paths";
    let bytes = synthetic_runtime_file(6090, |bytes| {
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-manifest-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic manifest stream");

    compare_manifest_result(cpp.manifest.as_ref(), rust.manifest(), name)
        .unwrap_or_else(|err| panic!("{err}"));

    let manifest = rust.manifest().expect("decoded manifest data");
    assert_eq!(manifest.resolve_name(7), Some("position"));
    assert_eq!(manifest.resolve_name(u32::MAX), Some("wrapped"));
    assert_eq!(manifest.resolve_name(404), None);
    assert_eq!(manifest.resolve_path(3), Some(&[7, 9, 11][..]));
    assert_eq!(manifest.resolve_path(u32::MAX), Some(&[13, 21][..]));
    assert_eq!(manifest.resolve_path(404), None);

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

    let name = "partial_malformed_manifest";
    let bytes = synthetic_runtime_file(6091, |bytes| {
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-manifest-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports partial malformed manifest stream");

    compare_manifest_result(cpp.manifest.as_ref(), rust.manifest(), name)
        .unwrap_or_else(|err| panic!("{err}"));

    let manifest = rust
        .manifest()
        .expect("C++ keeps the manifest resolver after decode failure");
    assert_eq!(manifest.resolve_name(7), Some("position"));
    assert_eq!(manifest.resolve_path(3), Some(&[7, 0][..]));
}

#[test]
fn cpp_probe_matches_rust_file_level_stored_property_values_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ file-level registry property comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    for fixture_path in [
        "car_widgets_v01.riv",
        "databind_solo_to_enum.riv",
        "hosted_image_file.riv",
        "viewmodel_runtime_file.riv",
    ] {
        compare_file_level_stored_property_values(&probe, fixture_path);
    }
}

fn compare_file_level_stored_property_values(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_reference_file_with_property_values(probe, fixture_path);
    let rust = read_rust_reference_runtime_file(fixture_path);
    let rust_assets = runtime_file_assets(&rust);
    let rust_view_models = runtime_file_view_models(&rust);
    let rust_enums = runtime_data_enums(&rust);

    for (asset_index, (cpp_asset, rust_asset)) in cpp.assets.iter().zip(&rust_assets).enumerate() {
        compare_cpp_registry_properties(
            &cpp_asset.property_values,
            rust_asset,
            &format!("file asset {asset_index} in reference fixture {fixture_path}"),
        );
    }

    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        compare_cpp_registry_properties(
            &cpp_view_model.property_values,
            rust_view_model.object,
            &format!("view model {view_model_index} in reference fixture {fixture_path}"),
        );

        for (property_index, (cpp_property, rust_property)) in cpp_view_model
            .properties
            .iter()
            .zip(&rust_view_model.properties)
            .enumerate()
        {
            compare_cpp_registry_properties(
                &cpp_property.property_values,
                rust_property,
                &format!(
                    "view model {view_model_index} property {property_index} in reference fixture {fixture_path}"
                ),
            );
        }

        for (instance_index, (cpp_instance, rust_instance)) in cpp_view_model
            .instances
            .iter()
            .zip(&rust_view_model.instances)
            .enumerate()
        {
            compare_cpp_registry_properties(
                &cpp_instance.property_values,
                rust_instance.object,
                &format!(
                    "view model {view_model_index} instance {instance_index} in reference fixture {fixture_path}"
                ),
            );
            for (value_index, (cpp_value, rust_value)) in cpp_instance
                .values
                .iter()
                .zip(&rust_instance.values)
                .enumerate()
            {
                compare_cpp_registry_properties(
                    &cpp_value.property_values,
                    rust_value.object,
                    &format!(
                        "view model {view_model_index} instance {instance_index} value {value_index} in reference fixture {fixture_path}"
                    ),
                );
                for (item_index, (cpp_item, rust_item)) in cpp_value
                    .items
                    .iter()
                    .zip(&rust_value.list_items)
                    .enumerate()
                {
                    compare_cpp_registry_properties(
                        &cpp_item.property_values,
                        rust_item,
                        &format!(
                            "view model {view_model_index} instance {instance_index} value {value_index} list item {item_index} in reference fixture {fixture_path}"
                        ),
                    );
                }
            }
        }
    }

    for (enum_index, (cpp_enum, rust_enum)) in cpp.enums.iter().zip(&rust_enums).enumerate() {
        compare_cpp_registry_properties(
            &cpp_enum.property_values,
            rust_enum.object,
            &format!("data enum {enum_index} in reference fixture {fixture_path}"),
        );

        for (value_index, (cpp_value, rust_value)) in
            cpp_enum.values.iter().zip(&rust_enum.values).enumerate()
        {
            compare_cpp_registry_properties(
                &cpp_value.property_values,
                rust_value,
                &format!(
                    "data enum {enum_index} value {value_index} in reference fixture {fixture_path}"
                ),
            );
        }
    }
}

fn compare_file_view_models_and_enums(probe: &Path, fixture_path: &str) {
    let path = reference_fixture(fixture_path);
    let cpp = read_cpp_probe_path_with_options(
        probe,
        &path,
        fixture_path,
        CppProbePropertyValueMode::None,
        false,
        true,
    );
    let rust = read_rust_reference_runtime_file(fixture_path);
    let rust_view_models = runtime_file_view_models(&rust);
    let rust_enums = runtime_data_enums(&rust);

    assert_eq!(
        cpp.view_models.len(),
        rust_view_models.len(),
        "view model count mismatch for reference fixture {fixture_path}"
    );

    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        assert_eq!(
            cpp_view_model.index, view_model_index,
            "C++ view model index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_view_model.core_type, rust_view_model.object.type_key,
            "view model {view_model_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_view_model.name,
            rust_view_model
                .object
                .string_property("name")
                .unwrap_or_default(),
            "view model {view_model_index} name mismatch for reference fixture {fixture_path}"
        );
        compare_view_model_properties(
            &rust,
            &cpp_view_model.properties,
            &rust_view_model.properties,
            view_model_index,
            fixture_path,
        );
        compare_view_model_instances(
            &rust,
            &cpp_view_model.instances,
            &rust_view_model.instances,
            view_model_index,
            fixture_path,
        );
        compare_view_model_default_instance(
            &rust,
            cpp_view_model,
            rust_view_model,
            view_model_index,
            fixture_path,
        );
    }

    assert_eq!(
        cpp.enums.len(),
        rust_enums.len(),
        "data enum count mismatch for reference fixture {fixture_path}"
    );

    for (enum_index, (cpp_enum, rust_enum)) in cpp.enums.iter().zip(&rust_enums).enumerate() {
        assert_eq!(
            cpp_enum.index, enum_index,
            "C++ data enum index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_enum.core_type, rust_enum.object.type_key,
            "data enum {enum_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_enum.name,
            rust_enum.object.string_property("name").unwrap_or_default(),
            "data enum {enum_index} name mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_enum.values.len(),
            rust_enum.values.len(),
            "data enum {enum_index} value count mismatch for reference fixture {fixture_path}"
        );
        for (value_index, (cpp_value, rust_value)) in
            cpp_enum.values.iter().zip(&rust_enum.values).enumerate()
        {
            assert_eq!(
                cpp_value.index, value_index,
                "data enum {enum_index} C++ value index mismatch for reference fixture {fixture_path}"
            );
            assert_eq!(
                cpp_value.core_type, rust_value.type_key,
                "data enum {enum_index} value {value_index} core type mismatch for reference fixture {fixture_path}"
            );
            assert_eq!(
                cpp_value.key,
                rust_value.string_property("key").unwrap_or_default(),
                "data enum {enum_index} value {value_index} key mismatch for reference fixture {fixture_path}"
            );
            assert_eq!(
                cpp_value.value,
                rust_value.string_property("value").unwrap_or_default(),
                "data enum {enum_index} value {value_index} value mismatch for reference fixture {fixture_path}"
            );
        }
        compare_data_enum_lookup_result(&rust, cpp_enum, enum_index, fixture_path)
            .unwrap_or_else(|err| panic!("{err}"));
    }
}

fn compare_data_enum_lookup_result(
    rust_file: &RuntimeFile,
    cpp_enum: &CppProbeDataEnum,
    data_enum_index: usize,
    label: &str,
) -> Result<(), String> {
    let prefix = format!("{label}: data enum {data_enum_index}");

    for (lookup_index, lookup) in cpp_enum.key_lookups.iter().enumerate() {
        let rust_value =
            rust_file.data_enum_value_for_key_bytes(data_enum_index, lookup.key.as_bytes());
        let rust_value_index =
            rust_file.data_enum_value_index_for_key_bytes(data_enum_index, lookup.key.as_bytes());

        if lookup.value_index < 0 {
            if rust_value.is_some() {
                return Err(format!(
                    "{prefix} key lookup {lookup_index} for {:?} should miss value, Rust {:?}",
                    lookup.key, rust_value
                ));
            }
            if rust_value_index.is_some() {
                return Err(format!(
                    "{prefix} key lookup {lookup_index} for {:?} should miss index, Rust {:?}",
                    lookup.key, rust_value_index
                ));
            }
            continue;
        }

        let expected_index = lookup.value_index as usize;
        if rust_value_index != Some(expected_index) {
            return Err(format!(
                "{prefix} key lookup {lookup_index} for {:?} index mismatch, C++ {}, Rust {:?}",
                lookup.key, lookup.value_index, rust_value_index
            ));
        }
        if rust_value != Some(lookup.value.as_bytes()) {
            return Err(format!(
                "{prefix} key lookup {lookup_index} for {:?} value mismatch, C++ {:?}, Rust {:?}",
                lookup.key, lookup.value, rust_value
            ));
        }
    }

    for (lookup_index, lookup) in cpp_enum.index_lookups.iter().enumerate() {
        let rust_value = rust_file.data_enum_value_for_index(data_enum_index, lookup.index);
        let rust_value_index =
            rust_file.data_enum_value_index_for_index(data_enum_index, lookup.index);

        if lookup.value_index < 0 {
            if rust_value.is_some() {
                return Err(format!(
                    "{prefix} index lookup {lookup_index} for {} should miss value, Rust {:?}",
                    lookup.index, rust_value
                ));
            }
            if rust_value_index.is_some() {
                return Err(format!(
                    "{prefix} index lookup {lookup_index} for {} should miss index, Rust {:?}",
                    lookup.index, rust_value_index
                ));
            }
            continue;
        }

        let expected_index = lookup.value_index as usize;
        if rust_value_index != Some(expected_index) {
            return Err(format!(
                "{prefix} index lookup {lookup_index} for {} index mismatch, C++ {}, Rust {:?}",
                lookup.index, lookup.value_index, rust_value_index
            ));
        }
        if rust_value != Some(lookup.value.as_bytes()) {
            return Err(format!(
                "{prefix} index lookup {lookup_index} for {} value mismatch, C++ {:?}, Rust {:?}",
                lookup.index, lookup.value, rust_value
            ));
        }
    }

    Ok(())
}

fn compare_view_model_property_enum_lookup_result(
    rust_file: &RuntimeFile,
    cpp_property: &CppProbeViewModelProperty,
    rust_property: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    if cpp_property.enum_key_lookups.is_empty() && cpp_property.enum_index_lookups.is_empty() {
        return Ok(());
    }

    for (lookup_index, lookup) in cpp_property.enum_key_lookups.iter().enumerate() {
        let rust_value = rust_file.view_model_property_enum_value_for_key_bytes_object(
            rust_property,
            lookup.key.as_bytes(),
        );
        let rust_value_index = rust_file.view_model_property_enum_value_index_for_key_bytes_object(
            rust_property,
            lookup.key.as_bytes(),
        );

        if lookup.value_index < 0 {
            if rust_value.is_some() {
                return Err(format!(
                    "{label} enum key lookup {lookup_index} for {:?} should miss value, Rust {:?}",
                    lookup.key, rust_value
                ));
            }
            if rust_value_index.is_some() {
                return Err(format!(
                    "{label} enum key lookup {lookup_index} for {:?} should miss index, Rust {:?}",
                    lookup.key, rust_value_index
                ));
            }
            continue;
        }

        let expected_index = lookup.value_index as usize;
        if rust_value_index != Some(expected_index) {
            return Err(format!(
                "{label} enum key lookup {lookup_index} for {:?} index mismatch, C++ {}, Rust {:?}",
                lookup.key, lookup.value_index, rust_value_index
            ));
        }
        if rust_value != Some(lookup.value.as_bytes()) {
            return Err(format!(
                "{label} enum key lookup {lookup_index} for {:?} value mismatch, C++ {:?}, Rust {:?}",
                lookup.key, lookup.value, rust_value
            ));
        }
    }

    for (lookup_index, lookup) in cpp_property.enum_index_lookups.iter().enumerate() {
        let rust_value =
            rust_file.view_model_property_enum_value_for_index_object(rust_property, lookup.index);
        let rust_value_index = rust_file
            .view_model_property_enum_value_index_for_index_object(rust_property, lookup.index);

        if lookup.value_index < 0 {
            if rust_value.is_some() {
                return Err(format!(
                    "{label} enum index lookup {lookup_index} for {} should miss value, Rust {:?}",
                    lookup.index, rust_value
                ));
            }
            if rust_value_index.is_some() {
                return Err(format!(
                    "{label} enum index lookup {lookup_index} for {} should miss index, Rust {:?}",
                    lookup.index, rust_value_index
                ));
            }
            continue;
        }

        let expected_index = lookup.value_index as usize;
        if rust_value_index != Some(expected_index) {
            return Err(format!(
                "{label} enum index lookup {lookup_index} for {} index mismatch, C++ {}, Rust {:?}",
                lookup.index, lookup.value_index, rust_value_index
            ));
        }
        if rust_value != Some(lookup.value.as_bytes()) {
            return Err(format!(
                "{label} enum index lookup {lookup_index} for {} value mismatch, C++ {:?}, Rust {:?}",
                lookup.index, lookup.value, rust_value
            ));
        }
    }

    Ok(())
}

#[test]
fn cpp_probe_matches_rust_view_model_instance_view_model_references_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ view-model instance reference comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let name = "view_model_instance_view_model_references";
    let bytes = synthetic_runtime_file(6078, |bytes| {
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
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-view-model-instance-references-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_inner(&probe, &path, name, false);
    let _ = std::fs::remove_file(&path);
    let rust =
        read_runtime_file(&bytes).expect("Rust imports synthetic view-model reference stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert_eq!(cpp.view_models.len(), rust_view_models.len());
    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        compare_view_model_instances(
            &rust,
            &cpp_view_model.instances,
            &rust_view_model.instances,
            view_model_index,
            name,
        );
    }

    let root_instances = &rust_view_models[1].instances;
    assert!(
        rust.referenced_view_model_instance_for_value_object(root_instances[0].values[0].object)
            .is_none(),
        "C++ leaves ViewModelInstanceViewModel unresolved before an ArtboardImporter exists"
    );
    let reference = rust
        .referenced_view_model_instance_for_value_object(root_instances[1].values[0].object)
        .expect("post-artboard ViewModelInstanceViewModel resolves through C++ import");
    assert_eq!(reference.view_model_index, 0);
    assert_eq!(reference.instance_index, 1);
    assert_eq!(reference.object.string_property("name"), Some("item1"));
}

#[test]
fn cpp_probe_matches_rust_data_context_lookups_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ data-context lookup comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let mut manifest_names = Vec::new();
    push_var_uint(&mut manifest_names, 4);
    for (id, name) in [
        (10, "child"),
        (11, "label"),
        (12, "amount"),
        (13, "fallback"),
    ] {
        push_var_uint(&mut manifest_names, id);
        push_var_uint(&mut manifest_names, name.len() as u64);
        manifest_names.extend_from_slice(name.as_bytes());
    }

    let mut manifest_bytes = Vec::new();
    push_var_uint(&mut manifest_bytes, 0);
    push_var_uint(&mut manifest_bytes, manifest_names.len() as u64);
    manifest_bytes.extend_from_slice(&manifest_names);

    let name = "data_context_lookups";
    let bytes = synthetic_runtime_file(6101, |bytes| {
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Child");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "label");
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
        push_object_with_properties(bytes, "ViewModelPropertyNumber", |bytes| {
            push_string_property(bytes, "ViewModelPropertyNumber", "name", "amount");
        });
        push_object_with_properties(bytes, "ViewModel", |bytes| {
            push_string_property(bytes, "ViewModel", "name", "Fallback");
        });
        push_object_with_properties(bytes, "ViewModelPropertyString", |bytes| {
            push_string_property(bytes, "ViewModelPropertyString", "name", "fallback");
        });
        push_empty_object(bytes, "Backboard");
        push_empty_object(bytes, "ManifestAsset");
        push_object_with_properties(bytes, "FileAssetContents", |bytes| {
            push_bytes_property(bytes, "FileAssetContents", "bytes", &manifest_bytes);
        });
        push_empty_object(bytes, "Artboard");
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "child-instance");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "root-instance");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstanceViewModel", |bytes| {
            push_uint_property(
                bytes,
                "ViewModelInstanceViewModel",
                "viewModelPropertyId",
                0,
            );
            push_uint_property(bytes, "ViewModelInstanceViewModel", "propertyValue", 0);
        });
        push_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 1);
        });
        push_object_with_properties(bytes, "ViewModelInstance", |bytes| {
            push_string_property(bytes, "ViewModelInstance", "name", "fallback-instance");
            push_uint_property(bytes, "ViewModelInstance", "viewModelId", 2);
        });
        push_object_with_properties(bytes, "ViewModelInstanceString", |bytes| {
            push_uint_property(bytes, "ViewModelInstanceString", "viewModelPropertyId", 0);
        });
    });

    let path = std::env::temp_dir().join(format!(
        "rive-rust-cpp-data-context-lookups-{}-{name}.riv",
        std::process::id()
    ));
    std::fs::write(&path, &bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let cpp = read_cpp_probe_path_with_extra_options(
        &probe,
        &path,
        name,
        CppProbePropertyValueMode::None,
        false,
        true,
        true,
    );
    let _ = std::fs::remove_file(&path);
    let rust = read_runtime_file(&bytes).expect("Rust imports synthetic data-context stream");
    let rust_view_models = runtime_file_view_models(&rust);

    assert!(
        !cpp.data_context_lookups.is_empty(),
        "C++ probe should emit data-context lookup cases"
    );
    let lookup_kinds = cpp
        .data_context_lookups
        .iter()
        .map(|lookup| lookup.kind.as_str())
        .collect::<BTreeSet<_>>();
    for required_kind in [
        "absoluteProperty",
        "absoluteInstance",
        "relativeProperty",
        "relativeInstance",
        "propertyFromPath",
        "absolutePropertyParentFallback",
        "relativePropertyParentFallback",
    ] {
        assert!(
            lookup_kinds.contains(required_kind),
            "missing C++ data-context lookup kind {required_kind}"
        );
    }

    for (lookup_index, lookup) in cpp.data_context_lookups.iter().enumerate() {
        let current_instance = rust_view_models[lookup.current_view_model_index].instances
            [lookup.current_instance_index]
            .object;
        let mut chain = vec![current_instance];
        if let (Some(parent_view_model_index), Some(parent_instance_index)) =
            (lookup.parent_view_model_index, lookup.parent_instance_index)
        {
            chain.push(
                rust_view_models[parent_view_model_index].instances[parent_instance_index].object,
            );
        }

        match lookup.kind.as_str() {
            "absoluteProperty" | "absolutePropertyParentFallback" => {
                let rust_value =
                    rust.data_context_view_model_property_for_instance_chain(&chain, &lookup.path);
                compare_data_context_lookup_value(
                    &rust,
                    &rust_view_models,
                    lookup.value.as_ref(),
                    rust_value,
                    lookup_index,
                    &lookup.kind,
                    name,
                );
                assert!(
                    lookup.instance.is_none(),
                    "C++ property lookup {lookup_index} unexpectedly emitted an instance"
                );
            }
            "relativeProperty" | "relativePropertyParentFallback" => {
                let rust_value = rust.data_context_relative_view_model_property_for_instance_chain(
                    &chain,
                    &lookup.path,
                );
                compare_data_context_lookup_value(
                    &rust,
                    &rust_view_models,
                    lookup.value.as_ref(),
                    rust_value,
                    lookup_index,
                    &lookup.kind,
                    name,
                );
                assert!(
                    lookup.instance.is_none(),
                    "C++ relative property lookup {lookup_index} unexpectedly emitted an instance"
                );
            }
            "propertyFromPath" => {
                let rust_value = rust.view_model_instance_property_from_path_for_object(
                    current_instance,
                    &lookup.path,
                );
                compare_data_context_lookup_value(
                    &rust,
                    &rust_view_models,
                    lookup.value.as_ref(),
                    rust_value,
                    lookup_index,
                    &lookup.kind,
                    name,
                );
                assert!(
                    lookup.instance.is_none(),
                    "C++ propertyFromPath lookup {lookup_index} unexpectedly emitted an instance"
                );
            }
            "absoluteInstance" => {
                let rust_instance =
                    rust.data_context_view_model_instance_for_instance_chain(&chain, &lookup.path);
                compare_data_context_lookup_instance(
                    &rust_view_models,
                    lookup.instance.as_ref(),
                    rust_instance.as_ref(),
                    lookup_index,
                    &lookup.kind,
                    name,
                );
                assert!(
                    lookup.value.is_none(),
                    "C++ instance lookup {lookup_index} unexpectedly emitted a value"
                );
            }
            "relativeInstance" => {
                let rust_instance = rust
                    .data_context_relative_view_model_instance_for_instance_chain(
                        &chain,
                        &lookup.path,
                    );
                compare_data_context_lookup_instance(
                    &rust_view_models,
                    lookup.instance.as_ref(),
                    rust_instance.as_ref(),
                    lookup_index,
                    &lookup.kind,
                    name,
                );
                assert!(
                    lookup.value.is_none(),
                    "C++ relative instance lookup {lookup_index} unexpectedly emitted a value"
                );
            }
            other => panic!("unexpected C++ data-context lookup kind {other}"),
        }
    }
}

fn compare_data_context_lookup_value(
    rust_file: &RuntimeFile,
    rust_view_models: &[nuxie_binary::RuntimeViewModel<'_>],
    cpp_value: Option<&CppProbeDataContextValue>,
    rust_value: Option<&RuntimeObject>,
    lookup_index: usize,
    kind: &str,
    label: &str,
) {
    match (cpp_value, rust_value) {
        (None, None) => {}
        (Some(cpp_value), Some(rust_value)) => {
            let expected = rust_view_models[cpp_value.view_model_index].instances
                [cpp_value.instance_index]
                .values[cpp_value.value_index]
                .object;
            assert_eq!(
                expected.id, rust_value.id,
                "{label} data-context lookup {lookup_index} {kind} value object mismatch"
            );
            assert_eq!(
                cpp_value.core_type, rust_value.type_key,
                "{label} data-context lookup {lookup_index} {kind} value core type mismatch"
            );
            assert_eq!(
                Some(cpp_value.view_model_property_id),
                rust_value.uint_property("viewModelPropertyId"),
                "{label} data-context lookup {lookup_index} {kind} value property id mismatch"
            );
            assert_eq!(
                Some(cpp_value.name.as_str()),
                rust_file.view_model_instance_value_name_for_object(rust_value),
                "{label} data-context lookup {lookup_index} {kind} value name mismatch"
            );
        }
        (Some(cpp_value), None) => panic!(
            "{label} data-context lookup {lookup_index} {kind} missing Rust value for C++ core type {}",
            cpp_value.core_type
        ),
        (None, Some(rust_value)) => panic!(
            "{label} data-context lookup {lookup_index} {kind} returned unexpected Rust value {}",
            rust_value.type_name
        ),
    }
}

fn compare_data_context_lookup_instance(
    rust_view_models: &[nuxie_binary::RuntimeViewModel<'_>],
    cpp_instance: Option<&CppProbeViewModelInstanceReference>,
    rust_instance: Option<&nuxie_binary::RuntimeViewModelInstanceReference<'_>>,
    lookup_index: usize,
    kind: &str,
    label: &str,
) {
    match (cpp_instance, rust_instance) {
        (None, None) => {}
        (Some(cpp_instance), Some(rust_instance)) => {
            let expected = rust_view_models[cpp_instance.view_model_index].instances
                [cpp_instance.instance_index]
                .object;
            assert_eq!(
                expected.id, rust_instance.object.id,
                "{label} data-context lookup {lookup_index} {kind} instance object mismatch"
            );
            assert_eq!(
                cpp_instance.view_model_index, rust_instance.view_model_index,
                "{label} data-context lookup {lookup_index} {kind} instance view-model index mismatch"
            );
            assert_eq!(
                cpp_instance.instance_index, rust_instance.instance_index,
                "{label} data-context lookup {lookup_index} {kind} instance index mismatch"
            );
            assert_eq!(
                cpp_instance.core_type, rust_instance.object.type_key,
                "{label} data-context lookup {lookup_index} {kind} instance core type mismatch"
            );
            assert_eq!(
                Some(cpp_instance.name.as_str()),
                rust_instance.object.string_property("name"),
                "{label} data-context lookup {lookup_index} {kind} instance name mismatch"
            );
            assert_eq!(
                Some(cpp_instance.view_model_id),
                rust_instance.object.uint_property("viewModelId"),
                "{label} data-context lookup {lookup_index} {kind} instance view-model id mismatch"
            );
        }
        (Some(cpp_instance), None) => panic!(
            "{label} data-context lookup {lookup_index} {kind} missing Rust instance for C++ core type {}",
            cpp_instance.core_type
        ),
        (None, Some(rust_instance)) => panic!(
            "{label} data-context lookup {lookup_index} {kind} returned unexpected Rust instance {}",
            rust_instance.object.type_name
        ),
    }
}

fn compare_view_model_properties(
    rust_file: &RuntimeFile,
    cpp_properties: &[CppProbeViewModelProperty],
    rust_properties: &[&RuntimeObject],
    view_model_index: usize,
    fixture_path: &str,
) {
    assert_eq!(
        cpp_properties.len(),
        rust_properties.len(),
        "view model {view_model_index} property count mismatch for reference fixture {fixture_path}"
    );
    for (property_index, (cpp_property, rust_property)) in
        cpp_properties.iter().zip(rust_properties).enumerate()
    {
        assert_eq!(
            cpp_property.index, property_index,
            "view model {view_model_index} C++ property index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_property.core_type, rust_property.type_key,
            "view model {view_model_index} property {property_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_property.name,
            rust_property.string_property("name").unwrap_or_default(),
            "view model {view_model_index} property {property_index} name mismatch for reference fixture {fixture_path}"
        );
        compare_view_model_property_enum_lookup_result(
            rust_file,
            cpp_property,
            rust_property,
            &format!(
                "view model {view_model_index} property {property_index} in reference fixture {fixture_path}"
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
    }
}

fn compare_view_model_name_lookups(
    rust_file: &RuntimeFile,
    cpp_view_model: &CppProbeViewModel,
    rust_view_model: &nuxie_binary::RuntimeViewModel<'_>,
    view_model_index: usize,
    label: &str,
) {
    for lookup in &cpp_view_model.property_name_lookups {
        let rust_property = rust_file.view_model_property_named(view_model_index, &lookup.name);
        match (lookup.result_index, rust_property) {
            (None, None) => {}
            (Some(property_index), Some(rust_property)) => {
                let expected = rust_view_model.properties[property_index];
                assert_eq!(
                    expected.id, rust_property.id,
                    "{label} view model {view_model_index} property lookup {:?} object mismatch",
                    lookup.name
                );
                assert_eq!(
                    lookup.core_type,
                    Some(rust_property.type_key),
                    "{label} view model {view_model_index} property lookup {:?} core type mismatch",
                    lookup.name
                );
            }
            (None, Some(rust_property)) => panic!(
                "{label} view model {view_model_index} property lookup {:?} unexpectedly resolved in Rust to {}",
                lookup.name, rust_property.type_name
            ),
            (Some(property_index), None) => panic!(
                "{label} view model {view_model_index} property lookup {:?} missing Rust property index {property_index}",
                lookup.name
            ),
        }
    }

    for lookup in &cpp_view_model.instance_name_lookups {
        let rust_instance = rust_file.view_model_instance_named(view_model_index, &lookup.name);
        match (lookup.result_index, rust_instance.as_ref()) {
            (None, None) => {}
            (Some(instance_index), Some(rust_instance)) => {
                let expected = rust_view_model.instances[instance_index].object;
                assert_eq!(
                    expected.id, rust_instance.object.id,
                    "{label} view model {view_model_index} instance lookup {:?} object mismatch",
                    lookup.name
                );
                assert_eq!(
                    lookup.core_type,
                    Some(rust_instance.object.type_key),
                    "{label} view model {view_model_index} instance lookup {:?} core type mismatch",
                    lookup.name
                );
                assert_eq!(
                    instance_index, rust_instance.instance_index,
                    "{label} view model {view_model_index} instance lookup {:?} index mismatch",
                    lookup.name
                );
            }
            (None, Some(rust_instance)) => panic!(
                "{label} view model {view_model_index} instance lookup {:?} unexpectedly resolved in Rust to {}",
                lookup.name, rust_instance.object.type_name
            ),
            (Some(instance_index), None) => panic!(
                "{label} view model {view_model_index} instance lookup {:?} missing Rust instance index {instance_index}",
                lookup.name
            ),
        }
    }

    for (instance_index, (cpp_instance, rust_instance)) in cpp_view_model
        .instances
        .iter()
        .zip(&rust_view_model.instances)
        .enumerate()
    {
        for lookup in &cpp_instance.value_name_lookups {
            let rust_value = rust_file
                .view_model_instance_value_named_for_object(rust_instance.object, &lookup.name);
            match (lookup.result_index, rust_value) {
                (None, None) => {}
                (Some(value_index), Some(rust_value)) => {
                    let expected = rust_instance.values[value_index].object;
                    assert_eq!(
                        expected.id, rust_value.id,
                        "{label} view model {view_model_index} instance {instance_index} value lookup {:?} object mismatch",
                        lookup.name
                    );
                    assert_eq!(
                        lookup.core_type,
                        Some(rust_value.type_key),
                        "{label} view model {view_model_index} instance {instance_index} value lookup {:?} core type mismatch",
                        lookup.name
                    );
                }
                (None, Some(rust_value)) => panic!(
                    "{label} view model {view_model_index} instance {instance_index} value lookup {:?} unexpectedly resolved in Rust to {}",
                    lookup.name, rust_value.type_name
                ),
                (Some(value_index), None) => panic!(
                    "{label} view model {view_model_index} instance {instance_index} value lookup {:?} missing Rust value index {value_index}",
                    lookup.name
                ),
            }
        }
    }
}

fn compare_view_model_symbol_lookups(
    rust_file: &RuntimeFile,
    cpp_view_model: &CppProbeViewModel,
    rust_view_model: &nuxie_binary::RuntimeViewModel<'_>,
    view_model_index: usize,
    label: &str,
) {
    for lookup in &cpp_view_model.property_symbol_lookups {
        let rust_property =
            rust_file.view_model_property_for_symbol(view_model_index, lookup.symbol);
        match (lookup.result_index, rust_property) {
            (None, None) => {}
            (Some(property_index), Some(rust_property)) => {
                let expected = rust_view_model.properties[property_index];
                assert_eq!(
                    expected.id, rust_property.id,
                    "{label} view model {view_model_index} property symbol lookup {} object mismatch",
                    lookup.symbol
                );
                assert_eq!(
                    lookup.core_type,
                    Some(rust_property.type_key),
                    "{label} view model {view_model_index} property symbol lookup {} core type mismatch",
                    lookup.symbol
                );
            }
            (None, Some(rust_property)) => panic!(
                "{label} view model {view_model_index} property symbol lookup {} unexpectedly resolved in Rust to {}",
                lookup.symbol, rust_property.type_name
            ),
            (Some(property_index), None) => panic!(
                "{label} view model {view_model_index} property symbol lookup {} missing Rust property index {property_index}",
                lookup.symbol
            ),
        }
    }

    for (instance_index, (cpp_instance, rust_instance)) in cpp_view_model
        .instances
        .iter()
        .zip(&rust_view_model.instances)
        .enumerate()
    {
        for lookup in &cpp_instance.value_symbol_lookups {
            let rust_value = rust_file
                .view_model_instance_value_for_symbol_object(rust_instance.object, lookup.symbol);
            match (lookup.result_index, rust_value) {
                (None, None) => {}
                (Some(value_index), Some(rust_value)) => {
                    let expected = rust_instance.values[value_index].object;
                    assert_eq!(
                        expected.id, rust_value.id,
                        "{label} view model {view_model_index} instance {instance_index} value symbol lookup {} object mismatch",
                        lookup.symbol
                    );
                    assert_eq!(
                        lookup.core_type,
                        Some(rust_value.type_key),
                        "{label} view model {view_model_index} instance {instance_index} value symbol lookup {} core type mismatch",
                        lookup.symbol
                    );
                }
                (None, Some(rust_value)) => panic!(
                    "{label} view model {view_model_index} instance {instance_index} value symbol lookup {} unexpectedly resolved in Rust to {}",
                    lookup.symbol, rust_value.type_name
                ),
                (Some(value_index), None) => panic!(
                    "{label} view model {view_model_index} instance {instance_index} value symbol lookup {} missing Rust value index {value_index}",
                    lookup.symbol
                ),
            }
        }
    }
}

fn compare_view_model_instance_value_id_lookups_result(
    rust_file: &RuntimeFile,
    cpp_instance: &CppProbeViewModelInstance,
    rust_instance: &RuntimeViewModelInstance<'_>,
    view_model_index: usize,
    instance_index: usize,
    label: &str,
) -> Result<(), String> {
    for lookup in &cpp_instance.value_id_lookups {
        let rust_value = rust_file.view_model_instance_value_for_property_id_object(
            rust_instance.object,
            lookup.property_id,
        );
        match (lookup.result_index, rust_value) {
            (None, None) => {}
            (Some(value_index), Some(rust_value)) => {
                let Some(expected) = rust_instance
                    .values
                    .get(value_index)
                    .map(|value| value.object)
                else {
                    return Err(format!(
                        "{label}: view model {view_model_index} instance {instance_index} property-id lookup {} C++ index {value_index} out of Rust range",
                        lookup.property_id
                    ));
                };
                if expected.id != rust_value.id {
                    return Err(format!(
                        "{label}: view model {view_model_index} instance {instance_index} property-id lookup {} object mismatch, expected Rust object {}, got {}",
                        lookup.property_id, expected.id, rust_value.id
                    ));
                }
                if lookup.core_type != Some(rust_value.type_key) {
                    return Err(format!(
                        "{label}: view model {view_model_index} instance {instance_index} property-id lookup {} core type mismatch, C++ {:?}, Rust {}",
                        lookup.property_id, lookup.core_type, rust_value.type_key
                    ));
                }
            }
            (None, Some(rust_value)) => {
                return Err(format!(
                    "{label}: view model {view_model_index} instance {instance_index} property-id lookup {} unexpectedly resolved in Rust to {}",
                    lookup.property_id, rust_value.type_name
                ));
            }
            (Some(value_index), None) => {
                return Err(format!(
                    "{label}: view model {view_model_index} instance {instance_index} property-id lookup {} missing Rust value index {value_index}",
                    lookup.property_id
                ));
            }
        }
    }
    Ok(())
}

fn compare_view_model_default_instance(
    rust_file: &RuntimeFile,
    cpp_view_model: &CppProbeViewModel,
    rust_view_model: &nuxie_binary::RuntimeViewModel<'_>,
    view_model_index: usize,
    label: &str,
) {
    let rust_instance = rust_file.view_model_default_instance(view_model_index);
    match (
        cpp_view_model.default_instance.as_ref(),
        rust_instance.as_ref(),
    ) {
        (None, None) => {}
        (Some(cpp_instance), Some(rust_instance)) => {
            let expected = rust_view_model.instances[cpp_instance.instance_index].object;
            assert_eq!(
                expected.id, rust_instance.object.id,
                "{label} view model {view_model_index} default instance object mismatch"
            );
            assert_eq!(
                cpp_instance.view_model_index, rust_instance.view_model_index,
                "{label} view model {view_model_index} default instance view-model index mismatch"
            );
            assert_eq!(
                cpp_instance.instance_index, rust_instance.instance_index,
                "{label} view model {view_model_index} default instance index mismatch"
            );
            assert_eq!(
                cpp_instance.core_type, rust_instance.object.type_key,
                "{label} view model {view_model_index} default instance core type mismatch"
            );
            assert_eq!(
                Some(cpp_instance.name.as_str()),
                rust_instance.object.string_property("name"),
                "{label} view model {view_model_index} default instance name mismatch"
            );
            assert_eq!(
                Some(cpp_instance.view_model_id),
                rust_instance.object.uint_property("viewModelId"),
                "{label} view model {view_model_index} default instance view-model id mismatch"
            );
        }
        (Some(cpp_instance), None) => panic!(
            "{label} view model {view_model_index} missing Rust default instance index {}",
            cpp_instance.instance_index
        ),
        (None, Some(rust_instance)) => panic!(
            "{label} view model {view_model_index} unexpectedly resolved Rust default instance {}",
            rust_instance.object.type_name
        ),
    }
}

fn compare_view_model_default_instance_result(
    rust_file: &RuntimeFile,
    cpp_view_model: &CppProbeViewModel,
    rust_view_model: &nuxie_binary::RuntimeViewModel<'_>,
    view_model_index: usize,
    label: &str,
) -> Result<(), String> {
    let rust_instance = rust_file.view_model_default_instance(view_model_index);
    match (
        cpp_view_model.default_instance.as_ref(),
        rust_instance.as_ref(),
    ) {
        (None, None) => Ok(()),
        (Some(cpp_instance), Some(rust_instance)) => {
            let Some(expected) = rust_view_model
                .instances
                .get(cpp_instance.instance_index)
                .map(|instance| instance.object)
            else {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance C++ index {} out of Rust range",
                    cpp_instance.instance_index
                ));
            };
            if expected.id != rust_instance.object.id {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance object mismatch, expected Rust object {}, got {}",
                    expected.id, rust_instance.object.id
                ));
            }
            if cpp_instance.view_model_index != rust_instance.view_model_index {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance view-model index mismatch, C++ {}, Rust {}",
                    cpp_instance.view_model_index, rust_instance.view_model_index
                ));
            }
            if cpp_instance.instance_index != rust_instance.instance_index {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance index mismatch, C++ {}, Rust {}",
                    cpp_instance.instance_index, rust_instance.instance_index
                ));
            }
            if cpp_instance.core_type != rust_instance.object.type_key {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance core type mismatch, C++ {}, Rust {}",
                    cpp_instance.core_type, rust_instance.object.type_key
                ));
            }
            let rust_name = rust_instance
                .object
                .string_property("name")
                .unwrap_or_default();
            if cpp_instance.name != rust_name {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance name mismatch, C++ {:?}, Rust {:?}",
                    cpp_instance.name, rust_name
                ));
            }
            if rust_instance.object.uint_property("viewModelId") != Some(cpp_instance.view_model_id)
            {
                return Err(format!(
                    "{label}: view model {view_model_index} default instance view-model id mismatch, C++ {}, Rust {:?}",
                    cpp_instance.view_model_id,
                    rust_instance.object.uint_property("viewModelId")
                ));
            }
            Ok(())
        }
        (Some(cpp_instance), None) => Err(format!(
            "{label}: view model {view_model_index} missing Rust default instance index {}",
            cpp_instance.instance_index
        )),
        (None, Some(rust_instance)) => Err(format!(
            "{label}: view model {view_model_index} unexpectedly resolved Rust default instance {}",
            rust_instance.object.type_name
        )),
    }
}

fn compare_artboard_view_model_result(
    rust_file: &RuntimeFile,
    cpp_artboard: &CppProbeArtboard,
    rust_artboard: &RuntimeObject,
    artboard_index: usize,
    label: &str,
) -> Result<(), String> {
    let rust_view_model = rust_file.resolved_view_model_for_artboard_object(rust_artboard);
    match (cpp_artboard.view_model.as_ref(), rust_view_model.as_ref()) {
        (None, None) => Ok(()),
        (Some(cpp_view_model), Some(rust_view_model)) => {
            let expected = rust_file
                .view_model(cpp_view_model.view_model_index)
                .ok_or_else(|| {
                    format!(
                        "{label}: artboard {artboard_index} C++ view model index {} is out of Rust range",
                        cpp_view_model.view_model_index
                    )
                })?;
            if expected.object.id != rust_view_model.object.id {
                return Err(format!(
                    "{label}: artboard {artboard_index} view model object mismatch, expected Rust object {}, got {}",
                    expected.object.id, rust_view_model.object.id
                ));
            }
            if cpp_view_model.view_model_index != rust_view_model.view_model_index {
                return Err(format!(
                    "{label}: artboard {artboard_index} view model index mismatch, C++ {}, Rust {}",
                    cpp_view_model.view_model_index, rust_view_model.view_model_index
                ));
            }
            if cpp_view_model.core_type != rust_view_model.object.type_key {
                return Err(format!(
                    "{label}: artboard {artboard_index} view model core type mismatch, C++ {}, Rust {}",
                    cpp_view_model.core_type, rust_view_model.object.type_key
                ));
            }
            let rust_name = rust_view_model
                .object
                .string_property("name")
                .unwrap_or_default();
            if cpp_view_model.name != rust_name {
                return Err(format!(
                    "{label}: artboard {artboard_index} view model name mismatch, C++ {:?}, Rust {:?}",
                    cpp_view_model.name, rust_name
                ));
            }
            Ok(())
        }
        (Some(cpp_view_model), None) => Err(format!(
            "{label}: artboard {artboard_index} missing Rust view model index {}",
            cpp_view_model.view_model_index
        )),
        (None, Some(rust_view_model)) => Err(format!(
            "{label}: artboard {artboard_index} unexpectedly resolved Rust view model {}",
            rust_view_model.object.type_name
        )),
    }
}

fn compare_view_model_instances(
    rust_file: &RuntimeFile,
    cpp_instances: &[CppProbeViewModelInstance],
    rust_instances: &[RuntimeViewModelInstance<'_>],
    view_model_index: usize,
    fixture_path: &str,
) {
    assert_eq!(
        cpp_instances.len(),
        rust_instances.len(),
        "view model {view_model_index} instance count mismatch for reference fixture {fixture_path}"
    );
    for (instance_index, (cpp_instance, rust_instance)) in
        cpp_instances.iter().zip(rust_instances).enumerate()
    {
        assert_eq!(
            cpp_instance.index, instance_index,
            "view model {view_model_index} C++ instance index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_instance.core_type, rust_instance.object.type_key,
            "view model {view_model_index} instance {instance_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_instance.name,
            rust_instance
                .object
                .string_property("name")
                .unwrap_or_default(),
            "view model {view_model_index} instance {instance_index} name mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            rust_instance.object.uint_property("viewModelId"),
            Some(cpp_instance.view_model_id),
            "view model {view_model_index} instance {instance_index} view model id mismatch for reference fixture {fixture_path}"
        );
        compare_view_model_instance_values(
            rust_file,
            &cpp_instance.values,
            &rust_instance.values,
            view_model_index,
            instance_index,
            fixture_path,
        );
        compare_view_model_instance_value_id_lookups_result(
            rust_file,
            cpp_instance,
            rust_instance,
            view_model_index,
            instance_index,
            fixture_path,
        )
        .unwrap_or_else(|err| panic!("{err}"));
    }
}

fn compare_view_model_instance_values(
    rust_file: &RuntimeFile,
    cpp_values: &[CppProbeViewModelInstanceValue],
    rust_values: &[nuxie_binary::RuntimeViewModelInstanceValue<'_>],
    view_model_index: usize,
    instance_index: usize,
    fixture_path: &str,
) {
    assert_eq!(
        cpp_values.len(),
        rust_values.len(),
        "view model {view_model_index} instance {instance_index} value count mismatch for reference fixture {fixture_path}"
    );
    for (value_index, (cpp_value, rust_value)) in cpp_values.iter().zip(rust_values).enumerate() {
        assert_eq!(
            cpp_value.index, value_index,
            "view model {view_model_index} instance {instance_index} C++ value index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_value.core_type, rust_value.object.type_key,
            "view model {view_model_index} instance {instance_index} value {value_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            rust_value.object.uint_property("viewModelPropertyId"),
            Some(cpp_value.view_model_property_id),
            "view model {view_model_index} instance {instance_index} value {value_index} property id mismatch for reference fixture {fixture_path}"
        );
        assert_view_model_instance_view_model_reference(
            rust_file,
            cpp_value.reference_view_model_instance.as_ref(),
            rust_value.object,
            &format!(
                "view model {view_model_index} instance {instance_index} value {value_index} reference for reference fixture {fixture_path}"
            ),
        );
        compare_view_model_instance_asset_snapshot_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!(
                "view model {view_model_index} instance {instance_index} value {value_index} asset snapshot for reference fixture {fixture_path}"
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        compare_view_model_instance_enum_runtime_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!(
                "view model {view_model_index} instance {instance_index} value {value_index} enum runtime for reference fixture {fixture_path}"
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        compare_view_model_instance_value_runtime_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!(
                "view model {view_model_index} instance {instance_index} value {value_index} runtime value for reference fixture {fixture_path}"
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        compare_view_model_instance_source_data_value_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!(
                "view model {view_model_index} instance {instance_index} value {value_index} source data value for reference fixture {fixture_path}"
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
        compare_view_model_instance_list_items(
            rust_file,
            &cpp_value.items,
            &rust_value.list_items,
            view_model_index,
            instance_index,
            value_index,
            fixture_path,
        );
    }
}

fn assert_view_model_instance_view_model_reference(
    rust_file: &RuntimeFile,
    cpp_reference: Option<&CppProbeViewModelInstanceReference>,
    rust_value: &RuntimeObject,
    label: &str,
) {
    compare_view_model_instance_view_model_reference_result(
        rust_file,
        cpp_reference,
        rust_value,
        label,
    )
    .unwrap_or_else(|err| panic!("{err}"));
}

fn compare_view_model_instance_view_model_reference_result(
    rust_file: &RuntimeFile,
    cpp_reference: Option<&CppProbeViewModelInstanceReference>,
    rust_value: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let rust_reference = rust_file.referenced_view_model_instance_for_value_object(rust_value);
    match (cpp_reference, rust_reference) {
        (None, None) => Ok(()),
        (Some(cpp), Some(rust)) => {
            if cpp.view_model_index != rust.view_model_index {
                return Err(format!(
                    "{label} view model index mismatch, C++ {}, Rust {}",
                    cpp.view_model_index, rust.view_model_index
                ));
            }
            if cpp.instance_index != rust.instance_index {
                return Err(format!(
                    "{label} instance index mismatch, C++ {}, Rust {}",
                    cpp.instance_index, rust.instance_index
                ));
            }
            if cpp.core_type != rust.object.type_key {
                return Err(format!(
                    "{label} core type mismatch, C++ {}, Rust {}",
                    cpp.core_type, rust.object.type_key
                ));
            }
            let rust_name = rust.object.string_property("name").unwrap_or_default();
            if cpp.name != rust_name {
                return Err(format!(
                    "{label} name mismatch, C++ {:?}, Rust {:?}",
                    cpp.name, rust_name
                ));
            }
            if rust.object.uint_property("viewModelId") != Some(cpp.view_model_id) {
                return Err(format!(
                    "{label} view model id mismatch, C++ {}, Rust {:?}",
                    cpp.view_model_id,
                    rust.object.uint_property("viewModelId")
                ));
            }
            Ok(())
        }
        (None, Some(rust)) => Err(format!(
            "{label} unexpectedly resolved in Rust to view model {} instance {} ({})",
            rust.view_model_index, rust.instance_index, rust.object.type_name
        )),
        (Some(cpp), None) => Err(format!(
            "{label} did not resolve in Rust but C++ resolved to view model {} instance {} ({})",
            cpp.view_model_index, cpp.instance_index, cpp.name
        )),
    }
}

fn compare_view_model_instance_asset_snapshot_result(
    rust_file: &RuntimeFile,
    cpp_value: &CppProbeViewModelInstanceValue,
    rust_value: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let rust_snapshot = rust_file.view_model_instance_asset_file_assets_for_object(rust_value);
    if cpp_value.asset_snapshot.len() != rust_snapshot.len() {
        return Err(format!(
            "{label}: snapshot count mismatch, C++ {}, Rust {}",
            cpp_value.asset_snapshot.len(),
            rust_snapshot.len()
        ));
    }

    for (asset_index, (cpp_asset, rust_asset)) in cpp_value
        .asset_snapshot
        .iter()
        .zip(&rust_snapshot)
        .enumerate()
    {
        compare_view_model_instance_asset_snapshot_item_result(
            cpp_asset,
            rust_asset,
            asset_index,
            &format!("{label}: asset {asset_index}"),
        )?;
    }

    let rust_resolved =
        rust_file.resolved_file_asset_for_view_model_instance_asset_object(rust_value);
    match (cpp_value.resolved_asset.as_ref(), rust_resolved) {
        (None, None) => Ok(()),
        (Some(cpp_asset), Some(rust_asset)) => {
            compare_view_model_instance_asset_snapshot_item_result(
                cpp_asset,
                rust_asset,
                cpp_asset.index,
                &format!("{label}: resolved asset"),
            )
        }
        (None, Some(rust_asset)) => Err(format!(
            "{label}: Rust resolved asset {} ({}) but C++ did not",
            rust_asset.id, rust_asset.type_name
        )),
        (Some(cpp_asset), None) => Err(format!(
            "{label}: C++ resolved asset {} ({}) but Rust did not",
            cpp_asset.index, cpp_asset.name
        )),
    }
}

fn compare_view_model_instance_asset_snapshot_item_result(
    cpp_asset: &CppProbeViewModelInstanceAsset,
    rust_asset: &RuntimeObject,
    expected_index: usize,
    label: &str,
) -> Result<(), String> {
    if cpp_asset.index != expected_index {
        return Err(format!(
            "{label}: C++ index mismatch, got {}",
            cpp_asset.index
        ));
    }
    if cpp_asset.core_type != rust_asset.type_key {
        return Err(format!(
            "{label}: core type mismatch, C++ {}, Rust {}",
            cpp_asset.core_type, rust_asset.type_key
        ));
    }
    let rust_name = rust_asset.string_property("name").unwrap_or_default();
    if cpp_asset.name != rust_name {
        return Err(format!(
            "{label}: name mismatch, C++ {:?}, Rust {:?}",
            cpp_asset.name, rust_name
        ));
    }
    if rust_asset.uint_property("assetId") != Some(cpp_asset.asset_id) {
        return Err(format!(
            "{label}: asset id mismatch, C++ {}, Rust {:?}",
            cpp_asset.asset_id,
            rust_asset.uint_property("assetId")
        ));
    }
    let rust_cdn_base_url = rust_asset.string_property("cdnBaseUrl").unwrap_or_default();
    if cpp_asset.cdn_base_url != rust_cdn_base_url {
        return Err(format!(
            "{label}: CDN base URL mismatch, C++ {:?}, Rust {:?}",
            cpp_asset.cdn_base_url, rust_cdn_base_url
        ));
    }
    let rust_cdn_uuid_string = rust_asset.file_asset_cdn_uuid_string().unwrap_or_default();
    if cpp_asset.cdn_uuid_string != rust_cdn_uuid_string {
        return Err(format!(
            "{label}: CDN UUID string mismatch, C++ {:?}, Rust {:?}",
            cpp_asset.cdn_uuid_string, rust_cdn_uuid_string
        ));
    }
    let rust_file_extension = rust_asset.file_asset_extension().unwrap_or_default();
    if cpp_asset.file_extension != rust_file_extension {
        return Err(format!(
            "{label}: extension mismatch, C++ {:?}, Rust {:?}",
            cpp_asset.file_extension, rust_file_extension
        ));
    }
    let rust_unique_name = rust_asset.file_asset_unique_name().unwrap_or_default();
    if cpp_asset.unique_name != rust_unique_name {
        return Err(format!(
            "{label}: unique name mismatch, C++ {:?}, Rust {:?}",
            cpp_asset.unique_name, rust_unique_name
        ));
    }
    let rust_unique_filename = rust_asset.file_asset_unique_filename().unwrap_or_default();
    if cpp_asset.unique_filename != rust_unique_filename {
        return Err(format!(
            "{label}: unique filename mismatch, C++ {:?}, Rust {:?}",
            cpp_asset.unique_filename, rust_unique_filename
        ));
    }

    Ok(())
}

fn compare_manifest_result(
    cpp_manifest: Option<&CppProbeManifest>,
    rust_manifest: Option<RuntimeManifest>,
    label: &str,
) -> Result<(), String> {
    match (cpp_manifest, rust_manifest) {
        (None, None) => Ok(()),
        (Some(cpp), Some(rust)) => {
            if cpp.names.len() != rust.names.len() {
                return Err(format!(
                    "{label}: manifest name count mismatch, C++ {}, Rust {}",
                    cpp.names.len(),
                    rust.names.len()
                ));
            }
            for (index, (cpp_name, (rust_id, rust_name))) in
                cpp.names.iter().zip(&rust.names).enumerate()
            {
                if cpp_name.id != *rust_id {
                    return Err(format!(
                        "{label}: manifest name {index} id mismatch, C++ {}, Rust {}",
                        cpp_name.id, rust_id
                    ));
                }
                let Some(rust_value) = rust_name.as_str() else {
                    return Err(format!(
                        "{label}: manifest name {} is not UTF-8 in Rust",
                        cpp_name.id
                    ));
                };
                if cpp_name.value != rust_value {
                    return Err(format!(
                        "{label}: manifest name {} value mismatch, C++ {:?}, Rust {:?}",
                        cpp_name.id, cpp_name.value, rust_value
                    ));
                }
            }

            if cpp.paths.len() != rust.paths.len() {
                return Err(format!(
                    "{label}: manifest path count mismatch, C++ {}, Rust {}",
                    cpp.paths.len(),
                    rust.paths.len()
                ));
            }
            for (index, (cpp_path, (rust_id, rust_path))) in
                cpp.paths.iter().zip(&rust.paths).enumerate()
            {
                if cpp_path.id != *rust_id {
                    return Err(format!(
                        "{label}: manifest path {index} id mismatch, C++ {}, Rust {}",
                        cpp_path.id, rust_id
                    ));
                }
                if cpp_path.path != *rust_path {
                    return Err(format!(
                        "{label}: manifest path {} mismatch, C++ {:?}, Rust {:?}",
                        cpp_path.id, cpp_path.path, rust_path
                    ));
                }
            }
            Ok(())
        }
        (None, Some(rust)) => Err(format!(
            "{label}: Rust decoded manifest with {} names and {} paths but C++ did not",
            rust.names.len(),
            rust.paths.len()
        )),
        (Some(cpp), None) => Err(format!(
            "{label}: C++ decoded manifest with {} names and {} paths but Rust did not",
            cpp.names.len(),
            cpp.paths.len()
        )),
    }
}

fn compare_view_model_instance_list_items(
    rust_file: &RuntimeFile,
    cpp_items: &[CppProbeViewModelInstanceListItem],
    rust_items: &[&RuntimeObject],
    view_model_index: usize,
    instance_index: usize,
    value_index: usize,
    fixture_path: &str,
) {
    assert_eq!(
        cpp_items.len(),
        rust_items.len(),
        "view model {view_model_index} instance {instance_index} value {value_index} list item count mismatch for reference fixture {fixture_path}"
    );
    for (item_index, (cpp_item, rust_item)) in cpp_items.iter().zip(rust_items).enumerate() {
        assert_eq!(
            cpp_item.index, item_index,
            "view model {view_model_index} instance {instance_index} value {value_index} C++ list item index mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            cpp_item.core_type, rust_item.type_key,
            "view model {view_model_index} instance {instance_index} value {value_index} list item {item_index} core type mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            rust_item.uint_property("viewModelId"),
            Some(cpp_item.view_model_id),
            "view model {view_model_index} instance {instance_index} value {value_index} list item {item_index} view model id mismatch for reference fixture {fixture_path}"
        );
        assert_eq!(
            rust_item.uint_property("viewModelInstanceId"),
            Some(cpp_item.view_model_instance_id),
            "view model {view_model_index} instance {instance_index} value {value_index} list item {item_index} view model instance id mismatch for reference fixture {fixture_path}"
        );
        compare_view_model_instance_list_item_reference_result(
            rust_file,
            cpp_item.referenced_view_model_instance.as_ref(),
            rust_item,
            &format!(
                "view model {view_model_index} instance {instance_index} value {value_index} list item {item_index} reference for reference fixture {fixture_path}"
            ),
        )
        .unwrap_or_else(|err| panic!("{err}"));
    }
}

fn assert_view_model_instance_value_property_link(
    rust_file: &RuntimeFile,
    cpp_value: &CppProbeViewModelInstanceValue,
    rust_value: &nuxie_binary::RuntimeViewModelInstanceValue<'_>,
    label: &str,
) {
    compare_view_model_instance_value_property_link_result(
        rust_file,
        cpp_value,
        rust_value.object,
        label,
    )
    .unwrap_or_else(|err| panic!("{err}"));
}

fn compare_view_model_instance_value_property_link_result(
    rust_file: &RuntimeFile,
    cpp_value: &CppProbeViewModelInstanceValue,
    rust_value: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let rust_property = rust_file.view_model_property_for_instance_value_object(rust_value);
    match (cpp_value.view_model_property_core_type, rust_property) {
        (None, None) => {}
        (Some(cpp_core_type), Some(rust_property)) => {
            if cpp_core_type != rust_property.type_key {
                return Err(format!(
                    "{label}: linked property core type mismatch, C++ {}, Rust {}",
                    cpp_core_type, rust_property.type_key
                ));
            }
        }
        (None, Some(rust_property)) => {
            return Err(format!(
                "{label}: Rust resolved linked property {} ({}) but C++ did not",
                rust_property.id, rust_property.type_name
            ));
        }
        (Some(cpp_core_type), None) => {
            return Err(format!(
                "{label}: C++ resolved linked property core type {cpp_core_type} but Rust did not"
            ));
        }
    }

    match (cpp_value.view_model_property_name.as_deref(), rust_property) {
        (None, None) => {}
        (Some(cpp_name), Some(rust_property)) => {
            let rust_name = rust_property.string_property("name").unwrap_or_default();
            if cpp_name != rust_name {
                return Err(format!(
                    "{label}: linked property name mismatch, C++ {:?}, Rust {:?}",
                    cpp_name, rust_name
                ));
            }
        }
        (None, Some(rust_property)) => {
            return Err(format!(
                "{label}: Rust resolved linked property name {:?} but C++ did not",
                rust_property.string_property("name")
            ));
        }
        (Some(cpp_name), None) => {
            return Err(format!(
                "{label}: C++ resolved linked property name {cpp_name:?} but Rust did not"
            ));
        }
    }

    let rust_value_name = rust_file
        .view_model_instance_value_name_for_object(rust_value)
        .unwrap_or_default();
    if cpp_value.name != rust_value_name {
        return Err(format!(
            "{label}: ViewModelInstanceValue::name() mismatch, C++ {:?}, Rust {:?}",
            cpp_value.name, rust_value_name
        ));
    }

    Ok(())
}

fn compare_view_model_instance_enum_runtime_result(
    rust_file: &RuntimeFile,
    cpp_value: &CppProbeViewModelInstanceValue,
    rust_value: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let rust_values = rust_file.view_model_instance_enum_value_keys_for_object(rust_value);
    match (cpp_value.enum_runtime.as_ref(), rust_values.as_ref()) {
        (None, None) => return Ok(()),
        (None, Some(rust_values)) => {
            return Err(format!(
                "{label}: Rust resolved enum runtime values {:?} but C++ did not",
                rust_values
            ));
        }
        (Some(cpp_enum), None) => {
            return Err(format!(
                "{label}: C++ resolved enum runtime {:?} but Rust did not",
                cpp_enum.enum_type
            ));
        }
        (Some(cpp_enum), Some(rust_values)) => {
            let rust_value_key = rust_file
                .view_model_instance_enum_value_key_for_object(rust_value)
                .unwrap_or_default();
            if rust_value_key != cpp_enum.value.as_bytes() {
                return Err(format!(
                    "{label}: enum runtime value mismatch, C++ {:?}, Rust {:?}",
                    cpp_enum.value, rust_value_key
                ));
            }

            let rust_value_index = rust_file
                .view_model_instance_enum_value_index_for_object(rust_value)
                .unwrap_or_default();
            if rust_value_index != cpp_enum.value_index {
                return Err(format!(
                    "{label}: enum runtime value index mismatch, C++ {}, Rust {}",
                    cpp_enum.value_index, rust_value_index
                ));
            }

            let cpp_values = cpp_enum
                .values
                .iter()
                .map(|value| value.as_bytes())
                .collect::<Vec<_>>();
            if *rust_values != cpp_values {
                return Err(format!(
                    "{label}: enum runtime values mismatch, C++ {:?}, Rust {:?}",
                    cpp_enum.values, rust_values
                ));
            }

            let rust_enum_type = rust_file
                .view_model_instance_enum_type_for_object(rust_value)
                .unwrap_or_default();
            if rust_enum_type != cpp_enum.enum_type.as_bytes() {
                return Err(format!(
                    "{label}: enum runtime enum type mismatch, C++ {:?}, Rust {:?}",
                    cpp_enum.enum_type, rust_enum_type
                ));
            }
        }
    }

    Ok(())
}

fn compare_view_model_instance_value_runtime_result(
    rust_file: &RuntimeFile,
    cpp_value: &CppProbeViewModelInstanceValue,
    rust_value: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let cpp_runtime = cpp_value.value_runtime.as_ref().ok_or_else(|| {
        format!("{label}: C++ probe did not emit a view-model instance value runtime")
    })?;
    let rust_data_type = rust_file
        .view_model_instance_value_data_type_for_object(rust_value)
        .ok_or_else(|| format!("{label}: Rust did not resolve a value runtime data type"))?;
    if cpp_runtime.data_type != rust_data_type.as_cpp_u32() {
        return Err(format!(
            "{label}: runtime data type mismatch, C++ {}, Rust {}",
            cpp_runtime.data_type,
            rust_data_type.as_cpp_u32()
        ));
    }

    match rust_data_type {
        RuntimeDataType::Number => {
            let rust_value = rust_file
                .view_model_instance_number_value_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve number value"))?;
            let cpp_value = cpp_runtime
                .number_value
                .ok_or_else(|| format!("{label}: C++ did not emit number value"))?;
            if (cpp_value - rust_value).abs() > 0.00001 {
                return Err(format!(
                    "{label}: number value mismatch, C++ {cpp_value}, Rust {rust_value}"
                ));
            }
        }
        RuntimeDataType::String => {
            let rust_value = rust_file
                .view_model_instance_string_value_for_object(rust_value)
                .unwrap_or_default();
            let cpp_value = cpp_runtime.string_value.as_deref().unwrap_or_default();
            if cpp_value != rust_value {
                return Err(format!(
                    "{label}: string value mismatch, C++ {:?}, Rust {:?}",
                    cpp_value, rust_value
                ));
            }
        }
        RuntimeDataType::Boolean => {
            let rust_value = rust_file
                .view_model_instance_boolean_value_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve boolean value"))?;
            if cpp_runtime.boolean_value != Some(rust_value) {
                return Err(format!(
                    "{label}: boolean value mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.boolean_value
                ));
            }
        }
        RuntimeDataType::Color => {
            let rust_value = rust_file
                .view_model_instance_color_value_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve color value"))?;
            if cpp_runtime.color_value != Some(rust_value) {
                return Err(format!(
                    "{label}: color value mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.color_value
                ));
            }
        }
        RuntimeDataType::List => {
            let rust_value = rust_file
                .view_model_instance_list_size_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve list size"))?;
            if cpp_runtime.list_size != Some(rust_value) {
                return Err(format!(
                    "{label}: list size mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.list_size
                ));
            }
        }
        RuntimeDataType::Trigger => {
            let rust_value = rust_file
                .view_model_instance_trigger_count_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve trigger count"))?;
            if cpp_runtime.trigger_count != Some(rust_value) {
                return Err(format!(
                    "{label}: trigger count mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.trigger_count
                ));
            }
        }
        RuntimeDataType::ViewModel => {
            let rust_value = rust_file
                .view_model_instance_view_model_index_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve view-model index"))?;
            if cpp_runtime.view_model_index != Some(rust_value) {
                return Err(format!(
                    "{label}: view-model index mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.view_model_index
                ));
            }
        }
        RuntimeDataType::SymbolListIndex => {
            let rust_value = rust_file
                .view_model_instance_symbol_list_index_value_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve symbol-list index"))?;
            if cpp_runtime.integer_value != Some(rust_value) {
                return Err(format!(
                    "{label}: symbol-list index mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.integer_value
                ));
            }
        }
        RuntimeDataType::AssetImage => {
            let rust_value = rust_file
                .view_model_instance_asset_index_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve asset index"))?;
            if cpp_runtime.asset_index != Some(rust_value) {
                return Err(format!(
                    "{label}: asset index mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.asset_index
                ));
            }
        }
        RuntimeDataType::AssetFont => {
            let rust_value = rust_file
                .view_model_instance_font_asset_index_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve font asset index"))?;
            if cpp_runtime.asset_index != Some(rust_value) {
                return Err(format!(
                    "{label}: font asset index mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.asset_index
                ));
            }
        }
        RuntimeDataType::Artboard => {
            let rust_value = rust_file
                .view_model_instance_artboard_index_for_object(rust_value)
                .ok_or_else(|| format!("{label}: Rust did not resolve artboard index"))?;
            if cpp_runtime.artboard_index != Some(rust_value) {
                return Err(format!(
                    "{label}: artboard index mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_runtime.artboard_index
                ));
            }
        }
        RuntimeDataType::EnumType
        | RuntimeDataType::None
        | RuntimeDataType::Integer
        | RuntimeDataType::Input
        | RuntimeDataType::Any => {}
    }

    Ok(())
}

fn compare_view_model_instance_source_data_value_result(
    rust_file: &RuntimeFile,
    cpp_value: &CppProbeViewModelInstanceValue,
    rust_value: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let cpp_data_value = cpp_value
        .source_data_value
        .as_ref()
        .ok_or_else(|| format!("{label}: C++ probe did not emit a view-model source data value"))?;
    let rust_data_value = rust_file
        .view_model_instance_source_data_value_for_object(rust_value)
        .ok_or_else(|| format!("{label}: Rust did not resolve a source data value"))?;
    if cpp_data_value.data_type != rust_data_value.data_type().as_cpp_u32() {
        return Err(format!(
            "{label}: source data value type mismatch, C++ {}, Rust {}",
            cpp_data_value.data_type,
            rust_data_value.data_type().as_cpp_u32()
        ));
    }

    match rust_data_value {
        RuntimeDataValue::None => {}
        RuntimeDataValue::Number(rust_value) => {
            let cpp_value = cpp_data_value
                .number_value
                .ok_or_else(|| format!("{label}: C++ did not emit number source value"))?;
            if (cpp_value - rust_value).abs() > 0.00001 {
                return Err(format!(
                    "{label}: source number mismatch, C++ {cpp_value}, Rust {rust_value}"
                ));
            }
        }
        RuntimeDataValue::String(rust_value) => {
            let rust_value = std::str::from_utf8(rust_value).unwrap_or_default();
            let cpp_value = cpp_data_value.string_value.as_deref().unwrap_or_default();
            if cpp_value != rust_value {
                return Err(format!(
                    "{label}: source string mismatch, C++ {:?}, Rust {:?}",
                    cpp_value, rust_value
                ));
            }
        }
        RuntimeDataValue::Boolean(rust_value) => {
            if cpp_data_value.boolean_value != Some(rust_value) {
                return Err(format!(
                    "{label}: source boolean mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_data_value.boolean_value
                ));
            }
        }
        RuntimeDataValue::Color(rust_value) => {
            if cpp_data_value.color_value != Some(rust_value) {
                return Err(format!(
                    "{label}: source color mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_data_value.color_value
                ));
            }
        }
        RuntimeDataValue::Enum { value, data_enum } => {
            if cpp_data_value.integer_value != Some(value) {
                return Err(format!(
                    "{label}: source enum integer mismatch, C++ {:?}, Rust {value}",
                    cpp_data_value.integer_value
                ));
            }
            let rust_enum_core_type = data_enum.map(|data_enum| data_enum.object.type_key);
            if cpp_data_value.enum_data_core_type != rust_enum_core_type {
                return Err(format!(
                    "{label}: source enum data core type mismatch, C++ {:?}, Rust {:?}",
                    cpp_data_value.enum_data_core_type, rust_enum_core_type
                ));
            }
        }
        RuntimeDataValue::Trigger(rust_value)
        | RuntimeDataValue::SymbolListIndex(rust_value)
        | RuntimeDataValue::AssetImage(rust_value)
        | RuntimeDataValue::AssetFont(rust_value)
        | RuntimeDataValue::Artboard(rust_value) => {
            if cpp_data_value.integer_value != Some(rust_value) {
                return Err(format!(
                    "{label}: source integer mismatch, C++ {:?}, Rust {rust_value}",
                    cpp_data_value.integer_value
                ));
            }
        }
        RuntimeDataValue::List(rust_items) => {
            if cpp_data_value.list_size != Some(rust_items.len()) {
                return Err(format!(
                    "{label}: source list size mismatch, C++ {:?}, Rust {}",
                    cpp_data_value.list_size,
                    rust_items.len()
                ));
            }
        }
        RuntimeDataValue::ViewModel(rust_reference) => {
            match (
                cpp_value.reference_view_model_instance.as_ref(),
                rust_reference,
            ) {
                (None, None) => {}
                (Some(cpp_reference), Some(rust_reference)) => {
                    if cpp_reference.view_model_index != rust_reference.view_model_index {
                        return Err(format!(
                            "{label}: source view-model index mismatch, C++ {}, Rust {}",
                            cpp_reference.view_model_index, rust_reference.view_model_index
                        ));
                    }
                    if cpp_reference.instance_index != rust_reference.instance_index {
                        return Err(format!(
                            "{label}: source view-model instance mismatch, C++ {}, Rust {}",
                            cpp_reference.instance_index, rust_reference.instance_index
                        ));
                    }
                    if cpp_reference.core_type != rust_reference.object.type_key {
                        return Err(format!(
                            "{label}: source view-model core type mismatch, C++ {}, Rust {}",
                            cpp_reference.core_type, rust_reference.object.type_key
                        ));
                    }
                }
                (None, Some(reference)) => {
                    return Err(format!(
                        "{label}: Rust source view-model value resolved to view model {} instance {} but C++ did not",
                        reference.view_model_index, reference.instance_index
                    ));
                }
                (Some(reference), None) => {
                    return Err(format!(
                        "{label}: C++ source view-model value resolved to view model {} instance {} but Rust did not",
                        reference.view_model_index, reference.instance_index
                    ));
                }
            }
        }
    }

    Ok(())
}

fn compare_view_model_instance_list_item_reference_result(
    rust_file: &RuntimeFile,
    cpp_reference: Option<&CppProbeViewModelInstanceReference>,
    rust_item: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let rust_reference = rust_file.referenced_view_model_instance_for_list_item_object(rust_item);
    match (cpp_reference, rust_reference.as_ref()) {
        (None, None) => Ok(()),
        (Some(cpp_reference), Some(rust_reference)) => {
            if cpp_reference.view_model_index != rust_reference.view_model_index {
                return Err(format!(
                    "{label}: referenced view-model index mismatch, C++ {}, Rust {}",
                    cpp_reference.view_model_index, rust_reference.view_model_index
                ));
            }
            if cpp_reference.instance_index != rust_reference.instance_index {
                return Err(format!(
                    "{label}: referenced instance index mismatch, C++ {}, Rust {}",
                    cpp_reference.instance_index, rust_reference.instance_index
                ));
            }
            if cpp_reference.core_type != rust_reference.object.type_key {
                return Err(format!(
                    "{label}: referenced instance core type mismatch, C++ {}, Rust {}",
                    cpp_reference.core_type, rust_reference.object.type_key
                ));
            }
            let rust_name = rust_reference
                .object
                .string_property("name")
                .unwrap_or_default();
            if cpp_reference.name != rust_name {
                return Err(format!(
                    "{label}: referenced instance name mismatch, C++ {:?}, Rust {:?}",
                    cpp_reference.name, rust_name
                ));
            }
            if rust_reference.object.uint_property("viewModelId")
                != Some(cpp_reference.view_model_id)
            {
                return Err(format!(
                    "{label}: referenced instance view-model id mismatch, C++ {}, Rust {:?}",
                    cpp_reference.view_model_id,
                    rust_reference.object.uint_property("viewModelId")
                ));
            }
            Ok(())
        }
        (None, Some(rust_reference)) => Err(format!(
            "{label}: Rust resolved referenced instance {} ({}) but C++ did not",
            rust_reference.object.id, rust_reference.object.type_name
        )),
        (Some(cpp_reference), None) => Err(format!(
            "{label}: C++ resolved referenced instance core type {} but Rust did not",
            cpp_reference.core_type
        )),
    }
}

fn rust_list_item_for_probe_item<'a>(
    rust_file: &'a RuntimeFile,
    cpp_item: &CppProbeArtboardListItemArtboard,
    label: &str,
) -> Result<&'a RuntimeObject, String> {
    let view_models = rust_file.view_models();
    let view_model = view_models
        .get(cpp_item.owner_view_model_index)
        .ok_or_else(|| {
            format!(
                "{label}: missing owner view model {}",
                cpp_item.owner_view_model_index
            )
        })?;
    let instance = view_model
        .instances
        .get(cpp_item.owner_instance_index)
        .ok_or_else(|| {
            format!(
                "{label}: missing owner instance {}",
                cpp_item.owner_instance_index
            )
        })?;
    let value = instance
        .values
        .get(cpp_item.value_index)
        .ok_or_else(|| format!("{label}: missing owner list value {}", cpp_item.value_index))?;
    value
        .list_items
        .get(cpp_item.item_index)
        .copied()
        .ok_or_else(|| format!("{label}: missing list item {}", cpp_item.item_index))
}

fn cpp_int_from_runtime_uint(value: u64) -> i64 {
    i64::from(value as u32 as i32)
}

fn compare_artboard_component_list_result(
    rust_file: &RuntimeFile,
    cpp_list: &CppProbeArtboardComponentList,
    rust_list: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    if cpp_list.core_type != rust_list.type_key {
        return Err(format!(
            "{label}: core type mismatch, C++ {}, Rust {}",
            cpp_list.core_type, rust_list.type_key
        ));
    }

    let cpp_rules = cpp_list
        .map_rules
        .iter()
        .map(|rule| (rule.view_model_id, rule.artboard_id))
        .collect::<BTreeMap<_, _>>();
    let mut rust_rules = BTreeMap::new();
    for rule in rust_file.artboard_component_list_map_rules_for_object(rust_list) {
        rust_rules.insert(
            cpp_int_from_runtime_uint(rule.view_model_id),
            cpp_int_from_runtime_uint(rule.artboard_id),
        );
    }
    if cpp_rules != rust_rules {
        return Err(format!(
            "{label}: ArtboardComponentList map-rule table mismatch, C++ {:?}, Rust {:?}",
            cpp_rules, rust_rules
        ));
    }

    for cpp_item in &cpp_list.item_artboards {
        let item_label = format!(
            "{label}: owner VM {} instance {} value {} item {}",
            cpp_item.owner_view_model_index,
            cpp_item.owner_instance_index,
            cpp_item.value_index,
            cpp_item.item_index
        );
        let rust_item = rust_list_item_for_probe_item(rust_file, cpp_item, &item_label)?;
        if rust_item.uint_property("viewModelId") != Some(cpp_item.view_model_id) {
            return Err(format!(
                "{item_label}: list item viewModelId mismatch, C++ {}, Rust {:?}",
                cpp_item.view_model_id,
                rust_item.uint_property("viewModelId")
            ));
        }
        if rust_item.uint_property("viewModelInstanceId") != Some(cpp_item.view_model_instance_id) {
            return Err(format!(
                "{item_label}: list item viewModelInstanceId mismatch, C++ {}, Rust {:?}",
                cpp_item.view_model_instance_id,
                rust_item.uint_property("viewModelInstanceId")
            ));
        }

        let rust_artboard = rust_file
            .resolved_artboard_for_artboard_component_list_item_objects(rust_list, rust_item);
        match (cpp_item.artboard.as_ref(), rust_artboard.as_ref()) {
            (None, None) => {}
            (Some(cpp_artboard), Some(rust_artboard)) => {
                if cpp_artboard.index != rust_artboard.artboard_index {
                    return Err(format!(
                        "{item_label}: artboard index mismatch, C++ {}, Rust {}",
                        cpp_artboard.index, rust_artboard.artboard_index
                    ));
                }
                if cpp_artboard.core_type != rust_artboard.object.type_key {
                    return Err(format!(
                        "{item_label}: artboard core type mismatch, C++ {}, Rust {}",
                        cpp_artboard.core_type, rust_artboard.object.type_key
                    ));
                }
                let rust_name = rust_artboard
                    .object
                    .string_property("name")
                    .unwrap_or_default();
                if cpp_artboard.name != rust_name {
                    return Err(format!(
                        "{item_label}: artboard name mismatch, C++ {:?}, Rust {:?}",
                        cpp_artboard.name, rust_name
                    ));
                }
                if rust_artboard.object.uint_property("viewModelId")
                    != Some(cpp_artboard.view_model_id)
                {
                    return Err(format!(
                        "{item_label}: artboard viewModelId mismatch, C++ {}, Rust {:?}",
                        cpp_artboard.view_model_id,
                        rust_artboard.object.uint_property("viewModelId")
                    ));
                }
            }
            (None, Some(rust_artboard)) => {
                return Err(format!(
                    "{item_label}: Rust resolved artboard {} ({}) but C++ did not",
                    rust_artboard.artboard_index,
                    rust_artboard
                        .object
                        .string_property("name")
                        .unwrap_or_default()
                ));
            }
            (Some(cpp_artboard), None) => {
                return Err(format!(
                    "{item_label}: C++ resolved artboard {} ({}) but Rust did not",
                    cpp_artboard.index, cpp_artboard.name
                ));
            }
        }
    }

    Ok(())
}

fn compare_view_model_instance_values_result(
    rust_file: &RuntimeFile,
    cpp_values: &[CppProbeViewModelInstanceValue],
    rust_values: &[nuxie_binary::RuntimeViewModelInstanceValue<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_values.len() != rust_values.len() {
        return Err(format!(
            "{label}: value count mismatch, C++ {}, Rust {}",
            cpp_values.len(),
            rust_values.len()
        ));
    }

    for (value_index, (cpp_value, rust_value)) in cpp_values.iter().zip(rust_values).enumerate() {
        if cpp_value.index != value_index {
            return Err(format!(
                "{label}: value {value_index} C++ index mismatch, got {}",
                cpp_value.index
            ));
        }
        if cpp_value.core_type != rust_value.object.type_key {
            return Err(format!(
                "{label}: value {value_index} core type mismatch, C++ {}, Rust {}",
                cpp_value.core_type, rust_value.object.type_key
            ));
        }
        if rust_value.object.uint_property("viewModelPropertyId")
            != Some(cpp_value.view_model_property_id)
        {
            return Err(format!(
                "{label}: value {value_index} property id mismatch, C++ {}, Rust {:?}",
                cpp_value.view_model_property_id,
                rust_value.object.uint_property("viewModelPropertyId")
            ));
        }
        compare_view_model_instance_value_property_link_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!("{label}: value {value_index} property link"),
        )?;
        compare_view_model_instance_view_model_reference_result(
            rust_file,
            cpp_value.reference_view_model_instance.as_ref(),
            rust_value.object,
            &format!("{label}: value {value_index} reference"),
        )?;
        compare_view_model_instance_asset_snapshot_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!("{label}: value {value_index} asset snapshot"),
        )?;
        compare_view_model_instance_enum_runtime_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!("{label}: value {value_index} enum runtime"),
        )?;
        compare_view_model_instance_value_runtime_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!("{label}: value {value_index} runtime value"),
        )?;
        compare_view_model_instance_source_data_value_result(
            rust_file,
            cpp_value,
            rust_value.object,
            &format!("{label}: value {value_index} source data value"),
        )?;
        if cpp_value.items.len() != rust_value.list_items.len() {
            return Err(format!(
                "{label}: value {value_index} list item count mismatch, C++ {}, Rust {}",
                cpp_value.items.len(),
                rust_value.list_items.len()
            ));
        }
        for (item_index, (cpp_item, rust_item)) in cpp_value
            .items
            .iter()
            .zip(&rust_value.list_items)
            .enumerate()
        {
            if cpp_item.index != item_index {
                return Err(format!(
                    "{label}: value {value_index} list item {item_index} C++ index mismatch, got {}",
                    cpp_item.index
                ));
            }
            if cpp_item.core_type != rust_item.type_key {
                return Err(format!(
                    "{label}: value {value_index} list item {item_index} core type mismatch, C++ {}, Rust {}",
                    cpp_item.core_type, rust_item.type_key
                ));
            }
            if rust_item.uint_property("viewModelId") != Some(cpp_item.view_model_id) {
                return Err(format!(
                    "{label}: value {value_index} list item {item_index} view model id mismatch, C++ {}, Rust {:?}",
                    cpp_item.view_model_id,
                    rust_item.uint_property("viewModelId")
                ));
            }
            if rust_item.uint_property("viewModelInstanceId")
                != Some(cpp_item.view_model_instance_id)
            {
                return Err(format!(
                    "{label}: value {value_index} list item {item_index} view model instance id mismatch, C++ {}, Rust {:?}",
                    cpp_item.view_model_instance_id,
                    rust_item.uint_property("viewModelInstanceId")
                ));
            }
            compare_view_model_instance_list_item_reference_result(
                rust_file,
                cpp_item.referenced_view_model_instance.as_ref(),
                rust_item,
                &format!("{label}: value {value_index} list item {item_index} reference"),
            )?;
        }
    }

    Ok(())
}

#[test]
fn cpp_probe_matches_rust_artboard_animation_metadata_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ animation metadata comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    for fixture_path in [
        "animation/smi_test.riv",
        "animation/state_machine_transition.riv",
    ] {
        compare_artboard_animation_metadata(&probe, fixture_path);
    }

    for fixture_path in ["data_binding_test.riv"] {
        compare_reference_artboard_animation_metadata(&probe, fixture_path);
    }
}

fn compare_artboard_animation_metadata(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_file_with_property_values(probe, fixture_path);
    let rust = read_rust_runtime_file(fixture_path);
    compare_artboard_animation_metadata_from_imports(&cpp, &rust, fixture_path);
}

fn compare_reference_artboard_animation_metadata(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_reference_file_with_property_values(probe, fixture_path);
    let rust = read_rust_reference_runtime_file(fixture_path);
    compare_artboard_animation_metadata_from_imports(&cpp, &rust, fixture_path);
}

fn compare_artboard_animation_metadata_from_imports(
    cpp: &CppProbeFile,
    rust: &RuntimeFile,
    fixture_path: &str,
) {
    let ranges = runtime_artboard_ranges(&rust);

    assert_eq!(
        cpp.artboards.len(),
        ranges.len(),
        "artboard count mismatch for {fixture_path}"
    );

    for (artboard_index, (cpp_artboard, _range)) in cpp.artboards.iter().zip(&ranges).enumerate() {
        let rust_animations = rust.artboard_linear_animations(artboard_index);
        assert_eq!(
            cpp_artboard.animations.len(),
            rust_animations.len(),
            "artboard {artboard_index} animation count mismatch for {fixture_path}"
        );

        for (animation_index, (cpp_animation, rust_animation)) in cpp_artboard
            .animations
            .iter()
            .zip(&rust_animations)
            .enumerate()
        {
            assert_eq!(
                cpp_animation.index, animation_index,
                "C++ animation index mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                cpp_animation.core_type, rust_animation.object.type_key,
                "animation {animation_index} core type mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                cpp_animation.name,
                rust_animation
                    .object
                    .string_property("name")
                    .unwrap_or_default(),
                "animation {animation_index} name mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                rust_animation.object.uint_property("fps"),
                Some(cpp_animation.fps),
                "animation {animation_index} fps mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                rust_animation.object.uint_property("duration"),
                Some(cpp_animation.duration),
                "animation {animation_index} duration mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_float_eq(
                cpp_animation.speed,
                rust_animation.object.double_property("speed"),
                &format!(
                    "animation {animation_index} speed in artboard {artboard_index} for {fixture_path}"
                ),
            );
            assert_eq!(
                rust_animation.object.uint_property("loopValue"),
                Some(cpp_animation.loop_value),
                "animation {animation_index} loop value mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                rust_animation.object.uint_property("workStart"),
                Some(cpp_animation.work_start),
                "animation {animation_index} work start mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                rust_animation.object.uint_property("workEnd"),
                Some(cpp_animation.work_end),
                "animation {animation_index} work end mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                rust_animation.object.bool_property("enableWorkArea"),
                Some(cpp_animation.enable_work_area),
                "animation {animation_index} enable work area mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                rust_animation.object.bool_property("quantize"),
                Some(cpp_animation.quantize),
                "animation {animation_index} quantize mismatch in artboard {artboard_index} for {fixture_path}"
            );
            compare_cpp_registry_properties(
                &cpp_animation.property_values,
                rust_animation.object,
                &format!(
                    "animation {animation_index} in artboard {artboard_index} for {fixture_path}"
                ),
            );
            compare_keyed_objects(
                &cpp_animation.keyed_objects,
                &rust_animation.keyed_objects,
                cpp_animation.fps,
                &format!(
                    "animation {animation_index} in artboard {artboard_index} for {fixture_path}"
                ),
            );
        }

        let rust_state_machines = rust.artboard_state_machine_graphs(artboard_index);
        assert_eq!(
            cpp_artboard.state_machines.len(),
            rust_state_machines.len(),
            "artboard {artboard_index} state machine count mismatch for {fixture_path}"
        );

        for (state_machine_index, (cpp_state_machine, rust_state_machine)) in cpp_artboard
            .state_machines
            .iter()
            .zip(&rust_state_machines)
            .enumerate()
        {
            assert_eq!(
                cpp_state_machine.index, state_machine_index,
                "C++ state machine index mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                cpp_state_machine.core_type, rust_state_machine.object.type_key,
                "state machine {state_machine_index} core type mismatch in artboard {artboard_index} for {fixture_path}"
            );
            assert_eq!(
                cpp_state_machine.name,
                rust_state_machine
                    .object
                    .string_property("name")
                    .unwrap_or_default(),
                "state machine {state_machine_index} name mismatch in artboard {artboard_index} for {fixture_path}"
            );
            compare_cpp_registry_properties(
                &cpp_state_machine.property_values,
                rust_state_machine.object,
                &format!(
                    "state machine {state_machine_index} in artboard {artboard_index} for {fixture_path}"
                ),
            );
            compare_state_machine_children(
                rust,
                cpp_state_machine,
                rust_state_machine,
                &format!(
                    "state machine {state_machine_index} in artboard {artboard_index} for {fixture_path}"
                ),
            );
        }

        let rust_data_binds = rust.artboard_data_binds(artboard_index);
        compare_artboard_data_binds(
            rust,
            &cpp_artboard.data_binds,
            &rust_data_binds,
            &format!("artboard {artboard_index} data binds for {fixture_path}"),
        );
    }
}

fn compare_artboard_data_binds(
    rust_file: &RuntimeFile,
    cpp_data_binds: &[Option<CppProbeArtboardDataBind>],
    rust_data_binds: &[nuxie_binary::RuntimeDataBind<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_data_binds.len(),
        rust_data_binds.len(),
        "{label} count mismatch"
    );

    for (data_bind_index, (cpp_data_bind, rust_data_bind)) in
        cpp_data_binds.iter().zip(rust_data_binds).enumerate()
    {
        let cpp_data_bind = cpp_data_bind
            .as_ref()
            .unwrap_or_else(|| panic!("{label} data bind {data_bind_index} is null in C++"));
        assert_eq!(
            cpp_data_bind.index, data_bind_index,
            "{label} data bind {data_bind_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_data_bind.core_type, rust_data_bind.object.type_key,
            "{label} data bind {data_bind_index} core type mismatch"
        );
        assert_eq!(
            rust_data_bind.object.uint_property("propertyKey"),
            Some(cpp_data_bind.property_key),
            "{label} data bind {data_bind_index} property key mismatch"
        );
        assert_eq!(
            rust_data_bind.object.uint_property("flags"),
            Some(cpp_data_bind.flags),
            "{label} data bind {data_bind_index} flags mismatch"
        );
        assert_eq!(
            rust_data_bind.object.uint_property("converterId"),
            Some(cpp_data_bind.converter_id),
            "{label} data bind {data_bind_index} converter id mismatch"
        );
        assert_eq!(
            rust_data_bind.converter.map(|converter| converter.type_key),
            cpp_data_bind.converter_core_type,
            "{label} data bind {data_bind_index} converter core type mismatch"
        );
        assert_eq!(
            data_converter_output_type(rust_file, rust_data_bind.converter),
            cpp_data_bind.converter_output_type,
            "{label} data bind {data_bind_index} converter output type mismatch"
        );
        assert_eq!(
            data_bind_source_output_type(rust_file, rust_data_bind.object),
            cpp_data_bind.data_bind_source_output_type,
            "{label} data bind {data_bind_index} source output type mismatch"
        );
        assert_eq!(
            data_bind_output_type(rust_file, rust_data_bind.object),
            cpp_data_bind.data_bind_output_type,
            "{label} data bind {data_bind_index} output type mismatch"
        );
        assert_eq!(
            data_bind_target_supports_push(rust_file, rust_data_bind.object),
            cpp_data_bind.target_supports_push,
            "{label} data bind {data_bind_index} target-supports-push mismatch"
        );
        assert_eq!(
            data_bind_uses_persisting_list(rust_file, rust_data_bind.object),
            cpp_data_bind.uses_persisting_list,
            "{label} data bind {data_bind_index} persisting-list mismatch"
        );
        assert_eq!(
            data_converter_operation_view_model_source_path_ids(rust_data_bind.converter),
            cpp_data_bind.converter_source_path_ids,
            "{label} data bind {data_bind_index} converter source path ids mismatch"
        );
        let rust_number_to_list_view_model =
            resolved_number_to_list_view_model(rust_file, rust_data_bind.converter);
        assert_eq!(
            rust_number_to_list_view_model
                .as_ref()
                .map(|view_model| view_model.object.type_key),
            cpp_data_bind.converter_number_to_list_view_model_core_type,
            "{label} data bind {data_bind_index} number-to-list view model core type mismatch"
        );
        assert_eq!(
            rust_number_to_list_view_model
                .as_ref()
                .and_then(|view_model| view_model.object.string_property("name")),
            cpp_data_bind
                .converter_number_to_list_view_model_name
                .as_deref(),
            "{label} data bind {data_bind_index} number-to-list view model name mismatch"
        );
        assert_eq!(
            data_bind_context_source_path_ids(rust_file, rust_data_bind.object),
            cpp_data_bind.data_bind_context_source_path_ids,
            "{label} data bind {data_bind_index} DataBindContext source path ids mismatch"
        );
        assert_eq!(
            data_bind_context_resolved_source_path_ids(rust_file, rust_data_bind.object),
            cpp_data_bind.resolved_data_bind_context_source_path_ids,
            "{label} data bind {data_bind_index} resolved DataBindContext source path ids mismatch"
        );
        assert_eq!(
            resolved_data_converter_interpolator_core_type(rust_file, rust_data_bind.converter),
            cpp_data_bind.converter_interpolator_core_type,
            "{label} data bind {data_bind_index} converter interpolator core type mismatch"
        );
        compare_data_converter_group_items(
            rust_file,
            &cpp_data_bind.converter_group_items,
            &resolved_data_bind_converter_group_items(rust_file, rust_data_bind.converter),
            &format!("{label} data bind {data_bind_index} converter group items"),
        );
        compare_data_converter_formula_tokens(
            &cpp_data_bind.converter_formula_tokens,
            &resolved_data_bind_converter_formula_tokens(rust_file, rust_data_bind.converter),
            &format!("{label} data bind {data_bind_index} converter formula tokens"),
        );
        assert_eq!(
            rust_data_bind.target.map(|target| target.type_key),
            cpp_data_bind.target_core_type,
            "{label} data bind {data_bind_index} target core type mismatch"
        );
        assert_eq!(
            rust_data_bind.target_local_id, cpp_data_bind.target_local,
            "{label} data bind {data_bind_index} target local mismatch"
        );
        compare_cpp_registry_properties(
            &cpp_data_bind.property_values,
            rust_data_bind.object,
            &format!("{label} data bind {data_bind_index}"),
        );
    }
}

fn compare_artboard_skins(
    cpp_skins: &[CppProbeSkin],
    rust_skins: &[nuxie_binary::RuntimeSkin<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_skins.len(),
        rust_skins.len(),
        "{label} skin count mismatch"
    );

    for (skin_index, (cpp_skin, rust_skin)) in cpp_skins.iter().zip(rust_skins).enumerate() {
        assert_eq!(
            cpp_skin.local_id, rust_skin.local_id,
            "{label} skin {skin_index} local id mismatch"
        );
        assert_eq!(
            cpp_skin.core_type, rust_skin.object.type_key,
            "{label} skin {skin_index} core type mismatch"
        );
        assert_eq!(
            cpp_skin.skinnable_local, rust_skin.skinnable_local_id,
            "{label} skin {skin_index} skinnable local mismatch"
        );
        assert_eq!(
            cpp_skin.skinnable_core_type,
            rust_skin.skinnable.map(|object| object.type_key),
            "{label} skin {skin_index} skinnable core type mismatch"
        );
        assert_eq!(
            cpp_skin.tendons.len(),
            rust_skin.tendons.len(),
            "{label} skin {skin_index} tendon count mismatch"
        );

        for (tendon_index, (cpp_tendon, rust_tendon)) in
            cpp_skin.tendons.iter().zip(&rust_skin.tendons).enumerate()
        {
            assert_eq!(
                cpp_tendon.index, tendon_index,
                "{label} skin {skin_index} tendon {tendon_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_tendon.local_id,
                Some(rust_tendon.local_id),
                "{label} skin {skin_index} tendon {tendon_index} local id mismatch"
            );
            assert_eq!(
                cpp_tendon.core_type, rust_tendon.object.type_key,
                "{label} skin {skin_index} tendon {tendon_index} core type mismatch"
            );
            assert_eq!(
                cpp_tendon.bone_local, rust_tendon.bone_local_id,
                "{label} skin {skin_index} tendon {tendon_index} bone local mismatch"
            );
            assert_eq!(
                cpp_tendon.bone_core_type,
                rust_tendon.bone.map(|object| object.type_key),
                "{label} skin {skin_index} tendon {tendon_index} bone core type mismatch"
            );
        }
    }
}

fn compare_artboard_meshes(
    cpp_meshes: &[CppProbeMesh],
    rust_meshes: &[nuxie_binary::RuntimeMesh<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_meshes.len(),
        rust_meshes.len(),
        "{label} mesh count mismatch"
    );

    for (mesh_index, (cpp_mesh, rust_mesh)) in cpp_meshes.iter().zip(rust_meshes).enumerate() {
        assert_eq!(
            cpp_mesh.local_id, rust_mesh.local_id,
            "{label} mesh {mesh_index} local id mismatch"
        );
        assert_eq!(
            cpp_mesh.core_type, rust_mesh.object.type_key,
            "{label} mesh {mesh_index} core type mismatch"
        );
        assert_eq!(
            cpp_mesh.vertices.len(),
            rust_mesh.vertices.len(),
            "{label} mesh {mesh_index} vertex count mismatch"
        );

        for (vertex_index, (cpp_vertex, rust_vertex)) in cpp_mesh
            .vertices
            .iter()
            .zip(&rust_mesh.vertices)
            .enumerate()
        {
            assert_eq!(
                cpp_vertex.index, vertex_index,
                "{label} mesh {mesh_index} vertex {vertex_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_vertex.local_id, rust_vertex.local_id,
                "{label} mesh {mesh_index} vertex {vertex_index} local id mismatch"
            );
            assert_eq!(
                cpp_vertex.core_type, rust_vertex.object.type_key,
                "{label} mesh {mesh_index} vertex {vertex_index} core type mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_local, rust_vertex.weight_local_id,
                "{label} mesh {mesh_index} vertex {vertex_index} weight local mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_core_type,
                rust_vertex.weight.map(|object| object.type_key),
                "{label} mesh {mesh_index} vertex {vertex_index} weight core type mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_values,
                rust_vertex
                    .weight
                    .and_then(|object| object.uint_property("values")),
                "{label} mesh {mesh_index} vertex {vertex_index} weight values mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_indices,
                rust_vertex
                    .weight
                    .and_then(|object| object.uint_property("indices")),
                "{label} mesh {mesh_index} vertex {vertex_index} weight indices mismatch"
            );
        }
    }
}

fn compare_artboard_paths(
    cpp_paths: &[CppProbePath],
    rust_paths: &[nuxie_binary::RuntimePath<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_paths.len(),
        rust_paths.len(),
        "{label} path count mismatch"
    );

    for (path_index, (cpp_path, rust_path)) in cpp_paths.iter().zip(rust_paths).enumerate() {
        assert_eq!(
            cpp_path.local_id, rust_path.local_id,
            "{label} path {path_index} local id mismatch"
        );
        assert_eq!(
            cpp_path.core_type, rust_path.object.type_key,
            "{label} path {path_index} core type mismatch"
        );
        assert_eq!(
            cpp_path.vertices.len(),
            rust_path.vertices.len(),
            "{label} path {path_index} vertex count mismatch"
        );

        for (vertex_index, (cpp_vertex, rust_vertex)) in cpp_path
            .vertices
            .iter()
            .zip(&rust_path.vertices)
            .enumerate()
        {
            assert_eq!(
                cpp_vertex.index, vertex_index,
                "{label} path {path_index} vertex {vertex_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_vertex.local_id, rust_vertex.local_id,
                "{label} path {path_index} vertex {vertex_index} local id mismatch"
            );
            assert_eq!(
                cpp_vertex.core_type, rust_vertex.object.type_key,
                "{label} path {path_index} vertex {vertex_index} core type mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_local, rust_vertex.weight_local_id,
                "{label} path {path_index} vertex {vertex_index} weight local mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_core_type,
                rust_vertex.weight.map(|object| object.type_key),
                "{label} path {path_index} vertex {vertex_index} weight core type mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_values,
                rust_vertex
                    .weight
                    .and_then(|object| object.uint_property("values")),
                "{label} path {path_index} vertex {vertex_index} weight values mismatch"
            );
            assert_eq!(
                cpp_vertex.weight_indices,
                rust_vertex
                    .weight
                    .and_then(|object| object.uint_property("indices")),
                "{label} path {path_index} vertex {vertex_index} weight indices mismatch"
            );
        }
    }
}

fn compare_artboard_n_slicer_details(
    cpp_details: &[CppProbeNSlicerDetails],
    rust_details: &[nuxie_binary::RuntimeNSlicerDetails<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_details.len(),
        rust_details.len(),
        "{label} NSlicerDetails count mismatch"
    );

    for (details_index, (cpp_detail, rust_detail)) in
        cpp_details.iter().zip(rust_details).enumerate()
    {
        assert_eq!(
            cpp_detail.local_id, rust_detail.local_id,
            "{label} NSlicerDetails {details_index} local id mismatch"
        );
        assert_eq!(
            cpp_detail.core_type, rust_detail.object.type_key,
            "{label} NSlicerDetails {details_index} core type mismatch"
        );
        compare_n_slicer_axes(
            &cpp_detail.x_axes,
            &rust_detail.x_axes,
            &format!("{label} NSlicerDetails {details_index} x axes"),
        );
        compare_n_slicer_axes(
            &cpp_detail.y_axes,
            &rust_detail.y_axes,
            &format!("{label} NSlicerDetails {details_index} y axes"),
        );
        compare_n_slicer_tile_modes(
            &cpp_detail.tile_modes,
            &rust_detail.tile_modes,
            &format!("{label} NSlicerDetails {details_index} tile modes"),
        );
    }
}

fn compare_n_slicer_axes(
    cpp_axes: &[CppProbeNSlicerAxis],
    rust_axes: &[nuxie_binary::RuntimeNSlicerAxis<'_>],
    label: &str,
) {
    assert_eq!(cpp_axes.len(), rust_axes.len(), "{label} count mismatch");
    for (axis_index, (cpp_axis, rust_axis)) in cpp_axes.iter().zip(rust_axes).enumerate() {
        assert_eq!(
            cpp_axis.index, axis_index,
            "{label} axis {axis_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_axis.local_id,
            Some(rust_axis.local_id),
            "{label} axis {axis_index} local id mismatch"
        );
        assert_eq!(
            cpp_axis.core_type, rust_axis.object.type_key,
            "{label} axis {axis_index} core type mismatch"
        );
    }
}

fn compare_n_slicer_tile_modes(
    cpp_tile_modes: &[CppProbeNSlicerTileMode],
    rust_tile_modes: &[nuxie_binary::RuntimeNSlicerTileMode<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_tile_modes.len(),
        rust_tile_modes.len(),
        "{label} count mismatch"
    );
    for (mode_index, (cpp_mode, rust_mode)) in
        cpp_tile_modes.iter().zip(rust_tile_modes).enumerate()
    {
        assert_eq!(
            cpp_mode.index, mode_index,
            "{label} tile mode {mode_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_mode.local_id,
            Some(rust_mode.local_id),
            "{label} tile mode {mode_index} local id mismatch"
        );
        assert_eq!(
            cpp_mode.core_type,
            Some(rust_mode.object.type_key),
            "{label} tile mode {mode_index} core type mismatch"
        );
        assert_eq!(
            cpp_mode.patch_index, rust_mode.patch_index,
            "{label} tile mode {mode_index} patch index mismatch"
        );
        assert_eq!(
            cpp_mode.style, rust_mode.style,
            "{label} tile mode {mode_index} style mismatch"
        );
    }
}

fn compare_artboard_shapes(
    cpp_shapes: &[CppProbeShape],
    rust_shapes: &[nuxie_binary::RuntimeShape<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_shapes.len(),
        rust_shapes.len(),
        "{label} shape count mismatch"
    );

    for (shape_index, (cpp_shape, rust_shape)) in cpp_shapes.iter().zip(rust_shapes).enumerate() {
        assert_eq!(
            cpp_shape.local_id, rust_shape.local_id,
            "{label} shape {shape_index} local id mismatch"
        );
        assert_eq!(
            cpp_shape.core_type, rust_shape.object.type_key,
            "{label} shape {shape_index} core type mismatch"
        );
        assert_eq!(
            cpp_shape.paths.len(),
            rust_shape.paths.len(),
            "{label} shape {shape_index} path count mismatch"
        );

        for (path_index, (cpp_path, rust_path)) in
            cpp_shape.paths.iter().zip(&rust_shape.paths).enumerate()
        {
            assert_eq!(
                cpp_path.index, path_index,
                "{label} shape {shape_index} path {path_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_path.local_id,
                Some(rust_path.local_id),
                "{label} shape {shape_index} path {path_index} local id mismatch"
            );
            assert_eq!(
                cpp_path.core_type, rust_path.object.type_key,
                "{label} shape {shape_index} path {path_index} core type mismatch"
            );
        }

        assert_eq!(
            cpp_shape.paints.len(),
            rust_shape.paints.len(),
            "{label} shape {shape_index} paint count mismatch"
        );

        for (paint_index, (cpp_paint, rust_paint)) in
            cpp_shape.paints.iter().zip(&rust_shape.paints).enumerate()
        {
            assert_eq!(
                cpp_paint.index, paint_index,
                "{label} shape {shape_index} paint {paint_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_paint.local_id,
                Some(rust_paint.local_id),
                "{label} shape {shape_index} paint {paint_index} local id mismatch"
            );
            assert_eq!(
                cpp_paint.core_type, rust_paint.object.type_key,
                "{label} shape {shape_index} paint {paint_index} core type mismatch"
            );
            assert_eq!(
                cpp_paint.mutator_local, rust_paint.mutator_local_id,
                "{label} shape {shape_index} paint {paint_index} mutator local mismatch"
            );
            assert_eq!(
                cpp_paint.mutator_core_type,
                rust_paint.mutator.map(|object| object.type_key),
                "{label} shape {shape_index} paint {paint_index} mutator core type mismatch"
            );
            assert_eq!(
                cpp_paint.feather_local, rust_paint.feather_local_id,
                "{label} shape {shape_index} paint {paint_index} feather local mismatch"
            );
            assert_eq!(
                cpp_paint.feather_core_type,
                rust_paint.feather.map(|object| object.type_key),
                "{label} shape {shape_index} paint {paint_index} feather core type mismatch"
            );
            assert_eq!(
                cpp_paint.effects.len(),
                rust_paint.effects.len(),
                "{label} shape {shape_index} paint {paint_index} effect count mismatch"
            );

            for (effect_index, (cpp_effect, rust_effect)) in cpp_paint
                .effects
                .iter()
                .zip(&rust_paint.effects)
                .enumerate()
            {
                assert_eq!(
                    cpp_effect.index, effect_index,
                    "{label} shape {shape_index} paint {paint_index} effect {effect_index} C++ index mismatch"
                );
                assert_eq!(
                    cpp_effect.local_id,
                    Some(rust_effect.local_id),
                    "{label} shape {shape_index} paint {paint_index} effect {effect_index} local id mismatch"
                );
                assert_eq!(
                    cpp_effect.core_type, rust_effect.object.type_key,
                    "{label} shape {shape_index} paint {paint_index} effect {effect_index} core type mismatch"
                );
                assert_eq!(
                    cpp_effect.target_group_effect_local, rust_effect.target_group_effect_local_id,
                    "{label} shape {shape_index} paint {paint_index} effect {effect_index} target group local mismatch"
                );
                assert_eq!(
                    cpp_effect.target_group_effect_core_type,
                    rust_effect
                        .target_group_effect
                        .map(|object| object.type_key),
                    "{label} shape {shape_index} paint {paint_index} effect {effect_index} target group core type mismatch"
                );
            }
            assert_eq!(
                cpp_paint.gradient_stops.len(),
                rust_paint.gradient_stops.len(),
                "{label} shape {shape_index} paint {paint_index} gradient stop count mismatch"
            );

            for (stop_index, (cpp_stop, rust_stop)) in cpp_paint
                .gradient_stops
                .iter()
                .zip(&rust_paint.gradient_stops)
                .enumerate()
            {
                assert_eq!(
                    cpp_stop.index, stop_index,
                    "{label} shape {shape_index} paint {paint_index} gradient stop {stop_index} C++ index mismatch"
                );
                assert_eq!(
                    cpp_stop.local_id,
                    Some(rust_stop.local_id),
                    "{label} shape {shape_index} paint {paint_index} gradient stop {stop_index} local id mismatch"
                );
                assert_eq!(
                    cpp_stop.core_type, rust_stop.object.type_key,
                    "{label} shape {shape_index} paint {paint_index} gradient stop {stop_index} core type mismatch"
                );
            }
        }
    }
}

fn compare_artboard_shape_paint_containers(
    cpp_containers: &[CppProbeShapePaintContainer],
    rust_containers: &[nuxie_binary::RuntimeShapePaintContainer<'_>],
    label: &str,
) {
    assert_eq!(
        cpp_containers.len(),
        rust_containers.len(),
        "{label} shape paint container count mismatch"
    );

    for (container_index, (cpp_container, rust_container)) in
        cpp_containers.iter().zip(rust_containers).enumerate()
    {
        assert_eq!(
            cpp_container.local_id, rust_container.local_id,
            "{label} shape paint container {container_index} local id mismatch"
        );
        assert_eq!(
            cpp_container.core_type, rust_container.object.type_key,
            "{label} shape paint container {container_index} core type mismatch"
        );
        assert_eq!(
            cpp_container.paints.len(),
            rust_container.paints.len(),
            "{label} shape paint container {container_index} paint count mismatch"
        );

        for (paint_index, (cpp_paint, rust_paint)) in cpp_container
            .paints
            .iter()
            .zip(&rust_container.paints)
            .enumerate()
        {
            assert_eq!(
                cpp_paint.index, paint_index,
                "{label} shape paint container {container_index} paint {paint_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_paint.local_id,
                Some(rust_paint.local_id),
                "{label} shape paint container {container_index} paint {paint_index} local id mismatch"
            );
            assert_eq!(
                cpp_paint.core_type, rust_paint.object.type_key,
                "{label} shape paint container {container_index} paint {paint_index} core type mismatch"
            );
            assert_eq!(
                cpp_paint.mutator_local, rust_paint.mutator_local_id,
                "{label} shape paint container {container_index} paint {paint_index} mutator local mismatch"
            );
            assert_eq!(
                cpp_paint.mutator_core_type,
                rust_paint.mutator.map(|object| object.type_key),
                "{label} shape paint container {container_index} paint {paint_index} mutator core type mismatch"
            );
            assert_eq!(
                cpp_paint.feather_local, rust_paint.feather_local_id,
                "{label} shape paint container {container_index} paint {paint_index} feather local mismatch"
            );
            assert_eq!(
                cpp_paint.feather_core_type,
                rust_paint.feather.map(|object| object.type_key),
                "{label} shape paint container {container_index} paint {paint_index} feather core type mismatch"
            );
            assert_eq!(
                cpp_paint.effects.len(),
                rust_paint.effects.len(),
                "{label} shape paint container {container_index} paint {paint_index} effect count mismatch"
            );
            for (effect_index, (cpp_effect, rust_effect)) in cpp_paint
                .effects
                .iter()
                .zip(&rust_paint.effects)
                .enumerate()
            {
                assert_eq!(
                    cpp_effect.index, effect_index,
                    "{label} shape paint container {container_index} paint {paint_index} effect {effect_index} C++ index mismatch"
                );
                assert_eq!(
                    cpp_effect.local_id,
                    Some(rust_effect.local_id),
                    "{label} shape paint container {container_index} paint {paint_index} effect {effect_index} local id mismatch"
                );
                assert_eq!(
                    cpp_effect.core_type, rust_effect.object.type_key,
                    "{label} shape paint container {container_index} paint {paint_index} effect {effect_index} core type mismatch"
                );
                assert_eq!(
                    cpp_effect.target_group_effect_local, rust_effect.target_group_effect_local_id,
                    "{label} shape paint container {container_index} paint {paint_index} effect {effect_index} target group local mismatch"
                );
                assert_eq!(
                    cpp_effect.target_group_effect_core_type,
                    rust_effect
                        .target_group_effect
                        .map(|object| object.type_key),
                    "{label} shape paint container {container_index} paint {paint_index} effect {effect_index} target group core type mismatch"
                );
            }
            assert_eq!(
                cpp_paint.gradient_stops.len(),
                rust_paint.gradient_stops.len(),
                "{label} shape paint container {container_index} paint {paint_index} gradient stop count mismatch"
            );
            for (stop_index, (cpp_stop, rust_stop)) in cpp_paint
                .gradient_stops
                .iter()
                .zip(&rust_paint.gradient_stops)
                .enumerate()
            {
                assert_eq!(
                    cpp_stop.index, stop_index,
                    "{label} shape paint container {container_index} paint {paint_index} gradient stop {stop_index} C++ index mismatch"
                );
                assert_eq!(
                    cpp_stop.local_id,
                    Some(rust_stop.local_id),
                    "{label} shape paint container {container_index} paint {paint_index} gradient stop {stop_index} local id mismatch"
                );
                assert_eq!(
                    cpp_stop.core_type, rust_stop.object.type_key,
                    "{label} shape paint container {container_index} paint {paint_index} gradient stop {stop_index} core type mismatch"
                );
            }
        }
    }
}

fn resolved_data_bind_converter_group_items<'a>(
    rust_file: &'a RuntimeFile,
    converter: Option<&'a RuntimeObject>,
) -> Vec<nuxie_binary::RuntimeDataConverterGroupItem<'a>> {
    converter
        .map(|converter| rust_file.data_converter_group_items_for_object(converter))
        .unwrap_or_default()
        .into_iter()
        .filter(|item| item.converter.is_some())
        .collect()
}

fn resolved_data_bind_converter_formula_tokens<'a>(
    rust_file: &'a RuntimeFile,
    converter: Option<&'a RuntimeObject>,
) -> Vec<&'a RuntimeObject> {
    converter
        .map(|converter| rust_file.data_converter_formula_tokens_for_object(converter))
        .unwrap_or_default()
}

fn data_converter_operation_view_model_source_path_ids(
    converter: Option<&RuntimeObject>,
) -> Option<Vec<u32>> {
    let converter = converter?;
    (converter.type_name == "DataConverterOperationViewModel")
        .then(|| converter.id_list_property("sourcePathIds"))
        .flatten()
}

fn data_converter_output_type(
    rust_file: &RuntimeFile,
    converter: Option<&RuntimeObject>,
) -> Option<u32> {
    converter
        .and_then(|converter| rust_file.data_converter_output_type_for_object(converter))
        .map(RuntimeDataType::as_cpp_u32)
}

fn data_bind_source_output_type(rust_file: &RuntimeFile, data_bind: &RuntimeObject) -> Option<u32> {
    rust_file
        .data_bind_source_output_type_for_object(data_bind)
        .map(RuntimeDataType::as_cpp_u32)
}

fn data_bind_output_type(rust_file: &RuntimeFile, data_bind: &RuntimeObject) -> Option<u32> {
    rust_file
        .data_bind_output_type_for_object(data_bind)
        .map(RuntimeDataType::as_cpp_u32)
}

fn data_bind_target_supports_push(
    rust_file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> Option<bool> {
    rust_file.data_bind_target_supports_push_for_object(data_bind)
}

fn data_bind_uses_persisting_list(
    rust_file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> Option<bool> {
    rust_file.data_bind_uses_persisting_list_for_object(data_bind)
}

fn data_bind_context_source_path_ids(
    rust_file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> Option<Vec<u32>> {
    rust_file.data_bind_context_source_path_ids_for_object(data_bind)
}

fn data_bind_context_resolved_source_path_ids(
    rust_file: &RuntimeFile,
    data_bind: &RuntimeObject,
) -> Option<Vec<u32>> {
    rust_file.data_bind_context_resolved_source_path_ids_for_object(data_bind)
}

fn compare_converter_output(
    cpp_output: &CppProbeConverterOutput,
    rust_output: &RuntimeConvertedDataValue<'_>,
    label: &str,
) {
    assert_eq!(
        cpp_output.data_type,
        Some(rust_output.data_type().as_cpp_u32()),
        "{label} data type mismatch"
    );

    match rust_output {
        RuntimeConvertedDataValue::Number(value) => assert!(
            cpp_output
                .number_value
                .is_some_and(|cpp_value| (cpp_value - *value).abs() <= 0.0001),
            "{label} number mismatch, C++ {:?}, Rust {value}",
            cpp_output.number_value
        ),
        RuntimeConvertedDataValue::String(value) => assert_eq!(
            cpp_output.string_value.as_deref().map(str::as_bytes),
            Some(value.as_slice()),
            "{label} string mismatch"
        ),
        RuntimeConvertedDataValue::Boolean(value) => assert_eq!(
            cpp_output.boolean_value,
            Some(*value),
            "{label} boolean mismatch"
        ),
        RuntimeConvertedDataValue::Color(value) => assert_eq!(
            cpp_output.color_value,
            Some(*value),
            "{label} color mismatch"
        ),
        RuntimeConvertedDataValue::Enum { value, .. }
        | RuntimeConvertedDataValue::Integer(value)
        | RuntimeConvertedDataValue::Trigger(value)
        | RuntimeConvertedDataValue::SymbolListIndex(value)
        | RuntimeConvertedDataValue::AssetImage(value)
        | RuntimeConvertedDataValue::AssetFont(value)
        | RuntimeConvertedDataValue::Artboard(value) => assert_eq!(
            cpp_output.integer_value,
            Some(*value),
            "{label} integer mismatch"
        ),
        RuntimeConvertedDataValue::List(items) => assert_eq!(
            cpp_output.list_size,
            Some(items.len()),
            "{label} list size mismatch"
        ),
        RuntimeConvertedDataValue::GeneratedList(items) => assert_eq!(
            cpp_output.list_size,
            Some(items.len()),
            "{label} generated list size mismatch"
        ),
        RuntimeConvertedDataValue::None | RuntimeConvertedDataValue::ViewModel(_) => {}
    }
}

fn compare_interpolator_state_sample(
    rust: &RuntimeFile,
    cpp_sample: &CppProbeConverterSample,
    label: &str,
    converter_index: usize,
) {
    let rust_output = run_interpolator_state_sample(rust, label, converter_index, false)
        .unwrap_or_else(|| panic!("{label} Rust stateful interpolator conversion returned None"));
    compare_converter_output(&cpp_sample.output, &rust_output, label);

    let rust_reverse_output = run_interpolator_state_sample(rust, label, converter_index, true)
        .unwrap_or_else(|| {
            panic!("{label} Rust stateful interpolator reverse conversion returned None")
        });
    compare_converter_output(
        &cpp_sample.reverse_output,
        &rust_reverse_output,
        &format!("{label} reverse"),
    );
}

fn run_interpolator_state_sample<'a>(
    rust: &'a RuntimeFile,
    label: &str,
    converter_index: usize,
    reverse: bool,
) -> Option<RuntimeConvertedDataValue<'a>> {
    let mut state = RuntimeDataConverterInterpolatorState::new();
    let start_number: RuntimeDataValue<'a> = RuntimeDataValue::Number(0.0);
    let target_number: RuntimeDataValue<'a> = RuntimeDataValue::Number(10.0);
    let retarget_number: RuntimeDataValue<'a> = RuntimeDataValue::Number(20.0);
    let cubic_target_number: RuntimeDataValue<'a> = RuntimeDataValue::Number(100.0);
    let start_color: RuntimeDataValue<'a> = RuntimeDataValue::Color(0xff000000);
    let target_color: RuntimeDataValue<'a> = RuntimeDataValue::Color(0xffffffff);

    match label {
        "interpolator_number_after_half_duration" => {
            interpolator_state_convert(rust, converter_index, &mut state, &start_number, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            interpolator_state_convert(rust, converter_index, &mut state, &target_number, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.5)?;
            interpolator_state_convert(rust, converter_index, &mut state, &target_number, reverse)
        }
        "interpolator_number_midflight_retarget" => {
            interpolator_state_convert(rust, converter_index, &mut state, &start_number, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            interpolator_state_convert(rust, converter_index, &mut state, &target_number, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.5)?;
            interpolator_state_convert(
                rust,
                converter_index,
                &mut state,
                &retarget_number,
                reverse,
            )?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.25)?;
            interpolator_state_convert(rust, converter_index, &mut state, &retarget_number, reverse)
        }
        "interpolator_color_after_half_duration" => {
            interpolator_state_convert(rust, converter_index, &mut state, &start_color, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            interpolator_state_convert(rust, converter_index, &mut state, &target_color, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.5)?;
            interpolator_state_convert(rust, converter_index, &mut state, &target_color, reverse)
        }
        "interpolator_cubic_after_quarter_duration" => {
            interpolator_state_convert(rust, converter_index, &mut state, &start_number, reverse)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.1)?;
            interpolator_state_convert(
                rust,
                converter_index,
                &mut state,
                &cubic_target_number,
                reverse,
            )?;
            rust.data_converter_interpolator_advance(converter_index, &mut state, 0.25)?;
            interpolator_state_convert(
                rust,
                converter_index,
                &mut state,
                &cubic_target_number,
                reverse,
            )
        }
        _ => None,
    }
}

fn interpolator_state_convert<'a>(
    rust: &'a RuntimeFile,
    converter_index: usize,
    state: &mut RuntimeDataConverterInterpolatorState,
    input: &RuntimeDataValue<'a>,
    reverse: bool,
) -> Option<RuntimeConvertedDataValue<'a>> {
    if reverse {
        rust.data_converter_interpolator_reverse_convert(converter_index, state, input)
    } else {
        rust.data_converter_interpolator_convert(converter_index, state, input)
    }
}

fn compare_stateful_group_sample(
    rust: &RuntimeFile,
    cpp_sample: &CppProbeConverterSample,
    label: &str,
    converter_index: usize,
) {
    let rust_output = run_stateful_group_sample(rust, converter_index, false)
        .unwrap_or_else(|| panic!("{label} Rust stateful group conversion returned None"));
    compare_converter_output(&cpp_sample.output, &rust_output, label);

    let rust_reverse_output = run_stateful_group_sample(rust, converter_index, true)
        .unwrap_or_else(|| panic!("{label} Rust stateful group reverse conversion returned None"));
    compare_converter_output(
        &cpp_sample.reverse_output,
        &rust_reverse_output,
        &format!("{label} reverse"),
    );
}

fn run_stateful_group_sample<'a>(
    rust: &'a RuntimeFile,
    converter_index: usize,
    reverse: bool,
) -> Option<RuntimeConvertedDataValue<'a>> {
    let mut state = RuntimeDataConverterState::new();
    let start_number: RuntimeDataValue<'a> = RuntimeDataValue::Number(0.0);
    let target_number: RuntimeDataValue<'a> = RuntimeDataValue::Number(10.0);

    stateful_group_convert(rust, converter_index, &mut state, &start_number, reverse)?;
    rust.data_converter_stateful_advance(converter_index, &mut state, 0.1)?;
    rust.data_converter_stateful_advance(converter_index, &mut state, 0.1)?;
    stateful_group_convert(rust, converter_index, &mut state, &target_number, reverse)?;
    rust.data_converter_stateful_advance(converter_index, &mut state, 0.5)?;
    stateful_group_convert(rust, converter_index, &mut state, &target_number, reverse)
}

fn stateful_group_convert<'a>(
    rust: &'a RuntimeFile,
    converter_index: usize,
    state: &mut RuntimeDataConverterState,
    input: &RuntimeDataValue<'a>,
    reverse: bool,
) -> Option<RuntimeConvertedDataValue<'a>> {
    if reverse {
        rust.data_converter_stateful_reverse_convert(converter_index, state, input)
    } else {
        rust.data_converter_stateful_convert(converter_index, state, input)
    }
}

fn resolved_data_converter_interpolator_core_type(
    rust_file: &RuntimeFile,
    converter: Option<&RuntimeObject>,
) -> Option<u16> {
    converter
        .and_then(|converter| rust_file.resolved_interpolator_for_data_converter_object(converter))
        .map(|interpolator| interpolator.type_key)
}

fn resolved_number_to_list_view_model<'a>(
    rust_file: &'a RuntimeFile,
    converter: Option<&RuntimeObject>,
) -> Option<nuxie_binary::RuntimeViewModel<'a>> {
    converter.and_then(|converter| {
        rust_file.resolved_view_model_for_number_to_list_converter_object(converter)
    })
}

fn compare_data_converter_group_items(
    rust_file: &RuntimeFile,
    cpp_items: &[CppProbeConverterGroupItem],
    rust_items: &[nuxie_binary::RuntimeDataConverterGroupItem<'_>],
    label: &str,
) {
    assert_eq!(cpp_items.len(), rust_items.len(), "{label} count mismatch");
    for (item_index, (cpp_item, rust_item)) in cpp_items.iter().zip(rust_items).enumerate() {
        assert_eq!(
            cpp_item.index, item_index,
            "{label} item {item_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_item.core_type, rust_item.object.type_key,
            "{label} item {item_index} core type mismatch"
        );
        assert_eq!(
            cpp_item.converter_core_type,
            rust_item.converter.map(|converter| converter.type_key),
            "{label} item {item_index} converter core type mismatch"
        );
        assert_eq!(
            cpp_item.converter_output_type,
            data_converter_output_type(rust_file, rust_item.converter),
            "{label} item {item_index} converter output type mismatch"
        );
        assert_eq!(
            cpp_item.converter_source_path_ids,
            data_converter_operation_view_model_source_path_ids(rust_item.converter),
            "{label} item {item_index} converter source path ids mismatch"
        );
        assert_eq!(
            cpp_item.interpolator_core_type,
            resolved_data_converter_interpolator_core_type(rust_file, rust_item.converter),
            "{label} item {item_index} interpolator core type mismatch"
        );
    }
}

fn compare_data_converter_formula_tokens(
    cpp_tokens: &[CppProbeConverterFormulaToken],
    rust_tokens: &[&RuntimeObject],
    label: &str,
) {
    assert_eq!(
        cpp_tokens.len(),
        rust_tokens.len(),
        "{label} count mismatch"
    );
    for (token_index, (cpp_token, rust_token)) in cpp_tokens.iter().zip(rust_tokens).enumerate() {
        assert_eq!(
            cpp_token.index, token_index,
            "{label} token {token_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_token.core_type, rust_token.type_key,
            "{label} token {token_index} core type mismatch"
        );
        compare_cpp_registry_properties(
            &cpp_token.property_values,
            rust_token,
            &format!("{label} token {token_index}"),
        );
    }
}

fn compare_data_converter_formula_tokens_result(
    cpp_tokens: &[CppProbeConverterFormulaToken],
    rust_tokens: &[&RuntimeObject],
    label: &str,
) -> Result<(), String> {
    if cpp_tokens.len() != rust_tokens.len() {
        return Err(format!(
            "{label} count mismatch, C++ {}, Rust {}",
            cpp_tokens.len(),
            rust_tokens.len()
        ));
    }

    for (token_index, (cpp_token, rust_token)) in cpp_tokens.iter().zip(rust_tokens).enumerate() {
        if cpp_token.index != token_index {
            return Err(format!(
                "{label} token {token_index} C++ index mismatch, got {}",
                cpp_token.index
            ));
        }
        if cpp_token.core_type != rust_token.type_key {
            return Err(format!(
                "{label} token {token_index} core type mismatch, C++ {}, Rust {}",
                cpp_token.core_type, rust_token.type_key
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_token.property_values,
            rust_token,
            &format!("{label} token {token_index}"),
        )?;
    }

    Ok(())
}

fn compare_data_converter_group_items_result(
    rust_file: &RuntimeFile,
    cpp_items: &[CppProbeConverterGroupItem],
    rust_items: &[nuxie_binary::RuntimeDataConverterGroupItem<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_items.len() != rust_items.len() {
        return Err(format!(
            "{label} count mismatch, C++ {}, Rust {}",
            cpp_items.len(),
            rust_items.len()
        ));
    }

    for (item_index, (cpp_item, rust_item)) in cpp_items.iter().zip(rust_items).enumerate() {
        if cpp_item.index != item_index {
            return Err(format!(
                "{label} item {item_index} C++ index mismatch, got {}",
                cpp_item.index
            ));
        }
        if cpp_item.core_type != rust_item.object.type_key {
            return Err(format!(
                "{label} item {item_index} core type mismatch, C++ {}, Rust {}",
                cpp_item.core_type, rust_item.object.type_key
            ));
        }
        let rust_converter_core_type = rust_item.converter.map(|converter| converter.type_key);
        if cpp_item.converter_core_type != rust_converter_core_type {
            return Err(format!(
                "{label} item {item_index} converter core type mismatch, C++ {:?}, Rust {:?}",
                cpp_item.converter_core_type, rust_converter_core_type
            ));
        }
        let rust_converter_output_type = data_converter_output_type(rust_file, rust_item.converter);
        if cpp_item.converter_output_type != rust_converter_output_type {
            return Err(format!(
                "{label} item {item_index} converter output type mismatch, C++ {:?}, Rust {:?}",
                cpp_item.converter_output_type, rust_converter_output_type
            ));
        }
        let rust_source_path_ids =
            data_converter_operation_view_model_source_path_ids(rust_item.converter);
        if cpp_item.converter_source_path_ids != rust_source_path_ids {
            return Err(format!(
                "{label} item {item_index} converter source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_item.converter_source_path_ids, rust_source_path_ids
            ));
        }
        let rust_interpolator_core_type =
            resolved_data_converter_interpolator_core_type(rust_file, rust_item.converter);
        if cpp_item.interpolator_core_type != rust_interpolator_core_type {
            return Err(format!(
                "{label} item {item_index} interpolator core type mismatch, C++ {:?}, Rust {:?}",
                cpp_item.interpolator_core_type, rust_interpolator_core_type
            ));
        }
    }

    Ok(())
}

fn compare_state_machine_children(
    rust_file: &RuntimeFile,
    cpp_state_machine: &CppProbeStateMachine,
    rust_children: &nuxie_binary::RuntimeStateMachine<'_>,
    label: &str,
) {
    assert_eq!(
        cpp_state_machine.layer_count,
        rust_children.layers.len(),
        "{label} layer count mismatch"
    );
    assert_eq!(
        cpp_state_machine.layers.len(),
        rust_children.layers.len(),
        "{label} layer list length mismatch"
    );
    for (layer_index, (cpp_layer, rust_layer)) in cpp_state_machine
        .layers
        .iter()
        .zip(&rust_children.layers)
        .enumerate()
    {
        let cpp_layer = cpp_layer
            .as_ref()
            .unwrap_or_else(|| panic!("{label} layer {layer_index} is null in C++"));
        assert_eq!(
            cpp_layer.index, layer_index,
            "{label} layer {layer_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_layer.core_type, rust_layer.object.type_key,
            "{label} layer {layer_index} core type mismatch"
        );
        assert_eq!(
            cpp_layer.name,
            rust_layer
                .object
                .string_property("name")
                .unwrap_or_default(),
            "{label} layer {layer_index} name mismatch"
        );
        assert_eq!(
            cpp_layer.state_count, rust_layer.state_count,
            "{label} layer {layer_index} state count mismatch"
        );
        compare_cpp_registry_properties(
            &cpp_layer.property_values,
            rust_layer.object,
            &format!("{label} layer {layer_index}"),
        );
    }

    assert_eq!(
        cpp_state_machine.input_count,
        rust_children.inputs.len(),
        "{label} input count mismatch"
    );
    assert_eq!(
        cpp_state_machine.inputs.len(),
        rust_children.inputs.len(),
        "{label} input list length mismatch"
    );
    for (input_index, (cpp_input, rust_input)) in cpp_state_machine
        .inputs
        .iter()
        .zip(&rust_children.inputs)
        .enumerate()
    {
        let cpp_input = cpp_input
            .as_ref()
            .unwrap_or_else(|| panic!("{label} input {input_index} is null in C++"));
        assert_eq!(
            cpp_input.index, input_index,
            "{label} input {input_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_input.core_type, rust_input.type_key,
            "{label} input {input_index} core type mismatch"
        );
        assert_eq!(
            cpp_input.name,
            rust_input.string_property("name").unwrap_or_default(),
            "{label} input {input_index} name mismatch"
        );
        compare_cpp_registry_properties(
            &cpp_input.property_values,
            rust_input,
            &format!("{label} input {input_index}"),
        );
    }

    assert_eq!(
        cpp_state_machine.listener_count,
        rust_children.listeners.len(),
        "{label} listener count mismatch"
    );
    assert_eq!(
        cpp_state_machine.listeners.len(),
        rust_children.listeners.len(),
        "{label} listener list length mismatch"
    );
    for (listener_index, (cpp_listener, rust_listener)) in cpp_state_machine
        .listeners
        .iter()
        .zip(&rust_children.listeners)
        .enumerate()
    {
        let cpp_listener = cpp_listener
            .as_ref()
            .unwrap_or_else(|| panic!("{label} listener {listener_index} is null in C++"));
        assert_eq!(
            cpp_listener.index, listener_index,
            "{label} listener {listener_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_listener.core_type, rust_listener.object.type_key,
            "{label} listener {listener_index} core type mismatch"
        );
        assert_eq!(
            cpp_listener.name,
            rust_listener
                .object
                .string_property("name")
                .unwrap_or_default(),
            "{label} listener {listener_index} name mismatch"
        );
        assert_eq!(
            rust_listener.object.uint_property("targetId"),
            Some(cpp_listener.target_id),
            "{label} listener {listener_index} target id mismatch"
        );
        assert_eq!(
            cpp_listener.data_bind_path_ids,
            data_bind_path_ids_for_referencer(rust_file, rust_listener.object),
            "{label} listener {listener_index} data-bind path mismatch"
        );
        assert_eq!(
            cpp_listener.resolved_data_bind_path_ids,
            resolved_data_bind_path_ids_for_referencer(rust_file, rust_listener.object),
            "{label} listener {listener_index} resolved data-bind path mismatch"
        );
        compare_cpp_registry_properties(
            &cpp_listener.property_values,
            rust_listener.object,
            &format!("{label} listener {listener_index}"),
        );
        assert_eq!(
            cpp_listener.action_count,
            rust_listener.actions.len(),
            "{label} listener {listener_index} action count mismatch"
        );
        assert_eq!(
            cpp_listener.actions.len(),
            rust_listener.actions.len(),
            "{label} listener {listener_index} action list length mismatch"
        );
        for (action_index, (cpp_action, rust_action)) in cpp_listener
            .actions
            .iter()
            .zip(&rust_listener.actions)
            .enumerate()
        {
            let rust_action = rust_action.object;
            let cpp_action = cpp_action.as_ref().unwrap_or_else(|| {
                panic!("{label} listener {listener_index} action {action_index} is null in C++")
            });
            assert_eq!(
                cpp_action.index, action_index,
                "{label} listener {listener_index} action {action_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_action.core_type, rust_action.type_key,
                "{label} listener {listener_index} action {action_index} core type mismatch"
            );
            assert_eq!(
                rust_action.uint_property("flags"),
                Some(cpp_action.flags),
                "{label} listener {listener_index} action {action_index} flags mismatch"
            );
            assert_eq!(
                cpp_action.data_bind_path_ids,
                data_bind_path_ids_for_referencer(rust_file, rust_action),
                "{label} listener {listener_index} action {action_index} data-bind path mismatch"
            );
            assert_eq!(
                cpp_action.resolved_data_bind_path_ids,
                resolved_data_bind_path_ids_for_referencer(rust_file, rust_action),
                "{label} listener {listener_index} action {action_index} resolved data-bind path mismatch"
            );
            compare_cpp_registry_properties(
                &cpp_action.property_values,
                rust_action,
                &format!("{label} listener {listener_index} action {action_index}"),
            );
        }
        assert_eq!(
            cpp_listener.listener_input_type_count,
            rust_listener.listener_input_types.len(),
            "{label} listener {listener_index} listener input type count mismatch"
        );
        assert_eq!(
            cpp_listener.listener_input_types.len(),
            rust_listener.listener_input_types.len(),
            "{label} listener {listener_index} listener input type list length mismatch"
        );
        for (input_type_index, (cpp_input_type, rust_input_type)) in cpp_listener
            .listener_input_types
            .iter()
            .zip(&rust_listener.listener_input_types)
            .enumerate()
        {
            let cpp_input_type = cpp_input_type.as_ref().unwrap_or_else(|| {
                panic!(
                    "{label} listener {listener_index} input type {input_type_index} is null in C++"
                )
            });
            assert_eq!(
                cpp_input_type.index, input_type_index,
                "{label} listener {listener_index} input type {input_type_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_input_type.core_type, rust_input_type.type_key,
                "{label} listener {listener_index} input type {input_type_index} core type mismatch"
            );
            assert_eq!(
                rust_input_type.uint_property("listenerTypeValue"),
                Some(cpp_input_type.listener_type_value),
                "{label} listener {listener_index} input type {input_type_index} listener type value mismatch"
            );
            assert_eq!(
                cpp_input_type.data_bind_path_ids,
                data_bind_path_ids_for_referencer(rust_file, rust_input_type),
                "{label} listener {listener_index} input type {input_type_index} data-bind path mismatch"
            );
            assert_eq!(
                cpp_input_type.resolved_data_bind_path_ids,
                resolved_data_bind_path_ids_for_referencer(rust_file, rust_input_type),
                "{label} listener {listener_index} input type {input_type_index} resolved data-bind path mismatch"
            );
            assert_eq!(
                cpp_input_type.view_model_path_ids_buffer,
                rust_file
                    .listener_input_type_view_model_path_ids_buffer_for_object(rust_input_type),
                "{label} listener {listener_index} input type {input_type_index} view-model path ids buffer mismatch"
            );
            compare_cpp_registry_properties(
                &cpp_input_type.property_values,
                rust_input_type,
                &format!("{label} listener {listener_index} input type {input_type_index}"),
            );
        }
    }

    assert_eq!(
        cpp_state_machine.data_bind_count,
        rust_children.data_binds.len(),
        "{label} data bind count mismatch"
    );
    assert_eq!(
        cpp_state_machine.data_binds.len(),
        rust_children.data_binds.len(),
        "{label} data bind list length mismatch"
    );
    for (data_bind_index, (cpp_data_bind, rust_data_bind)) in cpp_state_machine
        .data_binds
        .iter()
        .zip(&rust_children.data_binds)
        .enumerate()
    {
        let cpp_data_bind = cpp_data_bind
            .as_ref()
            .unwrap_or_else(|| panic!("{label} data bind {data_bind_index} is null in C++"));
        assert_eq!(
            cpp_data_bind.index, data_bind_index,
            "{label} data bind {data_bind_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_data_bind.core_type, rust_data_bind.type_key,
            "{label} data bind {data_bind_index} core type mismatch"
        );
        assert_eq!(
            rust_data_bind.uint_property("propertyKey"),
            Some(cpp_data_bind.property_key),
            "{label} data bind {data_bind_index} property key mismatch"
        );
        assert_eq!(
            rust_data_bind.uint_property("flags"),
            Some(cpp_data_bind.flags),
            "{label} data bind {data_bind_index} flags mismatch"
        );
        assert_eq!(
            rust_data_bind.uint_property("converterId"),
            Some(cpp_data_bind.converter_id),
            "{label} data bind {data_bind_index} converter id mismatch"
        );
        assert_eq!(
            rust_file
                .resolved_data_converter_for_data_bind_object(rust_data_bind)
                .map(|converter| converter.type_key),
            cpp_data_bind.converter_core_type,
            "{label} data bind {data_bind_index} converter core type mismatch"
        );
        let rust_converter = rust_file.resolved_data_converter_for_data_bind_object(rust_data_bind);
        assert_eq!(
            data_converter_output_type(rust_file, rust_converter),
            cpp_data_bind.converter_output_type,
            "{label} data bind {data_bind_index} converter output type mismatch"
        );
        assert_eq!(
            data_bind_source_output_type(rust_file, rust_data_bind),
            cpp_data_bind.data_bind_source_output_type,
            "{label} data bind {data_bind_index} source output type mismatch"
        );
        assert_eq!(
            data_bind_output_type(rust_file, rust_data_bind),
            cpp_data_bind.data_bind_output_type,
            "{label} data bind {data_bind_index} output type mismatch"
        );
        assert_eq!(
            data_bind_target_supports_push(rust_file, rust_data_bind),
            cpp_data_bind.target_supports_push,
            "{label} data bind {data_bind_index} target-supports-push mismatch"
        );
        assert_eq!(
            data_bind_uses_persisting_list(rust_file, rust_data_bind),
            cpp_data_bind.uses_persisting_list,
            "{label} data bind {data_bind_index} persisting-list mismatch"
        );
        assert_eq!(
            data_converter_operation_view_model_source_path_ids(rust_converter),
            cpp_data_bind.converter_source_path_ids,
            "{label} data bind {data_bind_index} converter source path ids mismatch"
        );
        assert_eq!(
            resolved_data_converter_interpolator_core_type(rust_file, rust_converter),
            cpp_data_bind.converter_interpolator_core_type,
            "{label} data bind {data_bind_index} converter interpolator core type mismatch"
        );
        compare_data_converter_group_items(
            rust_file,
            &cpp_data_bind.converter_group_items,
            &resolved_data_bind_converter_group_items(rust_file, rust_converter),
            &format!("{label} data bind {data_bind_index} converter group items"),
        );
        compare_data_converter_formula_tokens(
            &cpp_data_bind.converter_formula_tokens,
            &resolved_data_bind_converter_formula_tokens(rust_file, rust_converter),
            &format!("{label} data bind {data_bind_index} converter formula tokens"),
        );
        compare_cpp_registry_properties(
            &cpp_data_bind.property_values,
            rust_data_bind,
            &format!("{label} data bind {data_bind_index}"),
        );
    }

    assert_eq!(
        cpp_state_machine.scripted_object_count,
        rust_children.scripted_objects.len(),
        "{label} scripted object count mismatch"
    );
    assert_eq!(
        cpp_state_machine.scripted_objects.len(),
        rust_children.scripted_objects.len(),
        "{label} scripted object list length mismatch"
    );
    for (scripted_object_index, (cpp_scripted_object, rust_scripted_object)) in cpp_state_machine
        .scripted_objects
        .iter()
        .zip(&rust_children.scripted_objects)
        .enumerate()
    {
        assert_eq!(
            cpp_scripted_object.index, scripted_object_index,
            "{label} scripted object {scripted_object_index} C++ index mismatch"
        );
        assert_eq!(
            cpp_scripted_object.core_type, rust_scripted_object.object.type_key,
            "{label} scripted object {scripted_object_index} core type mismatch"
        );
        assert_eq!(
            cpp_scripted_object.input_count,
            rust_scripted_object.inputs.len(),
            "{label} scripted object {scripted_object_index} input count mismatch"
        );
        assert_eq!(
            cpp_scripted_object.inputs.len(),
            rust_scripted_object.inputs.len(),
            "{label} scripted object {scripted_object_index} input list length mismatch"
        );
        for (input_index, (cpp_input, rust_input)) in cpp_scripted_object
            .inputs
            .iter()
            .zip(&rust_scripted_object.inputs)
            .enumerate()
        {
            assert_eq!(
                cpp_input.index, input_index,
                "{label} scripted object {scripted_object_index} input {input_index} C++ index mismatch"
            );
            assert_eq!(
                cpp_input.core_type, rust_input.type_key,
                "{label} scripted object {scripted_object_index} input {input_index} core type mismatch"
            );
            assert_eq!(
                cpp_input.name,
                rust_input.string_property("name").unwrap_or_default(),
                "{label} scripted object {scripted_object_index} input {input_index} name mismatch"
            );
            compare_cpp_registry_properties(
                &cpp_input.property_values,
                rust_input,
                &format!("{label} scripted object {scripted_object_index} input {input_index}"),
            );
        }
    }
}

fn compare_keyed_objects(
    cpp_keyed_objects: &[CppProbeKeyedObject],
    rust_keyed_objects: &[nuxie_binary::RuntimeKeyedObject<'_>],
    animation_fps: u64,
    label: &str,
) {
    assert_eq!(
        cpp_keyed_objects.len(),
        rust_keyed_objects.len(),
        "{label} keyed object count mismatch"
    );

    for (keyed_object_index, (cpp_keyed_object, rust_keyed_object)) in
        cpp_keyed_objects.iter().zip(rust_keyed_objects).enumerate()
    {
        assert_eq!(
            cpp_keyed_object.index, keyed_object_index,
            "{label} C++ keyed object index mismatch"
        );
        assert_eq!(
            cpp_keyed_object.core_type, rust_keyed_object.object.type_key,
            "{label} keyed object {keyed_object_index} core type mismatch"
        );
        assert_eq!(
            rust_keyed_object.object.uint_property("objectId"),
            Some(cpp_keyed_object.object_id),
            "{label} keyed object {keyed_object_index} object id mismatch"
        );
        compare_cpp_registry_properties(
            &cpp_keyed_object.property_values,
            rust_keyed_object.object,
            &format!("{label} keyed object {keyed_object_index}"),
        );
        assert_eq!(
            cpp_keyed_object.keyed_properties.len(),
            rust_keyed_object.keyed_properties.len(),
            "{label} keyed object {keyed_object_index} keyed property count mismatch"
        );

        for (keyed_property_index, (cpp_keyed_property, rust_keyed_property)) in cpp_keyed_object
            .keyed_properties
            .iter()
            .zip(&rust_keyed_object.keyed_properties)
            .enumerate()
        {
            assert_eq!(
                cpp_keyed_property.index, keyed_property_index,
                "{label} keyed object {keyed_object_index} C++ keyed property index mismatch"
            );
            assert_eq!(
                cpp_keyed_property.core_type, rust_keyed_property.object.type_key,
                "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} core type mismatch"
            );
            assert_eq!(
                rust_keyed_property.object.uint_property("propertyKey"),
                Some(cpp_keyed_property.property_key),
                "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} property key mismatch"
            );
            compare_cpp_registry_properties(
                &cpp_keyed_property.property_values,
                rust_keyed_property.object,
                &format!(
                    "{label} keyed object {keyed_object_index} keyed property {keyed_property_index}"
                ),
            );

            match (
                &cpp_keyed_property.first_key_frame,
                rust_keyed_property.first_key_frame,
            ) {
                (None, None) => {}
                (Some(cpp_key_frame), Some(rust_key_frame)) => {
                    assert_eq!(
                        cpp_key_frame.core_type, rust_key_frame.type_key,
                        "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} first keyframe core type mismatch"
                    );
                    assert_eq!(
                        rust_key_frame.uint_property("frame"),
                        Some(cpp_key_frame.frame),
                        "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} first keyframe frame mismatch"
                    );
                    let expected_seconds = rust_key_frame
                        .uint_property("frame")
                        .map(|frame| frame as f32 / animation_fps as f32);
                    assert_float_eq(
                        cpp_key_frame.seconds,
                        expected_seconds,
                        &format!(
                            "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} first keyframe seconds"
                        ),
                    );
                    compare_cpp_registry_properties(
                        &cpp_key_frame.property_values,
                        rust_key_frame,
                        &format!(
                            "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} first keyframe"
                        ),
                    );
                }
                (None, Some(rust_key_frame)) => panic!(
                    "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} has no C++ first keyframe but Rust has {}",
                    rust_key_frame.type_name
                ),
                (Some(cpp_key_frame), None) => panic!(
                    "{label} keyed object {keyed_object_index} keyed property {keyed_property_index} has no Rust first keyframe but C++ has core type {}",
                    cpp_key_frame.core_type
                ),
            }
        }
    }
}

#[test]
fn cpp_probe_matches_rust_stored_registry_property_values_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ registry property comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    for fixture_path in [
        "minimal/long_name.riv",
        "minimal/two_artboards.riv",
        "graph/clipping_and_draw_order.riv",
        "graph/dependency_test.riv",
        "graph/draw_rule_cycle.riv",
        "animation/smi_test.riv",
        "animation/state_machine_transition.riv",
    ] {
        compare_stored_registry_property_values(&probe, fixture_path);
    }
}

fn compare_stored_registry_property_values(probe: &Path, fixture_path: &str) {
    let cpp = read_cpp_probe_file_with_property_values(probe, fixture_path);
    let rust = read_rust_runtime_file(fixture_path);
    let ranges = runtime_artboard_ranges(&rust);

    assert_eq!(
        cpp.artboards.len(),
        ranges.len(),
        "artboard count mismatch for {fixture_path}"
    );

    for (artboard_index, (cpp_artboard, range)) in cpp.artboards.iter().zip(&ranges).enumerate() {
        let rust_objects = runtime_artboard_local_objects(&rust, *range);
        for (local_id, (cpp_object, rust_object)) in
            cpp_artboard.objects.iter().zip(&rust_objects).enumerate()
        {
            let (Some(cpp_object), Some(rust_object)) = (cpp_object, rust_object) else {
                continue;
            };
            if rust_object.type_name == "Artboard" {
                continue;
            }

            compare_cpp_registry_properties(
                &cpp_object.property_values,
                rust_object,
                &format!("object {local_id} in artboard {artboard_index} for {fixture_path}"),
            );
        }
    }
}

fn compare_cpp_registry_properties(
    cpp_properties: &[CppProbePropertyValue],
    rust_object: &RuntimeObject,
    label: &str,
) {
    compare_cpp_registry_properties_result(cpp_properties, rust_object, label)
        .unwrap_or_else(|err| panic!("{err}"));
}

fn compare_cpp_registry_properties_result(
    cpp_properties: &[CppProbePropertyValue],
    rust_object: &RuntimeObject,
    label: &str,
) -> Result<(), String> {
    let mut failures = Vec::new();
    for cpp_property in cpp_properties {
        let Some((name, kind)) = stored_getter_property(rust_object, cpp_property.key) else {
            continue;
        };
        if cpp_property.kind != kind {
            failures.push(format!(
                "{label} property {} kind mismatch, C++ {:?}, Rust {:?}",
                cpp_property.key, cpp_property.kind, kind
            ));
            continue;
        }

        if let Err(err) = compare_cpp_property_value_result(
            cpp_property,
            rust_object,
            name,
            &format!("{label} property {name} ({})", cpp_property.key),
        ) {
            failures.push(err);
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn stored_getter_property(
    object: &RuntimeObject,
    key: u16,
) -> Option<(&'static str, CppProbeFieldKind)> {
    let definition = definition_by_type_key(object.type_key)?;
    let property = primary_property_by_key_in_hierarchy(definition, key)?;
    if !property.stores_field || property.override_get || property.virtual_ || property.pure_virtual
    {
        // C++ CoreRegistry getters for these properties may expose resolved runtime state
        // instead of the stored import member that nuxie-binary preserves.
        return None;
    }

    let kind = core_registry_getter_field_kind_by_property_key(key)?;
    Some((property.name, CppProbeFieldKind::from_field_kind(kind)?))
}

fn primary_property_by_key_in_hierarchy(
    definition: &nuxie_schema::Definition,
    key: u16,
) -> Option<&'static nuxie_schema::Property> {
    definition
        .properties
        .iter()
        .find(|property| property.key.int == key)
        .or_else(|| {
            definition.ancestors.iter().find_map(|ancestor| {
                let ancestor = definition_by_name(ancestor)?;
                ancestor
                    .properties
                    .iter()
                    .find(|property| property.key.int == key)
            })
        })
}

fn compare_cpp_property_value_result(
    cpp_property: &CppProbePropertyValue,
    rust_object: &RuntimeObject,
    name: &str,
    label: &str,
) -> Result<(), String> {
    match cpp_property.kind {
        CppProbeFieldKind::Uint => {
            let expected = cpp_property
                .value
                .as_u64()
                .ok_or_else(|| format!("{label} C++ value is not uint"))?;
            let actual = rust_object.uint_property(name);
            if actual != Some(expected) {
                return Err(format!("{label} mismatch: C++ {expected}, Rust {actual:?}"));
            }
        }
        CppProbeFieldKind::String => {
            let expected = cpp_property
                .value
                .as_str()
                .ok_or_else(|| format!("{label} C++ value is not string"))?;
            let actual = rust_object.string_property(name);
            if actual != Some(expected) {
                return Err(format!(
                    "{label} mismatch: C++ {expected:?}, Rust {actual:?}"
                ));
            }
        }
        CppProbeFieldKind::Double => {
            let expected = cpp_double_value(cpp_property, label)?;
            let actual = rust_object.double_property(name);
            if !float_matches(expected, actual) {
                return Err(format!("{label} mismatch: C++ {expected}, Rust {actual:?}"));
            }
        }
        CppProbeFieldKind::Color => {
            let expected = cpp_property
                .value
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| format!("{label} C++ value is not color"))?;
            let actual = rust_object.color_property(name);
            if actual != Some(expected) {
                return Err(format!("{label} mismatch: C++ {expected}, Rust {actual:?}"));
            }
        }
        CppProbeFieldKind::Bool => {
            let expected = cpp_property
                .value
                .as_bool()
                .ok_or_else(|| format!("{label} C++ value is not bool"))?;
            let actual = rust_object.bool_property(name);
            if actual != Some(expected) {
                return Err(format!("{label} mismatch: C++ {expected}, Rust {actual:?}"));
            }
        }
    }
    Ok(())
}

fn cpp_double_value(cpp_property: &CppProbePropertyValue, label: &str) -> Result<f32, String> {
    if let Some(value) = cpp_property.value.as_f64() {
        return Ok(value as f32);
    }

    match cpp_property.value.as_str() {
        Some("nan") => Ok(f32::NAN),
        Some("inf") => Ok(f32::INFINITY),
        Some("-inf") => Ok(f32::NEG_INFINITY),
        _ => Err(format!("{label} C++ value is not double")),
    }
}

#[test]
fn cpp_failed_import_diagnostics_match_rust_import_status_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ failed import comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let cases = [
        (
            "contextless_known_objects_drop",
            synthetic_runtime_file(6041, |bytes| {
                push_empty_object(bytes, "Node");
                push_empty_object(bytes, "Artboard");
                push_empty_object(bytes, "ImageAsset");
                push_empty_object(bytes, "FileAssetContents");
                push_empty_object(bytes, "LinearAnimation");
                push_empty_object(bytes, "StateMachine");
            }),
        ),
        (
            "valid_basic_import_stack_keeps_objects",
            synthetic_runtime_file(6042, |bytes| {
                push_empty_object(bytes, "Backboard");
                push_empty_object(bytes, "Artboard");
                push_empty_object(bytes, "Node");
                push_empty_object(bytes, "ImageAsset");
                push_empty_object(bytes, "FileAssetContents");
                push_empty_object(bytes, "LinearAnimation");
            }),
        ),
        (
            "file_asset_referencers_require_backboard_context",
            synthetic_runtime_file(6076, |bytes| {
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
            }),
        ),
        (
            "listener_input_type_contexts",
            synthetic_runtime_file(6047, |bytes| {
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
            }),
        ),
        (
            "transition_importer_contexts",
            synthetic_runtime_file(6048, |bytes| {
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
            }),
        ),
        (
            "transition_input_condition_validation",
            synthetic_runtime_file(6049, |bytes| {
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
            }),
        ),
        (
            "latest_importer_consumes_null_object",
            synthetic_runtime_file(6059, |bytes| {
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
            }),
        ),
        (
            "non_consuming_latest_importer_allows_older_null_consumer",
            synthetic_runtime_file(6071, |bytes| {
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
            }),
        ),
        (
            "blend_input_validation",
            synthetic_runtime_file(6053, |bytes| {
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
            }),
        ),
        (
            "listener_input_change_validation",
            synthetic_runtime_file(6054, |bytes| {
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
            }),
        ),
        (
            "nested_listener_input_change_validation",
            synthetic_runtime_file(6056, |bytes| {
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
            }),
        ),
    ];

    for (name, bytes) in cases {
        let cpp_failures = cpp_failed_import_type_keys(&probe, name, &bytes);
        let rust = read_runtime_file(&bytes)
            .unwrap_or_else(|err| panic!("Rust failed to decode {name}: {err:#}"));
        let rust_failures = rust_dropped_import_type_keys(&rust);

        assert_eq!(
            rust_failures,
            cpp_failures,
            "failed object->import type-key sequence mismatch for {name}\nC++:  {}\nRust: {}",
            format_type_keys(&cpp_failures),
            format_type_keys(&rust_failures)
        );
    }
}

#[test]
fn cpp_custom_import_methods_are_tracked_by_binary_import_model() {
    let src_dir = reference_runtime_dir().join("src");
    if !src_dir.exists() {
        eprintln!(
            "skipping C++ custom import method audit; reference src not found at {}; set RIVE_RUNTIME_DIR",
            src_dir.display()
        );
        return;
    }

    let mut cpp_files = Vec::new();
    collect_cpp_paths(&src_dir, &mut cpp_files);

    let mut actual = BTreeSet::new();
    for path in cpp_files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&contents);
        for rest in compact.split("StatusCode").skip(1) {
            let Some((class_name, _)) = rest.split_once("::import(ImportStack&") else {
                continue;
            };
            if !is_cpp_identifier(class_name) {
                continue;
            }
            actual.insert(class_name.to_owned());
        }
    }

    let expected = [
        "Artboard",
        "AudioEvent",
        "BlendAnimation",
        "BlendAnimationDirect",
        "BlendState1DInput",
        "BlendState1DViewModel",
        "Component",
        "DataBind",
        "DataBindPath",
        "DataConverter",
        "DataConverterGroupItem",
        "DataEnumValue",
        "FileAsset",
        "FileAssetContents",
        "FormulaToken",
        "GamepadInput",
        "Image",
        "KeyFrame",
        "KeyFrameInterpolator",
        "KeyboardInput",
        "KeyedObject",
        "KeyedProperty",
        "LayerState",
        "LinearAnimation",
        "ListenerAction",
        "ListenerInputChange",
        "ListenerInputType",
        "ListenerViewModelChange",
        "NestedArtboard",
        "ScriptInputArtboard",
        "ScriptInputBoolean",
        "ScriptInputColor",
        "ScriptInputNumber",
        "ScriptInputString",
        "ScriptInputTrigger",
        "ScriptInputViewModelProperty",
        "ScriptedDataConverter",
        "ScriptedDrawable",
        "ScriptedInterpolator",
        "ScriptedListenerAction",
        "ScriptedPathEffect",
        "ScriptedTransitionCondition",
        "ScrollConstraint",
        "ScrollPhysics",
        "SemanticInput",
        "StateMachine",
        "StateMachineFireAction",
        "StateMachineFireTrigger",
        "StateMachineInput",
        "StateMachineLayer",
        "StateMachineListener",
        "StateMachineListenerSingle",
        "StateTransition",
        "TextStyle",
        "TransitionComparator",
        "TransitionCondition",
        "TransitionInputCondition",
        "TransitionPropertyViewModelComparator",
        "ViewModelInstance",
        "ViewModelInstanceAsset",
        "ViewModelInstanceListItem",
        "ViewModelInstanceValue",
        "ViewModelInstanceViewModel",
        "ViewModelProperty",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "C++ custom import(ImportStack&) surface changed; audit object_imports_successfully, update synthetic import-status coverage, then refresh this list"
    );
}

#[test]
fn cpp_scroll_physics_import_methods_are_tracked_by_binary_import_model() {
    let scroll_physics_path =
        reference_runtime_dir().join("src/constraints/scrolling/scroll_physics.cpp");
    let scroll_constraint_path =
        reference_runtime_dir().join("src/constraints/scrolling/scroll_constraint.cpp");
    if !scroll_physics_path.exists() || !scroll_constraint_path.exists() {
        eprintln!(
            "skipping C++ scroll-physics import audit; reference sources not found at {} and {}; set RIVE_RUNTIME_DIR",
            scroll_physics_path.display(),
            scroll_constraint_path.display()
        );
        return;
    }

    let scroll_physics_source = std::fs::read_to_string(&scroll_physics_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", scroll_physics_path.display()));
    let compact_scroll_physics = compact_cpp_source(&scroll_physics_source);
    let scroll_physics_import = compact_cpp_function_body(
        &compact_scroll_physics,
        "ScrollPhysics::import(ImportStack&importStack)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing ScrollPhysics::import in {}",
            scroll_physics_path.display()
        )
    });
    assert_eq!(
        scroll_physics_import,
        "autobackboardImporter=importStack.latest<BackboardImporter>(BackboardBase::typeKey);if(backboardImporter!=nullptr){backboardImporter->addPhysics(this);}else{returnStatusCode::MissingObject;}returnStatusCode::Ok;",
        "ScrollPhysics::import changed; audit RuntimeFile::scroll_physics and scroll-physics import status"
    );

    let scroll_constraint_source = std::fs::read_to_string(&scroll_constraint_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", scroll_constraint_path.display()));
    let compact_scroll_constraint = compact_cpp_source(&scroll_constraint_source);
    let scroll_constraint_import = compact_cpp_function_body(
        &compact_scroll_constraint,
        "ScrollConstraint::import(ImportStack&importStack)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing ScrollConstraint::import in {}",
            scroll_constraint_path.display()
        )
    });
    assert_eq!(
        scroll_constraint_import,
        "autobackboardImporter=importStack.latest<BackboardImporter>(BackboardBase::typeKey);if(backboardImporter!=nullptr){std::vector<ScrollPhysics*>physicsObjects=backboardImporter->physics();if(physicsId()!=-1&&physicsId()<physicsObjects.size()){autophys=physicsObjects[physicsId()];if(phys!=nullptr){autocloned=phys->clone()->as<ScrollPhysics>();physics(cloned);}}}else{returnStatusCode::MissingObject;}returnSuper::import(importStack);",
        "ScrollConstraint::import changed; audit RuntimeFile::resolved_scroll_physics_for_constraint"
    );
}

#[test]
fn cpp_custom_encoded_property_decoders_are_tracked_by_binary_import_model() {
    let generated_dir = reference_runtime_dir().join("include/rive/generated");
    if !generated_dir.exists() {
        eprintln!(
            "skipping C++ encoded property decoder audit; generated headers not found at {}; set RIVE_RUNTIME_DIR",
            generated_dir.display()
        );
        return;
    }

    let mut headers = Vec::new();
    collect_hpp_paths(&generated_dir, &mut headers);

    let mut actual = BTreeSet::new();
    for path in headers {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let Some(class_name) = cpp_generated_base_class_name(&contents) else {
            continue;
        };

        for line in contents.lines() {
            let Some(rest) = line.trim_start().strip_prefix("virtual void decode") else {
                continue;
            };
            let Some((property_suffix, _)) = rest.split_once('(') else {
                continue;
            };
            actual.insert(format!("{class_name}::decode{property_suffix}"));
        }
    }

    let expected = [
        "DataBindContext::decodeSourcePathIds",
        "DataBindPath::decodePath",
        "DataConverterOperationViewModel::decodeSourcePathIds",
        "FileAsset::decodeCdnUuid",
        "FileAssetContents::decodeBytes",
        "FileAssetContents::decodeSignature",
        "ListenerInputTypeViewModel::decodeViewModelPathIds",
        "Mesh::decodeTriangleIndexBytes",
        "NestedArtboard::decodeDataBindPathIds",
        "ScriptInputViewModelProperty::decodeDataBindPathIds",
        "StateMachineFireTrigger::decodeViewModelPathIds",
        "StateMachineListenerSingle::decodeViewModelPathIds",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "C++ encoded property decoder surface changed; audit decoded-buffer helpers, C++ probe coverage, and binary import tests before refreshing this list"
    );
}

#[test]
fn cpp_custom_encoded_property_copiers_are_tracked_by_binary_import_model() {
    let generated_dir = reference_runtime_dir().join("include/rive/generated");
    if !generated_dir.exists() {
        eprintln!(
            "skipping C++ encoded property copier audit; generated headers not found at {}; set RIVE_RUNTIME_DIR",
            generated_dir.display()
        );
        return;
    }

    let mut headers = Vec::new();
    collect_hpp_paths(&generated_dir, &mut headers);

    let mut actual = BTreeSet::new();
    for path in headers {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let Some(class_name) = cpp_generated_base_class_name(&contents) else {
            continue;
        };

        let compact = compact_cpp_source(&contents);
        for rest in compact.split("virtualvoidcopy").skip(1) {
            let Some((property_suffix, after_name)) = rest.split_once('(') else {
                continue;
            };
            let Some((signature_tail, _)) = after_name.split_once(';') else {
                continue;
            };
            if signature_tail.contains("const") && signature_tail.contains("=0") {
                actual.insert(format!("{class_name}::copy{property_suffix}"));
            }
        }
    }

    let expected = [
        "DataBindContext::copySourcePathIds",
        "DataBindPath::copyPath",
        "DataConverterOperationViewModel::copySourcePathIds",
        "FileAsset::copyCdnUuid",
        "FileAssetContents::copyBytes",
        "FileAssetContents::copySignature",
        "ListenerInputTypeViewModel::copyViewModelPathIds",
        "Mesh::copyTriangleIndexBytes",
        "NestedArtboard::copyDataBindPathIds",
        "ScriptInputViewModelProperty::copyDataBindPathIds",
        "StateMachineFireTrigger::copyViewModelPathIds",
        "StateMachineListenerSingle::copyViewModelPathIds",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "C++ encoded property copier surface changed; audit clone/copy paths that preserve decoded buffers before refreshing this list"
    );
}

#[test]
fn cpp_data_converter_output_type_overrides_are_tracked_by_binary_import_model() {
    let include_dir = reference_runtime_dir().join("include/rive");
    let converters_dir = include_dir.join("data_bind/converters");
    let scripted_converter = include_dir.join("scripted/scripted_data_converter.hpp");
    if !converters_dir.exists() || !scripted_converter.exists() {
        eprintln!(
            "skipping C++ data-converter outputType audit; reference headers not found under {}; set RIVE_RUNTIME_DIR",
            include_dir.display()
        );
        return;
    }

    let mut headers = Vec::new();
    collect_hpp_paths(&converters_dir, &mut headers);
    headers.push(scripted_converter);

    let mut actual = BTreeMap::new();
    for path in headers {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&contents);
        if !compact.contains("DataTypeoutputType()override") {
            continue;
        }
        let class_name = cpp_header_data_converter_class_name(&contents)
            .unwrap_or_else(|| panic!("missing converter class name in {}", path.display()));
        let output_type = if class_name == "DataConverterGroup" {
            "dynamic".to_owned()
        } else {
            cpp_inline_output_type_return(&compact)
                .unwrap_or_else(|| panic!("missing inline outputType return in {}", path.display()))
        };
        actual.insert(class_name, output_type);
    }

    let expected = [
        ("DataConverterBooleanNegate", "boolean"),
        ("DataConverterFormula", "number"),
        ("DataConverterGroup", "dynamic"),
        ("DataConverterInterpolator", "input"),
        ("DataConverterListToLength", "number"),
        ("DataConverterNumberToList", "list"),
        ("DataConverterOperation", "number"),
        ("DataConverterRangeMapper", "number"),
        ("DataConverterRounder", "number"),
        ("DataConverterStringPad", "string"),
        ("DataConverterStringRemoveZeros", "string"),
        ("DataConverterStringTrim", "string"),
        ("DataConverterToNumber", "number"),
        ("DataConverterToString", "string"),
        ("DataConverterTrigger", "trigger"),
        ("ScriptedDataConverter", "any"),
    ]
    .into_iter()
    .map(|(class_name, output_type)| (class_name.to_owned(), output_type.to_owned()))
    .collect::<BTreeMap<_, _>>();

    assert_eq!(
        actual, expected,
        "C++ DataConverter::outputType override surface changed; audit RuntimeDataType mapping, group output traversal, and C++ probe coverage before refreshing this list"
    );
}

#[test]
fn cpp_data_converter_behavior_overrides_are_tracked_by_binary_import_model() {
    let include_dir = reference_runtime_dir().join("include/rive");
    let converters_dir = include_dir.join("data_bind/converters");
    let scripted_converter = include_dir.join("scripted/scripted_data_converter.hpp");
    if !converters_dir.exists() || !scripted_converter.exists() {
        eprintln!(
            "skipping C++ data-converter behavior audit; reference headers not found under {}; set RIVE_RUNTIME_DIR",
            include_dir.display()
        );
        return;
    }

    let mut headers = Vec::new();
    collect_hpp_paths(&converters_dir, &mut headers);
    headers.push(scripted_converter);

    let mut actual = BTreeMap::<String, BTreeSet<String>>::new();
    for path in headers {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let Some(class_name) = cpp_header_data_converter_class_name(&contents) else {
            continue;
        };
        let compact = compact_cpp_source(&contents);
        let mut methods = BTreeSet::new();
        if compact.contains("DataValue*convert(DataValue*value,DataBind*dataBind)override") {
            methods.insert("convert".to_owned());
        }
        if compact.contains("DataValue*reverseConvert(DataValue*value,DataBind*dataBind)override") {
            methods.insert("reverseConvert".to_owned());
        }
        if compact.contains("booladvance(floatelapsedTime)override")
            || compact.contains("booladvance(floatelapsedSeconds)override")
        {
            methods.insert("advance".to_owned());
        }
        if !methods.is_empty() {
            actual.insert(class_name, methods);
        }
    }

    let expected = [
        (
            "DataConverterBooleanNegate",
            &["convert", "reverseConvert"][..],
        ),
        ("DataConverterFormula", &["convert", "reverseConvert"]),
        (
            "DataConverterGroup",
            &["advance", "convert", "reverseConvert"],
        ),
        (
            "DataConverterInterpolator",
            &["advance", "convert", "reverseConvert"],
        ),
        ("DataConverterListToLength", &["convert"]),
        ("DataConverterNumberToList", &["convert"]),
        (
            "DataConverterOperationValue",
            &["convert", "reverseConvert"],
        ),
        (
            "DataConverterOperationViewModel",
            &["convert", "reverseConvert"],
        ),
        ("DataConverterRangeMapper", &["convert", "reverseConvert"]),
        ("DataConverterRounder", &["convert"]),
        ("DataConverterStringPad", &["convert"]),
        ("DataConverterStringRemoveZeros", &["convert"]),
        ("DataConverterStringTrim", &["convert"]),
        (
            "DataConverterSystemDegsToRads",
            &["convert", "reverseConvert"],
        ),
        (
            "DataConverterSystemNormalizer",
            &["convert", "reverseConvert"],
        ),
        ("DataConverterToNumber", &["convert"]),
        ("DataConverterToString", &["convert"]),
        ("DataConverterTrigger", &["convert"]),
        (
            "ScriptedDataConverter",
            &["advance", "convert", "reverseConvert"],
        ),
    ]
    .into_iter()
    .map(|(class_name, methods)| {
        (
            class_name.to_owned(),
            methods
                .iter()
                .map(|method| (*method).to_owned())
                .collect::<BTreeSet<_>>(),
        )
    })
    .collect::<BTreeMap<_, _>>();

    assert_eq!(
        actual, expected,
        "C++ DataConverter behavior override surface changed; audit conversion/reverse/stateful Rust behavior, C++ probe samples, and docs before refreshing this list"
    );
}

#[test]
fn cpp_base_data_converter_methods_are_tracked_by_binary_import_model() {
    let include_path =
        reference_runtime_dir().join("include/rive/data_bind/converters/data_converter.hpp");
    let source_path = reference_runtime_dir().join("src/data_bind/converters/data_converter.cpp");
    if !include_path.exists() || !source_path.exists() {
        eprintln!(
            "skipping C++ base data-converter audit; reference files not found at {} and {}; set RIVE_RUNTIME_DIR",
            include_path.display(),
            source_path.display()
        );
        return;
    }

    let header = std::fs::read_to_string(&include_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", include_path.display()));
    let compact_header = compact_cpp_source(&header);
    assert!(
        compact_header
            .contains("virtualDataValue*convert(DataValue*value,DataBind*dataBind){returnvalue;};"),
        "DataConverter::convert base implementation changed; audit Rust base pass-through dispatch"
    );
    assert!(
        compact_header.contains(
            "virtualDataValue*reverseConvert(DataValue*value,DataBind*dataBind){returnvalue;};"
        ),
        "DataConverter::reverseConvert base implementation changed; audit Rust reverse pass-through dispatch"
    );
    assert!(
        compact_header.contains("virtualDataTypeoutputType(){returnDataType::none;};"),
        "DataConverter::outputType base implementation changed; audit RuntimeDataType::None mapping"
    );

    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let compact_source = compact_cpp_source(&source);
    let advance =
        compact_cpp_function_body(&compact_source, "DataConverter::advance(floatelapsedTime)")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataConverter::advance in {}",
                    source_path.display()
                )
            });
    assert_eq!(
        advance, "returnfalse;",
        "DataConverter::advance base implementation changed; audit stateful converter advance fallback"
    );
}

#[test]
fn cpp_data_converter_bind_context_methods_are_tracked_by_binary_import_model() {
    let base_header_path =
        reference_runtime_dir().join("include/rive/data_bind/converters/data_converter.hpp");
    let base_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter.cpp");
    let group_header_path =
        reference_runtime_dir().join("include/rive/data_bind/converters/data_converter_group.hpp");
    let group_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_group.cpp");
    let operation_view_model_header_path = reference_runtime_dir()
        .join("include/rive/data_bind/converters/data_converter_operation_viewmodel.hpp");
    let operation_view_model_source_path = reference_runtime_dir()
        .join("src/data_bind/converters/data_converter_operation_viewmodel.cpp");
    let formula_header_path = reference_runtime_dir()
        .join("include/rive/data_bind/converters/data_converter_formula.hpp");
    let formula_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_formula.cpp");

    let required_paths = [
        &base_header_path,
        &base_source_path,
        &group_header_path,
        &group_source_path,
        &operation_view_model_header_path,
        &operation_view_model_source_path,
        &formula_header_path,
        &formula_source_path,
    ];
    if required_paths.iter().any(|path| !path.exists()) {
        eprintln!(
            "skipping C++ data-converter bind-context audit; reference files not found under {}; set RIVE_RUNTIME_DIR",
            reference_runtime_dir().display()
        );
        return;
    }

    let base_header = std::fs::read_to_string(&base_header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", base_header_path.display()));
    let compact_base_header = compact_cpp_source(&base_header);
    assert_compact_contains_in_order(
        &compact_base_header,
        &[
            "virtualvoidbindFromContext(DataContext*dataContext,DataBind*dataBind);",
            "virtualvoidunbind();",
            "virtualvoidupdate();",
            "virtualbooladvance(floatelapsedTime);",
            "virtualvoidreset(){};",
            "DataBind*m_parentDataBind;",
        ],
        "DataConverter live binding surface changed; audit RuntimeFile::data_converter_bind_context_effect",
    );

    let group_header = std::fs::read_to_string(&group_header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", group_header_path.display()));
    let operation_view_model_header = std::fs::read_to_string(&operation_view_model_header_path)
        .unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                operation_view_model_header_path.display()
            )
        });
    let formula_header = std::fs::read_to_string(&formula_header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", formula_header_path.display()));

    assert!(
        compact_cpp_source(&group_header)
            .contains("voidbindFromContext(DataContext*dataContext,DataBind*dataBind)override;"),
        "DataConverterGroup bindFromContext override changed; audit RuntimeFile::data_converter_bind_context_effect"
    );
    assert!(
        compact_cpp_source(&operation_view_model_header)
            .contains("voidbindFromContext(DataContext*dataContext,DataBind*dataBind)override;"),
        "DataConverterOperationViewModel bindFromContext override changed; audit RuntimeFile::data_converter_bind_context_effect"
    );
    assert!(
        compact_cpp_source(&formula_header)
            .contains("voidbindFromContext(DataContext*dataContext,DataBind*dataBind)override;"),
        "DataConverterFormula bindFromContext override changed; audit RuntimeFile::data_converter_bind_context_effect"
    );

    let base_source = std::fs::read_to_string(&base_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", base_source_path.display()));
    let compact_base_source = compact_cpp_source(&base_source);
    let base_bind = compact_cpp_function_body(
        &compact_base_source,
        "DataConverter::bindFromContext(DataContext*dataContext,DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverter::bindFromContext in {}",
            base_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &base_bind,
        &[
            "m_parentDataBind=dataBind;",
            "bindDataBindsFromContext(dataContext);",
        ],
        "DataConverter::bindFromContext changed; audit RuntimeFile::data_converter_bind_context_effect",
    );

    let group_source = std::fs::read_to_string(&group_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", group_source_path.display()));
    let compact_group_source = compact_cpp_source(&group_source);
    let group_bind = compact_cpp_function_body(
        &compact_group_source,
        "DataConverterGroup::bindFromContext(DataContext*dataContext,DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterGroup::bindFromContext in {}",
            group_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &group_bind,
        &[
            "DataConverter::bindFromContext(dataContext,dataBind);",
            "for(auto&item:m_items){",
            "autoconverter=item->converter();",
            "if(converter!=nullptr){",
            "converter->bindFromContext(dataContext,dataBind);",
        ],
        "DataConverterGroup::bindFromContext changed; audit RuntimeFile::data_converter_bind_context_effect",
    );

    let operation_view_model_source = std::fs::read_to_string(&operation_view_model_source_path)
        .unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                operation_view_model_source_path.display()
            )
        });
    let compact_operation_view_model_source = compact_cpp_source(&operation_view_model_source);
    let operation_view_model_bind = compact_cpp_function_body(
        &compact_operation_view_model_source,
        "DataConverterOperationViewModel::bindFromContext(DataContext*dataContext,DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterOperationViewModel::bindFromContext in {}",
            operation_view_model_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &operation_view_model_bind,
        &[
            "DataConverter::bindFromContext(dataContext,dataBind);",
            "autopropertyValue=dataContext->getViewModelProperty(m_SourcePathIdsBuffer);",
            "if(propertyValue!=nullptr&&propertyValue->is<ViewModelInstanceNumber>()){",
            "m_source=propertyValue->as<ViewModelInstanceNumber>();",
            "m_source->addDependent(dataBind);",
        ],
        "DataConverterOperationViewModel::bindFromContext changed; audit RuntimeFile::data_converter_bind_context_effect",
    );

    let formula_source = std::fs::read_to_string(&formula_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", formula_source_path.display()));
    let compact_formula_source = compact_cpp_source(&formula_source);
    let formula_bind = compact_cpp_function_body(
        &compact_formula_source,
        "DataConverterFormula::bindFromContext(DataContext*dataContext,DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterFormula::bindFromContext in {}",
            formula_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &formula_bind,
        &[
            "DataConverter::bindFromContext(dataContext,dataBind);",
            "if(dataBind&&dataBind->source()){",
            "m_source=ref_rcp<ViewModelInstanceValue>(dataBind->source());",
            "m_source->addDependent(this);",
        ],
        "DataConverterFormula::bindFromContext changed; audit RuntimeFile::data_converter_bind_context_effect",
    );
}

#[test]
fn cpp_data_converter_lifecycle_methods_are_tracked_by_binary_import_model() {
    let base_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter.cpp");
    let group_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_group.cpp");
    let formula_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_formula.cpp");
    let interpolator_header_path = reference_runtime_dir()
        .join("include/rive/data_bind/converters/data_converter_interpolator.hpp");
    let interpolator_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_interpolator.cpp");
    let range_mapper_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_range_mapper.cpp");
    let string_pad_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_string_pad.cpp");
    let string_trim_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_string_trim.cpp");
    let to_string_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_to_string.cpp");
    let number_to_list_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_number_to_list.cpp");
    let operation_value_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_operation_value.cpp");
    let rounder_source_path =
        reference_runtime_dir().join("src/data_bind/converters/data_converter_rounder.cpp");
    let random_mode_path = reference_runtime_dir().join("include/rive/random_mode.hpp");

    let required_paths = [
        &base_source_path,
        &group_source_path,
        &formula_source_path,
        &interpolator_header_path,
        &interpolator_source_path,
        &range_mapper_source_path,
        &string_pad_source_path,
        &string_trim_source_path,
        &to_string_source_path,
        &number_to_list_source_path,
        &operation_value_source_path,
        &rounder_source_path,
        &random_mode_path,
    ];
    if required_paths.iter().any(|path| !path.exists()) {
        eprintln!(
            "skipping C++ data-converter lifecycle audit; reference files not found under {}; set RIVE_RUNTIME_DIR",
            reference_runtime_dir().display()
        );
        return;
    }

    let base_source = std::fs::read_to_string(&base_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", base_source_path.display()));
    let compact_base_source = compact_cpp_source(&base_source);
    let base_unbind = compact_cpp_function_body(&compact_base_source, "DataConverter::unbind()")
        .unwrap_or_else(|| {
            panic!(
                "missing DataConverter::unbind in {}",
                base_source_path.display()
            )
        });
    assert_eq!(
        base_unbind, "unbindDataBinds();",
        "DataConverter::unbind changed; audit RuntimeFile::data_converter_unbind_effect"
    );

    let mark_dirty =
        compact_cpp_function_body(&compact_base_source, "DataConverter::markConverterDirty()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataConverter::markConverterDirty in {}",
                    base_source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &mark_dirty,
        &[
            "if(m_parentDataBind!=nullptr){",
            "m_parentDataBind->addDirt(ComponentDirt::Dependents|(m_parentDataBind->targetOrigin()?ComponentDirt::BindingsTarget:ComponentDirt::Bindings),false);",
        ],
        "DataConverter::markConverterDirty changed; audit RuntimeFile::data_converter_mark_dirty_effect",
    );

    let add_dirty = compact_cpp_function_body(
        &compact_base_source,
        "DataConverter::addDirtyDataBind(DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverter::addDirtyDataBind in {}",
            base_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &add_dirty,
        &[
            "markConverterDirty();",
            "DataBindContainer::addDirtyDataBind(dataBind);",
        ],
        "DataConverter::addDirtyDataBind changed; audit RuntimeFile::data_converter_add_dirty_data_bind_effect",
    );

    let base_update = compact_cpp_function_body(&compact_base_source, "DataConverter::update()")
        .unwrap_or_else(|| {
            panic!(
                "missing DataConverter::update in {}",
                base_source_path.display()
            )
        });
    assert_eq!(
        base_update, "updateDataBinds(false);",
        "DataConverter::update changed; audit RuntimeFile::data_converter_update_effect"
    );

    let group_source = std::fs::read_to_string(&group_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", group_source_path.display()));
    let compact_group_source = compact_cpp_source(&group_source);
    let group_unbind =
        compact_cpp_function_body(&compact_group_source, "DataConverterGroup::unbind()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataConverterGroup::unbind in {}",
                    group_source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &group_unbind,
        &[
            "for(auto&item:m_items){",
            "autoconverter=item->converter();",
            "if(converter!=nullptr){",
            "converter->unbind();",
        ],
        "DataConverterGroup::unbind changed; audit RuntimeFile::data_converter_unbind_effect",
    );
    assert!(
        !group_unbind.contains("DataConverter::unbind"),
        "DataConverterGroup::unbind now calls the base unbind; audit owned data-bind cleanup modeling"
    );

    let group_update =
        compact_cpp_function_body(&compact_group_source, "DataConverterGroup::update()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataConverterGroup::update in {}",
                    group_source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &group_update,
        &[
            "for(auto&item:m_items){",
            "autoconverter=item->converter();",
            "if(converter!=nullptr){",
            "converter->update();",
        ],
        "DataConverterGroup::update changed; audit RuntimeFile::data_converter_update_effect",
    );
    assert!(
        !group_update.contains("DataConverter::update"),
        "DataConverterGroup::update now calls the base update; audit owned data-bind update modeling"
    );

    let group_reset =
        compact_cpp_function_body(&compact_group_source, "DataConverterGroup::reset()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataConverterGroup::reset in {}",
                    group_source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &group_reset,
        &[
            "for(auto&item:m_items){",
            "autoconverter=item->converter();",
            "if(converter!=nullptr){",
            "converter->reset();",
        ],
        "DataConverterGroup::reset changed; audit RuntimeFile::data_converter_reset_effect",
    );

    let interpolator_header =
        std::fs::read_to_string(&interpolator_header_path).unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                interpolator_header_path.display()
            )
        });
    let compact_interpolator_header = compact_cpp_source(&interpolator_header);
    assert!(
        compact_interpolator_header.contains("voidreset()override;"),
        "DataConverterInterpolator reset override changed; audit RuntimeFile::data_converter_reset_effect"
    );
    assert!(
        compact_interpolator_header
            .contains("voidreset(){dispose();m_isSmoothingAnimation=false;m_initialized=false;}"),
        "InterpolatorAdvancer::reset changed; audit RuntimeFile::data_converter_reset_effect"
    );

    let interpolator_source =
        std::fs::read_to_string(&interpolator_source_path).unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                interpolator_source_path.display()
            )
        });
    let compact_interpolator_source = compact_cpp_source(&interpolator_source);
    let interpolator_reset = compact_cpp_function_body(
        &compact_interpolator_source,
        "DataConverterInterpolator::reset()",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterInterpolator::reset in {}",
            interpolator_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &interpolator_reset,
        &["m_advanceCount=0;", "m_advancer.reset();"],
        "DataConverterInterpolator::reset changed; audit RuntimeFile::data_converter_reset_effect",
    );

    let interpolator_duration_changed = compact_cpp_function_body(
        &compact_interpolator_source,
        "DataConverterInterpolator::durationChanged()",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterInterpolator::durationChanged in {}",
            interpolator_source_path.display()
        )
    });
    assert_eq!(
        interpolator_duration_changed, "markConverterDirty();",
        "DataConverterInterpolator::durationChanged changed; audit RuntimeFile::data_converter_property_change_effect"
    );

    let range_mapper_source =
        std::fs::read_to_string(&range_mapper_source_path).unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                range_mapper_source_path.display()
            )
        });
    let compact_range_mapper_source = compact_cpp_source(&range_mapper_source);
    for callback in [
        "DataConverterRangeMapper::minInputChanged()",
        "DataConverterRangeMapper::maxInputChanged()",
        "DataConverterRangeMapper::minOutputChanged()",
        "DataConverterRangeMapper::maxOutputChanged()",
    ] {
        let body = compact_cpp_function_body(&compact_range_mapper_source, callback)
            .unwrap_or_else(|| {
                panic!(
                    "missing {callback} in {}",
                    range_mapper_source_path.display()
                )
            });
        assert_eq!(
            body, "markConverterDirty();",
            "{callback} changed; audit RuntimeFile::data_converter_property_change_effect"
        );
    }
    assert!(
        !compact_range_mapper_source.contains("DataConverterRangeMapper::flagsChanged"),
        "DataConverterRangeMapper::flagsChanged was added; audit RuntimeFile::data_converter_property_change_effect"
    );

    let string_pad_source = std::fs::read_to_string(&string_pad_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", string_pad_source_path.display()));
    let compact_string_pad_source = compact_cpp_source(&string_pad_source);
    for callback in [
        "DataConverterStringPad::lengthChanged()",
        "DataConverterStringPad::padTypeChanged()",
        "DataConverterStringPad::textChanged()",
    ] {
        let body =
            compact_cpp_function_body(&compact_string_pad_source, callback).unwrap_or_else(|| {
                panic!("missing {callback} in {}", string_pad_source_path.display())
            });
        assert_eq!(
            body, "markConverterDirty();",
            "{callback} changed; audit RuntimeFile::data_converter_property_change_effect"
        );
    }

    let string_trim_source =
        std::fs::read_to_string(&string_trim_source_path).unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                string_trim_source_path.display()
            )
        });
    let compact_string_trim_source = compact_cpp_source(&string_trim_source);
    let trim_type_changed = compact_cpp_function_body(
        &compact_string_trim_source,
        "DataConverterStringTrim::trimTypeChanged()",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterStringTrim::trimTypeChanged in {}",
            string_trim_source_path.display()
        )
    });
    assert_eq!(
        trim_type_changed, "markConverterDirty();",
        "DataConverterStringTrim::trimTypeChanged changed; audit RuntimeFile::data_converter_property_change_effect"
    );

    let to_string_source = std::fs::read_to_string(&to_string_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", to_string_source_path.display()));
    let compact_to_string_source = compact_cpp_source(&to_string_source);
    for callback in [
        "DataConverterToString::decimalsChanged()",
        "DataConverterToString::colorFormatChanged()",
    ] {
        let body = compact_cpp_function_body(&compact_to_string_source, callback)
            .unwrap_or_else(|| panic!("missing {callback} in {}", to_string_source_path.display()));
        assert_eq!(
            body, "markConverterDirty();",
            "{callback} changed; audit RuntimeFile::data_converter_property_change_effect"
        );
    }

    let number_to_list_source = std::fs::read_to_string(&number_to_list_source_path)
        .unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                number_to_list_source_path.display()
            )
        });
    let compact_number_to_list_source = compact_cpp_source(&number_to_list_source);
    let view_model_id_changed = compact_cpp_function_body(
        &compact_number_to_list_source,
        "DataConverterNumberToList::viewModelIdChanged()",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterNumberToList::viewModelIdChanged in {}",
            number_to_list_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &view_model_id_changed,
        &["clearItems();", "markConverterDirty();"],
        "DataConverterNumberToList::viewModelIdChanged changed; audit RuntimeFile::data_converter_property_change_effect",
    );

    let operation_value_source = std::fs::read_to_string(&operation_value_source_path)
        .unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                operation_value_source_path.display()
            )
        });
    let compact_operation_value_source = compact_cpp_source(&operation_value_source);
    let operation_value_changed = compact_cpp_function_body(
        &compact_operation_value_source,
        "DataConverterOperationValue::operationValueChanged()",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterOperationValue::operationValueChanged in {}",
            operation_value_source_path.display()
        )
    });
    assert_eq!(
        operation_value_changed, "markConverterDirty();",
        "DataConverterOperationValue::operationValueChanged changed; audit RuntimeFile::data_converter_property_change_effect"
    );

    let rounder_source = std::fs::read_to_string(&rounder_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", rounder_source_path.display()));
    let compact_rounder_source = compact_cpp_source(&rounder_source);
    assert!(
        !compact_rounder_source.contains("DataConverterRounder::decimalsChanged"),
        "DataConverterRounder::decimalsChanged was added; audit RuntimeFile::data_converter_property_change_effect"
    );

    let formula_source = std::fs::read_to_string(&formula_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", formula_source_path.display()));
    let compact_formula_source = compact_cpp_source(&formula_source);
    let formula_add_dirt = compact_cpp_function_body(
        &compact_formula_source,
        "DataConverterFormula::addDirt(ComponentDirtvalue,boolrecurse)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataConverterFormula::addDirt in {}",
            formula_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &formula_add_dirt,
        &[
            "if(randomModeValue()==static_cast<uint32_t>(RandomMode::sourceChange)){",
            "m_randoms.clear();",
        ],
        "DataConverterFormula::addDirt changed; audit RuntimeFile::data_converter_formula_add_dirt_effect",
    );

    let formula_unbind =
        compact_cpp_function_body(&compact_formula_source, "DataConverterFormula::unbind()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataConverterFormula::unbind in {}",
                    formula_source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &formula_unbind,
        &[
            "if(m_source){",
            "m_source->removeDependent(this);",
            "m_source=nullptr;",
        ],
        "DataConverterFormula::unbind changed; audit RuntimeFile::data_converter_unbind_effect",
    );
    assert!(
        !formula_unbind.contains("DataConverter::unbind"),
        "DataConverterFormula::unbind now calls the base unbind; audit owned data-bind cleanup modeling"
    );

    let random_mode = std::fs::read_to_string(&random_mode_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", random_mode_path.display()));
    assert!(
        compact_cpp_source(&random_mode).contains("sourceChange=2,"),
        "RandomMode::sourceChange discriminant changed; audit RuntimeFile::data_converter_formula_add_dirt_effect"
    );
}

#[test]
fn cpp_data_bind_context_binding_methods_are_tracked_by_binary_import_model() {
    let header_path = reference_runtime_dir().join("include/rive/data_bind/data_bind_context.hpp");
    let source_path = reference_runtime_dir().join("src/data_bind/data_bind_context.cpp");
    if !header_path.exists() || !source_path.exists() {
        eprintln!(
            "skipping C++ data-bind-context binding audit; reference files not found at {} and {}; set RIVE_RUNTIME_DIR",
            header_path.display(),
            source_path.display()
        );
        return;
    }

    let header = std::fs::read_to_string(&header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header_path.display()));
    let compact_header = compact_cpp_source(&header);
    assert_compact_contains_in_order(
        &compact_header,
        &[
            "voidbindFromContext(DataContext*dataContext);",
            "voidresolvePath();",
            "boolm_isPathResolved=false;",
        ],
        "DataBindContext live binding state changed; audit RuntimeFile::data_bind_context_bind_effect",
    );

    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let compact_source = compact_cpp_source(&source);

    let resolve_path = compact_cpp_function_body(&compact_source, "DataBindContext::resolvePath()")
        .unwrap_or_else(|| {
            panic!(
                "missing DataBindContext::resolvePath in {}",
                source_path.display()
            )
        });
    assert_compact_contains_in_order(
        &resolve_path,
        &[
            "if(!isNameBased()||m_isPathResolved){",
            "return;",
            "m_isPathResolved=true;",
            "if(m_SourcePathIdsBuffer.size()>0){",
            "autopathId=m_SourcePathIdsBuffer[0];",
            "if(m_file){",
            "autodataResolver=m_file->dataResolver();",
            "if(dataResolver){",
            "autoresolvedPath=dataResolver->resolvePath(pathId);",
            "if(!resolvedPath.empty()){",
            "m_SourcePathIdsBuffer=resolvedPath;",
        ],
        "DataBindContext::resolvePath changed; audit RuntimeFile::data_bind_context_bind_effect and resolved source-path helpers",
    );

    let bind_from_context = compact_cpp_function_body(
        &compact_source,
        "DataBindContext::bindFromContext(DataContext*dataContext)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContext::bindFromContext in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &bind_from_context,
        &[
            "if(dataContext!=nullptr){",
            "resolvePath();",
            "autovmSource=isNameBased()&&file()?dataContext->getRelativeViewModelProperty(m_SourcePathIdsBuffer,file()->dataResolver()):dataContext->getViewModelProperty(m_SourcePathIdsBuffer);",
            "if(m_Source==nullptr||vmSource!=m_Source.get()){",
            "if(vmSource!=nullptr){",
            "clearSource();",
            "source(ref_rcp(vmSource));",
            "bind();",
            "else{",
            "unbind();",
            "else{",
            "addDirt(reconcileDirt(),true);",
            "if(m_dataConverter!=nullptr){",
            "m_dataConverter->bindFromContext(dataContext,this);",
        ],
        "DataBindContext::bindFromContext changed; audit RuntimeFile::data_bind_context_bind_effect",
    );
}

#[test]
fn cpp_data_bind_flag_helpers_are_tracked_by_binary_import_model() {
    let include_path = reference_runtime_dir().join("include/rive/data_bind_flags.hpp");
    let component_dirt_path = reference_runtime_dir().join("include/rive/component_dirt.hpp");
    let data_bind_header_path =
        reference_runtime_dir().join("include/rive/data_bind/data_bind.hpp");
    let source_path = reference_runtime_dir().join("src/data_bind/data_bind.cpp");
    let core_source_path = reference_runtime_dir().join("src/core.cpp");
    let component_source_path = reference_runtime_dir().join("src/component.cpp");
    let artboard_source_path = reference_runtime_dir().join("src/artboard.cpp");
    let state_machine_instance_source_path =
        reference_runtime_dir().join("src/animation/state_machine_instance.cpp");
    if !include_path.exists()
        || !component_dirt_path.exists()
        || !data_bind_header_path.exists()
        || !source_path.exists()
        || !core_source_path.exists()
        || !component_source_path.exists()
        || !artboard_source_path.exists()
        || !state_machine_instance_source_path.exists()
    {
        eprintln!(
            "skipping C++ data-bind flag audit; reference files not found at {}, {}, {}, {}, {}, {}, {}, and {}; set RIVE_RUNTIME_DIR",
            include_path.display(),
            component_dirt_path.display(),
            data_bind_header_path.display(),
            source_path.display(),
            core_source_path.display(),
            component_source_path.display(),
            artboard_source_path.display(),
            state_machine_instance_source_path.display()
        );
        return;
    }

    let header = std::fs::read_to_string(&include_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", include_path.display()));
    let compact_header = compact_cpp_source(&header);
    assert_compact_contains_in_order(
        &compact_header,
        &[
            "Direction=1<<0,",
            "TwoWay=1<<1,",
            "Once=1<<2,",
            "SourceToTargetRunsFirst=1<<3,",
            "NameBased=1<<4,",
            "ToTarget=0,",
            "ToSource=1<<0,",
        ],
        "DataBindFlags bit layout changed; audit Rust data-bind flag helpers and update-queue classification",
    );

    let component_dirt = std::fs::read_to_string(&component_dirt_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", component_dirt_path.display()));
    let compact_component_dirt = compact_cpp_source(&component_dirt);
    assert_compact_contains_in_order(
        &compact_component_dirt,
        &[
            "enumclassComponentDirt:unsignedshort{",
            "None=0,",
            "Collapsed=1<<0,",
            "Dependents=1<<1,",
            "Components=1<<2,",
            "DrawOrder=1<<3,",
            "Path=1<<4,",
            "TextShape=1<<4,",
            "Skin=1<<4,",
            "Vertices=1<<5,",
            "TextCoverage=1<<5,",
            "Transform=1<<6,",
            "WorldTransform=1<<7,",
            "RenderOpacity=1<<8,",
            "Paint=1<<9,",
            "Stops=1<<10,",
            "LayoutStyle=1<<11,",
            "Bindings=1<<12,",
            "NSlicer=1<<13,",
            "BindingsTarget=1<<13,",
            "ScriptUpdate=1<<14,",
            "Clipping=1<<15,",
            "Filthy=0xFFFE",
        ],
        "ComponentDirt bit layout changed; audit RuntimeComponentDirt and data-bind dirt helpers",
    );

    let data_bind_header = std::fs::read_to_string(&data_bind_header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", data_bind_header_path.display()));
    let compact_data_bind_header = compact_cpp_source(&data_bind_header);
    assert_compact_contains_in_order(
        &compact_data_bind_header,
        &[
            "enumclassFlag:uint8_t{",
            "Collapsed=1<<0,",
            "InDirtyList=1<<1,",
            "InPersistingList=1<<2,",
            "SuppressDirt=1<<3,",
            "Observing=1<<4,",
            "TargetOrigin=1<<5,",
        ],
        "DataBind internal flag layout changed; audit Rust data-bind runtime-state helpers",
    );

    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let compact_source = compact_cpp_source(&source);

    let is_main_to_source =
        compact_cpp_function_body(&compact_source, "DataBind::isMainToSource()").unwrap_or_else(
            || {
                panic!(
                    "missing DataBind::isMainToSource in {}",
                    source_path.display()
                )
            },
        );
    assert_eq!(
        is_main_to_source,
        "autoflagsValue=static_cast<DataBindFlags>(flags());return(flagsValue&DataBindFlags::Direction)==DataBindFlags::ToSource;",
        "DataBind::isMainToSource changed; audit RuntimeFile::data_bind_is_main_to_source"
    );

    let source_to_target_runs_first =
        compact_cpp_function_body(&compact_source, "DataBind::sourceToTargetRunsFirst()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataBind::sourceToTargetRunsFirst in {}",
                    source_path.display()
                )
            });
    assert_eq!(
        source_to_target_runs_first,
        "autoflagsValue=static_cast<DataBindFlags>(flags());return(flagsValue&DataBindFlags::SourceToTargetRunsFirst)==DataBindFlags::SourceToTargetRunsFirst;",
        "DataBind::sourceToTargetRunsFirst changed; audit RuntimeFile::data_bind_source_to_target_runs_first"
    );

    let binds_once = compact_cpp_function_body(&compact_source, "DataBind::bindsOnce()")
        .unwrap_or_else(|| panic!("missing DataBind::bindsOnce in {}", source_path.display()));
    assert_eq!(
        binds_once,
        "autoflagsValue=static_cast<DataBindFlags>(flags());returnenums::is_flag_set(flagsValue,DataBindFlags::Once);",
        "DataBind::bindsOnce changed; audit RuntimeFile::data_bind_binds_once"
    );

    let to_source = compact_cpp_function_body(&compact_source, "DataBind::toSource()")
        .unwrap_or_else(|| panic!("missing DataBind::toSource in {}", source_path.display()));
    assert_eq!(
        to_source,
        "autoflagsValue=static_cast<DataBindFlags>(flags());returnenums::any_flag_set(flagsValue,DataBindFlags::TwoWay|DataBindFlags::ToSource);",
        "DataBind::toSource changed; audit RuntimeFile::data_bind_to_source and update-queue classification"
    );

    let to_target = compact_cpp_function_body(&compact_source, "DataBind::toTarget()")
        .unwrap_or_else(|| panic!("missing DataBind::toTarget in {}", source_path.display()));
    assert_eq!(
        to_target,
        "autoflagsValue=static_cast<DataBindFlags>(flags());returnenums::is_flag_set(flagsValue,DataBindFlags::TwoWay)||!enums::is_flag_set(flagsValue,DataBindFlags::ToSource);",
        "DataBind::toTarget changed; audit RuntimeFile::data_bind_to_target and update-queue classification"
    );

    let is_name_based = compact_cpp_function_body(&compact_source, "DataBind::isNameBased()")
        .unwrap_or_else(|| panic!("missing DataBind::isNameBased in {}", source_path.display()));
    assert_eq!(
        is_name_based,
        "autoflagsValue=static_cast<DataBindFlags>(flags());returnenums::is_flag_set(flagsValue,DataBindFlags::NameBased);",
        "DataBind::isNameBased changed; audit RuntimeFile::data_bind_is_name_based"
    );

    let can_skip = compact_cpp_function_body(&compact_source, "DataBind::canSkip()")
        .unwrap_or_else(|| panic!("missing DataBind::canSkip in {}", source_path.display()));
    assert_eq!(
        can_skip,
        "returnm_target&&m_target->is<Component>()&&m_target->as<Component>()->isCollapsed()&&propertyKey()!=LayoutComponentStyleBase::displayValuePropertyKey;",
        "DataBind::canSkip changed; audit RuntimeFile::data_bind_can_skip"
    );

    let advance = compact_cpp_function_body(&compact_source, "DataBind::advance(floatelapsedTime)")
        .unwrap_or_else(|| panic!("missing DataBind::advance in {}", source_path.display()));
    assert_eq!(
        advance,
        "if(converter()&&m_Source&&!hasFlag(Flag::Collapsed)){returnconverter()->advance(elapsedTime);}returnfalse;",
        "DataBind::advance changed; audit RuntimeFile::data_bind_stateful_advance"
    );

    let collapse =
        compact_cpp_function_body(&compact_source, "DataBind::collapse(boolisCollapsed)")
            .unwrap_or_else(|| panic!("missing DataBind::collapse in {}", source_path.display()));
    assert_eq!(
        collapse,
        "if(hasFlag(Flag::Collapsed)==isCollapsed||propertyKey()==LayoutComponentStyleBase::displayValuePropertyKey||!targetSupportsPush()){return;}setFlag(Flag::Collapsed,isCollapsed);if(!isCollapsed&&m_Dirt!=ComponentDirt::None&&m_container){m_container->addDirtyDataBind(this);}",
        "DataBind::collapse changed; audit RuntimeFile::data_bind_collapse_effect"
    );

    let update = compact_cpp_function_body(&compact_source, "DataBind::update(ComponentDirtvalue)")
        .unwrap_or_else(|| panic!("missing DataBind::update in {}", source_path.display()));
    assert_compact_contains_in_order(
        &update,
        &[
            "if(m_Source!=nullptr&&m_ContextValue!=nullptr){",
            "if((value&ComponentDirt::Bindings)==ComponentDirt::Bindings){",
            "autoflagsValue=static_cast<DataBindFlags>(flags());",
            "if(toTarget()){",
            "suppressDirt(true);",
            "m_ContextValue->apply(m_target,propertyKey(),(flagsValue&DataBindFlags::Direction)==DataBindFlags::ToTarget,this);",
            "suppressDirt(false);",
        ],
        "DataBind::update changed; audit RuntimeFile::data_bind_update_effect",
    );

    let update_dependents =
        compact_cpp_function_body(&compact_source, "DataBind::updateDependents()").unwrap_or_else(
            || {
                panic!(
                    "missing DataBind::updateDependents in {}",
                    source_path.display()
                )
            },
        );
    assert_eq!(
        update_dependents, "if(m_dataConverter){m_dataConverter->update();}",
        "DataBind::updateDependents changed; audit RuntimeFile::data_bind_update_effect"
    );

    let update_source_binding = compact_cpp_function_body(
        &compact_source,
        "DataBind::updateSourceBinding(boolinvalidate)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBind::updateSourceBinding in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &update_source_binding,
        &[
            "if(toSource()&&m_target&&m_ContextValue!=nullptr){",
            "if(invalidate){m_ContextValue->invalidate();}",
            "m_ContextValue->applyToSource(m_target,propertyKey(),isMainToSource(),this);",
        ],
        "DataBind::updateSourceBinding changed; audit RuntimeFile::data_bind_update_effect",
    );

    let source = compact_cpp_function_body(
        &compact_source,
        "DataBind::source(rcp<ViewModelInstanceValue>value)",
    )
    .unwrap_or_else(|| panic!("missing DataBind::source in {}", source_path.display()));
    assert_compact_contains_in_order(
        &source,
        &[
            "if(!bindsOnce()){",
            "value->addDependent(this);",
            "m_Source=value;",
            "if(m_Source&&target()&&target()->is<ArtboardComponentList>()){",
            "target()->as<ArtboardComponentList>()->shouldResetInstances(m_Source->coreType()==ViewModelInstanceNumberBase::typeKey);",
        ],
        "DataBind::source changed; audit RuntimeFile::data_bind_source_effect",
    );

    let clear_source = compact_cpp_function_body(&compact_source, "DataBind::clearSource()")
        .unwrap_or_else(|| panic!("missing DataBind::clearSource in {}", source_path.display()));
    assert_compact_contains_in_order(
        &clear_source,
        &[
            "if(m_Source!=nullptr){",
            "if(!bindsOnce()){",
            "m_Source->removeDependent(this);",
            "m_Source=nullptr;",
        ],
        "DataBind::clearSource changed; audit RuntimeFile::data_bind_clear_source_effect",
    );

    let bind = compact_cpp_function_body(&compact_source, "DataBind::bind()")
        .unwrap_or_else(|| panic!("missing DataBind::bind in {}", source_path.display()));
    assert_compact_contains_in_order(
        &bind,
        &[
            "deletem_ContextValue;",
            "m_ContextValue=nullptr;",
            "switch(outputType()){",
            "caseDataType::number:",
            "m_ContextValue=newDataBindContextValueNumber(this);",
            "caseDataType::any:",
            "m_ContextValue=newDataBindContextValueAny(this);",
            "if(m_dataConverter){",
            "m_dataConverter->reset();",
            "if(hasFlag(Flag::Observing)&&m_target!=nullptr){",
            "m_target->removePropertyObserver(this);",
            "setFlag(Flag::Observing,false);",
            "if(toSource()&&m_target!=nullptr&&targetSupportsPush()){",
            "m_target->addPropertyObserver(this);",
            "setFlag(Flag::Observing,true);",
            "addDirt(reconcileDirt(),true);",
        ],
        "DataBind::bind changed; audit RuntimeFile::data_bind_bind_effect",
    );

    let reconcile_dirt = compact_cpp_function_body(&compact_source, "DataBind::reconcileDirt()")
        .unwrap_or_else(|| {
            panic!(
                "missing DataBind::reconcileDirt in {}",
                source_path.display()
            )
        });
    assert_eq!(
        reconcile_dirt,
        "return(toTarget()?ComponentDirt::Bindings:ComponentDirt::None)|(toSource()?ComponentDirt::BindingsTarget:ComponentDirt::None);",
        "DataBind::reconcileDirt changed; audit RuntimeFile::data_bind_reconcile_dirt"
    );

    let target = compact_cpp_function_body(&compact_source, "DataBind::target(Core*value)")
        .unwrap_or_else(|| panic!("missing DataBind::target in {}", source_path.display()));
    assert_compact_contains_in_order(
        &target,
        &[
            "if(m_target==value){",
            "return;",
            "if(hasFlag(Flag::Observing)&&m_target!=nullptr){",
            "m_target->removePropertyObserver(this);",
            "setFlag(Flag::Observing,false);",
            "m_target=value;",
            "if(toSource()&&m_target!=nullptr&&targetSupportsPush()){",
            "m_target->addPropertyObserver(this);",
            "setFlag(Flag::Observing,true);",
        ],
        "DataBind::target changed; audit RuntimeFile::data_bind_target_effect",
    );

    let unbind = compact_cpp_function_body(&compact_source, "DataBind::unbind()")
        .unwrap_or_else(|| panic!("missing DataBind::unbind in {}", source_path.display()));
    assert_compact_contains_in_order(
        &unbind,
        &[
            "clearSource();",
            "if(hasFlag(Flag::Observing)&&m_target!=nullptr){",
            "m_target->removePropertyObserver(this);",
            "setFlag(Flag::Observing,false);",
            "if(m_dataConverter!=nullptr){",
            "m_dataConverter->unbind();",
            "if(m_ContextValue!=nullptr){",
            "deletem_ContextValue;",
            "m_ContextValue=nullptr;",
        ],
        "DataBind::unbind changed; audit RuntimeFile::data_bind_unbind_effect",
    );

    let add_dirt = compact_cpp_function_body(
        &compact_source,
        "DataBind::addDirt(ComponentDirtvalue,boolrecurse)",
    )
    .unwrap_or_else(|| panic!("missing DataBind::addDirt in {}", source_path.display()));
    assert_compact_contains_in_order(
        &add_dirt,
        &[
            "if(hasFlag(Flag::SuppressDirt)||(m_Dirt&value)==value){",
            "return;",
            "boolhasSource=enums::is_flag_set(value,ComponentDirt::Bindings);",
            "boolhasTarget=enums::is_flag_set(value,ComponentDirt::BindingsTarget);",
            "if(hasSource&&hasTarget){",
            "setFlag(Flag::TargetOrigin,!sourceToTargetRunsFirst());",
            "elseif(hasTarget){",
            "setFlag(Flag::TargetOrigin,true);",
            "elseif(hasSource){",
            "setFlag(Flag::TargetOrigin,false);",
            "m_Dirt|=value;",
            "if(enums::is_flag_set(m_Dirt,ComponentDirt::Dependents)&&m_ContextValue!=nullptr){",
            "m_ContextValue->invalidate();",
            "if(m_container&&!hasFlag(Flag::Collapsed)){",
            "m_container->addDirtyDataBind(this);",
        ],
        "DataBind::addDirt changed; audit RuntimeFile::data_bind_add_dirt_effect",
    );

    let core_source = std::fs::read_to_string(&core_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", core_source_path.display()));
    let compact_core_source = compact_cpp_source(&core_source);
    let notify_property_changed = compact_cpp_function_body(
        &compact_core_source,
        "Core::notifyPropertyChanged(uint16_tpropertyKey)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing Core::notifyPropertyChanged in {}",
            core_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &notify_property_changed,
        &[
            "if(m_firstObserver==nullptr){",
            "return;",
            "for(DataBind*o=m_firstObserver;o!=nullptr;o=o->nextObserver()){",
            "if(o->propertyKey()==propertyKey){",
            "o->addDirt(ComponentDirt::BindingsTarget,false);",
        ],
        "Core::notifyPropertyChanged changed; audit target-origin DataBind dirt modeling",
    );

    let initialize = compact_cpp_function_body(&compact_source, "DataBind::initialize()")
        .unwrap_or_else(|| panic!("missing DataBind::initialize in {}", source_path.display()));
    assert_eq!(
        initialize,
        "if(target()&&target()->is<Component>()){target()->as<Component>()->addCollapsable(this);}",
        "DataBind::initialize changed; audit RuntimeFile::data_bind_initialize_effect"
    );

    let relink = compact_cpp_function_body(&compact_source, "DataBind::relinkDataBind()")
        .unwrap_or_else(|| {
            panic!(
                "missing DataBind::relinkDataBind in {}",
                source_path.display()
            )
        });
    assert_eq!(
        relink, "m_container->rebuildDataBind(this);",
        "DataBind::relinkDataBind changed; audit RuntimeFile::data_bind_relink_effect"
    );

    let component_source = std::fs::read_to_string(&component_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", component_source_path.display()));
    let compact_component_source = compact_cpp_source(&component_source);
    let add_collapsable = compact_cpp_function_body(
        &compact_component_source,
        "Component::addCollapsable(DataBind*collapsable)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing Component::addCollapsable in {}",
            component_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &add_collapsable,
        &[
            "autosizeBefore=m_collapsables.size();",
            "m_collapsables.pushUnique(collapsable);",
            "if(m_collapsables.size()!=sizeBefore){",
            "collapsable->collapse(isCollapsed());",
        ],
        "Component::addCollapsable changed; audit RuntimeFile::data_bind_initialize_effect",
    );

    let artboard_source = std::fs::read_to_string(&artboard_source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", artboard_source_path.display()));
    let compact_artboard_source = compact_cpp_source(&artboard_source);
    let artboard_rebuild = compact_cpp_function_body(
        &compact_artboard_source,
        "Artboard::rebuildDataBind(DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing Artboard::rebuildDataBind in {}",
            artboard_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &artboard_rebuild,
        &[
            "if(dataBind->is<DataBindContext>()){",
            "dataBind->as<DataBindContext>()->bindFromContext(m_DataContext.get());",
        ],
        "Artboard::rebuildDataBind changed; audit RuntimeFile::data_bind_relink_effect",
    );

    let state_machine_instance_source =
        std::fs::read_to_string(&state_machine_instance_source_path).unwrap_or_else(|err| {
            panic!(
                "failed to read {}: {err}",
                state_machine_instance_source_path.display()
            )
        });
    let compact_state_machine_instance_source = compact_cpp_source(&state_machine_instance_source);
    let state_machine_rebuild = compact_cpp_function_body(
        &compact_state_machine_instance_source,
        "StateMachineInstance::rebuildDataBind(DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing StateMachineInstance::rebuildDataBind in {}",
            state_machine_instance_source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &state_machine_rebuild,
        &[
            "if(dataBind->is<DataBindContext>()){",
            "dataBind->as<DataBindContext>()->bindFromContext(m_DataContext.get());",
        ],
        "StateMachineInstance::rebuildDataBind changed; audit RuntimeFile::data_bind_relink_effect",
    );
}

#[test]
fn cpp_data_bind_container_update_queues_are_tracked_by_binary_import_model() {
    let include_path =
        reference_runtime_dir().join("include/rive/data_bind/data_bind_container.hpp");
    let source_path = reference_runtime_dir().join("src/data_bind/data_bind_container.cpp");
    if !include_path.exists() || !source_path.exists() {
        eprintln!(
            "skipping C++ data-bind container queue audit; reference files not found at {} and {}; set RIVE_RUNTIME_DIR",
            include_path.display(),
            source_path.display()
        );
        return;
    }

    let header = std::fs::read_to_string(&include_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", include_path.display()));
    let compact_header = compact_cpp_source(&header);
    assert_compact_contains_in_order(
        &compact_header,
        &[
            "std::vector<DataBind*>m_dataBinds;",
            "std::vector<DataBind*>m_persistingDataBinds;",
            "std::vector<DataBind*>m_dirtyToSourceDataBinds;",
            "std::vector<DataBind*>m_pendingDirtyToSourceDataBinds;",
            "std::vector<DataBind*>m_dirtyDataBinds;",
            "std::vector<DataBind*>m_pendingDirtyDataBinds;",
            "std::vector<DataBind*>m_pendingAdditions;",
            "std::vector<DataBind*>m_pendingRemovals;",
            "DataContext*m_dataContext=nullptr;",
            "boolm_isProcessing=false;",
        ],
        "DataBindContainer queue members changed; audit RuntimeDataBindUpdateQueue and add/remove effect classification",
    );

    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let compact_source = compact_cpp_source(&source);

    let unbind_data_binds =
        compact_cpp_function_body(&compact_source, "DataBindContainer::unbindDataBinds()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataBindContainer::unbindDataBinds in {}",
                    source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &unbind_data_binds,
        &[
            "for(auto&dataBind:m_dataBinds){",
            "dataBind->unbind();",
            "m_dataContext=nullptr;",
        ],
        "DataBindContainer::unbindDataBinds changed; audit RuntimeFile::data_bind_container_unbind_effect",
    );

    let bind_data_binds_from_context = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::bindDataBindsFromContext(DataContext*dataContext)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::bindDataBindsFromContext in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &bind_data_binds_from_context,
        &[
            "for(auto&dataBind:m_dataBinds){",
            "if(dataBind->is<DataBindContext>()){",
            "dataBind->as<DataBindContext>()->bindFromContext(dataContext);",
            "m_dataContext=dataContext;",
        ],
        "DataBindContainer::bindDataBindsFromContext changed; audit RuntimeFile::data_bind_container_bind_context_effect",
    );

    let advance_data_binds = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::advanceDataBinds(floatelapsedSeconds)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::advanceDataBinds in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &advance_data_binds,
        &[
            "if(m_dataBinds.size()==0){returnfalse;}",
            "booldidUpdate=false;",
            "for(auto&dataBind:m_dataBinds){",
            "if(dataBind->advance(elapsedSeconds)){",
            "didUpdate=true;",
            "returndidUpdate;",
        ],
        "DataBindContainer::advanceDataBinds changed; audit RuntimeFile::data_bind_container_advance_effect",
    );

    let add_data_bind = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::addDataBind(DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::addDataBind in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &add_data_bind,
        &[
            "if(m_isProcessing){m_pendingAdditions.push_back(dataBind);return;}",
            "m_dataBinds.push_back(dataBind);",
            "if(dataBind->toSource()&&!dataBind->targetSupportsPush()){",
            "m_persistingDataBinds.push_back(dataBind);",
            "dataBind->inPersistingList(true);",
            "dataBind->container(this);",
            "if(m_dataContext&&dataBind->is<DataBindContext>()){",
            "dataBind->as<DataBindContext>()->bindFromContext(m_dataContext);",
            "updateDataBind(dataBind,true);",
        ],
        "DataBindContainer::addDataBind changed; audit RuntimeFile::data_bind_add_effect, data_bind_uses_persisting_list, and data_bind_update_queue",
    );

    let remove_data_bind = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::removeDataBind(DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::removeDataBind in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &remove_data_bind,
        &[
            "if(m_isProcessing){m_pendingRemovals.push_back(dataBind);return;}",
            "autoeraseOne=[dataBind](std::vector<DataBind*>&v){",
            "v.erase(std::remove(v.begin(),v.end(),dataBind),v.end());",
            "eraseOne(m_dataBinds);",
            "if(dataBind->inPersistingList()){",
            "eraseOne(m_persistingDataBinds);",
            "dataBind->inPersistingList(false);",
            "if(dataBind->inDirtyList()){",
            "eraseOne(m_dirtyToSourceDataBinds);",
            "eraseOne(m_pendingDirtyToSourceDataBinds);",
            "eraseOne(m_dirtyDataBinds);",
            "eraseOne(m_pendingDirtyDataBinds);",
            "dataBind->inDirtyList(false);",
            "dataBind->container(nullptr);",
        ],
        "DataBindContainer::removeDataBind changed; audit RuntimeFile::data_bind_remove_effect",
    );

    let add_dirty_data_bind = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::addDirtyDataBind(DataBind*dataBind)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::addDirtyDataBind in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &add_dirty_data_bind,
        &[
            "if(dataBind->toSource()&&dataBind->inPersistingList()){return;}",
            "if(dataBind->inDirtyList()){return;}",
            "auto&insertingList=dataBind->toSource()?",
            "(m_isProcessing?m_pendingDirtyToSourceDataBinds:m_dirtyToSourceDataBinds)",
            ":(m_isProcessing?m_pendingDirtyDataBinds:m_dirtyDataBinds);",
            "insertingList.push_back(dataBind);",
            "dataBind->inDirtyList(true);",
        ],
        "DataBindContainer::addDirtyDataBind queue selection changed; audit RuntimeDataBindUpdateQueue",
    );

    let update_data_bind = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::updateDataBind(DataBind*dataBind,boolapplyTargetToSource)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::updateDataBind in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &update_data_bind,
        &[
            "autod=dataBind->dirt();",
            "if((d&ComponentDirt::Dependents)==ComponentDirt::Dependents){",
            "dataBind->updateDependents();",
            "boolwantsTargetToSource=applyTargetToSource&&(dataBind->inPersistingList()||(d&ComponentDirt::BindingsTarget)==ComponentDirt::BindingsTarget);",
            "if(wantsTargetToSource&&!dataBind->sourceToTargetRunsFirst()){",
            "dataBind->updateSourceBinding();",
            "if(d!=ComponentDirt::None){",
            "dataBind->dirt(ComponentDirt::None);",
            "dataBind->update(d);",
            "if(wantsTargetToSource&&dataBind->sourceToTargetRunsFirst()){",
            "dataBind->updateSourceBinding();",
        ],
        "DataBindContainer::updateDataBind order changed; audit RuntimeFile::data_bind_update_effect",
    );

    let sort_data_binds =
        compact_cpp_function_body(&compact_source, "DataBindContainer::sortDataBinds()")
            .unwrap_or_else(|| {
                panic!(
                    "missing DataBindContainer::sortDataBinds in {}",
                    source_path.display()
                )
            });
    assert_compact_contains_in_order(
        &sort_data_binds,
        &[
            "size_tcurrentToSourceIndex=0;",
            "for(size_ti=0;i<m_dataBinds.size();i++){",
            "if(m_dataBinds[i]->toSource()){",
            "if(i!=currentToSourceIndex){",
            "std::iter_swap(m_dataBinds.begin()+currentToSourceIndex,m_dataBinds.begin()+i);",
            "currentToSourceIndex+=1;",
        ],
        "DataBindContainer::sortDataBinds changed; audit RuntimeFile::sorted_data_bind_ids",
    );

    let update_data_binds = compact_cpp_function_body(
        &compact_source,
        "DataBindContainer::updateDataBinds(boolapplyTargetToSource)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing DataBindContainer::updateDataBinds in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &update_data_binds,
        &[
            "if(m_isProcessing){return;}",
            "if(m_persistingDataBinds.size()==0&&m_dirtyToSourceDataBinds.size()==0&&m_dirtyDataBinds.size()==0){return;}",
            "m_isProcessing=true;",
            "for(auto&dataBind:m_persistingDataBinds){",
            "if(!dataBind->canSkip()){updateDataBind(dataBind,applyTargetToSource);}",
            "for(auto&dataBind:m_dirtyToSourceDataBinds){",
            "dataBind->inDirtyList(false);",
            "updateDataBind(dataBind,applyTargetToSource);",
            "for(auto&dataBind:m_dirtyDataBinds){",
            "dataBind->inDirtyList(false);",
            "updateDataBind(dataBind,applyTargetToSource);",
            "m_dirtyToSourceDataBinds.clear();",
            "m_dirtyDataBinds.clear();",
            "if(m_pendingDirtyToSourceDataBinds.size()>0){m_dirtyToSourceDataBinds.swap(m_pendingDirtyToSourceDataBinds);}",
            "if(m_pendingDirtyDataBinds.size()>0){m_dirtyDataBinds.swap(m_pendingDirtyDataBinds);}",
            "m_isProcessing=false;",
            "if(!m_pendingAdditions.empty()){",
            "std::vector<DataBind*>additions;",
            "additions.swap(m_pendingAdditions);",
            "for(auto*dataBind:additions){",
            "addDataBind(dataBind);",
            "if(!m_pendingRemovals.empty()){",
            "std::vector<DataBind*>removals;",
            "removals.swap(m_pendingRemovals);",
            "for(auto*dataBind:removals){",
            "removeDataBind(dataBind);",
        ],
        "DataBindContainer::updateDataBinds queue drain order changed; audit RuntimeFile::data_bind_container_update_effect",
    );
}

#[test]
fn cpp_custom_on_added_methods_are_tracked_by_binary_import_model() {
    let src_dir = reference_runtime_dir().join("src");
    if !src_dir.exists() {
        eprintln!(
            "skipping C++ onAdded method audit; reference src not found at {}; set RIVE_RUNTIME_DIR",
            src_dir.display()
        );
        return;
    }

    let mut cpp_files = Vec::new();
    collect_cpp_paths(&src_dir, &mut cpp_files);

    let mut actual = BTreeSet::new();
    for path in cpp_files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&contents);
        for rest in compact.split("StatusCode").skip(1) {
            let Some((signature, _)) = rest.split_once('(') else {
                continue;
            };
            let Some((class_name, method_name)) = signature.split_once("::") else {
                continue;
            };
            if !is_cpp_identifier(class_name) || !is_cpp_identifier(method_name) {
                continue;
            }
            if matches!(method_name, "onAddedDirty" | "onAddedClean") {
                actual.insert(format!("{class_name}::{method_name}"));
            }
        }
    }

    let expected = [
        "Artboard::onAddedClean",
        "ArtboardListMapRule::onAddedDirty",
        "Axis::onAddedDirty",
        "AxisX::onAddedDirty",
        "AxisY::onAddedDirty",
        "BlendAnimation1D::onAddedClean",
        "BlendAnimation1D::onAddedDirty",
        "BlendAnimationDirect::onAddedClean",
        "BlendAnimationDirect::onAddedDirty",
        "Bone::onAddedClean",
        "ClippingShape::onAddedClean",
        "ClippingShape::onAddedDirty",
        "Component::onAddedDirty",
        "Constraint::onAddedDirty",
        "CubicInterpolator::onAddedDirty",
        "CubicInterpolatorComponent::onAddedDirty",
        "CubicValueInterpolator::onAddedDirty",
        "Dash::onAddedClean",
        "DashPath::onAddedClean",
        "DataBind::onAddedDirty",
        "DrawRules::onAddedClean",
        "DrawRules::onAddedDirty",
        "DrawTarget::onAddedClean",
        "DrawTarget::onAddedDirty",
        "Drawable::onAddedDirty",
        "ElasticInterpolator::onAddedDirty",
        "Feather::onAddedDirty",
        "FollowPathConstraint::onAddedClean",
        "GradientStop::onAddedDirty",
        "IKConstraint::onAddedClean",
        "InterpolatingKeyFrame::onAddedDirty",
        "Joystick::onAddedClean",
        "Joystick::onAddedDirty",
        "KeyedObject::onAddedClean",
        "KeyedObject::onAddedDirty",
        "KeyedProperty::onAddedClean",
        "KeyedProperty::onAddedDirty",
        "LayerState::onAddedClean",
        "LayerState::onAddedDirty",
        "LayoutComponent::onAddedClean",
        "LayoutComponent::onAddedDirty",
        "LayoutComponentStyle::onAddedDirty",
        "LinearAnimation::onAddedClean",
        "LinearAnimation::onAddedDirty",
        "LinearGradient::onAddedDirty",
        "Mesh::onAddedClean",
        "Mesh::onAddedDirty",
        "MeshVertex::onAddedDirty",
        "NSlicer::onAddedDirty",
        "NSlicerTileMode::onAddedDirty",
        "NestedAnimation::onAddedDirty",
        "NestedArtboard::onAddedClean",
        "NestedArtboardLayout::onAddedClean",
        "Path::onAddedClean",
        "PathVertex::onAddedDirty",
        "RootBone::onAddedClean",
        "ScriptInputArtboard::onAddedClean",
        "ScriptInputBoolean::onAddedClean",
        "ScriptInputColor::onAddedClean",
        "ScriptInputNumber::onAddedClean",
        "ScriptInputString::onAddedClean",
        "ScriptInputTrigger::onAddedClean",
        "ScriptInputViewModelProperty::onAddedClean",
        "ScriptedDrawable::onAddedDirty",
        "ScriptedPathEffect::onAddedClean",
        "ScriptedPathEffect::onAddedDirty",
        "ScrollBarConstraint::onAddedDirty",
        "ScrollConstraint::onAddedDirty",
        "Shape::onAddedClean",
        "Shape::onAddedDirty",
        "ShapePaint::onAddedClean",
        "Skin::onAddedDirty",
        "SolidColor::onAddedDirty",
        "Solo::onAddedClean",
        "StateMachine::onAddedClean",
        "StateMachine::onAddedDirty",
        "StateMachineInput::onAddedClean",
        "StateMachineInput::onAddedDirty",
        "StateMachineLayer::onAddedClean",
        "StateMachineLayer::onAddedDirty",
        "StateTransition::onAddedClean",
        "StateTransition::onAddedDirty",
        "TargetEffect::onAddedClean",
        "TargetedConstraint::onAddedDirty",
        "Tendon::onAddedClean",
        "Tendon::onAddedDirty",
        "TextFollowPathModifier::onAddedClean",
        "TextInput::onAddedClean",
        "TextInputDrawable::onAddedClean",
        "TextInputSelectedText::onAddedClean",
        "TextModifier::onAddedDirty",
        "TextModifierGroup::onAddedDirty",
        "TextModifierRange::onAddedDirty",
        "TextStyle::onAddedClean",
        "TextStyleAxis::onAddedDirty",
        "TextStyleFeature::onAddedDirty",
        "TextTargetModifier::onAddedDirty",
        "TextValueRun::onAddedClean",
        "TextValueRun::onAddedDirty",
        "TransformComponent::onAddedClean",
        "TransitionCondition::onAddedClean",
        "TransitionCondition::onAddedDirty",
        "TrimPath::onAddedClean",
        "TrimPath::onAddedDirty",
        "ViewModelInstanceValue::onAddedDirty",
        "Weight::onAddedDirty",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "C++ onAddedDirty/onAddedClean surface changed; audit import-resolution validation, graph projection effects, and C++ probe coverage before refreshing this list"
    );
}

#[test]
fn cpp_joystick_resolution_methods_are_tracked_by_binary_import_model() {
    let source_path = reference_runtime_dir().join("src/joystick.cpp");
    if !source_path.exists() {
        eprintln!(
            "skipping C++ joystick handle-source audit; reference source not found at {}; set RIVE_RUNTIME_DIR",
            source_path.display()
        );
        return;
    }

    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let compact_source = compact_cpp_source(&source);
    let on_added_dirty = compact_cpp_function_body(
        &compact_source,
        "Joystick::onAddedDirty(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing Joystick::onAddedDirty in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &on_added_dirty,
        &[
            "StatusCodestatus=Super::onAddedDirty(context);",
            "if(status!=StatusCode::Ok){returnstatus;}",
            "if(handleSourceId()!=Core::emptyId){",
            "autocoreObject=context->resolve(handleSourceId());",
            "if(coreObject==nullptr||!coreObject->is<TransformComponent>()){returnStatusCode::MissingObject;}",
            "m_handleSource=static_cast<TransformComponent*>(coreObject);",
            "returnStatusCode::Ok;",
        ],
        "Joystick::onAddedDirty changed; audit RuntimeFile::resolved_handle_source_for_joystick",
    );

    let on_added_clean = compact_cpp_function_body(
        &compact_source,
        "Joystick::onAddedClean(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing Joystick::onAddedClean in {}",
            source_path.display()
        )
    });
    assert_compact_contains_in_order(
        &on_added_clean,
        &[
            "m_xAnimation=artboard()->animation(xId());",
            "m_yAnimation=artboard()->animation(yId());",
            "returnStatusCode::Ok;",
        ],
        "Joystick::onAddedClean changed; audit RuntimeFile::resolved_x_animation_for_joystick and resolved_y_animation_for_joystick",
    );
}

#[test]
fn cpp_n_slicer_resolution_methods_are_tracked_by_binary_import_model() {
    let runtime_dir = reference_runtime_dir();
    let source_paths = [
        runtime_dir.join("src/layout/n_slicer_details.cpp"),
        runtime_dir.join("src/layout/n_slicer.cpp"),
        runtime_dir.join("src/layout/axis.cpp"),
        runtime_dir.join("src/layout/axis_x.cpp"),
        runtime_dir.join("src/layout/axis_y.cpp"),
        runtime_dir.join("src/layout/n_slicer_tile_mode.cpp"),
    ];
    if !source_paths.iter().all(|path| path.exists()) {
        eprintln!(
            "skipping C++ NSlicer audit; reference layout source not found at {}; set RIVE_RUNTIME_DIR",
            runtime_dir.display()
        );
        return;
    }

    let compact_sources = source_paths
        .iter()
        .map(|path| {
            let source = std::fs::read_to_string(path)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
            compact_cpp_source(&source)
        })
        .collect::<Vec<_>>();

    let n_slicer_details_from = compact_cpp_function_body(
        &compact_sources[0],
        "NSlicerDetails::from(Component*component)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing NSlicerDetails::from in {}",
            source_paths[0].display()
        )
    });
    assert_compact_contains_in_order(
        &n_slicer_details_from,
        &[
            "switch(component->coreType()){",
            "caseNSlicer::typeKey:returncomponent->as<NSlicer>();",
            "caseNSlicedNode::typeKey:returncomponent->as<NSlicedNode>();",
            "returnnullptr;",
        ],
        "NSlicerDetails::from changed; audit RuntimeFile::artboard_n_slicer_details and parent validation",
    );

    let add_axis_x =
        compact_cpp_function_body(&compact_sources[0], "NSlicerDetails::addAxisX(Axis*axis)")
            .unwrap_or_else(|| {
                panic!(
                    "missing NSlicerDetails::addAxisX in {}",
                    source_paths[0].display()
                )
            });
    assert!(
        add_axis_x.contains("m_xs.push_back(axis);"),
        "NSlicerDetails::addAxisX changed; audit RuntimeNSlicerDetails x-axis ordering"
    );
    let add_axis_y =
        compact_cpp_function_body(&compact_sources[0], "NSlicerDetails::addAxisY(Axis*axis)")
            .unwrap_or_else(|| {
                panic!(
                    "missing NSlicerDetails::addAxisY in {}",
                    source_paths[0].display()
                )
            });
    assert!(
        add_axis_y.contains("m_ys.push_back(axis);"),
        "NSlicerDetails::addAxisY changed; audit RuntimeNSlicerDetails y-axis ordering"
    );
    let add_tile_mode = compact_cpp_function_body(
        &compact_sources[0],
        "NSlicerDetails::addTileMode(intpatchIndex,NSlicerTileModeTypestyle)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing NSlicerDetails::addTileMode in {}",
            source_paths[0].display()
        )
    });
    assert!(
        add_tile_mode.contains("m_tileModes[patchIndex]=style;"),
        "NSlicerDetails::addTileMode changed; audit RuntimeNSlicerTileMode duplicate handling"
    );

    let n_slicer_dirty = compact_cpp_function_body(
        &compact_sources[1],
        "NSlicer::onAddedDirty(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing NSlicer::onAddedDirty in {}",
            source_paths[1].display()
        )
    });
    assert_compact_contains_in_order(
        &n_slicer_dirty,
        &[
            "StatusCodecode=Super::onAddedDirty(context);",
            "if(code!=StatusCode::Ok){returncode;}",
            "if(!parent()->is<Image>()){returnStatusCode::MissingObject;}",
            "parent()->as<Image>()->setMesh(m_sliceMesh.get());",
            "returnStatusCode::Ok;",
        ],
        "NSlicer::onAddedDirty changed; audit NSlicer parent validation",
    );

    let axis_dirty = compact_cpp_function_body(
        &compact_sources[2],
        "Axis::onAddedDirty(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing Axis::onAddedDirty in {}",
            source_paths[2].display()
        )
    });
    assert_compact_contains_in_order(
        &axis_dirty,
        &[
            "StatusCodecode=Super::onAddedDirty(context);",
            "if(code!=StatusCode::Ok){returncode;}",
            "if(NSlicerDetails::from(parent())==nullptr){returnStatusCode::MissingObject;}",
            "returnStatusCode::Ok;",
        ],
        "Axis::onAddedDirty changed; audit Axis parent validation",
    );

    let axis_x_dirty = compact_cpp_function_body(
        &compact_sources[3],
        "AxisX::onAddedDirty(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing AxisX::onAddedDirty in {}",
            source_paths[3].display()
        )
    });
    assert_compact_contains_in_order(
        &axis_x_dirty,
        &[
            "StatusCodecode=Super::onAddedDirty(context);",
            "if(code!=StatusCode::Ok){returncode;}",
            "NSlicerDetails::from(parent())->addAxisX(this);",
            "returnStatusCode::Ok;",
        ],
        "AxisX::onAddedDirty changed; audit RuntimeNSlicerDetails x-axis registration",
    );

    let axis_y_dirty = compact_cpp_function_body(
        &compact_sources[4],
        "AxisY::onAddedDirty(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing AxisY::onAddedDirty in {}",
            source_paths[4].display()
        )
    });
    assert_compact_contains_in_order(
        &axis_y_dirty,
        &[
            "StatusCodecode=Super::onAddedDirty(context);",
            "if(code!=StatusCode::Ok){returncode;}",
            "NSlicerDetails::from(parent())->addAxisY(this);",
            "returnStatusCode::Ok;",
        ],
        "AxisY::onAddedDirty changed; audit RuntimeNSlicerDetails y-axis registration",
    );

    let tile_mode_dirty = compact_cpp_function_body(
        &compact_sources[5],
        "NSlicerTileMode::onAddedDirty(CoreContext*context)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing NSlicerTileMode::onAddedDirty in {}",
            source_paths[5].display()
        )
    });
    assert_compact_contains_in_order(
        &tile_mode_dirty,
        &[
            "StatusCodecode=Super::onAddedDirty(context);",
            "if(code!=StatusCode::Ok){returncode;}",
            "NSlicerDetails*container=NSlicerDetails::from(parent());",
            "if(container==nullptr){returnStatusCode::MissingObject;}",
            "container->addTileMode(patchIndex(),NSlicerTileModeType(style()));",
            "returnStatusCode::Ok;",
        ],
        "NSlicerTileMode::onAddedDirty changed; audit RuntimeNSlicerDetails tile-mode registration",
    );
}

#[test]
fn cpp_importer_resolution_methods_are_tracked_by_binary_import_model() {
    let src_dir = reference_runtime_dir().join("src");
    if !src_dir.exists() {
        eprintln!(
            "skipping C++ importer resolution audit; reference src not found at {}; set RIVE_RUNTIME_DIR",
            src_dir.display()
        );
        return;
    }

    let mut cpp_files = Vec::new();
    collect_cpp_paths(&src_dir, &mut cpp_files);

    let mut resolves = BTreeSet::new();
    let mut null_readers = BTreeSet::new();
    for path in cpp_files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&contents);
        for rest in compact.split("StatusCode").skip(1) {
            let Some((signature, _)) = rest.split_once('(') else {
                continue;
            };
            let Some((class_name, method_name)) = signature.split_once("::") else {
                continue;
            };
            if !is_cpp_identifier(class_name) || !is_cpp_identifier(method_name) {
                continue;
            }
            if method_name == "resolve" {
                resolves.insert(format!("{class_name}::resolve"));
            }
        }

        for rest in compact.split("bool").skip(1) {
            let Some((signature, _)) = rest.split_once('(') else {
                continue;
            };
            let Some((class_name, method_name)) = signature.split_once("::") else {
                continue;
            };
            if !is_cpp_identifier(class_name) || !is_cpp_identifier(method_name) {
                continue;
            }
            if method_name == "readNullObject" {
                null_readers.insert(format!("{class_name}::readNullObject"));
            }
        }
    }

    let expected_resolves = [
        "ArtboardImporter::resolve",
        "BackboardImporter::resolve",
        "DataConverterFormulaImporter::resolve",
        "EnumImporter::resolve",
        "FileAssetImporter::resolve",
        "LayerStateImporter::resolve",
        "ListenerInputTypeGamepadImporter::resolve",
        "ListenerInputTypeKeyboardImporter::resolve",
        "ListenerInputTypeSemanticImporter::resolve",
        "ScriptedObjectImporter::resolve",
        "StateMachineImporter::resolve",
        "StateMachineLayerImporter::resolve",
        "StateMachineListenerImporter::resolve",
        "StateTransitionImporter::resolve",
        "TextAssetImporter::resolve",
        "TransitionViewModelConditionImporter::resolve",
        "ViewModelImporter::resolve",
        "ViewModelInstanceImporter::resolve",
        "ViewModelInstanceListImporter::resolve",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    let expected_null_readers = [
        "ArtboardImporter::readNullObject",
        "KeyedPropertyImporter::readNullObject",
        "StateMachineImporter::readNullObject",
        "StateMachineLayerImporter::readNullObject",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        resolves, expected_resolves,
        "C++ ImportStackObject::resolve surface changed; audit Rust import-resolution helpers and C++ probe coverage before refreshing this list"
    );
    assert_eq!(
        null_readers, expected_null_readers,
        "C++ ImportStackObject::readNullObject surface changed; audit NullObjectConsumer routing and synthetic null-object coverage before refreshing this list"
    );
}

#[test]
fn cpp_import_stack_core_mechanics_match_rust_model() {
    let path = reference_runtime_dir().join("include/rive/importers/import_stack.hpp");
    if !path.exists() {
        eprintln!(
            "skipping C++ ImportStack audit; reference import_stack.hpp not found at {}; set RIVE_RUNTIME_DIR",
            path.display()
        );
        return;
    }

    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let compact = compact_cpp_source(&source);

    let stack_object = compact_cpp_class_body(&compact, "ImportStackObject")
        .unwrap_or_else(|| panic!("missing ImportStackObject in {}", path.display()));
    assert!(
        stack_object.contains("virtualStatusCoderesolve(){returnStatusCode::Ok;}"),
        "ImportStackObject default resolve changed; audit Rust default importer resolution behavior"
    );
    assert!(
        stack_object.contains("virtualboolreadNullObject(){returnfalse;}"),
        "ImportStackObject default null-object handling changed; audit Rust null-object consumers"
    );

    let import_stack = compact_cpp_class_body(&compact, "ImportStack")
        .unwrap_or_else(|| panic!("missing ImportStack in {}", path.display()));

    let latest = compact_cpp_function_body(&import_stack, "T*latest(uint16_tcoreType)")
        .unwrap_or_else(|| panic!("missing ImportStack::latest in {}", path.display()));
    assert_eq!(
        latest,
        "autoitr=m_latests.find(coreType);if(itr==m_latests.end()){returnnullptr;}returnstatic_cast<T*>(itr->second.get());",
        "ImportStack::latest changed; audit Rust latest importer lookups"
    );

    let make_latest = compact_cpp_function_body(
        &import_stack,
        "makeLatest(uint16_tcoreType,std::unique_ptr<ImportStackObject>object)",
    )
    .unwrap_or_else(|| panic!("missing ImportStack::makeLatest in {}", path.display()));
    assert_compact_contains_in_order(
        &make_latest,
        &[
            "autoitr=m_latests.find(coreType);",
            "if(itr!=m_latests.end()){",
            "autostackObject=itr->second.get();",
            "autolastAddedItr=std::find(m_lastAdded.begin(),m_lastAdded.end(),stackObject);",
            "if(lastAddedItr!=m_lastAdded.end()){m_lastAdded.erase(lastAddedItr);}",
            "StatusCodecode=stackObject->resolve();",
            "if(code!=StatusCode::Ok){m_latests.erase(coreType);returncode;}",
            "if(object==nullptr){m_latests.erase(coreType);}",
            "else{m_lastAdded.push_back(object.get());m_latests[coreType]=std::move(object);}",
            "returnStatusCode::Ok;",
        ],
        "ImportStack::makeLatest changed; audit Rust latest/last-added replacement semantics",
    );

    let resolve = compact_cpp_function_body(&import_stack, "StatusCoderesolve()")
        .unwrap_or_else(|| panic!("missing ImportStack::resolve in {}", path.display()));
    assert_compact_contains_in_order(
        &resolve,
        &[
            "StatusCodereturnCode=StatusCode::Ok;",
            "for(autoitr=m_lastAdded.rbegin();itr!=m_lastAdded.rend();itr++){",
            "StatusCodecode=(*itr)->resolve();",
            "if(code!=StatusCode::Ok){returnCode=code;break;}",
            "m_latests.clear();",
            "m_lastAdded.clear();",
            "returnreturnCode;",
        ],
        "ImportStack::resolve changed; audit Rust final import-stack resolution order and error mapping",
    );

    let read_null_object = compact_cpp_function_body(&import_stack, "boolreadNullObject()")
        .unwrap_or_else(|| panic!("missing ImportStack::readNullObject in {}", path.display()));
    assert_compact_contains_in_order(
        &read_null_object,
        &[
            "for(autoitr=m_lastAdded.rbegin();itr!=m_lastAdded.rend();itr++){",
            "if((*itr)->readNullObject()){returntrue;}",
            "returnfalse;",
        ],
        "ImportStack::readNullObject changed; audit Rust latest null-object consumer ordering",
    );
}

#[test]
fn cpp_auxiliary_status_methods_are_tracked_by_binary_import_model() {
    let src_dir = reference_runtime_dir().join("src");
    if !src_dir.exists() {
        eprintln!(
            "skipping C++ auxiliary StatusCode method audit; reference src not found at {}; set RIVE_RUNTIME_DIR",
            src_dir.display()
        );
        return;
    }

    let mut cpp_files = Vec::new();
    collect_cpp_paths(&src_dir, &mut cpp_files);

    let mut actual = BTreeSet::new();
    for path in cpp_files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&contents);
        for rest in compact.split("StatusCode").skip(1) {
            let Some((signature, _)) = rest.split_once('(') else {
                continue;
            };
            let Some((class_name, method_name)) = signature.split_once("::") else {
                continue;
            };
            if !is_cpp_identifier(class_name) || !is_cpp_identifier(method_name) {
                continue;
            }
            if method_name == "import"
                || method_name == "resolve"
                || matches!(method_name, "onAddedDirty" | "onAddedClean")
            {
                continue;
            }
            actual.insert(format!("{class_name}::{method_name}"));
        }
    }

    let expected = [
        "Artboard::initialize",
        "FileAssetReferencer::registerReferencer",
        "ShapePaintMutator::initPaintMutator",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "C++ auxiliary StatusCode method surface changed; audit import-resolution validation, file-asset referencer registration, paint/effect validation, and probe coverage before refreshing this list"
    );
}

#[test]
fn cpp_core_field_type_ids_match_runtime_header_contract() {
    let rive_dir = reference_runtime_dir().join("include/rive");
    let field_types_dir = rive_dir.join("core/field_types");
    if !field_types_dir.exists() {
        eprintln!(
            "skipping C++ field type id audit; reference field types not found at {}; set RIVE_RUNTIME_DIR",
            field_types_dir.display()
        );
        return;
    }

    let mut ids = BTreeMap::new();
    for (class_name, header_name) in [
        ("CoreUintType", "core_uint_type.hpp"),
        ("CoreUint64Type", "core_uint64_type.hpp"),
        ("CoreStringType", "core_string_type.hpp"),
        ("CoreBytesType", "core_bytes_type.hpp"),
        ("CoreDoubleType", "core_double_type.hpp"),
        ("CoreColorType", "core_color_type.hpp"),
        ("CoreBoolType", "core_bool_type.hpp"),
    ] {
        let id = parse_cpp_core_field_type_id(&field_types_dir.join(header_name), class_name);
        ids.insert(class_name, id);
    }

    assert_eq!(ids["CoreUintType"], 0);
    assert_eq!(ids["CoreUint64Type"], 0);
    assert_eq!(ids["CoreStringType"], 1);
    assert_eq!(ids["CoreBytesType"], 1);
    assert_eq!(ids["CoreDoubleType"], 2);
    assert_eq!(ids["CoreColorType"], 3);
    assert_eq!(ids["CoreBoolType"], 4);
    assert_eq!(
        ids["CoreUintType"], ids["CoreUint64Type"],
        "uint32 and uint64 must share the runtime-header field id"
    );
    assert_eq!(
        ids["CoreStringType"], ids["CoreBytesType"],
        "C++ runtime header only stores one two-bit field id for string/bytes payloads"
    );
    assert!(
        ids["CoreBoolType"] > 3,
        "bool is a CoreRegistry field kind but cannot be represented in RuntimeHeader's two-bit ToC"
    );

    let header_path = rive_dir.join("runtime_header.hpp");
    let header = std::fs::read_to_string(&header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header_path.display()));
    let compact_header = compact_cpp_source(&header);

    assert!(
        compact_header.contains("intcurrentBit=8;"),
        "RuntimeHeader field-id unpacking changed; audit RuntimeHeader::read in Rust"
    );
    assert!(
        compact_header.contains("fieldIndex=(currentInt>>currentBit)&3;"),
        "RuntimeHeader no longer masks two-bit field ids; audit HeaderFieldKind"
    );
    assert!(
        compact_header.contains("currentBit+=2;"),
        "RuntimeHeader field-id width changed; audit HeaderFieldKind"
    );
    assert!(
        compact_header.contains("if(currentBit==8)"),
        "RuntimeHeader packed field-id word cadence changed; audit RuntimeHeader::read in Rust"
    );
}

#[test]
fn cpp_runtime_header_read_matches_binary_header_model() {
    let header_path = reference_runtime_dir().join("include/rive/runtime_header.hpp");
    let file_path = reference_runtime_dir().join("src/file.cpp");
    if !header_path.exists() || !file_path.exists() {
        eprintln!(
            "skipping C++ RuntimeHeader audit; reference files not found at {} and {}; set RIVE_RUNTIME_DIR",
            header_path.display(),
            file_path.display()
        );
        return;
    }

    let header = std::fs::read_to_string(&header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header_path.display()));
    let compact_header = compact_cpp_source(&header);

    assert!(
        compact_header.contains("staticconstexprcharkRuntimeHeaderFingerprint[]=\"RIVE\";"),
        "RuntimeHeader fingerprint changed; audit Rust header fingerprint validation"
    );

    let property_field_id =
        compact_cpp_function_body(&compact_header, "propertyFieldId(intpropertyKey)const")
            .unwrap_or_else(|| {
                panic!(
                    "missing RuntimeHeader::propertyFieldId in {}",
                    header_path.display()
                )
            });
    assert_eq!(
        property_field_id,
        "autoitr=m_PropertyToFieldIndex.find(propertyKey);if(itr==m_PropertyToFieldIndex.end()){return-1;}returnitr->second;",
        "RuntimeHeader missing-ToC behavior changed; audit Rust unknown-property fallback"
    );

    let header_read = compact_cpp_function_body(
        &compact_header,
        "read(BinaryReader&reader,RuntimeHeader&header)",
    )
    .unwrap_or_else(|| panic!("missing RuntimeHeader::read in {}", header_path.display()));
    assert_compact_contains_in_order(
        &header_read,
        &[
            "for(inti=0;i<4;i++){autob=reader.readByte();if(kRuntimeHeaderFingerprint[i]!=b){returnfalse;}}",
            "header.m_MajorVersion=reader.readVarUintAs<int>();",
            "if(reader.didOverflow()){returnfalse;}",
            "header.m_MinorVersion=reader.readVarUintAs<int>();",
            "if(reader.didOverflow()){returnfalse;}",
            "header.m_FileId=reader.readVarUintAs<int>();",
            "if(reader.didOverflow()){returnfalse;}",
            "std::vector<int>propertyKeys;",
            "for(intpropertyKey=reader.readVarUintAs<int>();propertyKey!=0;propertyKey=reader.readVarUintAs<int>())",
            "propertyKeys.push_back(propertyKey);",
            "if(reader.didOverflow()){returnfalse;}",
            "intcurrentInt=0;",
            "intcurrentBit=8;",
            "for(autopropertyKey:propertyKeys)",
            "if(currentBit==8){currentInt=reader.readUint32();currentBit=0;}",
            "intfieldIndex=(currentInt>>currentBit)&3;",
            "header.m_PropertyToFieldIndex[propertyKey]=fieldIndex;",
            "currentBit+=2;",
            "if(reader.didOverflow()){returnfalse;}",
            "returntrue;",
        ],
        "RuntimeHeader::read changed; audit Rust header integer, property-ToC, and packed field-id decoding",
    );

    let file = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", file_path.display()));
    let compact_file = compact_cpp_source(&file);
    let import = compact_cpp_function_body(
        &compact_file,
        "File::import(Span<constuint8_t>bytes,Factory*factory,ImportResult*result,rcp<FileAssetLoader>assetLoader,ScriptingVM*vm)",
    )
    .unwrap_or_else(|| panic!("missing File::import in {}", file_path.display()));
    assert_compact_contains_in_order(
        &import,
        &[
            "BinaryReaderreader(bytes);",
            "RuntimeHeaderheader;",
            "if(!RuntimeHeader::read(reader,header)){",
            "*result=ImportResult::malformed;",
            "returnnullptr;",
            "if(header.majorVersion()!=majorVersion){",
            "header.minorVersion()",
            "*result=ImportResult::unsupportedVersion;",
            "returnnullptr;",
            "autoreadResult=file->read(reader,header);",
        ],
        "File::import version gate changed; audit Rust import result categories",
    );
    assert!(
        !import.contains("header.minorVersion()!=minorVersion"),
        "File::import now rejects minor-version mismatches; audit Rust unsupported-version behavior"
    );
}

#[test]
fn cpp_file_read_loop_matches_import_stack_model() {
    let path = reference_runtime_dir().join("src/file.cpp");
    if !path.exists() {
        eprintln!(
            "skipping C++ File::read audit; reference file.cpp not found at {}; set RIVE_RUNTIME_DIR",
            path.display()
        );
        return;
    }

    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let compact = compact_cpp_source(&source);
    let body = compact_cpp_function_body(
        &compact,
        "File::read(BinaryReader&reader,constRuntimeHeader&header)",
    )
    .unwrap_or_else(|| panic!("missing File::read in {}", path.display()));

    assert_compact_contains_in_order(
        &body,
        &[
            "ImportStackimportStack;",
            "Core*lastBindableObject=nullptr;",
            "while(!reader.reachedEnd()){",
            "autoobject=readRuntimeObject(reader,header);",
            "if(object==nullptr){importStack.readNullObject();continue;}",
            "if(!object->is<DataBind>()){lastBindableObject=object;}elseif(lastBindableObject!=nullptr){object->as<DataBind>()->target(lastBindableObject);}",
            "if(object->import(importStack)==StatusCode::Ok){",
            "caseBackboard::typeKey:m_backboard=object->as<Backboard>();break;",
            "caseArtboard::typeKey:{Artboard*ab=object->as<Artboard>();ab->m_Factory=m_factory;m_artboards.push_back(ab);}",
            "caseImageAsset::typeKey:caseFontAsset::typeKey:caseAudioAsset::typeKey:caseBlobAsset::typeKey:caseScriptAsset::typeKey:caseShaderAsset::typeKey:",
            "m_fileAssets.push_back(rcp<FileAsset>(fa));",
            "caseViewModel::typeKey:",
            "m_ViewModels.push_back(vmc);",
            "caseDataEnum::typeKey:caseDataEnumCustom::typeKey:",
            "m_Enums.push_back(de);",
            "caseViewModelPropertyEnumCustom::typeKey:",
            "vme->dataEnum(m_Enums[vme->enumId()]);",
            "else{if(lastBindableObject==object){lastBindableObject=nullptr;}fprintf(stderr,\"Failedtoimportobjectoftype%d\\n\",object->coreType());deleteobject;continue;}",
            "std::unique_ptr<ImportStackObject>stackObject=nullptr;",
            "autostackType=object->coreType();",
            "switch(stackType){",
        ],
        "File::read collection/import loop changed; audit Rust import-status and file-collection modeling",
    );

    let stack_cases = compact_cpp_type_key_switch_cases(&body, "switch(stackType)")
        .unwrap_or_else(|| panic!("missing File::read stackType switch in {}", path.display()));
    let expected_stack_cases = [
        "AnyState::typeKey",
        "AnimationState::typeKey",
        "Artboard::typeKey",
        "ArtboardComponentList::typeKey",
        "AudioAsset::typeKey",
        "Backboard::typeKey",
        "BindablePropertyArtboard::typeKey",
        "BindablePropertyAsset::typeKey",
        "BindablePropertyBoolean::typeKey",
        "BindablePropertyColor::typeKey",
        "BindablePropertyEnum::typeKey",
        "BindablePropertyInteger::typeKey",
        "BindablePropertyList::typeKey",
        "BindablePropertyNumber::typeKey",
        "BindablePropertyString::typeKey",
        "BindablePropertyTrigger::typeKey",
        "BindablePropertyViewModel::typeKey",
        "BlendState1DInput::typeKey",
        "BlendState1DViewModel::typeKey",
        "BlendStateDirect::typeKey",
        "BlendStateTransition::typeKey",
        "BlobAsset::typeKey",
        "DataBindPathBase::typeKey",
        "DataConverterFormulaBase::typeKey",
        "DataConverterGroupBase::typeKey",
        "DataConverterNumberToList::typeKey",
        "DataEnumCustom::typeKey",
        "EntryState::typeKey",
        "ExitState::typeKey",
        "FontAsset::typeKey",
        "ImageAsset::typeKey",
        "KeyedObject::typeKey",
        "KeyedProperty::typeKey",
        "LinearAnimation::typeKey",
        "ListenerInputTypeGamepadBase::typeKey",
        "ListenerInputTypeKeyboardBase::typeKey",
        "ListenerInputTypeSemanticBase::typeKey",
        "ManifestAsset::typeKey",
        "NestedArtboard::typeKey",
        "NestedArtboardLayout::typeKey",
        "NestedArtboardLeaf::typeKey",
        "ScriptAsset::typeKey",
        "ScriptInputArtboard::typeKey",
        "ScriptedDataConverter::typeKey",
        "ScriptedDrawable::typeKey",
        "ScriptedInterpolator::typeKey",
        "ScriptedLayout::typeKey",
        "ScriptedListenerAction::typeKey",
        "ScriptedPathEffect::typeKey",
        "ScriptedTransitionCondition::typeKey",
        "ShaderAsset::typeKey",
        "StateMachine::typeKey",
        "StateMachineLayer::typeKey",
        "StateMachineListener::typeKey",
        "StateMachineListenerSingle::typeKey",
        "StateTransition::typeKey",
        "TransitionArtboardCondition::typeKey",
        "TransitionFocusCondition::typeKey",
        "TransitionViewModelCondition::typeKey",
        "ViewModel::typeKey",
        "ViewModelInstance::typeKey",
        "ViewModelInstanceList::typeKey",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(
        stack_cases, expected_stack_cases,
        "File::read import stack switch changed; audit CppImportStack modeling and probe coverage before refreshing this list"
    );

    assert_compact_contains_in_order(
        &body,
        &[
            "caseKeyedProperty::typeKey:{autoimporter=importStack.latest<LinearAnimationImporter>(LinearAnimation::typeKey);if(importer==nullptr){returnImportResult::malformed;}",
            "caseStateMachineLayer::typeKey:{autoartboardImporter=importStack.latest<ArtboardImporter>(ArtboardBase::typeKey);if(artboardImporter==nullptr){returnImportResult::malformed;}",
            "caseDataConverterNumberToList::typeKey:object->as<DataConverterNumberToList>()->file(this);break;",
            "caseScriptInputArtboard::typeKey:object->as<ScriptInputArtboard>()->file(this);break;",
            "if(importStack.makeLatest(stackType,std::move(stackObject))!=StatusCode::Ok){",
            "returnImportResult::malformed;}",
            "if(object->is<StateMachineLayerComponent>()&&importStack.makeLatest(StateMachineLayerComponent::typeKey,std::make_unique<StateMachineLayerComponentImporter>(object->as<StateMachineLayerComponent>()))!=StatusCode::Ok){returnImportResult::malformed;}",
            "if(object->is<DataConverter>()){m_DataConverters.push_back(object->as<DataConverter>());}",
            "elseif(object->is<KeyFrameInterpolator>()){",
            "if(artboardImporter==nullptr){m_keyframeInterpolators.push_back(object->as<KeyFrameInterpolator>());}",
            "if(object->is<ScriptedInterpolator>()){m_scriptedInterpolators.push_back(object->as<ScriptedInterpolator>());}",
            "elseif(object->is<ScrollPhysics>()){m_scrollPhysics.push_back(object->as<ScrollPhysics>());}",
            "autoresolved=importStack.resolve();",
            "return!reader.hasError()&&resolved==StatusCode::Ok?ImportResult::success:ImportResult::malformed;",
        ],
        "File::read stack/finalization flow changed; audit Rust importer context validation and final error mapping",
    );
}

#[test]
fn cpp_core_field_deserializers_match_binary_reader_model() {
    let field_types_dir = reference_runtime_dir().join("src/core/field_types");
    if !field_types_dir.exists() {
        eprintln!(
            "skipping C++ field deserializer audit; reference field type sources not found at {}; set RIVE_RUNTIME_DIR",
            field_types_dir.display()
        );
        return;
    }

    for (class_name, source_name, expected_body) in [
        (
            "CoreUintType",
            "core_uint_type.cpp",
            "returnreader.readVarUintAs<unsignedint>();",
        ),
        (
            "CoreUint64Type",
            "core_uint64_type.cpp",
            "returnreader.readVarUint64();",
        ),
        (
            "CoreStringType",
            "core_string_type.cpp",
            "returnreader.readString();",
        ),
        (
            "CoreBytesType",
            "core_bytes_type.cpp",
            "returnreader.readBytes();",
        ),
        (
            "CoreDoubleType",
            "core_double_type.cpp",
            "returnreader.readFloat32();",
        ),
        (
            "CoreColorType",
            "core_color_type.cpp",
            "returnreader.readUint32();",
        ),
        (
            "CoreBoolType",
            "core_bool_type.cpp",
            "returnreader.readByte()==1;",
        ),
    ] {
        let path = field_types_dir.join(source_name);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&source);
        let body = compact_cpp_function_body(
            &compact,
            &format!("{class_name}::deserialize(BinaryReader&reader)"),
        )
        .unwrap_or_else(|| panic!("missing {class_name}::deserialize in {}", path.display()));

        assert_eq!(
            body, expected_body,
            "{class_name}::deserialize changed; audit BinaryReader primitive decoding parity before refreshing this list"
        );
    }
}

#[test]
fn cpp_constructor_stored_field_default_overrides_are_tracked_by_binary_model() {
    let src_dir = reference_runtime_dir().join("src");
    if !src_dir.exists() {
        eprintln!(
            "skipping C++ constructor default audit; reference src not found at {}; set RIVE_RUNTIME_DIR",
            src_dir.display()
        );
        return;
    }

    let mut cpp_files = Vec::new();
    collect_cpp_paths(&src_dir, &mut cpp_files);

    let mut actual = BTreeMap::new();
    for path in cpp_files {
        if path
            .components()
            .any(|component| component.as_os_str() == "generated")
        {
            continue;
        }

        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&contents);

        for definition in DEFINITIONS {
            let signature = format!("{}::{}()", definition.name, definition.name);
            let Some(constructor) = compact_cpp_constructor(&compact, &signature) else {
                continue;
            };

            for (member, property_name) in cpp_stored_members_in_hierarchy(definition) {
                if let Some(value) =
                    compact_cpp_member_assignment(&constructor.initializers, &member)
                        .or_else(|| compact_cpp_member_assignment(&constructor.body, &member))
                {
                    actual.insert(format!("{}.{}", definition.name, property_name), value);
                }
            }
        }
    }

    let expected = [("Artboard.clip".to_owned(), "true".to_owned())]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        actual, expected,
        "C++ constructor stored-field default override surface changed; audit RuntimeObject::stored_field_initializer, omitted-property tests, and docs before refreshing this list"
    );
}

#[test]
fn cpp_file_asset_helpers_match_binary_asset_metadata_model() {
    let assets_src_dir = reference_runtime_dir().join("src/assets");
    let assets_include_dir = reference_runtime_dir().join("include/rive/assets");
    let file_asset_path = assets_src_dir.join("file_asset.cpp");
    if !file_asset_path.exists() || !assets_include_dir.exists() {
        eprintln!(
            "skipping C++ file-asset helper audit; reference asset sources not found at {} and {}; set RIVE_RUNTIME_DIR",
            file_asset_path.display(),
            assets_include_dir.display()
        );
        return;
    }

    let file_asset = std::fs::read_to_string(&file_asset_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", file_asset_path.display()));
    let compact_file_asset = compact_cpp_source(&file_asset);

    let unique_name =
        compact_cpp_function_body(&compact_file_asset, "FileAsset::uniqueName()const")
            .unwrap_or_else(|| {
                panic!(
                    "missing FileAsset::uniqueName in {}",
                    file_asset_path.display()
                )
            });
    assert_compact_contains_in_order(
        &unique_name,
        &[
            "std::stringuniqueName=name();",
            "std::size_tfinalDot=uniqueName.rfind('.');",
            "if(finalDot!=std::string::npos){uniqueName=uniqueName.substr(0,finalDot);}",
            "returnuniqueName+\"-\"+std::to_string(assetId());",
        ],
        "FileAsset::uniqueName changed; audit RuntimeObject::file_asset_unique_name_bytes",
    );

    let unique_filename =
        compact_cpp_function_body(&compact_file_asset, "FileAsset::uniqueFilename()const")
            .unwrap_or_else(|| {
                panic!(
                    "missing FileAsset::uniqueFilename in {}",
                    file_asset_path.display()
                )
            });
    assert_eq!(
        unique_filename, "returnuniqueName()+\".\"+fileExtension();",
        "FileAsset::uniqueFilename changed; audit RuntimeObject::file_asset_unique_filename_bytes"
    );

    let decode_cdn_uuid = compact_cpp_function_body(
        &compact_file_asset,
        "FileAsset::decodeCdnUuid(Span<constuint8_t>value)",
    )
    .unwrap_or_else(|| {
        panic!(
            "missing FileAsset::decodeCdnUuid in {}",
            file_asset_path.display()
        )
    });
    assert_eq!(
        decode_cdn_uuid, "m_cdnUuid=std::vector<uint8_t>(value.begin(),value.end());",
        "FileAsset::decodeCdnUuid changed; audit raw cdnUuid payload preservation"
    );

    let cdn_uuid_str =
        compact_cpp_function_body(&compact_file_asset, "FileAsset::cdnUuidStr()const")
            .unwrap_or_else(|| {
                panic!(
                    "missing FileAsset::cdnUuidStr in {}",
                    file_asset_path.display()
                )
            });
    assert_compact_contains_in_order(
        &cdn_uuid_str,
        &[
            "constexpruint8_tuuidSize=16;",
            "if(m_cdnUuid.size()!=uuidSize){return\"\";}",
            "indices{3,2,1,0,5,4,7,6,9,8,15,14,13,12,11,10};",
            "ss<<std::hex<<std::setfill('0');",
            "for(intidx:indices){",
            "ss<<std::setw(2)",
            "<<static_cast<unsignedint>(m_cdnUuid[idx]);",
            "if(idx==0||idx==4||idx==6||idx==8)ss<<'-';",
            "returnss.str();",
        ],
        "FileAsset::cdnUuidStr changed; audit format_cpp_file_asset_cdn_uuid",
    );

    let actual_extensions = [
        (
            "ImageAsset",
            assets_src_dir.join("image_asset.cpp"),
            "std::stringImageAsset::fileExtension()const{return\"png\";}",
        ),
        (
            "FontAsset",
            assets_src_dir.join("font_asset.cpp"),
            "std::stringFontAsset::fileExtension()const{return\"ttf\";}",
        ),
        (
            "AudioAsset",
            assets_src_dir.join("audio_asset.cpp"),
            "std::stringAudioAsset::fileExtension()const{return\"wav\";}",
        ),
        (
            "BlobAsset",
            assets_src_dir.join("blob_asset.cpp"),
            "std::stringBlobAsset::fileExtension()const{return\"blob\";}",
        ),
        (
            "ScriptAsset",
            assets_include_dir.join("script_asset.hpp"),
            "std::stringfileExtension()constoverride{return\"lua\";}",
        ),
        (
            "ShaderAsset",
            assets_include_dir.join("shader_asset.hpp"),
            "std::stringfileExtension()constoverride{return\"rstb\";}",
        ),
        (
            "ManifestAsset",
            assets_src_dir.join("manifest_asset.cpp"),
            "std::stringManifestAsset::fileExtension()const{return\"man\";}",
        ),
    ]
    .into_iter()
    .map(|(class_name, path, expected_snippet)| {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let compact = compact_cpp_source(&source);
        assert!(
            compact.contains(expected_snippet),
            "{class_name} fileExtension changed in {}; audit cpp_file_asset_extension",
            path.display()
        );
        class_name.to_owned()
    })
    .collect::<BTreeSet<_>>();

    let expected_extensions = [
        "AudioAsset",
        "BlobAsset",
        "FontAsset",
        "ImageAsset",
        "ManifestAsset",
        "ScriptAsset",
        "ShaderAsset",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_extensions, expected_extensions,
        "C++ FileAsset extension override surface changed; audit cpp_file_asset_extension"
    );
}

#[test]
fn cpp_binary_reader_methods_match_runtime_reader_model() {
    let source_path = reference_runtime_dir().join("src/core/binary_reader.cpp");
    let header_path = reference_runtime_dir().join("include/rive/core/binary_reader.hpp");
    if !source_path.exists() || !header_path.exists() {
        eprintln!(
            "skipping C++ BinaryReader audit; reference reader not found at {} and {}; set RIVE_RUNTIME_DIR",
            source_path.display(),
            header_path.display()
        );
        return;
    }

    let source = std::fs::read_to_string(&source_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", source_path.display()));
    let compact_source = compact_cpp_source(&source);

    let reached_end = compact_cpp_function_body(&compact_source, "BinaryReader::reachedEnd()const")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::reachedEnd in {}",
                source_path.display()
            )
        });
    assert_eq!(
        reached_end, "returnm_Position==m_Bytes.end()||didOverflow()||didIntRangeError();",
        "BinaryReader end/error state changed; audit Rust reader EOF and malformed-stream handling"
    );

    let overflow = compact_cpp_function_body(&compact_source, "BinaryReader::overflow()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::overflow in {}",
                source_path.display()
            )
        });
    assert_eq!(
        overflow, "m_Overflowed=true;m_Position=m_Bytes.end();",
        "BinaryReader overflow behavior changed; audit Rust truncated-payload handling"
    );

    let int_range_error =
        compact_cpp_function_body(&compact_source, "BinaryReader::intRangeError()").unwrap_or_else(
            || {
                panic!(
                    "missing BinaryReader::intRangeError in {}",
                    source_path.display()
                )
            },
        );
    assert_eq!(
        int_range_error, "m_IntRangeError=true;m_Position=m_Bytes.end();",
        "BinaryReader integer range error behavior changed; audit Rust varuint range checks"
    );

    let read_var_uint = compact_cpp_function_body(&compact_source, "BinaryReader::readVarUint64()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readVarUint64 in {}",
                source_path.display()
            )
        });
    for expected in [
        "autoreadBytes=decode_uint_leb(m_Position,m_Bytes.end(),&value);",
        "if(readBytes==0){overflow();return0;}",
        "m_Position+=readBytes;returnvalue;",
    ] {
        assert!(
            read_var_uint.contains(expected),
            "BinaryReader::readVarUint64 changed; audit Rust varuint decoding and error semantics"
        );
    }

    let read_string_len =
        compact_cpp_function_body(&compact_source, "BinaryReader::readString(size_tlength)")
            .unwrap_or_else(|| {
                panic!(
                    "missing BinaryReader::readString(size_t) in {}",
                    source_path.display()
                )
            });
    for expected in [
        "decode_string(length,m_Position,m_Bytes.end(),&rawValue[0]);",
        "if(readBytes!=length){overflow();returnstd::string();}",
        "m_Position+=readBytes;returnstd::string(rawValue.data(),(size_t)length);",
    ] {
        assert!(
            read_string_len.contains(expected),
            "BinaryReader::readString(size_t) changed; audit Rust string decoding and overflow handling"
        );
    }

    let read_string = compact_cpp_function_body(&compact_source, "BinaryReader::readString()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readString in {}",
                source_path.display()
            )
        });
    assert_eq!(
        read_string,
        "uint64_tlength=readVarUint64();if(didOverflow()){returnstd::string();}returnreadString(length);",
        "BinaryReader::readString changed; audit Rust length-prefixed string decoding"
    );

    let read_bytes = compact_cpp_function_body(&compact_source, "BinaryReader::readBytes()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readBytes in {}",
                source_path.display()
            )
        });
    assert_eq!(
        read_bytes,
        "uint64_tlength=readVarUint64();if(didOverflow()){returnSpan<constuint8_t>(m_Position,0);}returnreadBytes((size_t)length);",
        "BinaryReader::readBytes changed; audit Rust length-prefixed bytes decoding"
    );

    let read_bytes_len =
        compact_cpp_function_body(&compact_source, "BinaryReader::readBytes(size_tlength)")
            .unwrap_or_else(|| {
                panic!(
                    "missing BinaryReader::readBytes(size_t) in {}",
                    source_path.display()
                )
            });
    assert_eq!(
        read_bytes_len,
        "constuint8_t*start=m_Position;m_Position+=length;return{start,(size_t)length};",
        "BinaryReader::readBytes(size_t) changed; audit Rust sized bytes reads and malformed-stream tests"
    );

    let read_float = compact_cpp_function_body(&compact_source, "BinaryReader::readFloat32()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readFloat32 in {}",
                source_path.display()
            )
        });
    for expected in [
        "autoreadBytes=decode_float(m_Position,m_Bytes.end(),&value);",
        "if(readBytes==0){overflow();return0.0f;}",
        "m_Position+=readBytes;returnvalue;",
    ] {
        assert!(
            read_float.contains(expected),
            "BinaryReader::readFloat32 changed; audit Rust f32 decoding and truncated fixed-width primitive tests"
        );
    }

    let read_byte = compact_cpp_function_body(&compact_source, "BinaryReader::readByte()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readByte in {}",
                source_path.display()
            )
        });
    assert_eq!(
        read_byte, "if(m_Bytes.end()-m_Position<1){overflow();return0;}return*m_Position++;",
        "BinaryReader::readByte changed; audit Rust byte decoding and bool semantics"
    );

    let read_u32 = compact_cpp_function_body(&compact_source, "BinaryReader::readUint32()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readUint32 in {}",
                source_path.display()
            )
        });
    for expected in [
        "autoreadBytes=decode_uint_32(m_Position,m_Bytes.end(),&value);",
        "if(readBytes==0){overflow();return0;}",
        "m_Position+=readBytes;returnvalue;",
    ] {
        assert!(
            read_u32.contains(expected),
            "BinaryReader::readUint32 changed; audit Rust u32/color decoding and truncated fixed-width primitive tests"
        );
    }

    let header = std::fs::read_to_string(&header_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header_path.display()));
    let compact_header = compact_cpp_source(&header);
    let read_var_uint_as = compact_cpp_function_body(&compact_header, "TreadVarUintAs()")
        .unwrap_or_else(|| {
            panic!(
                "missing BinaryReader::readVarUintAs in {}",
                header_path.display()
            )
        });
    assert_eq!(
        read_var_uint_as,
        "autovalue=this->readVarUint64();if(!fitsIn<T>(value)){value=0;this->intRangeError();}returnstatic_cast<T>(value);",
        "BinaryReader::readVarUintAs changed; audit Rust object/header/property integer range handling"
    );
}

#[test]
fn cpp_core_reader_decoders_match_binary_reader_model() {
    let path = reference_runtime_dir().join("include/rive/core/reader.h");
    if !path.exists() {
        eprintln!(
            "skipping C++ core reader decoder audit; reference reader.h not found at {}; set RIVE_RUNTIME_DIR",
            path.display()
        );
        return;
    }

    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let compact = compact_cpp_source(&source);

    let leb = compact_cpp_function_body(
        &compact,
        "decode_uint_leb(constuint8_t*buf,constuint8_t*buf_end,uint64_t*r)",
    )
    .unwrap_or_else(|| panic!("missing decode_uint_leb in {}", path.display()));
    assert_compact_contains_in_order(
        &leb,
        &[
            "constuint8_t*p=buf;",
            "uint8_tshift=0;",
            "uint64_tresult=0;",
            "uint8_tbyte;",
            "do{",
            "if(p>=buf_end){return0;}",
            "byte=*p++;",
            "result|=((uint64_t)(byte&0x7f))<<shift;",
            "shift+=7;",
            "}while((byte&0x80)!=0);",
            "*r=result;",
            "returnp-buf;",
        ],
        "decode_uint_leb changed; audit Rust LEB128 decoding, overwide encodings, and truncated varuint errors",
    );

    let leb32 = compact_cpp_function_body(
        &compact,
        "decode_uint_leb32(constuint8_t*buf,constuint8_t*buf_end,uint32_t*r)",
    )
    .unwrap_or_else(|| panic!("missing decode_uint_leb32 in {}", path.display()));
    assert_compact_contains_in_order(
        &leb32,
        &[
            "constuint8_t*p=buf;",
            "uint8_tshift=0;",
            "uint32_tresult=0;",
            "if(p>=buf_end){return0;}",
            "byte=*p++;",
            "result|=((uint32_t)(byte&0x7f))<<shift;",
            "shift+=7;",
            "}while((byte&0x80)!=0);",
            "*r=result;",
            "returnp-buf;",
        ],
        "decode_uint_leb32 changed; audit Rust embedded uint32 list decoding",
    );

    let decode_string = compact_cpp_function_body(
        &compact,
        "decode_string(uint64_tstr_len,constuint8_t*buf,constuint8_t*buf_end,char*char_buf)",
    )
    .unwrap_or_else(|| panic!("missing decode_string in {}", path.display()));
    assert_compact_contains_in_order(
        &decode_string,
        &[
            "if(buf_end<buf+str_len){return0;}",
            "constuint8_t*p=buf;",
            "for(inti=0;i<str_len;i++){char_buf[i]=*p++;}",
            "char_buf[str_len]='\\0';",
            "returnstr_len;",
        ],
        "decode_string changed; audit Rust length-prefixed string/bytes truncation and raw-byte preservation",
    );

    let decode_float = compact_cpp_function_body(
        &compact,
        "decode_float(constuint8_t*buf,constuint8_t*buf_end,float*r)",
    )
    .unwrap_or_else(|| panic!("missing decode_float in {}", path.display()));
    assert_compact_contains_in_order(
        &decode_float,
        &[
            "if(buf_end-buf<(unsigned)sizeof(float)){return0;}",
            "if(is_big_endian()){uint8_tinverted[4]={buf[3],buf[2],buf[1],buf[0]};memcpy(r,inverted,sizeof(float));}",
            "else{memcpy(r,buf,sizeof(float));}",
            "returnsizeof(float);",
        ],
        "decode_float changed; audit Rust f32 decoding and fixed-width truncation behavior",
    );

    let decode_u16 = compact_cpp_function_body(
        &compact,
        "decode_uint_16(constuint8_t*buf,constuint8_t*buf_end,uint16_t*r)",
    )
    .unwrap_or_else(|| panic!("missing decode_uint_16 in {}", path.display()));
    assert_compact_contains_in_order(
        &decode_u16,
        &[
            "if(buf_end-buf<(unsigned)sizeof(uint16_t)){return0;}",
            "if(is_big_endian()){uint8_tinverted[2]={buf[1],buf[0]};memcpy(r,inverted,sizeof(uint16_t));}",
            "else{memcpy(r,buf,sizeof(uint16_t));}",
            "returnsizeof(uint16_t);",
        ],
        "decode_uint_16 changed; audit Rust embedded uint16 buffer decoding",
    );

    let decode_u32 = compact_cpp_function_body(
        &compact,
        "decode_uint_32(constuint8_t*buf,constuint8_t*buf_end,uint32_t*r)",
    )
    .unwrap_or_else(|| panic!("missing decode_uint_32 in {}", path.display()));
    assert_compact_contains_in_order(
        &decode_u32,
        &[
            "if(buf_end-buf<(unsigned)sizeof(uint32_t)){return0;}",
            "if(is_big_endian()){uint8_tinverted[4]={buf[3],buf[2],buf[1],buf[0]};memcpy(r,inverted,sizeof(uint32_t));}",
            "else{memcpy(r,buf,sizeof(uint32_t));}",
            "returnsizeof(uint32_t);",
        ],
        "decode_uint_32 changed; audit Rust color/header fixed-width decoding",
    );
}

#[test]
fn cpp_read_runtime_object_fallback_switch_matches_skip_model() {
    let path = reference_runtime_dir().join("src/file.cpp");
    if !path.exists() {
        eprintln!(
            "skipping C++ readRuntimeObject fallback audit; reference file.cpp not found at {}; set RIVE_RUNTIME_DIR",
            path.display()
        );
        return;
    }

    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let compact = compact_cpp_source(&source);
    let body = compact_cpp_function_body(
        &compact,
        "readRuntimeObject(BinaryReader&reader,constRuntimeHeader&header)",
    )
    .unwrap_or_else(|| panic!("missing readRuntimeObject in {}", path.display()));

    assert_compact_contains_in_order(
        &body,
        &[
            "autocoreObjectKey=reader.readVarUintAs<int>();",
            "autoobject=CoreRegistry::makeCoreInstance(coreObjectKey);",
            "while(true){",
            "autopropertyKey=reader.readVarUintAs<uint16_t>();",
            "if(propertyKey==0){",
            "break;",
            "if(reader.hasError()){deleteobject;returnnullptr;}",
            "if(object==nullptr||!object->deserialize(propertyKey,reader)){",
            "intid=CoreRegistry::propertyFieldId(propertyKey);",
            "if(id==-1){",
            "id=header.propertyFieldId(propertyKey);}",
            "if(id==-1){",
            "deleteobject;",
            "returnnullptr;",
            "switch(id){",
        ],
        "readRuntimeObject flow changed; audit Rust object construction, generated-deserialize dispatch, and unknown-property fallback",
    );
    assert!(
        body.contains("autocoreObjectKey=reader.readVarUintAs<int>();"),
        "readRuntimeObject object-key integer semantics changed; audit Rust object type-key decoding"
    );
    assert!(
        body.contains("autopropertyKey=reader.readVarUintAs<uint16_t>();"),
        "readRuntimeObject property-key integer semantics changed; audit Rust property-key decoding"
    );
    assert!(
        body.contains("intid=CoreRegistry::propertyFieldId(propertyKey);"),
        "readRuntimeObject no longer checks CoreRegistry before header ToC; audit unknown-property skipping"
    );
    assert!(
        body.contains("id=header.propertyFieldId(propertyKey);"),
        "readRuntimeObject header ToC fallback changed; audit unknown-property skipping"
    );

    let cases = compact_cpp_switch_cases(&body, "switch(id)").unwrap_or_else(|| {
        panic!(
            "missing readRuntimeObject fallback switch in {}",
            path.display()
        )
    });
    let expected = [
        "CoreColorType::id",
        "CoreDoubleType::id",
        "CoreStringType::id",
        "CoreUintType::id",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    assert_eq!(
        cases, expected,
        "readRuntimeObject fallback switch changed; audit skip_core_registry_value and bool/string/bytes fallback tests before refreshing this list"
    );

    assert_compact_contains_in_order(
        &body,
        &[
            "caseCoreUintType::id:",
            "reader.readVarUint64();",
            "break;",
            "caseCoreStringType::id:CoreStringType::deserialize(reader);break;",
            "caseCoreDoubleType::id:CoreDoubleType::deserialize(reader);break;",
            "caseCoreColorType::id:CoreColorType::deserialize(reader);break;",
            "if(object==nullptr){",
            "returnnullptr;",
            "returnobject;",
        ],
        "readRuntimeObject null-object completion changed; audit Rust abstract/unknown object slot handling",
    );
}

fn read_cpp_probe_file(probe: &Path, fixture_path: &str) -> CppProbeFile {
    read_cpp_probe_file_inner(probe, fixture_path, false)
}

fn read_cpp_probe_reference_file(probe: &Path, fixture_path: &str) -> CppProbeFile {
    let path = reference_fixture(fixture_path);
    read_cpp_probe_path_inner(probe, &path, fixture_path, false)
}

fn read_cpp_probe_reference_file_with_property_values(
    probe: &Path,
    fixture_path: &str,
) -> CppProbeFile {
    let path = reference_fixture(fixture_path);
    read_cpp_probe_path_inner(probe, &path, fixture_path, true)
}

fn read_cpp_probe_file_with_property_values(probe: &Path, fixture_path: &str) -> CppProbeFile {
    read_cpp_probe_file_inner(probe, fixture_path, true)
}

fn read_cpp_probe_file_inner(
    probe: &Path,
    fixture_path: &str,
    property_values: bool,
) -> CppProbeFile {
    let path = fixture(fixture_path);
    read_cpp_probe_path_inner(probe, &path, fixture_path, property_values)
}

fn read_cpp_probe_path_inner(
    probe: &Path,
    path: &Path,
    label: &str,
    property_values: bool,
) -> CppProbeFile {
    let property_value_mode = if property_values {
        CppProbePropertyValueMode::All
    } else {
        CppProbePropertyValueMode::None
    };
    read_cpp_probe_path_with_options(probe, path, label, property_value_mode, false, false)
}

#[derive(Clone, Copy)]
enum CppProbePropertyValueMode {
    None,
    All,
}

fn read_cpp_probe_path_with_options(
    probe: &Path,
    path: &Path,
    label: &str,
    property_value_mode: CppProbePropertyValueMode,
    no_advance: bool,
    complete_view_model_properties: bool,
) -> CppProbeFile {
    read_cpp_probe_path_with_extra_options(
        probe,
        path,
        label,
        property_value_mode,
        no_advance,
        complete_view_model_properties,
        false,
    )
}

fn read_cpp_probe_path_with_extra_options(
    probe: &Path,
    path: &Path,
    label: &str,
    property_value_mode: CppProbePropertyValueMode,
    no_advance: bool,
    complete_view_model_properties: bool,
    data_context_lookups: bool,
) -> CppProbeFile {
    let mut command = Command::new(probe);
    match property_value_mode {
        CppProbePropertyValueMode::None => {}
        CppProbePropertyValueMode::All => {
            command.arg("--property-values");
        }
    }
    if no_advance {
        command.arg("--no-advance");
    }
    if complete_view_model_properties {
        command.arg("--complete-view-model-properties");
    }
    if data_context_lookups {
        command.arg("--data-context-lookups");
    }
    let output = command
        .arg("--file")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    assert!(
        output.status.success(),
        "C++ probe failed for {label}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid probe JSON for {label}: {err}"))
}

fn read_cpp_converter_samples(probe: &Path) -> CppProbeConverterSamples {
    let output = Command::new(probe)
        .arg("--converter-samples")
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    assert!(
        output.status.success(),
        "C++ converter sample probe failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid converter sample probe JSON: {err}"))
}

fn read_cpp_number_to_list_samples(probe: &Path, path: &Path) -> CppProbeNumberToListSamples {
    let output = Command::new(probe)
        .arg("--number-to-list-samples")
        .arg("--file")
        .arg(path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    assert!(
        output.status.success(),
        "C++ number-to-list sample probe failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid number-to-list sample probe JSON: {err}"))
}

fn read_rust_runtime_file(fixture_path: &str) -> RuntimeFile {
    let path = fixture(fixture_path);
    read_rust_runtime_path(&path)
}

fn read_rust_reference_runtime_file(fixture_path: &str) -> RuntimeFile {
    let path = reference_fixture(fixture_path);
    read_rust_runtime_path(&path)
}

fn read_rust_runtime_path(path: &Path) -> RuntimeFile {
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    read_runtime_file(&bytes).unwrap_or_else(|err| {
        panic!("failed to import fixture {}: {err:#}", path.display());
    })
}

fn runtime_artboards(file: &RuntimeFile) -> Vec<&RuntimeObject> {
    file.artboards()
}

fn assert_float_eq(expected: f32, actual: Option<f32>, label: &str) {
    let actual = actual.unwrap_or_else(|| panic!("{label} missing in Rust import"));
    assert!(
        (expected - actual).abs() <= 0.0001,
        "{label} mismatch: C++ {expected}, Rust {actual}"
    );
}

fn float_matches(expected: f32, actual: Option<f32>) -> bool {
    actual.is_some_and(|actual| {
        if expected.is_nan() {
            actual.is_nan()
        } else if expected.is_infinite() {
            actual == expected
        } else {
            (expected - actual).abs() <= 0.0001
        }
    })
}

fn runtime_artboard_ranges(file: &RuntimeFile) -> Vec<(usize, usize)> {
    let starts = file
        .objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| match object {
            Some(object) if object.type_name == "Artboard" => Some(index),
            _ => None,
        })
        .collect::<Vec<_>>();

    starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            (
                *start,
                starts.get(index + 1).copied().unwrap_or(file.objects.len()),
            )
        })
        .collect()
}

fn runtime_artboard_range_by_index(
    file: &RuntimeFile,
    artboard_index: usize,
) -> Option<(usize, usize)> {
    runtime_artboard_ranges(file).get(artboard_index).copied()
}

fn runtime_artboard_local_objects(
    file: &RuntimeFile,
    range: (usize, usize),
) -> Vec<Option<&RuntimeObject>> {
    let mut objects = file.objects[range.0..range.1]
        .iter()
        .filter_map(|object| match object {
            None => Some(None),
            Some(object) if runtime_object_is_artboard_local(object) => Some(Some(object)),
            Some(_) => None,
        })
        .collect::<Vec<_>>();
    validate_runtime_artboard_local_objects(&mut objects);
    objects
}

fn validate_runtime_artboard_local_objects(objects: &mut [Option<&RuntimeObject>]) {
    loop {
        let mut changed = false;
        for index in 1..objects.len() {
            let Some(object) = objects[index] else {
                continue;
            };
            if !runtime_artboard_local_object_is_valid(object, objects) {
                objects[index] = None;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }
}

fn runtime_artboard_local_object_is_valid(
    object: &RuntimeObject,
    objects: &[Option<&RuntimeObject>],
) -> bool {
    let Some(definition) = definition_by_type_key(object.type_key) else {
        return false;
    };

    if definition.name == "Artboard" {
        return true;
    }

    if definition.is_a("Component") {
        let Some(parent) = local_object_reference(objects, object.uint_property("parentId")) else {
            return false;
        };
        if !runtime_object_is_container_component(parent) {
            return false;
        }
    }

    if definition.is_a("TargetedConstraint") {
        match local_object_reference(objects, object.uint_property("targetId")) {
            Some(target) => return runtime_object_is_transform_component(target),
            None => return !targeted_constraint_requires_target(definition.name),
        }
    }

    if definition.is_a("NestedAnimation") {
        let Some(parent) = local_object_reference(objects, object.uint_property("parentId")) else {
            return false;
        };
        return runtime_object_is_nested_artboard(parent);
    }

    if definition.is_a("TextStyle") {
        let Some(parent) = local_object_reference(objects, object.uint_property("parentId")) else {
            return false;
        };
        return runtime_object_is_text_interface(parent);
    }

    if definition.name == "ScrollBarConstraint" {
        let Some(scroll_constraint) =
            local_object_reference(objects, object.uint_property("scrollConstraintId"))
        else {
            return false;
        };
        return runtime_object_is_scroll_constraint(scroll_constraint);
    }

    if definition.name == "Feather" {
        let Some(parent) = local_object_reference(objects, object.uint_property("parentId")) else {
            return false;
        };
        return runtime_object_is_shape_paint(parent);
    }

    if definition.name == "ArtboardListMapRule" {
        let Some(parent) = local_object_reference(objects, object.uint_property("parentId")) else {
            return false;
        };
        return runtime_object_is_artboard_component_list(parent);
    }

    true
}

fn local_object_reference<'a>(
    objects: &[Option<&'a RuntimeObject>],
    id: Option<u64>,
) -> Option<&'a RuntimeObject> {
    let id = usize::try_from(id?).ok()?;
    objects.get(id).and_then(|object| *object)
}

fn runtime_file_assets(file: &RuntimeFile) -> Vec<&RuntimeObject> {
    file.file_assets()
}

fn runtime_file_view_models(file: &RuntimeFile) -> Vec<nuxie_binary::RuntimeViewModel<'_>> {
    file.view_models()
}

fn runtime_data_enums(file: &RuntimeFile) -> Vec<nuxie_binary::RuntimeDataEnum<'_>> {
    file.data_enums()
}

fn resolved_artboard_index_for_referencer(
    file: &RuntimeFile,
    object: &RuntimeObject,
) -> Option<usize> {
    let resolved = file.resolved_artboard_for_referencer_object(object)?;
    file.artboards()
        .into_iter()
        .position(|artboard| artboard.id == resolved.id)
}

fn data_bind_path_ids_for_referencer(
    file: &RuntimeFile,
    object: &RuntimeObject,
) -> Option<Vec<u32>> {
    file.data_bind_path_for_referencer_object(object)
        .map(|path| path.path_ids)
}

fn resolved_data_bind_path_ids_for_referencer(
    file: &RuntimeFile,
    object: &RuntimeObject,
) -> Option<Vec<u32>> {
    file.resolved_data_bind_path_ids_for_referencer_object(object)
}

fn scroll_physics_core_type_for_constraint(
    file: &RuntimeFile,
    object: &RuntimeObject,
) -> Option<u16> {
    file.resolved_scroll_physics_for_constraint_object(object)
        .map(|physics| physics.type_key)
}

fn runtime_object_is_artboard_local(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| {
        // Match import ownership, not just schema ancestry. C++ routes user
        // inputs into Artboard::objects(), while scripted inputs are usually
        // owned by ScriptedObject importers despite inheriting Component.
        (definition.is_a("Component")
            && !definition.name.starts_with("ScriptInput")
            && !definition.is_a("ScrollPhysics"))
            || definition.is_a("KeyFrameInterpolator")
            || definition.is_a("UserInput")
    })
}

fn runtime_object_is_container_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("ContainerComponent"))
}

fn runtime_object_is_artboard_component_list(object: &RuntimeObject) -> bool {
    object.type_name == "ArtboardComponentList"
}

fn runtime_object_is_transform_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("TransformComponent"))
}

fn runtime_object_is_nested_artboard(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("NestedArtboard"))
}

fn runtime_object_is_text_interface(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| matches!(definition.name, "Text" | "TextInput"))
}

fn runtime_object_is_scroll_constraint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.name == "ScrollConstraint")
}

fn runtime_object_is_shape_paint(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("ShapePaint"))
}

fn targeted_constraint_requires_target(type_name: &str) -> bool {
    !matches!(
        type_name,
        "RotationConstraint" | "ScaleConstraint" | "TranslationConstraint"
    )
}

fn runtime_object_is_component(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| definition.is_a("Component"))
}

#[test]
fn cpp_probe_agrees_on_unknown_property_null_object_fallback_when_available() {
    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ binary import comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let cases = [
        ("empty_file_header", Vec::new()),
        ("truncated_fingerprint_header", b"RIV".to_vec()),
        ("bad_fingerprint_header", b"RIVX\x07\x00\x00\x00".to_vec()),
        ("missing_header_major_version", b"RIVE".to_vec()),
        ("missing_header_minor_version", b"RIVE\x07".to_vec()),
        ("missing_header_file_id", b"RIVE\x07\x00".to_vec()),
        (
            "missing_header_property_toc_terminator",
            b"RIVE\x07\x00\x00".to_vec(),
        ),
        ("noncanonical_header_varuint_imports", {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(b"RIVE");
            bytes.extend_from_slice(&[0x87, 0x00]);
            push_var_uint(&mut bytes, 0);
            push_var_uint(&mut bytes, 0);
            push_var_uint(&mut bytes, 0);
            bytes
        }),
        ("overwide_header_varuint_imports", {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(b"RIVE");
            push_overwide_noncanonical_var_uint(&mut bytes, 7);
            push_var_uint(&mut bytes, 0);
            push_var_uint(&mut bytes, 0);
            push_overwide_noncanonical_var_uint(&mut bytes, 0);
            bytes
        }),
        (
            "overwide_object_property_string_length_varuints_import",
            synthetic_runtime_file(6037, |bytes| {
                push_overwide_noncanonical_var_uint(bytes, 2);
                push_overwide_noncanonical_var_uint(bytes, 4);
                push_overwide_noncanonical_var_uint(bytes, 2);
                bytes.extend_from_slice(b"ok");
                push_overwide_noncanonical_var_uint(bytes, 0);
            }),
        ),
        ("truncated_header_toc_field_ids", {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(b"RIVE");
            push_var_uint(&mut bytes, 7);
            push_var_uint(&mut bytes, 0);
            push_var_uint(&mut bytes, 0);
            push_var_uint(&mut bytes, 4);
            push_var_uint(&mut bytes, 0);
            bytes.extend_from_slice(&[0x00, 0x00, 0x00]);
            bytes
        }),
        (
            "header_major_i32_overflow",
            synthetic_runtime_header(i32::MAX as u64 + 1, 0, 0, &[]),
        ),
        (
            "header_minor_i32_overflow",
            synthetic_runtime_header(7, i32::MAX as u64 + 1, 0, &[]),
        ),
        (
            "header_file_id_i32_overflow",
            synthetic_runtime_header(7, 0, i32::MAX as u64 + 1, &[]),
        ),
        (
            "header_toc_key_i32_overflow",
            synthetic_runtime_header(7, 0, 0, &[i32::MAX as u64 + 1]),
        ),
        (
            "header_toc_key_i32_max_imports",
            synthetic_runtime_header(7, 0, 0, &[i32::MAX as u64]),
        ),
        (
            "duplicate_header_toc_key_later_field_wins",
            synthetic_runtime_file_with_toc_fields(6036, &[(65_000, 0), (65_000, 2)], |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 65_000);
                bytes.extend_from_slice(&12.5f32.to_le_bytes());
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "core_registry_fallback_beats_conflicting_header_toc",
            synthetic_runtime_file_with_toc_fields(6038, &[(13, 0)], |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 13);
                bytes.extend_from_slice(&12.5f32.to_le_bytes());
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "known_object_alternate_key_core_registry_beats_conflicting_header_toc",
            synthetic_runtime_file_with_toc_fields(6039, &[(9, 0)], |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, 9);
                bytes.extend_from_slice(&12.5f32.to_le_bytes());
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "unknown_object_type_i32_max_imports_as_null",
            synthetic_runtime_file(6022, |bytes| {
                push_var_uint(bytes, i32::MAX as u64);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "property_key_u16_max_missing_toc_imports_as_null",
            synthetic_runtime_file(6023, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, u16::MAX as u64);
            }),
        ),
        ("property_key_u16_max_header_toc_skips", {
            let mut bytes = synthetic_runtime_header(7, 0, 6024, &[u16::MAX as u64]);
            push_var_uint(&mut bytes, 70_000);
            push_var_uint(&mut bytes, u16::MAX as u64);
            push_var_uint(&mut bytes, u32::MAX as u64);
            push_var_uint(&mut bytes, 0);
            bytes
        }),
        (
            "missing_toc_at_eof",
            synthetic_runtime_file(6000, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 65_000);
            }),
        ),
        (
            "missing_toc_then_terminator",
            synthetic_runtime_file(6001, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 65_000);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "known_bool_fallback_at_eof",
            synthetic_runtime_file(6002, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 245);
                bytes.push(1);
            }),
        ),
        (
            "known_bool_fallback_payload_then_terminator",
            synthetic_runtime_file(6040, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 245);
                bytes.push(1);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "callback_fallback_at_eof",
            synthetic_runtime_file(6003, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 401);
            }),
        ),
        (
            "callback_fallback_then_terminator",
            synthetic_runtime_file(6004, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 401);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "bitmask_passthrough_fallback_at_eof",
            synthetic_runtime_file(6008, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 1027);
            }),
        ),
        (
            "bitmask_passthrough_fallback_then_terminator",
            synthetic_runtime_file(6009, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 1027);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "uint_value_u32_max",
            synthetic_runtime_file(6005, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 5);
                push_var_uint(bytes, u32::MAX as u64);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "uint_value_u32_overflow",
            synthetic_runtime_file(6006, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 5);
                push_var_uint(bytes, u64::from(u32::MAX) + 1);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "invalid_utf8_string_imports",
            synthetic_runtime_file(6007, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 4);
                push_var_uint(bytes, 2);
                bytes.extend_from_slice(&[0xff, 0xfe]);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "noncanonical_string_length_imports",
            synthetic_runtime_file(6034, |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, 4);
                bytes.extend_from_slice(&[0x82, 0x00]);
                bytes.extend_from_slice(b"ok");
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "property_key_u16_overflow",
            synthetic_runtime_file(6010, |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, u64::from(u16::MAX) + 1);
            }),
        ),
        (
            "object_type_i32_overflow",
            synthetic_runtime_file(6018, |bytes| {
                push_var_uint(bytes, i32::MAX as u64 + 1);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "known_object_alternate_key_imports_by_fallback",
            synthetic_runtime_file(6011, |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, 9);
                bytes.extend_from_slice(&12.5f32.to_le_bytes());
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "duplicate_stored_property_imports",
            synthetic_runtime_file(6030, |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, 13);
                bytes.extend_from_slice(&1.25f32.to_le_bytes());
                push_var_uint(bytes, 13);
                bytes.extend_from_slice(&2.5f32.to_le_bytes());
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "omitted_stored_properties_use_generated_defaults",
            synthetic_runtime_file(6031, |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, 0);
                push_var_uint(bytes, 18);
                push_var_uint(bytes, 0);
                push_var_uint(bytes, 653);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "abstract_object_imports_as_null_slot",
            synthetic_runtime_file(6012, |bytes| {
                push_var_uint(bytes, 10);
                push_var_uint(bytes, 4);
                push_var_uint(bytes, 8);
                bytes.extend_from_slice(b"abstract");
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "known_object_bitmask_passthrough_fallback_at_eof",
            synthetic_runtime_file(6016, |bytes| {
                push_var_uint(bytes, 134);
                push_var_uint(bytes, 1027);
            }),
        ),
        (
            "known_object_callback_fallback_at_eof",
            synthetic_runtime_file(6017, |bytes| {
                push_var_uint(bytes, 122);
                push_var_uint(bytes, 401);
            }),
        ),
        (
            "truncated_known_double_field",
            synthetic_runtime_file(6025, |bytes| {
                push_var_uint(bytes, 2);
                push_var_uint(bytes, 13);
                bytes.extend_from_slice(&[0x00, 0x00]);
            }),
        ),
        (
            "truncated_unknown_double_fallback",
            synthetic_runtime_file(6026, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 13);
                bytes.extend_from_slice(&[0x00, 0x00]);
            }),
        ),
        (
            "truncated_header_toc_double_fallback",
            synthetic_runtime_file_with_single_toc_field(6027, 65_000, 2, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 65_000);
                bytes.extend_from_slice(&[0x00, 0x00]);
            }),
        ),
        (
            "truncated_known_color_field",
            synthetic_runtime_file(6028, |bytes| {
                push_var_uint(bytes, 18);
                push_var_uint(bytes, 37);
                bytes.extend_from_slice(&[0x00, 0x00]);
            }),
        ),
        (
            "truncated_known_bool_field",
            synthetic_runtime_file(6029, |bytes| {
                push_var_uint(bytes, 129);
                push_var_uint(bytes, 245);
            }),
        ),
        (
            "noncanonical_bool_payload_imports_as_false",
            synthetic_runtime_file(6033, |bytes| {
                push_var_uint(bytes, 653);
                push_var_uint(bytes, 953);
                bytes.push(2);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "known_bytes_field_truncated_at_eof",
            synthetic_runtime_file(6019, |bytes| {
                push_var_uint(bytes, 109);
                push_var_uint(bytes, 223);
                push_var_uint(bytes, 4);
                bytes.extend_from_slice(&[1, 2]);
            }),
        ),
        (
            "known_bytes_field_truncated_then_terminator",
            synthetic_runtime_file(6020, |bytes| {
                push_var_uint(bytes, 109);
                push_var_uint(bytes, 223);
                push_var_uint(bytes, 4);
                bytes.extend_from_slice(&[1, 2]);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "mesh_missing_triangle_indices_malformed",
            synthetic_mesh_runtime_file(6043, None),
        ),
        (
            "mesh_out_of_range_triangle_index_malformed",
            synthetic_mesh_runtime_file(6044, Some(&[1])),
        ),
        (
            "mesh_valid_triangle_index_imports",
            synthetic_mesh_runtime_file(6045, Some(&[0])),
        ),
        (
            "mesh_valid_contour_vertex_triangle_index_imports",
            synthetic_mesh_runtime_file_with_vertex(6046, Some(&[0]), "ContourMeshVertex"),
        ),
        (
            "constraint_valid_transform_parent_imports",
            synthetic_constraint_runtime_file(6065, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "RotationConstraint", |bytes| {
                    push_uint_property(bytes, "RotationConstraint", "parentId", 1);
                    push_uint_property(bytes, "RotationConstraint", "targetId", 0);
                });
            }),
        ),
        (
            "constraint_wrong_transform_parent_malformed",
            synthetic_constraint_runtime_file(6066, |bytes| {
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
            }),
        ),
        (
            "ik_constraint_wrong_bone_parent_malformed",
            synthetic_constraint_runtime_file(6067, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "IKConstraint", |bytes| {
                    push_uint_property(bytes, "IKConstraint", "parentId", 1);
                    push_uint_property(bytes, "IKConstraint", "targetId", 0);
                });
            }),
        ),
        (
            "targeted_constraint_invalid_target_pruned_before_parent_malformed",
            synthetic_constraint_runtime_file(6070, |bytes| {
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
            }),
        ),
        (
            "joystick_missing_handle_source_imports_without_advance",
            synthetic_runtime_file(6102, |bytes| {
                push_empty_object(bytes, "Backboard");
                push_empty_object(bytes, "Artboard");
                push_object_with_properties(bytes, "Joystick", |bytes| {
                    push_uint_property(bytes, "Joystick", "parentId", 0);
                    push_uint_property(bytes, "Joystick", "handleSourceId", 99);
                });
            }),
        ),
        (
            "n_slicer_valid_parentage_imports",
            synthetic_n_slicer_runtime_file(6076, |bytes| {
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
                push_object_with_properties(bytes, "NSlicedNode", |bytes| {
                    push_uint_property(bytes, "NSlicedNode", "parentId", 0);
                });
                push_object_with_properties(bytes, "AxisX", |bytes| {
                    push_uint_property(bytes, "AxisX", "parentId", 6);
                });
            }),
        ),
        (
            "n_slicer_wrong_image_parent_keeps_slot_without_mesh_hookup",
            synthetic_n_slicer_runtime_file(6077, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "NSlicer", |bytes| {
                    push_uint_property(bytes, "NSlicer", "parentId", 1);
                });
            }),
        ),
        (
            "axis_wrong_n_slicer_parent_keeps_slot_without_registration",
            synthetic_n_slicer_runtime_file(6078, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "AxisX", |bytes| {
                    push_uint_property(bytes, "AxisX", "parentId", 1);
                });
            }),
        ),
        (
            "n_slicer_tile_mode_wrong_parent_keeps_slot_without_registration",
            synthetic_n_slicer_runtime_file(6079, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "NSlicerTileMode", |bytes| {
                    push_uint_property(bytes, "NSlicerTileMode", "parentId", 1);
                });
            }),
        ),
        (
            "trim_path_valid_mode_imports",
            synthetic_paint_effect_runtime_file(6068, |bytes| {
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
            }),
        ),
        (
            "trim_path_invalid_mode_malformed",
            synthetic_paint_effect_runtime_file(6069, |bytes| {
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
            }),
        ),
        (
            "paint_effect_valid_parentage_imports",
            synthetic_paint_effect_runtime_file(6057, |bytes| {
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
            }),
        ),
        (
            "paint_effect_dash_wrong_parent_malformed",
            synthetic_paint_effect_runtime_file(6058, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "Fill", |bytes| {
                    push_uint_property(bytes, "Fill", "parentId", 1);
                });
                push_object_with_properties(bytes, "Dash", |bytes| {
                    push_uint_property(bytes, "Dash", "parentId", 2);
                });
            }),
        ),
        (
            "paint_effect_wrong_effects_container_parent_malformed",
            synthetic_paint_effect_runtime_file(6059, |bytes| {
                push_object_with_properties(bytes, "DashPath", |bytes| {
                    push_uint_property(bytes, "DashPath", "parentId", 0);
                });
            }),
        ),
        (
            "paint_effect_duplicate_mutator_malformed",
            synthetic_paint_effect_runtime_file(6060, |bytes| {
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
            }),
        ),
        (
            "text_valid_parentage_imports",
            synthetic_text_runtime_file(6061, |bytes| {
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
            }),
        ),
        (
            "text_value_run_valid_parent_and_style_imports",
            synthetic_text_runtime_file(6073, |bytes| {
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
            }),
        ),
        (
            "text_value_run_non_text_parent_imports",
            synthetic_text_runtime_file(6074, |bytes| {
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
            }),
        ),
        (
            "text_value_run_non_paint_style_imports",
            synthetic_text_runtime_file(6075, |bytes| {
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
            }),
        ),
        (
            "text_style_feature_wrong_parent_malformed",
            synthetic_text_runtime_file(6062, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "TextStyleFeature", |bytes| {
                    push_uint_property(bytes, "TextStyleFeature", "parentId", 1);
                });
            }),
        ),
        (
            "text_style_axis_wrong_parent_malformed",
            synthetic_text_runtime_file(6063, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "TextStyleAxis", |bytes| {
                    push_uint_property(bytes, "TextStyleAxis", "parentId", 1);
                });
            }),
        ),
        (
            "text_input_drawable_wrong_parent_malformed",
            synthetic_text_runtime_file(6064, |bytes| {
                push_object_with_properties(bytes, "Shape", |bytes| {
                    push_uint_property(bytes, "Shape", "parentId", 0);
                });
                push_object_with_properties(bytes, "TextInputText", |bytes| {
                    push_uint_property(bytes, "TextInputText", "parentId", 1);
                });
            }),
        ),
        (
            "state_machine_layer_missing_required_state_malformed",
            synthetic_runtime_file(6055, |bytes| {
                push_empty_object(bytes, "Backboard");
                push_empty_object(bytes, "Artboard");
                push_empty_object(bytes, "StateMachine");
                push_empty_object(bytes, "StateMachineLayer");
                push_empty_object(bytes, "AnyState");
                push_empty_object(bytes, "EntryState");
            }),
        ),
        (
            "state_transition_invalid_state_to_no_placeholder",
            synthetic_runtime_file(6050, |bytes| {
                push_empty_object(bytes, "Backboard");
                push_empty_object(bytes, "Artboard");
                push_empty_object(bytes, "StateMachine");
                push_empty_object(bytes, "StateMachineLayer");
                push_empty_object(bytes, "AnyState");
                push_object_with_properties(bytes, "StateTransition", |bytes| {
                    push_uint_property(bytes, "StateTransition", "stateToId", 3);
                });
                push_empty_object(bytes, "EntryState");
                push_empty_object(bytes, "ExitState");
            }),
        ),
        (
            "state_transition_valid_state_to_with_placeholder",
            synthetic_runtime_file(6051, |bytes| {
                push_empty_object(bytes, "Backboard");
                push_empty_object(bytes, "Artboard");
                push_empty_object(bytes, "StateMachine");
                push_empty_object(bytes, "StateMachineLayer");
                push_empty_object(bytes, "AnyState");
                push_object_with_properties(bytes, "StateTransition", |bytes| {
                    push_uint_property(bytes, "StateTransition", "stateToId", 3);
                });
                push_empty_object(bytes, "LayerState");
                push_empty_object(bytes, "EntryState");
                push_empty_object(bytes, "ExitState");
            }),
        ),
        (
            "state_transition_invalid_state_to_even_with_placeholder",
            synthetic_runtime_file(6052, |bytes| {
                push_empty_object(bytes, "Backboard");
                push_empty_object(bytes, "Artboard");
                push_empty_object(bytes, "StateMachine");
                push_empty_object(bytes, "StateMachineLayer");
                push_empty_object(bytes, "AnyState");
                push_object_with_properties(bytes, "StateTransition", |bytes| {
                    push_uint_property(bytes, "StateTransition", "stateToId", 4);
                });
                push_empty_object(bytes, "LayerState");
                push_empty_object(bytes, "EntryState");
                push_empty_object(bytes, "ExitState");
            }),
        ),
        (
            "state_transition_valid_interpolator_imports",
            synthetic_runtime_file(6083, |bytes| {
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
                    push_uint_property(bytes, "StateTransition", "interpolatorId", 1);
                });
                push_empty_object(bytes, "EntryState");
                push_empty_object(bytes, "ExitState");
            }),
        ),
        (
            "state_transition_wrong_interpolator_kind_imports_unresolved",
            synthetic_runtime_file(6084, |bytes| {
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
                    push_uint_property(bytes, "StateTransition", "interpolatorId", 2);
                });
                push_empty_object(bytes, "EntryState");
                push_empty_object(bytes, "ExitState");
            }),
        ),
        (
            "state_transition_missing_interpolator_imports_unresolved",
            synthetic_runtime_file(6085, |bytes| {
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
                    push_uint_property(bytes, "StateTransition", "interpolatorId", 99);
                });
                push_empty_object(bytes, "EntryState");
                push_empty_object(bytes, "ExitState");
            }),
        ),
        (
            "unknown_bytes_fallback_truncated_at_eof",
            synthetic_runtime_file(6021, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 212);
                push_var_uint(bytes, 4);
                bytes.extend_from_slice(&[1, 2]);
            }),
        ),
        (
            "unknown_bytes_fallback_with_terminator_imports",
            synthetic_runtime_file(6032, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 212);
                push_var_uint(bytes, 4);
                bytes.extend_from_slice(&[0x00, 0xff, 0x10, 0x7f]);
                push_var_uint(bytes, 0);
            }),
        ),
        (
            "noncanonical_bytes_fallback_length_imports",
            synthetic_runtime_file(6035, |bytes| {
                push_var_uint(bytes, 70_000);
                push_var_uint(bytes, 212);
                bytes.extend_from_slice(&[0x84, 0x00]);
                bytes.extend_from_slice(&[0x00, 0xff, 0x10, 0x7f]);
                push_var_uint(bytes, 0);
            }),
        ),
        ("truncated_header_varuint", b"RIVE\x80".to_vec()),
        ("truncated_object_type_varuint", {
            let mut bytes = runtime_header_prefix(6013);
            bytes.push(0x80);
            bytes
        }),
        ("truncated_property_key_varuint", {
            let mut bytes = runtime_header_prefix(6014);
            push_var_uint(&mut bytes, 2);
            bytes.push(0x80);
            bytes
        }),
        ("truncated_string_length_varuint", {
            let mut bytes = runtime_header_prefix(6015);
            push_var_uint(&mut bytes, 2);
            push_var_uint(&mut bytes, 4);
            bytes.push(0x80);
            bytes
        }),
    ];

    for (name, bytes) in cases {
        let cpp_result = cpp_import_result(&probe, name, &bytes);
        let rust_result = rust_import_result(&bytes);

        assert_eq!(
            rust_result, cpp_result,
            "Rust/C++ import result mismatch for synthetic case {name}"
        );
    }
}

#[derive(Debug, Deserialize)]
struct CppProbeConverterSamples {
    samples: Vec<CppProbeConverterSample>,
}

#[derive(Debug, Deserialize)]
struct CppProbeNumberToListSamples {
    samples: Vec<CppProbeNumberToListSample>,
}

#[derive(Debug, Deserialize)]
struct CppProbeNumberToListSample {
    #[serde(rename = "converterIndex")]
    converter_index: usize,
    #[serde(rename = "converterCoreType")]
    converter_core_type: u16,
    output: CppProbeConverterOutput,
}

#[derive(Debug, Deserialize)]
struct CppProbeConverterSample {
    label: String,
    #[serde(rename = "converterCoreType")]
    converter_core_type: u16,
    #[serde(default, rename = "randomValues")]
    random_values: Vec<f32>,
    output: CppProbeConverterOutput,
    #[serde(default, rename = "reverseRandomValues")]
    reverse_random_values: Vec<f32>,
    #[serde(rename = "reverseOutput")]
    reverse_output: CppProbeConverterOutput,
}

#[derive(Debug, Deserialize)]
struct CppProbeConverterOutput {
    #[serde(default, rename = "dataType")]
    data_type: Option<u32>,
    #[serde(default, rename = "numberValue")]
    number_value: Option<f32>,
    #[serde(default, rename = "stringValue")]
    string_value: Option<String>,
    #[serde(default, rename = "booleanValue")]
    boolean_value: Option<bool>,
    #[serde(default, rename = "colorValue")]
    color_value: Option<u32>,
    #[serde(default, rename = "integerValue")]
    integer_value: Option<u64>,
    #[serde(default, rename = "listSize")]
    list_size: Option<usize>,
    #[serde(default, rename = "listItems")]
    list_items: Vec<Option<CppProbeConverterListItem>>,
}

#[derive(Debug, Deserialize)]
struct CppProbeConverterListItem {
    #[serde(rename = "viewModelId")]
    view_model_id: u64,
    #[serde(default, rename = "valueCoreTypes")]
    value_core_types: Vec<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    #[serde(default)]
    assets: Vec<CppProbeAsset>,
    #[serde(default)]
    manifest: Option<CppProbeManifest>,
    #[serde(default, rename = "viewModels")]
    view_models: Vec<CppProbeViewModel>,
    #[serde(default)]
    enums: Vec<CppProbeDataEnum>,
    #[serde(default, rename = "dataContextLookups")]
    data_context_lookups: Vec<CppProbeDataContextLookup>,
    artboards: Vec<CppProbeArtboard>,
}

#[derive(Debug, Deserialize)]
struct CppProbeManifest {
    #[serde(default)]
    names: Vec<CppProbeManifestName>,
    #[serde(default)]
    paths: Vec<CppProbeManifestPath>,
}

#[derive(Debug, Deserialize)]
struct CppProbeManifestName {
    id: i32,
    value: String,
}

#[derive(Debug, Deserialize)]
struct CppProbeManifestPath {
    id: i32,
    path: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct CppProbeAsset {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "assetId")]
    asset_id: u64,
    #[serde(rename = "cdnBaseUrl")]
    cdn_base_url: String,
    #[serde(rename = "cdnUuidString")]
    cdn_uuid_string: String,
    #[serde(rename = "fileExtension")]
    file_extension: String,
    #[serde(rename = "uniqueName")]
    unique_name: String,
    #[serde(rename = "uniqueFilename")]
    unique_filename: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModel {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default)]
    properties: Vec<CppProbeViewModelProperty>,
    #[serde(default, rename = "propertyNameLookups")]
    property_name_lookups: Vec<CppProbeNameLookup>,
    #[serde(default, rename = "propertySymbolLookups")]
    property_symbol_lookups: Vec<CppProbeSymbolLookup>,
    #[serde(default)]
    instances: Vec<CppProbeViewModelInstance>,
    #[serde(default, rename = "defaultInstance")]
    default_instance: Option<CppProbeViewModelInstanceReference>,
    #[serde(default, rename = "instanceNameLookups")]
    instance_name_lookups: Vec<CppProbeNameLookup>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelReference {
    #[serde(rename = "viewModelIndex")]
    view_model_index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelProperty {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "enumKeyLookups")]
    enum_key_lookups: Vec<CppProbeDataEnumKeyLookup>,
    #[serde(default, rename = "enumIndexLookups")]
    enum_index_lookups: Vec<CppProbeDataEnumIndexLookup>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstance {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "viewModelId")]
    view_model_id: u64,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default)]
    values: Vec<CppProbeViewModelInstanceValue>,
    #[serde(default, rename = "valueIdLookups")]
    value_id_lookups: Vec<CppProbePropertyIdLookup>,
    #[serde(default, rename = "valueNameLookups")]
    value_name_lookups: Vec<CppProbeNameLookup>,
    #[serde(default, rename = "valueSymbolLookups")]
    value_symbol_lookups: Vec<CppProbeSymbolLookup>,
}

#[derive(Debug, Deserialize)]
struct CppProbeNameLookup {
    name: String,
    #[serde(default, rename = "resultIndex")]
    result_index: Option<usize>,
    #[serde(default, rename = "coreType")]
    core_type: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbePropertyIdLookup {
    #[serde(rename = "propertyId")]
    property_id: u32,
    #[serde(default, rename = "resultIndex")]
    result_index: Option<usize>,
    #[serde(default, rename = "coreType")]
    core_type: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbeSymbolLookup {
    symbol: u8,
    #[serde(default, rename = "resultIndex")]
    result_index: Option<usize>,
    #[serde(default, rename = "coreType")]
    core_type: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceValue {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "viewModelPropertyId")]
    view_model_property_id: u64,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "viewModelPropertyCoreType")]
    view_model_property_core_type: Option<u16>,
    #[serde(default, rename = "viewModelPropertyName")]
    view_model_property_name: Option<String>,
    #[serde(default, rename = "enumRuntime")]
    enum_runtime: Option<CppProbeViewModelInstanceEnumRuntime>,
    #[serde(default, rename = "valueRuntime")]
    value_runtime: Option<CppProbeViewModelInstanceValueRuntime>,
    #[serde(default, rename = "sourceDataValue")]
    source_data_value: Option<CppProbeViewModelInstanceSourceDataValue>,
    #[serde(default, rename = "referenceViewModelInstance")]
    reference_view_model_instance: Option<CppProbeViewModelInstanceReference>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "assetSnapshot")]
    asset_snapshot: Vec<CppProbeViewModelInstanceAsset>,
    #[serde(default, rename = "resolvedAsset")]
    resolved_asset: Option<CppProbeViewModelInstanceAsset>,
    #[serde(default)]
    items: Vec<CppProbeViewModelInstanceListItem>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceValueRuntime {
    #[serde(rename = "dataType")]
    data_type: u32,
    #[serde(default, rename = "numberValue")]
    number_value: Option<f32>,
    #[serde(default, rename = "stringValue")]
    string_value: Option<String>,
    #[serde(default, rename = "booleanValue")]
    boolean_value: Option<bool>,
    #[serde(default, rename = "colorValue")]
    color_value: Option<u32>,
    #[serde(default, rename = "listSize")]
    list_size: Option<usize>,
    #[serde(default, rename = "triggerCount")]
    trigger_count: Option<u64>,
    #[serde(default, rename = "integerValue")]
    integer_value: Option<u64>,
    #[serde(default, rename = "viewModelIndex")]
    view_model_index: Option<u64>,
    #[serde(default, rename = "assetIndex")]
    asset_index: Option<u64>,
    #[serde(default, rename = "artboardIndex")]
    artboard_index: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceSourceDataValue {
    #[serde(rename = "dataType")]
    data_type: u32,
    #[serde(default, rename = "numberValue")]
    number_value: Option<f32>,
    #[serde(default, rename = "stringValue")]
    string_value: Option<String>,
    #[serde(default, rename = "booleanValue")]
    boolean_value: Option<bool>,
    #[serde(default, rename = "colorValue")]
    color_value: Option<u32>,
    #[serde(default, rename = "integerValue")]
    integer_value: Option<u64>,
    #[serde(default, rename = "listSize")]
    list_size: Option<usize>,
    #[serde(default, rename = "enumDataCoreType")]
    enum_data_core_type: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceEnumRuntime {
    value: String,
    #[serde(rename = "valueIndex")]
    value_index: usize,
    values: Vec<String>,
    #[serde(rename = "enumType")]
    enum_type: String,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceAsset {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "assetId")]
    asset_id: u64,
    #[serde(rename = "cdnBaseUrl")]
    cdn_base_url: String,
    #[serde(rename = "cdnUuidString")]
    cdn_uuid_string: String,
    #[serde(rename = "fileExtension")]
    file_extension: String,
    #[serde(rename = "uniqueName")]
    unique_name: String,
    #[serde(rename = "uniqueFilename")]
    unique_filename: String,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceReference {
    #[serde(rename = "viewModelIndex")]
    view_model_index: usize,
    #[serde(rename = "instanceIndex")]
    instance_index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "viewModelId")]
    view_model_id: u64,
}

#[derive(Debug, Deserialize)]
struct CppProbeDataContextLookup {
    kind: String,
    #[serde(rename = "currentViewModelIndex")]
    current_view_model_index: usize,
    #[serde(rename = "currentInstanceIndex")]
    current_instance_index: usize,
    #[serde(default, rename = "parentViewModelIndex")]
    parent_view_model_index: Option<usize>,
    #[serde(default, rename = "parentInstanceIndex")]
    parent_instance_index: Option<usize>,
    path: Vec<u32>,
    value: Option<CppProbeDataContextValue>,
    instance: Option<CppProbeViewModelInstanceReference>,
}

#[derive(Debug, Deserialize)]
struct CppProbeDataContextValue {
    #[serde(rename = "viewModelIndex")]
    view_model_index: usize,
    #[serde(rename = "instanceIndex")]
    instance_index: usize,
    #[serde(rename = "valueIndex")]
    value_index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "viewModelPropertyId")]
    view_model_property_id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CppProbeViewModelInstanceListItem {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "viewModelId")]
    view_model_id: u64,
    #[serde(rename = "viewModelInstanceId")]
    view_model_instance_id: u64,
    #[serde(default, rename = "referencedViewModelInstance")]
    referenced_view_model_instance: Option<CppProbeViewModelInstanceReference>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeDataEnum {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default)]
    values: Vec<CppProbeDataEnumValue>,
    #[serde(default, rename = "keyLookups")]
    key_lookups: Vec<CppProbeDataEnumKeyLookup>,
    #[serde(default, rename = "indexLookups")]
    index_lookups: Vec<CppProbeDataEnumIndexLookup>,
}

#[derive(Debug, Deserialize)]
struct CppProbeDataEnumValue {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    key: String,
    value: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeDataEnumKeyLookup {
    key: String,
    value: String,
    #[serde(rename = "valueIndex")]
    value_index: i32,
}

#[derive(Debug, Deserialize)]
struct CppProbeDataEnumIndexLookup {
    index: usize,
    value: String,
    #[serde(rename = "valueIndex")]
    value_index: i32,
}

#[derive(Debug, Deserialize)]
struct CppProbeArtboard {
    name: String,
    width: f32,
    height: f32,
    #[serde(rename = "objectCount")]
    object_count: usize,
    #[serde(default, rename = "viewModel")]
    view_model: Option<CppProbeViewModelReference>,
    objects: Vec<Option<CppProbeObject>>,
    components: Vec<CppProbeComponent>,
    #[serde(default, rename = "artboardComponentLists")]
    artboard_component_lists: Vec<CppProbeArtboardComponentList>,
    #[serde(default)]
    animations: Vec<CppProbeAnimation>,
    #[serde(default, rename = "stateMachines")]
    state_machines: Vec<CppProbeStateMachine>,
    #[serde(default, rename = "dataBinds")]
    data_binds: Vec<Option<CppProbeArtboardDataBind>>,
    #[serde(default)]
    meshes: Vec<CppProbeMesh>,
    #[serde(default)]
    paths: Vec<CppProbePath>,
    #[serde(default, rename = "nSlicerDetails")]
    n_slicer_details: Vec<CppProbeNSlicerDetails>,
    #[serde(default)]
    shapes: Vec<CppProbeShape>,
    #[serde(default, rename = "shapePaintContainers")]
    shape_paint_containers: Vec<CppProbeShapePaintContainer>,
    #[serde(default)]
    skins: Vec<CppProbeSkin>,
}

#[derive(Debug, Deserialize)]
struct CppProbeArtboardComponentList {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "mapRules")]
    map_rules: Vec<CppProbeArtboardListMapRule>,
    #[serde(default, rename = "itemArtboards")]
    item_artboards: Vec<CppProbeArtboardListItemArtboard>,
}

#[derive(Debug, Deserialize)]
struct CppProbeArtboardListMapRule {
    #[serde(rename = "viewModelId")]
    view_model_id: i64,
    #[serde(rename = "artboardId")]
    artboard_id: i64,
}

#[derive(Debug, Deserialize)]
struct CppProbeArtboardListItemArtboard {
    #[serde(rename = "ownerViewModelIndex")]
    owner_view_model_index: usize,
    #[serde(rename = "ownerInstanceIndex")]
    owner_instance_index: usize,
    #[serde(rename = "valueIndex")]
    value_index: usize,
    #[serde(rename = "itemIndex")]
    item_index: usize,
    #[serde(rename = "viewModelId")]
    view_model_id: u64,
    #[serde(rename = "viewModelInstanceId")]
    view_model_instance_id: u64,
    artboard: Option<CppProbeArtboardReference>,
}

#[derive(Debug, Deserialize)]
struct CppProbeArtboardReference {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "viewModelId")]
    view_model_id: u64,
}

#[derive(Debug, Deserialize)]
struct CppProbeObject {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "isComponent")]
    is_component: bool,
    #[serde(default, rename = "sourceArtboardIndex")]
    source_artboard_index: Option<usize>,
    #[serde(default, rename = "dataBindPathIds")]
    data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "resolvedDataBindPathIds")]
    resolved_data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "scrollPhysicsCoreType")]
    scroll_physics_core_type: Option<u16>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "parentId")]
    parent_id: u64,
    #[serde(rename = "parentLocal")]
    parent_local: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CppProbeShape {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default)]
    paths: Vec<CppProbeShapePath>,
    #[serde(default)]
    paints: Vec<CppProbeShapePaint>,
}

#[derive(Debug, Deserialize)]
struct CppProbeShapePaintContainer {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default)]
    paints: Vec<CppProbeShapePaint>,
}

#[derive(Debug, Deserialize)]
struct CppProbeShapePath {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "coreType")]
    core_type: u16,
}

#[derive(Debug, Deserialize)]
struct CppProbeShapePaint {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "mutatorLocal")]
    mutator_local: Option<usize>,
    #[serde(default, rename = "mutatorCoreType")]
    mutator_core_type: Option<u16>,
    #[serde(default, rename = "featherLocal")]
    feather_local: Option<usize>,
    #[serde(default, rename = "featherCoreType")]
    feather_core_type: Option<u16>,
    #[serde(default)]
    effects: Vec<CppProbeStrokeEffect>,
    #[serde(default, rename = "gradientStops")]
    gradient_stops: Vec<CppProbeGradientStop>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStrokeEffect {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "targetGroupEffectLocal")]
    target_group_effect_local: Option<usize>,
    #[serde(default, rename = "targetGroupEffectCoreType")]
    target_group_effect_core_type: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbeGradientStop {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "coreType")]
    core_type: u16,
}

#[derive(Debug, Deserialize)]
struct CppProbeSkin {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "skinnableLocal")]
    skinnable_local: Option<usize>,
    #[serde(default, rename = "skinnableCoreType")]
    skinnable_core_type: Option<u16>,
    #[serde(default)]
    tendons: Vec<CppProbeTendon>,
}

#[derive(Debug, Deserialize)]
struct CppProbeTendon {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "boneLocal")]
    bone_local: Option<usize>,
    #[serde(default, rename = "boneCoreType")]
    bone_core_type: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CppProbeMesh {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default)]
    vertices: Vec<CppProbeMeshVertex>,
}

#[derive(Debug, Deserialize)]
struct CppProbeMeshVertex {
    index: usize,
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "weightLocal")]
    weight_local: Option<usize>,
    #[serde(default, rename = "weightCoreType")]
    weight_core_type: Option<u16>,
    #[serde(default, rename = "weightValues")]
    weight_values: Option<u64>,
    #[serde(default, rename = "weightIndices")]
    weight_indices: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CppProbePath {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default)]
    vertices: Vec<CppProbePathVertex>,
}

#[derive(Debug, Deserialize)]
struct CppProbePathVertex {
    index: usize,
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "weightLocal")]
    weight_local: Option<usize>,
    #[serde(default, rename = "weightCoreType")]
    weight_core_type: Option<u16>,
    #[serde(default, rename = "weightValues")]
    weight_values: Option<u64>,
    #[serde(default, rename = "weightIndices")]
    weight_indices: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CppProbeNSlicerDetails {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "xAxes")]
    x_axes: Vec<CppProbeNSlicerAxis>,
    #[serde(default, rename = "yAxes")]
    y_axes: Vec<CppProbeNSlicerAxis>,
    #[serde(default, rename = "tileModes")]
    tile_modes: Vec<CppProbeNSlicerTileMode>,
}

#[derive(Debug, Deserialize)]
struct CppProbeNSlicerAxis {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(rename = "coreType")]
    core_type: u16,
}

#[derive(Debug, Deserialize)]
struct CppProbeNSlicerTileMode {
    index: usize,
    #[serde(default, rename = "localId")]
    local_id: Option<usize>,
    #[serde(default, rename = "coreType")]
    core_type: Option<u16>,
    #[serde(rename = "patchIndex")]
    patch_index: u64,
    style: u64,
}

#[derive(Debug, Deserialize)]
struct CppProbeAnimation {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    fps: u64,
    duration: u64,
    speed: f32,
    #[serde(rename = "loopValue")]
    loop_value: u64,
    #[serde(rename = "workStart")]
    work_start: u64,
    #[serde(rename = "workEnd")]
    work_end: u64,
    #[serde(rename = "enableWorkArea")]
    enable_work_area: bool,
    quantize: bool,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "keyedObjects")]
    keyed_objects: Vec<CppProbeKeyedObject>,
}

#[derive(Debug, Deserialize)]
struct CppProbeKeyedObject {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "objectId")]
    object_id: u64,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "keyedProperties")]
    keyed_properties: Vec<CppProbeKeyedProperty>,
}

#[derive(Debug, Deserialize)]
struct CppProbeKeyedProperty {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "propertyKey")]
    property_key: u64,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "firstKeyFrame")]
    first_key_frame: Option<CppProbeKeyFrame>,
}

#[derive(Debug, Deserialize)]
struct CppProbeKeyFrame {
    #[serde(rename = "coreType")]
    core_type: u16,
    frame: u64,
    seconds: f32,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachine {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "layerCount")]
    layer_count: usize,
    #[serde(rename = "inputCount")]
    input_count: usize,
    #[serde(rename = "listenerCount")]
    listener_count: usize,
    #[serde(rename = "dataBindCount")]
    data_bind_count: usize,
    #[serde(default, rename = "scriptedObjectCount")]
    scripted_object_count: usize,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default)]
    layers: Vec<Option<CppProbeStateMachineLayer>>,
    #[serde(default)]
    inputs: Vec<Option<CppProbeStateMachineInput>>,
    #[serde(default)]
    listeners: Vec<Option<CppProbeStateMachineListener>>,
    #[serde(default, rename = "dataBinds")]
    data_binds: Vec<Option<CppProbeStateMachineDataBind>>,
    #[serde(default, rename = "scriptedObjects")]
    scripted_objects: Vec<CppProbeStateMachineScriptedObject>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineScriptedObject {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "inputCount")]
    input_count: usize,
    #[serde(default)]
    inputs: Vec<CppProbeStateMachineScriptInput>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineScriptInput {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineInput {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineLayer {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "stateCount")]
    state_count: usize,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default)]
    states: Vec<Option<CppProbeLayerState>>,
}

#[derive(Debug, Deserialize)]
struct CppProbeLayerState {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "animationId")]
    animation_id: Option<u64>,
    #[serde(default, rename = "animationCoreType")]
    animation_core_type: Option<u16>,
    #[serde(rename = "fireActionCount")]
    fire_action_count: usize,
    #[serde(rename = "listenerActionCount")]
    listener_action_count: usize,
    #[serde(rename = "transitionCount")]
    transition_count: usize,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "fireActions")]
    fire_actions: Vec<Option<CppProbeStateMachineFireAction>>,
    #[serde(default, rename = "listenerActions")]
    listener_actions: Vec<Option<CppProbeListenerAction>>,
    #[serde(default)]
    transitions: Vec<Option<CppProbeStateTransition>>,
    #[serde(default, rename = "blendAnimations")]
    blend_animations: Vec<Option<CppProbeBlendAnimation>>,
}

#[derive(Debug, Deserialize)]
struct CppProbeBlendAnimation {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "animationId")]
    animation_id: u64,
    #[serde(default, rename = "animationIndex")]
    animation_index: Option<usize>,
    #[serde(default, rename = "animationCoreType")]
    animation_core_type: Option<u16>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineFireAction {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "eventId")]
    event_id: Option<u64>,
    #[serde(default, rename = "eventLocal")]
    event_local: Option<usize>,
    #[serde(default, rename = "eventCoreType")]
    event_core_type: Option<u16>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateTransition {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "stateToId")]
    state_to_id: u64,
    #[serde(default, rename = "stateToIndex")]
    state_to_index: Option<usize>,
    #[serde(default, rename = "stateToCoreType")]
    state_to_core_type: Option<u16>,
    #[serde(rename = "interpolatorId")]
    interpolator_id: u64,
    #[serde(default, rename = "interpolatorCoreType")]
    interpolator_core_type: Option<u16>,
    #[serde(default, rename = "exitBlendAnimationId")]
    exit_blend_animation_id: Option<u64>,
    #[serde(default, rename = "exitBlendAnimationIndex")]
    exit_blend_animation_index: Option<usize>,
    #[serde(default, rename = "exitBlendAnimationCoreType")]
    exit_blend_animation_core_type: Option<u16>,
    #[serde(default, rename = "exitAnimationIndex")]
    exit_animation_index: Option<usize>,
    #[serde(default, rename = "exitAnimationCoreType")]
    exit_animation_core_type: Option<u16>,
    #[serde(rename = "fireActionCount")]
    fire_action_count: usize,
    #[serde(rename = "listenerActionCount")]
    listener_action_count: usize,
    #[serde(rename = "conditionCount")]
    condition_count: usize,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "fireActions")]
    fire_actions: Vec<Option<CppProbeStateMachineFireAction>>,
    #[serde(default, rename = "listenerActions")]
    listener_actions: Vec<Option<CppProbeListenerAction>>,
    #[serde(default)]
    conditions: Vec<Option<CppProbeTransitionCondition>>,
}

#[derive(Debug, Deserialize)]
struct CppProbeTransitionCondition {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineListener {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    name: String,
    #[serde(rename = "targetId")]
    target_id: u64,
    #[serde(rename = "actionCount")]
    action_count: usize,
    #[serde(rename = "listenerInputTypeCount")]
    listener_input_type_count: usize,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
    #[serde(default, rename = "dataBindPathIds")]
    data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "resolvedDataBindPathIds")]
    resolved_data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default)]
    actions: Vec<Option<CppProbeListenerAction>>,
    #[serde(default, rename = "listenerInputTypes")]
    listener_input_types: Vec<Option<CppProbeListenerInputType>>,
}

#[derive(Debug, Deserialize)]
struct CppProbeListenerAction {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    flags: u64,
    #[serde(default, rename = "eventId")]
    event_id: Option<u64>,
    #[serde(default, rename = "eventLocal")]
    event_local: Option<usize>,
    #[serde(default, rename = "eventCoreType")]
    event_core_type: Option<u16>,
    #[serde(default, rename = "dataBindPathIds")]
    data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "resolvedDataBindPathIds")]
    resolved_data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeListenerInputType {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "listenerTypeValue")]
    listener_type_value: u64,
    #[serde(default, rename = "dataBindPathIds")]
    data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "resolvedDataBindPathIds")]
    resolved_data_bind_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "viewModelPathIdsBuffer")]
    view_model_path_ids_buffer: Option<Vec<u32>>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeStateMachineDataBind {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "propertyKey")]
    property_key: u64,
    flags: u64,
    #[serde(rename = "converterId")]
    converter_id: u64,
    #[serde(rename = "converterCoreType")]
    converter_core_type: Option<u16>,
    #[serde(default, rename = "converterOutputType")]
    converter_output_type: Option<u32>,
    #[serde(default, rename = "dataBindSourceOutputType")]
    data_bind_source_output_type: Option<u32>,
    #[serde(default, rename = "dataBindOutputType")]
    data_bind_output_type: Option<u32>,
    #[serde(default, rename = "targetSupportsPush")]
    target_supports_push: Option<bool>,
    #[serde(default, rename = "usesPersistingList")]
    uses_persisting_list: Option<bool>,
    #[serde(rename = "converterInterpolatorCoreType")]
    converter_interpolator_core_type: Option<u16>,
    #[serde(default, rename = "converterSourcePathIds")]
    converter_source_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "converterNumberToListViewModelCoreType")]
    converter_number_to_list_view_model_core_type: Option<u16>,
    #[serde(default, rename = "converterNumberToListViewModelName")]
    converter_number_to_list_view_model_name: Option<String>,
    #[serde(default, rename = "dataBindContextSourcePathIds")]
    data_bind_context_source_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "resolvedDataBindContextSourcePathIds")]
    resolved_data_bind_context_source_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "converterGroupItems")]
    converter_group_items: Vec<CppProbeConverterGroupItem>,
    #[serde(default, rename = "converterFormulaTokens")]
    converter_formula_tokens: Vec<CppProbeConverterFormulaToken>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeArtboardDataBind {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "propertyKey")]
    property_key: u64,
    flags: u64,
    #[serde(rename = "converterId")]
    converter_id: u64,
    #[serde(rename = "converterCoreType")]
    converter_core_type: Option<u16>,
    #[serde(default, rename = "converterOutputType")]
    converter_output_type: Option<u32>,
    #[serde(default, rename = "dataBindSourceOutputType")]
    data_bind_source_output_type: Option<u32>,
    #[serde(default, rename = "dataBindOutputType")]
    data_bind_output_type: Option<u32>,
    #[serde(default, rename = "targetSupportsPush")]
    target_supports_push: Option<bool>,
    #[serde(default, rename = "usesPersistingList")]
    uses_persisting_list: Option<bool>,
    #[serde(rename = "converterInterpolatorCoreType")]
    converter_interpolator_core_type: Option<u16>,
    #[serde(default, rename = "converterSourcePathIds")]
    converter_source_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "converterNumberToListViewModelCoreType")]
    converter_number_to_list_view_model_core_type: Option<u16>,
    #[serde(default, rename = "converterNumberToListViewModelName")]
    converter_number_to_list_view_model_name: Option<String>,
    #[serde(default, rename = "dataBindContextSourcePathIds")]
    data_bind_context_source_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "resolvedDataBindContextSourcePathIds")]
    resolved_data_bind_context_source_path_ids: Option<Vec<u32>>,
    #[serde(default, rename = "converterGroupItems")]
    converter_group_items: Vec<CppProbeConverterGroupItem>,
    #[serde(default, rename = "converterFormulaTokens")]
    converter_formula_tokens: Vec<CppProbeConverterFormulaToken>,
    #[serde(rename = "targetCoreType")]
    target_core_type: Option<u16>,
    #[serde(rename = "targetLocal")]
    target_local: Option<usize>,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbeConverterGroupItem {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "converterCoreType")]
    converter_core_type: Option<u16>,
    #[serde(default, rename = "converterOutputType")]
    converter_output_type: Option<u32>,
    #[serde(rename = "interpolatorCoreType")]
    interpolator_core_type: Option<u16>,
    #[serde(default, rename = "converterSourcePathIds")]
    converter_source_path_ids: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize)]
struct CppProbeConverterFormulaToken {
    index: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(default, rename = "propertyValues")]
    property_values: Vec<CppProbePropertyValue>,
}

#[derive(Debug, Deserialize)]
struct CppProbePropertyValue {
    key: u16,
    kind: CppProbeFieldKind,
    value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CppProbeFieldKind {
    Uint,
    String,
    Double,
    Color,
    Bool,
}

impl CppProbeFieldKind {
    fn from_field_kind(kind: FieldKind) -> Option<Self> {
        match kind {
            FieldKind::Uint => Some(Self::Uint),
            FieldKind::String => Some(Self::String),
            FieldKind::Double => Some(Self::Double),
            FieldKind::Color => Some(Self::Color),
            FieldKind::Bool => Some(Self::Bool),
            FieldKind::Bytes | FieldKind::Callback => None,
        }
    }
}

#[test]
fn cpp_accepted_unit_fixture_corpus_imports_and_matches_summary_when_enabled() {
    if std::env::var_os(RIVE_CPP_CORPUS_ENV).is_none() {
        eprintln!("skipping C++ fixture corpus comparison; set {RIVE_CPP_CORPUS_ENV}=1");
        return;
    }

    let Some(probe) = probe_path() else {
        eprintln!(
            "skipping C++ fixture corpus comparison; set RIVE_CPP_PROBE or run make cpp-probe"
        );
        return;
    };

    let fixture_dir = reference_unit_fixture_dir();
    if !fixture_dir.exists() {
        eprintln!(
            "skipping C++ fixture corpus comparison; reference fixtures not found at {}; set RIVE_RUNTIME_DIR",
            fixture_dir.display()
        );
        return;
    }

    let mut paths = Vec::new();
    collect_riv_paths(&fixture_dir, &mut paths);
    paths.sort();

    let mut cpp_accepted = 0usize;
    let mut rust_failures = Vec::new();
    let mut structural_failures = Vec::new();
    let mut property_value_failures = Vec::new();
    let mut import_status_failures = Vec::new();
    let mut import_result_failures = Vec::new();
    let mut cpp_rejected_with_result = 0usize;

    for path in paths {
        let relative = path.strip_prefix(&fixture_dir).unwrap_or(&path);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

        let cpp_result = cpp_import_file_result(&probe, &path);
        if cpp_result != Some(ProbeImportResult::Success) {
            if let Some(cpp_result) = cpp_result {
                cpp_rejected_with_result += 1;
                let rust_result = rust_import_result(&bytes);
                if rust_result != cpp_result {
                    import_result_failures.push(format!(
                        "{}: rejected import result mismatch, C++ {:?}, Rust {:?}",
                        relative.display(),
                        cpp_result,
                        rust_result
                    ));
                }
            }
            continue;
        }

        cpp_accepted += 1;

        let rust = match read_runtime_file(&bytes) {
            Ok(file) => file,
            Err(err) => {
                rust_failures.push(format!("{}: {err:#}", relative.display()));
                continue;
            }
        };

        let label = relative.display().to_string();
        let cpp = read_cpp_probe_path_with_options(
            &probe,
            &path,
            &label,
            CppProbePropertyValueMode::All,
            true,
            true,
        );
        if let Err(err) = compare_corpus_file_summary(&cpp, &rust, &label) {
            structural_failures.push(err);
        }
        if let Err(err) = compare_corpus_file_level_stored_property_values(&cpp, &rust, &label) {
            property_value_failures.push(err);
        }
        if let Err(err) = compare_corpus_artboard_stored_property_values(&cpp, &rust, &label) {
            property_value_failures.push(err);
        }

        let cpp_failed_imports = cpp_failed_import_file_type_keys(&probe, &path, &label);
        let rust_dropped_imports = rust_dropped_import_type_keys(&rust);
        if cpp_failed_imports != rust_dropped_imports {
            import_status_failures.push(format!(
                "{label}: failed import type-key sequence mismatch, C++ {}, Rust {}",
                format_type_keys(&cpp_failed_imports),
                format_type_keys(&rust_dropped_imports)
            ));
        }
    }

    assert!(
        cpp_accepted > 0,
        "C++ probe did not accept any fixtures from {}",
        fixture_dir.display()
    );
    assert!(
        rust_failures.is_empty(),
        "Rust failed to import C++-accepted fixtures:\n{}",
        rust_failures.join("\n")
    );
    assert!(
        structural_failures.is_empty(),
        "Rust/C++ corpus structural summary mismatches:\n{}",
        structural_failures.join("\n")
    );
    assert!(
        property_value_failures.is_empty(),
        "Rust/C++ corpus stored property value mismatches:\n{}",
        property_value_failures.join("\n")
    );
    assert!(
        import_status_failures.is_empty(),
        "Rust/C++ corpus import status mismatches:\n{}",
        import_status_failures.join("\n")
    );
    assert!(
        import_result_failures.is_empty(),
        "Rust/C++ corpus rejected import result mismatches:\n{}",
        import_result_failures.join("\n")
    );
    assert!(
        cpp_rejected_with_result > 0,
        "C++ probe did not report any classified rejected fixtures from {}",
        fixture_dir.display()
    );
}

fn compare_corpus_file_summary(
    cpp: &CppProbeFile,
    rust: &RuntimeFile,
    label: &str,
) -> Result<(), String> {
    let rust_artboards = runtime_artboards(rust);
    if cpp.artboards.len() != rust_artboards.len() {
        return Err(format!(
            "{label}: artboard count mismatch, C++ {}, Rust {}",
            cpp.artboards.len(),
            rust_artboards.len()
        ));
    }

    for (index, (cpp_artboard, rust_artboard)) in
        cpp.artboards.iter().zip(&rust_artboards).enumerate()
    {
        let rust_name = rust_artboard.string_property("name").unwrap_or_default();
        if cpp_artboard.name != rust_name {
            return Err(format!(
                "{label}: artboard {index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_artboard.name, rust_name
            ));
        }
        if !float_matches(cpp_artboard.width, rust_artboard.double_property("width"))
            || !float_matches(cpp_artboard.height, rust_artboard.double_property("height"))
        {
            return Err(format!(
                "{label}: artboard {index} bounds mismatch, C++ {}x{}, Rust {:?}x{:?}",
                cpp_artboard.width,
                cpp_artboard.height,
                rust_artboard.double_property("width"),
                rust_artboard.double_property("height")
            ));
        }
        compare_artboard_view_model_result(rust, cpp_artboard, rust_artboard, index, label)?;
        let Some(range) = runtime_artboard_range_by_index(rust, index) else {
            return Err(format!("{label}: missing Rust range for artboard {index}"));
        };
        let rust_local_objects = runtime_artboard_local_objects(rust, range);
        if cpp_artboard.object_count != rust_local_objects.len() {
            return Err(format!(
                "{label}: artboard {index} object slot count mismatch, C++ {}, Rust {}",
                cpp_artboard.object_count,
                rust_local_objects.len()
            ));
        }

        if cpp_artboard.objects.len() != rust_local_objects.len() {
            return Err(format!(
                "{label}: artboard {index} object list length mismatch, C++ {}, Rust {}",
                cpp_artboard.objects.len(),
                rust_local_objects.len()
            ));
        }

        for (local_id, (cpp_object, rust_object)) in cpp_artboard
            .objects
            .iter()
            .zip(&rust_local_objects)
            .enumerate()
        {
            match (cpp_object, rust_object) {
                (None, None) => {}
                (Some(cpp_object), Some(rust_object)) => {
                    if cpp_object.core_type != rust_object.type_key {
                        return Err(format!(
                            "{label}: artboard {index} object {local_id} core type mismatch, C++ {}, Rust {}",
                            cpp_object.core_type, rust_object.type_key
                        ));
                    }
                    let rust_source_artboard_index =
                        resolved_artboard_index_for_referencer(rust, rust_object);
                    if cpp_object.source_artboard_index != rust_source_artboard_index {
                        return Err(format!(
                            "{label}: artboard {index} object {local_id} source artboard mismatch, C++ {:?}, Rust {:?}",
                            cpp_object.source_artboard_index, rust_source_artboard_index
                        ));
                    }
                    let rust_path_ids = data_bind_path_ids_for_referencer(rust, rust_object);
                    if cpp_object.data_bind_path_ids != rust_path_ids {
                        return Err(format!(
                            "{label}: artboard {index} object {local_id} data-bind path mismatch, C++ {:?}, Rust {:?}",
                            cpp_object.data_bind_path_ids, rust_path_ids
                        ));
                    }
                    let rust_resolved_path_ids =
                        resolved_data_bind_path_ids_for_referencer(rust, rust_object);
                    if cpp_object.resolved_data_bind_path_ids != rust_resolved_path_ids {
                        return Err(format!(
                            "{label}: artboard {index} object {local_id} resolved data-bind path mismatch, C++ {:?}, Rust {:?}",
                            cpp_object.resolved_data_bind_path_ids, rust_resolved_path_ids
                        ));
                    }
                    let rust_scroll_physics_core_type =
                        scroll_physics_core_type_for_constraint(rust, rust_object);
                    if cpp_object.scroll_physics_core_type != rust_scroll_physics_core_type {
                        return Err(format!(
                            "{label}: artboard {index} object {local_id} scroll physics core type mismatch, C++ {:?}, Rust {:?}",
                            cpp_object.scroll_physics_core_type, rust_scroll_physics_core_type
                        ));
                    }
                }
                (None, Some(rust_object)) => {
                    return Err(format!(
                        "{label}: artboard {index} object {local_id} null mismatch, C++ null, Rust {}",
                        rust_object.type_key
                    ));
                }
                (Some(cpp_object), None) => {
                    return Err(format!(
                        "{label}: artboard {index} object {local_id} null mismatch, C++ {}, Rust null",
                        cpp_object.core_type
                    ));
                }
            }
        }

        let rust_components = rust_local_objects
            .iter()
            .enumerate()
            .filter_map(|(local_id, object)| {
                let object = object.as_ref()?;
                runtime_object_is_component(object).then_some((local_id, *object))
            })
            .collect::<BTreeMap<_, _>>();
        if cpp_artboard.components.len() != rust_components.len() {
            return Err(format!(
                "{label}: artboard {index} component count mismatch, C++ {}, Rust {}",
                cpp_artboard.components.len(),
                rust_components.len()
            ));
        }

        for cpp_component in &cpp_artboard.components {
            let Some(rust_component) = rust_components.get(&cpp_component.local_id) else {
                return Err(format!(
                    "{label}: artboard {index} missing Rust component local id {}",
                    cpp_component.local_id
                ));
            };
            if cpp_component.core_type != rust_component.type_key {
                return Err(format!(
                    "{label}: artboard {index} component {} core type mismatch, C++ {}, Rust {}",
                    cpp_component.local_id, cpp_component.core_type, rust_component.type_key
                ));
            }

            let rust_name = rust_component.string_property("name").unwrap_or_default();
            if cpp_component.name != rust_name {
                return Err(format!(
                    "{label}: artboard {index} component {} name mismatch, C++ {:?}, Rust {:?}",
                    cpp_component.local_id, cpp_component.name, rust_name
                ));
            }

            let rust_parent_id = rust_component.uint_property("parentId").unwrap_or(0);
            if cpp_component.parent_id != rust_parent_id {
                return Err(format!(
                    "{label}: artboard {index} component {} parent id mismatch, C++ {}, Rust {}",
                    cpp_component.local_id, cpp_component.parent_id, rust_parent_id
                ));
            }

            let rust_parent_local = if rust_component.type_name == "Artboard" {
                None
            } else {
                Some(rust_parent_id as usize)
            };
            if cpp_component.parent_local != rust_parent_local {
                return Err(format!(
                    "{label}: artboard {index} component {} resolved parent mismatch, C++ {:?}, Rust {:?}",
                    cpp_component.local_id, cpp_component.parent_local, rust_parent_local
                ));
            }
        }

        compare_corpus_artboard_geometry_relationships(cpp_artboard, rust, index, label)?;
        compare_corpus_artboard_shape_paint_containers(cpp_artboard, rust, index, label)?;
        compare_corpus_artboard_animation_summary(cpp_artboard, rust, index, label)?;
    }

    compare_manifest_result(cpp.manifest.as_ref(), rust.manifest(), label)?;

    let rust_assets = runtime_file_assets(rust);
    if cpp.assets.len() != rust_assets.len() {
        return Err(format!(
            "{label}: file asset count mismatch, C++ {}, Rust {}",
            cpp.assets.len(),
            rust_assets.len()
        ));
    }
    for (index, (cpp_asset, rust_asset)) in cpp.assets.iter().zip(&rust_assets).enumerate() {
        if cpp_asset.index != index {
            return Err(format!(
                "{label}: file asset {index} C++ index mismatch, got {}",
                cpp_asset.index
            ));
        }
        if cpp_asset.core_type != rust_asset.type_key {
            return Err(format!(
                "{label}: file asset {index} core type mismatch, C++ {}, Rust {}",
                cpp_asset.core_type, rust_asset.type_key
            ));
        }
        let rust_name = rust_asset.string_property("name").unwrap_or_default();
        if cpp_asset.name != rust_name {
            return Err(format!(
                "{label}: file asset {index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_asset.name, rust_name
            ));
        }
        if rust_asset.uint_property("assetId") != Some(cpp_asset.asset_id) {
            return Err(format!(
                "{label}: file asset {index} asset id mismatch, C++ {}, Rust {:?}",
                cpp_asset.asset_id,
                rust_asset.uint_property("assetId")
            ));
        }
        let rust_cdn_base_url = rust_asset.string_property("cdnBaseUrl").unwrap_or_default();
        if cpp_asset.cdn_base_url != rust_cdn_base_url {
            return Err(format!(
                "{label}: file asset {index} CDN base URL mismatch, C++ {:?}, Rust {:?}",
                cpp_asset.cdn_base_url, rust_cdn_base_url
            ));
        }
        let rust_cdn_uuid_string = rust_asset.file_asset_cdn_uuid_string().unwrap_or_default();
        if cpp_asset.cdn_uuid_string != rust_cdn_uuid_string {
            return Err(format!(
                "{label}: file asset {index} CDN UUID string mismatch, C++ {:?}, Rust {:?}",
                cpp_asset.cdn_uuid_string, rust_cdn_uuid_string
            ));
        }
        let rust_file_extension = rust_asset.file_asset_extension().unwrap_or_default();
        if cpp_asset.file_extension != rust_file_extension {
            return Err(format!(
                "{label}: file asset {index} extension mismatch, C++ {:?}, Rust {:?}",
                cpp_asset.file_extension, rust_file_extension
            ));
        }
        let rust_unique_name = rust_asset.file_asset_unique_name().unwrap_or_default();
        if cpp_asset.unique_name != rust_unique_name {
            return Err(format!(
                "{label}: file asset {index} unique name mismatch, C++ {:?}, Rust {:?}",
                cpp_asset.unique_name, rust_unique_name
            ));
        }
        let rust_unique_filename = rust_asset.file_asset_unique_filename().unwrap_or_default();
        if cpp_asset.unique_filename != rust_unique_filename {
            return Err(format!(
                "{label}: file asset {index} unique filename mismatch, C++ {:?}, Rust {:?}",
                cpp_asset.unique_filename, rust_unique_filename
            ));
        }
    }

    let rust_view_models = runtime_file_view_models(rust);
    if cpp.view_models.len() != rust_view_models.len() {
        return Err(format!(
            "{label}: view model count mismatch, C++ {}, Rust {}",
            cpp.view_models.len(),
            rust_view_models.len()
        ));
    }
    for (index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        if cpp_view_model.index != index {
            return Err(format!(
                "{label}: view model {index} C++ index mismatch, got {}",
                cpp_view_model.index
            ));
        }
        if cpp_view_model.core_type != rust_view_model.object.type_key {
            return Err(format!(
                "{label}: view model {index} core type mismatch, C++ {}, Rust {}",
                cpp_view_model.core_type, rust_view_model.object.type_key
            ));
        }
        let rust_name = rust_view_model
            .object
            .string_property("name")
            .unwrap_or_default();
        if cpp_view_model.name != rust_name {
            return Err(format!(
                "{label}: view model {index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_view_model.name, rust_name
            ));
        }
        if cpp_view_model.properties.len() != rust_view_model.properties.len() {
            return Err(format!(
                "{label}: view model {index} property count mismatch, C++ {}, Rust {}",
                cpp_view_model.properties.len(),
                rust_view_model.properties.len()
            ));
        }
        for (property_index, (cpp_property, rust_property)) in cpp_view_model
            .properties
            .iter()
            .zip(&rust_view_model.properties)
            .enumerate()
        {
            if cpp_property.index != property_index {
                return Err(format!(
                    "{label}: view model {index} property {property_index} C++ index mismatch, got {}",
                    cpp_property.index
                ));
            }
            if cpp_property.core_type != rust_property.type_key {
                return Err(format!(
                    "{label}: view model {index} property {property_index} core type mismatch, C++ {}, Rust {}",
                    cpp_property.core_type, rust_property.type_key
                ));
            }
            let rust_property_name = rust_property.string_property("name").unwrap_or_default();
            if cpp_property.name != rust_property_name {
                return Err(format!(
                    "{label}: view model {index} property {property_index} name mismatch, C++ {:?}, Rust {:?}",
                    cpp_property.name, rust_property_name
                ));
            }
            compare_view_model_property_enum_lookup_result(
                rust,
                cpp_property,
                rust_property,
                &format!("{label}: view model {index} property {property_index}"),
            )?;
        }
        if cpp_view_model.instances.len() != rust_view_model.instances.len() {
            return Err(format!(
                "{label}: view model {index} instance count mismatch, C++ {}, Rust {}",
                cpp_view_model.instances.len(),
                rust_view_model.instances.len()
            ));
        }
        for (instance_index, (cpp_instance, rust_instance)) in cpp_view_model
            .instances
            .iter()
            .zip(&rust_view_model.instances)
            .enumerate()
        {
            if cpp_instance.index != instance_index {
                return Err(format!(
                    "{label}: view model {index} instance {instance_index} C++ index mismatch, got {}",
                    cpp_instance.index
                ));
            }
            if cpp_instance.core_type != rust_instance.object.type_key {
                return Err(format!(
                    "{label}: view model {index} instance {instance_index} core type mismatch, C++ {}, Rust {}",
                    cpp_instance.core_type, rust_instance.object.type_key
                ));
            }
            let rust_instance_name = rust_instance
                .object
                .string_property("name")
                .unwrap_or_default();
            if cpp_instance.name != rust_instance_name {
                return Err(format!(
                    "{label}: view model {index} instance {instance_index} name mismatch, C++ {:?}, Rust {:?}",
                    cpp_instance.name, rust_instance_name
                ));
            }
            if rust_instance.object.uint_property("viewModelId") != Some(cpp_instance.view_model_id)
            {
                return Err(format!(
                    "{label}: view model {index} instance {instance_index} view model id mismatch, C++ {}, Rust {:?}",
                    cpp_instance.view_model_id,
                    rust_instance.object.uint_property("viewModelId")
                ));
            }

            compare_view_model_instance_values_result(
                rust,
                &cpp_instance.values,
                &rust_instance.values,
                &format!("{label}: view model {index} instance {instance_index}"),
            )?;
            compare_view_model_instance_value_id_lookups_result(
                rust,
                cpp_instance,
                rust_instance,
                index,
                instance_index,
                label,
            )?;
        }
        compare_view_model_default_instance_result(
            rust,
            cpp_view_model,
            rust_view_model,
            index,
            label,
        )?;
    }

    let rust_enums = runtime_data_enums(rust);
    if cpp.enums.len() != rust_enums.len() {
        return Err(format!(
            "{label}: data enum count mismatch, C++ {}, Rust {}",
            cpp.enums.len(),
            rust_enums.len()
        ));
    }
    for (index, (cpp_enum, rust_enum)) in cpp.enums.iter().zip(&rust_enums).enumerate() {
        if cpp_enum.index != index {
            return Err(format!(
                "{label}: data enum {index} C++ index mismatch, got {}",
                cpp_enum.index
            ));
        }
        if cpp_enum.core_type != rust_enum.object.type_key {
            return Err(format!(
                "{label}: data enum {index} core type mismatch, C++ {}, Rust {}",
                cpp_enum.core_type, rust_enum.object.type_key
            ));
        }
        let rust_name = rust_enum.object.string_property("name").unwrap_or_default();
        if cpp_enum.name != rust_name {
            return Err(format!(
                "{label}: data enum {index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_enum.name, rust_name
            ));
        }
        if cpp_enum.values.len() != rust_enum.values.len() {
            return Err(format!(
                "{label}: data enum {index} value count mismatch, C++ {}, Rust {}",
                cpp_enum.values.len(),
                rust_enum.values.len()
            ));
        }
        for (value_index, (cpp_value, rust_value)) in
            cpp_enum.values.iter().zip(&rust_enum.values).enumerate()
        {
            if cpp_value.index != value_index {
                return Err(format!(
                    "{label}: data enum {index} value {value_index} C++ index mismatch, got {}",
                    cpp_value.index
                ));
            }
            if cpp_value.core_type != rust_value.type_key {
                return Err(format!(
                    "{label}: data enum {index} value {value_index} core type mismatch, C++ {}, Rust {}",
                    cpp_value.core_type, rust_value.type_key
                ));
            }
            let rust_key = rust_value.string_property("key").unwrap_or_default();
            if cpp_value.key != rust_key {
                return Err(format!(
                    "{label}: data enum {index} value {value_index} key mismatch, C++ {:?}, Rust {:?}",
                    cpp_value.key, rust_key
                ));
            }
            let rust_value_value = rust_value.string_property("value").unwrap_or_default();
            if cpp_value.value != rust_value_value {
                return Err(format!(
                    "{label}: data enum {index} value {value_index} value mismatch, C++ {:?}, Rust {:?}",
                    cpp_value.value, rust_value_value
                ));
            }
        }
        compare_data_enum_lookup_result(rust, cpp_enum, index, label)?;
    }

    Ok(())
}

fn compare_corpus_artboard_geometry_relationships(
    cpp_artboard: &CppProbeArtboard,
    rust: &RuntimeFile,
    artboard_index: usize,
    label: &str,
) -> Result<(), String> {
    let mut failures = Vec::new();
    let prefix = format!("{label}: artboard {artboard_index}");

    push_comparison_result(
        &mut failures,
        compare_artboard_skins_result(
            &cpp_artboard.skins,
            &rust.artboard_skins(artboard_index),
            &prefix,
        ),
    );
    push_comparison_result(
        &mut failures,
        compare_artboard_meshes_result(
            &cpp_artboard.meshes,
            &rust.artboard_meshes(artboard_index),
            &prefix,
        ),
    );
    push_comparison_result(
        &mut failures,
        compare_artboard_paths_result(
            &cpp_artboard.paths,
            &rust.artboard_paths(artboard_index),
            &prefix,
        ),
    );
    push_comparison_result(
        &mut failures,
        compare_artboard_n_slicer_details_result(
            &cpp_artboard.n_slicer_details,
            &rust.artboard_n_slicer_details(artboard_index),
            &prefix,
        ),
    );
    push_comparison_result(
        &mut failures,
        compare_artboard_shapes_result(
            &cpp_artboard.shapes,
            &rust.artboard_shapes(artboard_index),
            &prefix,
        ),
    );

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn compare_artboard_skins_result(
    cpp_skins: &[CppProbeSkin],
    rust_skins: &[nuxie_binary::RuntimeSkin<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_skins.len() != rust_skins.len() {
        return Err(format!(
            "{label} skin count mismatch, C++ {}, Rust {}",
            cpp_skins.len(),
            rust_skins.len()
        ));
    }

    for (skin_index, (cpp_skin, rust_skin)) in cpp_skins.iter().zip(rust_skins).enumerate() {
        if cpp_skin.local_id != rust_skin.local_id {
            return Err(format!(
                "{label} skin {skin_index} local id mismatch, C++ {}, Rust {}",
                cpp_skin.local_id, rust_skin.local_id
            ));
        }
        if cpp_skin.core_type != rust_skin.object.type_key {
            return Err(format!(
                "{label} skin {skin_index} core type mismatch, C++ {}, Rust {}",
                cpp_skin.core_type, rust_skin.object.type_key
            ));
        }
        if cpp_skin.skinnable_local != rust_skin.skinnable_local_id {
            return Err(format!(
                "{label} skin {skin_index} skinnable local mismatch, C++ {:?}, Rust {:?}",
                cpp_skin.skinnable_local, rust_skin.skinnable_local_id
            ));
        }
        let rust_skinnable_core_type = rust_skin.skinnable.map(|object| object.type_key);
        if cpp_skin.skinnable_core_type != rust_skinnable_core_type {
            return Err(format!(
                "{label} skin {skin_index} skinnable core type mismatch, C++ {:?}, Rust {:?}",
                cpp_skin.skinnable_core_type, rust_skinnable_core_type
            ));
        }
        if cpp_skin.tendons.len() != rust_skin.tendons.len() {
            return Err(format!(
                "{label} skin {skin_index} tendon count mismatch, C++ {}, Rust {}",
                cpp_skin.tendons.len(),
                rust_skin.tendons.len()
            ));
        }

        for (tendon_index, (cpp_tendon, rust_tendon)) in
            cpp_skin.tendons.iter().zip(&rust_skin.tendons).enumerate()
        {
            if cpp_tendon.index != tendon_index {
                return Err(format!(
                    "{label} skin {skin_index} tendon {tendon_index} C++ index mismatch, got {}",
                    cpp_tendon.index
                ));
            }
            if cpp_tendon.local_id != Some(rust_tendon.local_id) {
                return Err(format!(
                    "{label} skin {skin_index} tendon {tendon_index} local id mismatch, C++ {:?}, Rust {}",
                    cpp_tendon.local_id, rust_tendon.local_id
                ));
            }
            if cpp_tendon.core_type != rust_tendon.object.type_key {
                return Err(format!(
                    "{label} skin {skin_index} tendon {tendon_index} core type mismatch, C++ {}, Rust {}",
                    cpp_tendon.core_type, rust_tendon.object.type_key
                ));
            }
            if cpp_tendon.bone_local != rust_tendon.bone_local_id {
                return Err(format!(
                    "{label} skin {skin_index} tendon {tendon_index} bone local mismatch, C++ {:?}, Rust {:?}",
                    cpp_tendon.bone_local, rust_tendon.bone_local_id
                ));
            }
            let rust_bone_core_type = rust_tendon.bone.map(|object| object.type_key);
            if cpp_tendon.bone_core_type != rust_bone_core_type {
                return Err(format!(
                    "{label} skin {skin_index} tendon {tendon_index} bone core type mismatch, C++ {:?}, Rust {:?}",
                    cpp_tendon.bone_core_type, rust_bone_core_type
                ));
            }
        }
    }

    Ok(())
}

fn compare_artboard_meshes_result(
    cpp_meshes: &[CppProbeMesh],
    rust_meshes: &[nuxie_binary::RuntimeMesh<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_meshes.len() != rust_meshes.len() {
        return Err(format!(
            "{label} mesh count mismatch, C++ {}, Rust {}",
            cpp_meshes.len(),
            rust_meshes.len()
        ));
    }

    for (mesh_index, (cpp_mesh, rust_mesh)) in cpp_meshes.iter().zip(rust_meshes).enumerate() {
        if cpp_mesh.local_id != rust_mesh.local_id {
            return Err(format!(
                "{label} mesh {mesh_index} local id mismatch, C++ {}, Rust {}",
                cpp_mesh.local_id, rust_mesh.local_id
            ));
        }
        if cpp_mesh.core_type != rust_mesh.object.type_key {
            return Err(format!(
                "{label} mesh {mesh_index} core type mismatch, C++ {}, Rust {}",
                cpp_mesh.core_type, rust_mesh.object.type_key
            ));
        }
        if cpp_mesh.vertices.len() != rust_mesh.vertices.len() {
            return Err(format!(
                "{label} mesh {mesh_index} vertex count mismatch, C++ {}, Rust {}",
                cpp_mesh.vertices.len(),
                rust_mesh.vertices.len()
            ));
        }

        for (vertex_index, (cpp_vertex, rust_vertex)) in cpp_mesh
            .vertices
            .iter()
            .zip(&rust_mesh.vertices)
            .enumerate()
        {
            compare_weighted_vertex_result(
                vertex_index,
                cpp_vertex.index,
                cpp_vertex.local_id,
                cpp_vertex.core_type,
                cpp_vertex.weight_local,
                cpp_vertex.weight_core_type,
                cpp_vertex.weight_values,
                cpp_vertex.weight_indices,
                rust_vertex.local_id,
                rust_vertex.object,
                rust_vertex.weight_local_id,
                rust_vertex.weight,
                &format!("{label} mesh {mesh_index} vertex {vertex_index}"),
            )?;
        }
    }

    Ok(())
}

fn compare_artboard_paths_result(
    cpp_paths: &[CppProbePath],
    rust_paths: &[nuxie_binary::RuntimePath<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_paths.len() != rust_paths.len() {
        return Err(format!(
            "{label} path count mismatch, C++ {}, Rust {}",
            cpp_paths.len(),
            rust_paths.len()
        ));
    }

    for (path_index, (cpp_path, rust_path)) in cpp_paths.iter().zip(rust_paths).enumerate() {
        if cpp_path.local_id != rust_path.local_id {
            return Err(format!(
                "{label} path {path_index} local id mismatch, C++ {}, Rust {}",
                cpp_path.local_id, rust_path.local_id
            ));
        }
        if cpp_path.core_type != rust_path.object.type_key {
            return Err(format!(
                "{label} path {path_index} core type mismatch, C++ {}, Rust {}",
                cpp_path.core_type, rust_path.object.type_key
            ));
        }
        if cpp_path.vertices.len() != rust_path.vertices.len() {
            return Err(format!(
                "{label} path {path_index} vertex count mismatch, C++ {}, Rust {}",
                cpp_path.vertices.len(),
                rust_path.vertices.len()
            ));
        }

        for (vertex_index, (cpp_vertex, rust_vertex)) in cpp_path
            .vertices
            .iter()
            .zip(&rust_path.vertices)
            .enumerate()
        {
            compare_weighted_vertex_result(
                vertex_index,
                cpp_vertex.index,
                cpp_vertex.local_id,
                cpp_vertex.core_type,
                cpp_vertex.weight_local,
                cpp_vertex.weight_core_type,
                cpp_vertex.weight_values,
                cpp_vertex.weight_indices,
                rust_vertex.local_id,
                rust_vertex.object,
                rust_vertex.weight_local_id,
                rust_vertex.weight,
                &format!("{label} path {path_index} vertex {vertex_index}"),
            )?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compare_weighted_vertex_result(
    expected_index: usize,
    cpp_index: usize,
    cpp_local_id: usize,
    cpp_core_type: u16,
    cpp_weight_local: Option<usize>,
    cpp_weight_core_type: Option<u16>,
    cpp_weight_values: Option<u64>,
    cpp_weight_indices: Option<u64>,
    rust_local_id: usize,
    rust_object: &RuntimeObject,
    rust_weight_local_id: Option<usize>,
    rust_weight: Option<&RuntimeObject>,
    label: &str,
) -> Result<(), String> {
    if cpp_index != expected_index {
        return Err(format!("{label} C++ index mismatch, got {}", cpp_index));
    }
    if cpp_local_id != rust_local_id {
        return Err(format!(
            "{label} local id mismatch, C++ {}, Rust {}",
            cpp_local_id, rust_local_id
        ));
    }
    if cpp_core_type != rust_object.type_key {
        return Err(format!(
            "{label} core type mismatch, C++ {}, Rust {}",
            cpp_core_type, rust_object.type_key
        ));
    }
    if cpp_weight_local != rust_weight_local_id {
        return Err(format!(
            "{label} weight local mismatch, C++ {:?}, Rust {:?}",
            cpp_weight_local, rust_weight_local_id
        ));
    }
    let rust_weight_core_type = rust_weight.map(|object| object.type_key);
    if cpp_weight_core_type != rust_weight_core_type {
        return Err(format!(
            "{label} weight core type mismatch, C++ {:?}, Rust {:?}",
            cpp_weight_core_type, rust_weight_core_type
        ));
    }
    let rust_weight_values = rust_weight.and_then(|object| object.uint_property("values"));
    if cpp_weight_values != rust_weight_values {
        return Err(format!(
            "{label} weight values mismatch, C++ {:?}, Rust {:?}",
            cpp_weight_values, rust_weight_values
        ));
    }
    let rust_weight_indices = rust_weight.and_then(|object| object.uint_property("indices"));
    if cpp_weight_indices != rust_weight_indices {
        return Err(format!(
            "{label} weight indices mismatch, C++ {:?}, Rust {:?}",
            cpp_weight_indices, rust_weight_indices
        ));
    }

    Ok(())
}

fn compare_artboard_n_slicer_details_result(
    cpp_details: &[CppProbeNSlicerDetails],
    rust_details: &[nuxie_binary::RuntimeNSlicerDetails<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_details.len() != rust_details.len() {
        return Err(format!(
            "{label} NSlicerDetails count mismatch, C++ {}, Rust {}",
            cpp_details.len(),
            rust_details.len()
        ));
    }

    for (details_index, (cpp_detail, rust_detail)) in
        cpp_details.iter().zip(rust_details).enumerate()
    {
        if cpp_detail.local_id != rust_detail.local_id {
            return Err(format!(
                "{label} NSlicerDetails {details_index} local id mismatch, C++ {}, Rust {}",
                cpp_detail.local_id, rust_detail.local_id
            ));
        }
        if cpp_detail.core_type != rust_detail.object.type_key {
            return Err(format!(
                "{label} NSlicerDetails {details_index} core type mismatch, C++ {}, Rust {}",
                cpp_detail.core_type, rust_detail.object.type_key
            ));
        }
        compare_n_slicer_axes_result(
            &cpp_detail.x_axes,
            &rust_detail.x_axes,
            &format!("{label} NSlicerDetails {details_index} x axes"),
        )?;
        compare_n_slicer_axes_result(
            &cpp_detail.y_axes,
            &rust_detail.y_axes,
            &format!("{label} NSlicerDetails {details_index} y axes"),
        )?;
        compare_n_slicer_tile_modes_result(
            &cpp_detail.tile_modes,
            &rust_detail.tile_modes,
            &format!("{label} NSlicerDetails {details_index} tile modes"),
        )?;
    }

    Ok(())
}

fn compare_n_slicer_axes_result(
    cpp_axes: &[CppProbeNSlicerAxis],
    rust_axes: &[nuxie_binary::RuntimeNSlicerAxis<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_axes.len() != rust_axes.len() {
        return Err(format!(
            "{label} count mismatch, C++ {}, Rust {}",
            cpp_axes.len(),
            rust_axes.len()
        ));
    }
    for (axis_index, (cpp_axis, rust_axis)) in cpp_axes.iter().zip(rust_axes).enumerate() {
        if cpp_axis.index != axis_index {
            return Err(format!(
                "{label} axis {axis_index} C++ index mismatch, got {}",
                cpp_axis.index
            ));
        }
        if cpp_axis.local_id != Some(rust_axis.local_id) {
            return Err(format!(
                "{label} axis {axis_index} local id mismatch, C++ {:?}, Rust {}",
                cpp_axis.local_id, rust_axis.local_id
            ));
        }
        if cpp_axis.core_type != rust_axis.object.type_key {
            return Err(format!(
                "{label} axis {axis_index} core type mismatch, C++ {}, Rust {}",
                cpp_axis.core_type, rust_axis.object.type_key
            ));
        }
    }
    Ok(())
}

fn compare_n_slicer_tile_modes_result(
    cpp_tile_modes: &[CppProbeNSlicerTileMode],
    rust_tile_modes: &[nuxie_binary::RuntimeNSlicerTileMode<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_tile_modes.len() != rust_tile_modes.len() {
        return Err(format!(
            "{label} count mismatch, C++ {}, Rust {}",
            cpp_tile_modes.len(),
            rust_tile_modes.len()
        ));
    }
    for (mode_index, (cpp_mode, rust_mode)) in
        cpp_tile_modes.iter().zip(rust_tile_modes).enumerate()
    {
        if cpp_mode.index != mode_index {
            return Err(format!(
                "{label} tile mode {mode_index} C++ index mismatch, got {}",
                cpp_mode.index
            ));
        }
        if cpp_mode.local_id != Some(rust_mode.local_id) {
            return Err(format!(
                "{label} tile mode {mode_index} local id mismatch, C++ {:?}, Rust {}",
                cpp_mode.local_id, rust_mode.local_id
            ));
        }
        if cpp_mode.core_type != Some(rust_mode.object.type_key) {
            return Err(format!(
                "{label} tile mode {mode_index} core type mismatch, C++ {:?}, Rust {}",
                cpp_mode.core_type, rust_mode.object.type_key
            ));
        }
        if cpp_mode.patch_index != rust_mode.patch_index {
            return Err(format!(
                "{label} tile mode {mode_index} patch index mismatch, C++ {}, Rust {}",
                cpp_mode.patch_index, rust_mode.patch_index
            ));
        }
        if cpp_mode.style != rust_mode.style {
            return Err(format!(
                "{label} tile mode {mode_index} style mismatch, C++ {}, Rust {}",
                cpp_mode.style, rust_mode.style
            ));
        }
    }
    Ok(())
}

fn compare_artboard_shapes_result(
    cpp_shapes: &[CppProbeShape],
    rust_shapes: &[nuxie_binary::RuntimeShape<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_shapes.len() != rust_shapes.len() {
        return Err(format!(
            "{label} shape count mismatch, C++ {}, Rust {}",
            cpp_shapes.len(),
            rust_shapes.len()
        ));
    }

    for (shape_index, (cpp_shape, rust_shape)) in cpp_shapes.iter().zip(rust_shapes).enumerate() {
        if cpp_shape.local_id != rust_shape.local_id {
            return Err(format!(
                "{label} shape {shape_index} local id mismatch, C++ {}, Rust {}",
                cpp_shape.local_id, rust_shape.local_id
            ));
        }
        if cpp_shape.core_type != rust_shape.object.type_key {
            return Err(format!(
                "{label} shape {shape_index} core type mismatch, C++ {}, Rust {}",
                cpp_shape.core_type, rust_shape.object.type_key
            ));
        }
        if cpp_shape.paths.len() != rust_shape.paths.len() {
            return Err(format!(
                "{label} shape {shape_index} path count mismatch, C++ {}, Rust {}",
                cpp_shape.paths.len(),
                rust_shape.paths.len()
            ));
        }
        for (path_index, (cpp_path, rust_path)) in
            cpp_shape.paths.iter().zip(&rust_shape.paths).enumerate()
        {
            if cpp_path.index != path_index {
                return Err(format!(
                    "{label} shape {shape_index} path {path_index} C++ index mismatch, got {}",
                    cpp_path.index
                ));
            }
            if cpp_path.local_id != Some(rust_path.local_id) {
                return Err(format!(
                    "{label} shape {shape_index} path {path_index} local id mismatch, C++ {:?}, Rust {}",
                    cpp_path.local_id, rust_path.local_id
                ));
            }
            if cpp_path.core_type != rust_path.object.type_key {
                return Err(format!(
                    "{label} shape {shape_index} path {path_index} core type mismatch, C++ {}, Rust {}",
                    cpp_path.core_type, rust_path.object.type_key
                ));
            }
        }
        if cpp_shape.paints.len() != rust_shape.paints.len() {
            return Err(format!(
                "{label} shape {shape_index} paint count mismatch, C++ {}, Rust {}",
                cpp_shape.paints.len(),
                rust_shape.paints.len()
            ));
        }
        for (paint_index, (cpp_paint, rust_paint)) in
            cpp_shape.paints.iter().zip(&rust_shape.paints).enumerate()
        {
            compare_shape_paint_result(
                cpp_paint,
                rust_paint,
                paint_index,
                &format!("{label} shape {shape_index} paint {paint_index}"),
            )?;
        }
    }

    Ok(())
}

fn compare_shape_paint_result(
    cpp_paint: &CppProbeShapePaint,
    rust_paint: &nuxie_binary::RuntimeShapePaint<'_>,
    paint_index: usize,
    label: &str,
) -> Result<(), String> {
    if cpp_paint.index != paint_index {
        return Err(format!(
            "{label} C++ index mismatch, got {}",
            cpp_paint.index
        ));
    }
    if cpp_paint.local_id != Some(rust_paint.local_id) {
        return Err(format!(
            "{label} local id mismatch, C++ {:?}, Rust {}",
            cpp_paint.local_id, rust_paint.local_id
        ));
    }
    if cpp_paint.core_type != rust_paint.object.type_key {
        return Err(format!(
            "{label} core type mismatch, C++ {}, Rust {}",
            cpp_paint.core_type, rust_paint.object.type_key
        ));
    }
    if cpp_paint.mutator_local != rust_paint.mutator_local_id {
        return Err(format!(
            "{label} mutator local mismatch, C++ {:?}, Rust {:?}",
            cpp_paint.mutator_local, rust_paint.mutator_local_id
        ));
    }
    let rust_mutator_core_type = rust_paint.mutator.map(|object| object.type_key);
    if cpp_paint.mutator_core_type != rust_mutator_core_type {
        return Err(format!(
            "{label} mutator core type mismatch, C++ {:?}, Rust {:?}",
            cpp_paint.mutator_core_type, rust_mutator_core_type
        ));
    }
    if cpp_paint.feather_local != rust_paint.feather_local_id {
        return Err(format!(
            "{label} feather local mismatch, C++ {:?}, Rust {:?}",
            cpp_paint.feather_local, rust_paint.feather_local_id
        ));
    }
    let rust_feather_core_type = rust_paint.feather.map(|object| object.type_key);
    if cpp_paint.feather_core_type != rust_feather_core_type {
        return Err(format!(
            "{label} feather core type mismatch, C++ {:?}, Rust {:?}",
            cpp_paint.feather_core_type, rust_feather_core_type
        ));
    }
    if cpp_paint.effects.len() != rust_paint.effects.len() {
        return Err(format!(
            "{label} effect count mismatch, C++ {}, Rust {}",
            cpp_paint.effects.len(),
            rust_paint.effects.len()
        ));
    }
    for (effect_index, (cpp_effect, rust_effect)) in cpp_paint
        .effects
        .iter()
        .zip(&rust_paint.effects)
        .enumerate()
    {
        if cpp_effect.index != effect_index {
            return Err(format!(
                "{label} effect {effect_index} C++ index mismatch, got {}",
                cpp_effect.index
            ));
        }
        if cpp_effect.local_id != Some(rust_effect.local_id) {
            return Err(format!(
                "{label} effect {effect_index} local id mismatch, C++ {:?}, Rust {}",
                cpp_effect.local_id, rust_effect.local_id
            ));
        }
        if cpp_effect.core_type != rust_effect.object.type_key {
            return Err(format!(
                "{label} effect {effect_index} core type mismatch, C++ {}, Rust {}",
                cpp_effect.core_type, rust_effect.object.type_key
            ));
        }
        if cpp_effect.target_group_effect_local != rust_effect.target_group_effect_local_id {
            return Err(format!(
                "{label} effect {effect_index} target group local mismatch, C++ {:?}, Rust {:?}",
                cpp_effect.target_group_effect_local, rust_effect.target_group_effect_local_id
            ));
        }
        let rust_target_group_effect_core_type = rust_effect
            .target_group_effect
            .map(|object| object.type_key);
        if cpp_effect.target_group_effect_core_type != rust_target_group_effect_core_type {
            return Err(format!(
                "{label} effect {effect_index} target group core type mismatch, C++ {:?}, Rust {:?}",
                cpp_effect.target_group_effect_core_type, rust_target_group_effect_core_type
            ));
        }
    }
    if cpp_paint.gradient_stops.len() != rust_paint.gradient_stops.len() {
        return Err(format!(
            "{label} gradient stop count mismatch, C++ {}, Rust {}",
            cpp_paint.gradient_stops.len(),
            rust_paint.gradient_stops.len()
        ));
    }
    for (stop_index, (cpp_stop, rust_stop)) in cpp_paint
        .gradient_stops
        .iter()
        .zip(&rust_paint.gradient_stops)
        .enumerate()
    {
        if cpp_stop.index != stop_index {
            return Err(format!(
                "{label} gradient stop {stop_index} C++ index mismatch, got {}",
                cpp_stop.index
            ));
        }
        if cpp_stop.local_id != Some(rust_stop.local_id) {
            return Err(format!(
                "{label} gradient stop {stop_index} local id mismatch, C++ {:?}, Rust {}",
                cpp_stop.local_id, rust_stop.local_id
            ));
        }
        if cpp_stop.core_type != rust_stop.object.type_key {
            return Err(format!(
                "{label} gradient stop {stop_index} core type mismatch, C++ {}, Rust {}",
                cpp_stop.core_type, rust_stop.object.type_key
            ));
        }
    }

    Ok(())
}

fn compare_corpus_artboard_shape_paint_containers(
    cpp_artboard: &CppProbeArtboard,
    rust: &RuntimeFile,
    artboard_index: usize,
    label: &str,
) -> Result<(), String> {
    let rust_containers = rust.artboard_shape_paint_containers(artboard_index);
    if cpp_artboard.shape_paint_containers.len() != rust_containers.len() {
        return Err(format!(
            "{label}: artboard {artboard_index} shape-paint container count mismatch, C++ {}, Rust {}",
            cpp_artboard.shape_paint_containers.len(),
            rust_containers.len()
        ));
    }

    for (container_index, (cpp_container, rust_container)) in cpp_artboard
        .shape_paint_containers
        .iter()
        .zip(&rust_containers)
        .enumerate()
    {
        if cpp_container.local_id != rust_container.local_id {
            return Err(format!(
                "{label}: artboard {artboard_index} shape-paint container {container_index} local id mismatch, C++ {}, Rust {}",
                cpp_container.local_id, rust_container.local_id
            ));
        }
        if cpp_container.core_type != rust_container.object.type_key {
            return Err(format!(
                "{label}: artboard {artboard_index} shape-paint container {container_index} core type mismatch, C++ {}, Rust {}",
                cpp_container.core_type, rust_container.object.type_key
            ));
        }
        if cpp_container.paints.len() != rust_container.paints.len() {
            return Err(format!(
                "{label}: artboard {artboard_index} shape-paint container {container_index} paint count mismatch, C++ {}, Rust {}",
                cpp_container.paints.len(),
                rust_container.paints.len()
            ));
        }

        for (paint_index, (cpp_paint, rust_paint)) in cpp_container
            .paints
            .iter()
            .zip(&rust_container.paints)
            .enumerate()
        {
            if cpp_paint.index != paint_index {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} C++ index mismatch, got {}",
                    cpp_paint.index
                ));
            }
            if cpp_paint.local_id != Some(rust_paint.local_id) {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} local id mismatch, C++ {:?}, Rust {}",
                    cpp_paint.local_id, rust_paint.local_id
                ));
            }
            if cpp_paint.core_type != rust_paint.object.type_key {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} core type mismatch, C++ {}, Rust {}",
                    cpp_paint.core_type, rust_paint.object.type_key
                ));
            }
            if cpp_paint.mutator_local != rust_paint.mutator_local_id {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} mutator local mismatch, C++ {:?}, Rust {:?}",
                    cpp_paint.mutator_local, rust_paint.mutator_local_id
                ));
            }
            let rust_mutator_core_type = rust_paint.mutator.map(|object| object.type_key);
            if cpp_paint.mutator_core_type != rust_mutator_core_type {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} mutator core type mismatch, C++ {:?}, Rust {:?}",
                    cpp_paint.mutator_core_type, rust_mutator_core_type
                ));
            }
            if cpp_paint.feather_local != rust_paint.feather_local_id {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} feather local mismatch, C++ {:?}, Rust {:?}",
                    cpp_paint.feather_local, rust_paint.feather_local_id
                ));
            }
            let rust_feather_core_type = rust_paint.feather.map(|object| object.type_key);
            if cpp_paint.feather_core_type != rust_feather_core_type {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} feather core type mismatch, C++ {:?}, Rust {:?}",
                    cpp_paint.feather_core_type, rust_feather_core_type
                ));
            }
            if cpp_paint.effects.len() != rust_paint.effects.len() {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} effect count mismatch, C++ {}, Rust {}",
                    cpp_paint.effects.len(),
                    rust_paint.effects.len()
                ));
            }
            for (effect_index, (cpp_effect, rust_effect)) in cpp_paint
                .effects
                .iter()
                .zip(&rust_paint.effects)
                .enumerate()
            {
                if cpp_effect.index != effect_index {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} effect {effect_index} C++ index mismatch, got {}",
                        cpp_effect.index
                    ));
                }
                if cpp_effect.local_id != Some(rust_effect.local_id) {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} effect {effect_index} local id mismatch, C++ {:?}, Rust {}",
                        cpp_effect.local_id, rust_effect.local_id
                    ));
                }
                if cpp_effect.core_type != rust_effect.object.type_key {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} effect {effect_index} core type mismatch, C++ {}, Rust {}",
                        cpp_effect.core_type, rust_effect.object.type_key
                    ));
                }
                if cpp_effect.target_group_effect_local != rust_effect.target_group_effect_local_id
                {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} effect {effect_index} target group local mismatch, C++ {:?}, Rust {:?}",
                        cpp_effect.target_group_effect_local,
                        rust_effect.target_group_effect_local_id
                    ));
                }
                let rust_target_group_effect_core_type = rust_effect
                    .target_group_effect
                    .map(|object| object.type_key);
                if cpp_effect.target_group_effect_core_type != rust_target_group_effect_core_type {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} effect {effect_index} target group core type mismatch, C++ {:?}, Rust {:?}",
                        cpp_effect.target_group_effect_core_type,
                        rust_target_group_effect_core_type
                    ));
                }
            }
            if cpp_paint.gradient_stops.len() != rust_paint.gradient_stops.len() {
                return Err(format!(
                    "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} gradient stop count mismatch, C++ {}, Rust {}",
                    cpp_paint.gradient_stops.len(),
                    rust_paint.gradient_stops.len()
                ));
            }
            for (stop_index, (cpp_stop, rust_stop)) in cpp_paint
                .gradient_stops
                .iter()
                .zip(&rust_paint.gradient_stops)
                .enumerate()
            {
                if cpp_stop.index != stop_index {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} gradient stop {stop_index} C++ index mismatch, got {}",
                        cpp_stop.index
                    ));
                }
                if cpp_stop.local_id != Some(rust_stop.local_id) {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} gradient stop {stop_index} local id mismatch, C++ {:?}, Rust {}",
                        cpp_stop.local_id, rust_stop.local_id
                    ));
                }
                if cpp_stop.core_type != rust_stop.object.type_key {
                    return Err(format!(
                        "{label}: artboard {artboard_index} shape-paint container {container_index} paint {paint_index} gradient stop {stop_index} core type mismatch, C++ {}, Rust {}",
                        cpp_stop.core_type, rust_stop.object.type_key
                    ));
                }
            }
        }
    }

    Ok(())
}

fn compare_corpus_artboard_animation_summary(
    cpp_artboard: &CppProbeArtboard,
    rust: &RuntimeFile,
    artboard_index: usize,
    label: &str,
) -> Result<(), String> {
    let rust_animations = rust.artboard_linear_animations(artboard_index);
    if cpp_artboard.animations.len() != rust_animations.len() {
        return Err(format!(
            "{label}: artboard {artboard_index} animation count mismatch, C++ {}, Rust {}",
            cpp_artboard.animations.len(),
            rust_animations.len()
        ));
    }

    for (animation_index, (cpp_animation, rust_animation)) in cpp_artboard
        .animations
        .iter()
        .zip(&rust_animations)
        .enumerate()
    {
        compare_corpus_animation(
            cpp_animation,
            rust_animation.object,
            &rust_animation.keyed_objects,
            artboard_index,
            animation_index,
            label,
        )?;
    }

    let rust_state_machines = rust.artboard_state_machine_graphs(artboard_index);

    if rust_animations.is_empty() && rust_state_machines.is_empty() {
        if cpp_artboard_state_machines_are_auto_generated(&cpp_artboard.state_machines) {
            return Ok(());
        }
        return Err(format!(
            "{label}: artboard {artboard_index} expected C++ auto-generated empty state machine, got {} state machines",
            cpp_artboard.state_machines.len()
        ));
    }

    if cpp_artboard.state_machines.len() != rust_state_machines.len() {
        return Err(format!(
            "{label}: artboard {artboard_index} state machine count mismatch, C++ {}, Rust {}",
            cpp_artboard.state_machines.len(),
            rust_state_machines.len()
        ));
    }

    for (state_machine_index, (cpp_state_machine, rust_state_machine)) in cpp_artboard
        .state_machines
        .iter()
        .zip(&rust_state_machines)
        .enumerate()
    {
        compare_corpus_state_machine(
            rust,
            cpp_state_machine,
            rust_state_machine,
            artboard_index,
            state_machine_index,
            label,
        )?;
    }

    compare_corpus_artboard_data_binds(
        rust,
        &cpp_artboard.data_binds,
        &rust.artboard_data_binds(artboard_index),
        artboard_index,
        label,
    )?;

    Ok(())
}

fn compare_corpus_animation(
    cpp_animation: &CppProbeAnimation,
    rust_animation: &RuntimeObject,
    rust_keyed_objects: &[nuxie_binary::RuntimeKeyedObject<'_>],
    artboard_index: usize,
    animation_index: usize,
    label: &str,
) -> Result<(), String> {
    let prefix = format!("{label}: artboard {artboard_index} animation {animation_index}");
    if cpp_animation.index != animation_index {
        return Err(format!(
            "{prefix} C++ index mismatch, got {}",
            cpp_animation.index
        ));
    }
    if cpp_animation.core_type != rust_animation.type_key {
        return Err(format!(
            "{prefix} core type mismatch, C++ {}, Rust {}",
            cpp_animation.core_type, rust_animation.type_key
        ));
    }
    let rust_name = rust_animation.string_property("name").unwrap_or_default();
    if cpp_animation.name != rust_name {
        return Err(format!(
            "{prefix} name mismatch, C++ {:?}, Rust {:?}",
            cpp_animation.name, rust_name
        ));
    }
    if rust_animation.uint_property("fps") != Some(cpp_animation.fps) {
        return Err(format!(
            "{prefix} fps mismatch, C++ {}, Rust {:?}",
            cpp_animation.fps,
            rust_animation.uint_property("fps")
        ));
    }
    if rust_animation.uint_property("duration") != Some(cpp_animation.duration) {
        return Err(format!(
            "{prefix} duration mismatch, C++ {}, Rust {:?}",
            cpp_animation.duration,
            rust_animation.uint_property("duration")
        ));
    }
    if !float_matches(cpp_animation.speed, rust_animation.double_property("speed")) {
        return Err(format!(
            "{prefix} speed mismatch, C++ {}, Rust {:?}",
            cpp_animation.speed,
            rust_animation.double_property("speed")
        ));
    }
    if rust_animation.uint_property("loopValue") != Some(cpp_animation.loop_value) {
        return Err(format!(
            "{prefix} loop value mismatch, C++ {}, Rust {:?}",
            cpp_animation.loop_value,
            rust_animation.uint_property("loopValue")
        ));
    }
    if rust_animation.uint_property("workStart") != Some(cpp_animation.work_start) {
        return Err(format!(
            "{prefix} work start mismatch, C++ {}, Rust {:?}",
            cpp_animation.work_start,
            rust_animation.uint_property("workStart")
        ));
    }
    if rust_animation.uint_property("workEnd") != Some(cpp_animation.work_end) {
        return Err(format!(
            "{prefix} work end mismatch, C++ {}, Rust {:?}",
            cpp_animation.work_end,
            rust_animation.uint_property("workEnd")
        ));
    }
    if rust_animation.bool_property("enableWorkArea") != Some(cpp_animation.enable_work_area) {
        return Err(format!(
            "{prefix} enable work area mismatch, C++ {}, Rust {:?}",
            cpp_animation.enable_work_area,
            rust_animation.bool_property("enableWorkArea")
        ));
    }
    if rust_animation.bool_property("quantize") != Some(cpp_animation.quantize) {
        return Err(format!(
            "{prefix} quantize mismatch, C++ {}, Rust {:?}",
            cpp_animation.quantize,
            rust_animation.bool_property("quantize")
        ));
    }

    compare_cpp_registry_properties_result(
        &cpp_animation.property_values,
        rust_animation,
        &prefix,
    )?;
    compare_corpus_keyed_objects(
        &cpp_animation.keyed_objects,
        rust_keyed_objects,
        cpp_animation.fps,
        &prefix,
    )
}

fn compare_corpus_keyed_objects(
    cpp_keyed_objects: &[CppProbeKeyedObject],
    rust_keyed_objects: &[nuxie_binary::RuntimeKeyedObject<'_>],
    animation_fps: u64,
    label: &str,
) -> Result<(), String> {
    if cpp_keyed_objects.len() != rust_keyed_objects.len() {
        return Err(format!(
            "{label} keyed object count mismatch, C++ {}, Rust {}",
            cpp_keyed_objects.len(),
            rust_keyed_objects.len()
        ));
    }

    for (keyed_object_index, (cpp_keyed_object, rust_keyed_object)) in
        cpp_keyed_objects.iter().zip(rust_keyed_objects).enumerate()
    {
        let prefix = format!("{label} keyed object {keyed_object_index}");
        if cpp_keyed_object.index != keyed_object_index {
            return Err(format!(
                "{prefix} C++ index mismatch, got {}",
                cpp_keyed_object.index
            ));
        }
        if cpp_keyed_object.core_type != rust_keyed_object.object.type_key {
            return Err(format!(
                "{prefix} core type mismatch, C++ {}, Rust {}",
                cpp_keyed_object.core_type, rust_keyed_object.object.type_key
            ));
        }
        if rust_keyed_object.object.uint_property("objectId") != Some(cpp_keyed_object.object_id) {
            return Err(format!(
                "{prefix} object id mismatch, C++ {}, Rust {:?}",
                cpp_keyed_object.object_id,
                rust_keyed_object.object.uint_property("objectId")
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_keyed_object.property_values,
            rust_keyed_object.object,
            &prefix,
        )?;
        if cpp_keyed_object.keyed_properties.len() != rust_keyed_object.keyed_properties.len() {
            return Err(format!(
                "{prefix} keyed property count mismatch, C++ {}, Rust {}",
                cpp_keyed_object.keyed_properties.len(),
                rust_keyed_object.keyed_properties.len()
            ));
        }
        for (keyed_property_index, (cpp_keyed_property, rust_keyed_property)) in cpp_keyed_object
            .keyed_properties
            .iter()
            .zip(&rust_keyed_object.keyed_properties)
            .enumerate()
        {
            let property_prefix = format!("{prefix} keyed property {keyed_property_index}");
            if cpp_keyed_property.index != keyed_property_index {
                return Err(format!(
                    "{property_prefix} C++ index mismatch, got {}",
                    cpp_keyed_property.index
                ));
            }
            if cpp_keyed_property.core_type != rust_keyed_property.object.type_key {
                return Err(format!(
                    "{property_prefix} core type mismatch, C++ {}, Rust {}",
                    cpp_keyed_property.core_type, rust_keyed_property.object.type_key
                ));
            }
            if rust_keyed_property.object.uint_property("propertyKey")
                != Some(cpp_keyed_property.property_key)
            {
                return Err(format!(
                    "{property_prefix} property key mismatch, C++ {}, Rust {:?}",
                    cpp_keyed_property.property_key,
                    rust_keyed_property.object.uint_property("propertyKey")
                ));
            }
            compare_cpp_registry_properties_result(
                &cpp_keyed_property.property_values,
                rust_keyed_property.object,
                &property_prefix,
            )?;

            match (
                &cpp_keyed_property.first_key_frame,
                rust_keyed_property.first_key_frame,
            ) {
                (None, None) => {}
                (Some(cpp_key_frame), Some(rust_key_frame)) => {
                    if cpp_key_frame.core_type != rust_key_frame.type_key {
                        return Err(format!(
                            "{property_prefix} first keyframe core type mismatch, C++ {}, Rust {}",
                            cpp_key_frame.core_type, rust_key_frame.type_key
                        ));
                    }
                    if rust_key_frame.uint_property("frame") != Some(cpp_key_frame.frame) {
                        return Err(format!(
                            "{property_prefix} first keyframe frame mismatch, C++ {}, Rust {:?}",
                            cpp_key_frame.frame,
                            rust_key_frame.uint_property("frame")
                        ));
                    }
                    let expected_seconds = rust_key_frame
                        .uint_property("frame")
                        .map(|frame| frame as f32 / animation_fps as f32);
                    if !float_matches(cpp_key_frame.seconds, expected_seconds) {
                        return Err(format!(
                            "{property_prefix} first keyframe seconds mismatch, C++ {}, Rust {:?}",
                            cpp_key_frame.seconds, expected_seconds
                        ));
                    }
                    compare_cpp_registry_properties_result(
                        &cpp_key_frame.property_values,
                        rust_key_frame,
                        &format!("{property_prefix} first keyframe"),
                    )?;
                }
                (None, Some(rust_key_frame)) => {
                    return Err(format!(
                        "{property_prefix} has no C++ first keyframe but Rust has {}",
                        rust_key_frame.type_name
                    ));
                }
                (Some(cpp_key_frame), None) => {
                    return Err(format!(
                        "{property_prefix} has no Rust first keyframe but C++ has core type {}",
                        cpp_key_frame.core_type
                    ));
                }
            }
        }
    }

    Ok(())
}

fn compare_corpus_state_machine(
    rust_file: &RuntimeFile,
    cpp_state_machine: &CppProbeStateMachine,
    rust_state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    artboard_index: usize,
    state_machine_index: usize,
    label: &str,
) -> Result<(), String> {
    let prefix = format!("{label}: artboard {artboard_index} state machine {state_machine_index}");
    if cpp_state_machine.index != state_machine_index {
        return Err(format!(
            "{prefix} C++ index mismatch, got {}",
            cpp_state_machine.index
        ));
    }
    if cpp_state_machine.core_type != rust_state_machine.object.type_key {
        return Err(format!(
            "{prefix} core type mismatch, C++ {}, Rust {}",
            cpp_state_machine.core_type, rust_state_machine.object.type_key
        ));
    }
    let rust_name = rust_state_machine
        .object
        .string_property("name")
        .unwrap_or_default();
    if cpp_state_machine.name != rust_name {
        return Err(format!(
            "{prefix} name mismatch, C++ {:?}, Rust {:?}",
            cpp_state_machine.name, rust_name
        ));
    }

    compare_cpp_registry_properties_result(
        &cpp_state_machine.property_values,
        rust_state_machine.object,
        &prefix,
    )?;
    compare_corpus_state_machine_children(rust_file, cpp_state_machine, rust_state_machine, &prefix)
}

fn compare_corpus_artboard_data_binds(
    rust_file: &RuntimeFile,
    cpp_data_binds: &[Option<CppProbeArtboardDataBind>],
    rust_data_binds: &[nuxie_binary::RuntimeDataBind<'_>],
    artboard_index: usize,
    label: &str,
) -> Result<(), String> {
    let prefix = format!("{label}: artboard {artboard_index} data binds");
    if cpp_data_binds.len() != rust_data_binds.len() {
        return Err(format!(
            "{prefix} count mismatch, C++ {}, Rust {}",
            cpp_data_binds.len(),
            rust_data_binds.len()
        ));
    }

    for (data_bind_index, (cpp_data_bind, rust_data_bind)) in
        cpp_data_binds.iter().zip(rust_data_binds).enumerate()
    {
        let Some(cpp_data_bind) = cpp_data_bind.as_ref() else {
            return Err(format!(
                "{prefix} data bind {data_bind_index} is null in C++"
            ));
        };
        let bind_prefix = format!("{prefix} data bind {data_bind_index}");
        if cpp_data_bind.index != data_bind_index {
            return Err(format!(
                "{bind_prefix} C++ index mismatch, got {}",
                cpp_data_bind.index
            ));
        }
        if cpp_data_bind.core_type != rust_data_bind.object.type_key {
            return Err(format!(
                "{bind_prefix} core type mismatch, C++ {}, Rust {}",
                cpp_data_bind.core_type, rust_data_bind.object.type_key
            ));
        }
        if rust_data_bind.object.uint_property("propertyKey") != Some(cpp_data_bind.property_key) {
            return Err(format!(
                "{bind_prefix} property key mismatch, C++ {}, Rust {:?}",
                cpp_data_bind.property_key,
                rust_data_bind.object.uint_property("propertyKey")
            ));
        }
        if rust_data_bind.object.uint_property("flags") != Some(cpp_data_bind.flags) {
            return Err(format!(
                "{bind_prefix} flags mismatch, C++ {}, Rust {:?}",
                cpp_data_bind.flags,
                rust_data_bind.object.uint_property("flags")
            ));
        }
        if rust_data_bind.object.uint_property("converterId") != Some(cpp_data_bind.converter_id) {
            return Err(format!(
                "{bind_prefix} converter id mismatch, C++ {}, Rust {:?}",
                cpp_data_bind.converter_id,
                rust_data_bind.object.uint_property("converterId")
            ));
        }
        let rust_converter_core_type = rust_data_bind.converter.map(|converter| converter.type_key);
        if rust_converter_core_type != cpp_data_bind.converter_core_type {
            return Err(format!(
                "{bind_prefix} converter core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_core_type, rust_converter_core_type
            ));
        }
        let rust_converter_output_type =
            data_converter_output_type(rust_file, rust_data_bind.converter);
        if rust_converter_output_type != cpp_data_bind.converter_output_type {
            return Err(format!(
                "{bind_prefix} converter output type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_output_type, rust_converter_output_type
            ));
        }
        let rust_source_output_type =
            data_bind_source_output_type(rust_file, rust_data_bind.object);
        if rust_source_output_type != cpp_data_bind.data_bind_source_output_type {
            return Err(format!(
                "{bind_prefix} source output type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.data_bind_source_output_type, rust_source_output_type
            ));
        }
        let rust_output_type = data_bind_output_type(rust_file, rust_data_bind.object);
        if rust_output_type != cpp_data_bind.data_bind_output_type {
            return Err(format!(
                "{bind_prefix} output type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.data_bind_output_type, rust_output_type
            ));
        }
        let rust_target_supports_push =
            data_bind_target_supports_push(rust_file, rust_data_bind.object);
        if rust_target_supports_push != cpp_data_bind.target_supports_push {
            return Err(format!(
                "{bind_prefix} target-supports-push mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.target_supports_push, rust_target_supports_push
            ));
        }
        let rust_uses_persisting_list =
            data_bind_uses_persisting_list(rust_file, rust_data_bind.object);
        if rust_uses_persisting_list != cpp_data_bind.uses_persisting_list {
            return Err(format!(
                "{bind_prefix} persisting-list mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.uses_persisting_list, rust_uses_persisting_list
            ));
        }
        let rust_converter_source_path_ids =
            data_converter_operation_view_model_source_path_ids(rust_data_bind.converter);
        if rust_converter_source_path_ids != cpp_data_bind.converter_source_path_ids {
            return Err(format!(
                "{bind_prefix} converter source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_source_path_ids, rust_converter_source_path_ids
            ));
        }
        let rust_number_to_list_view_model =
            resolved_number_to_list_view_model(rust_file, rust_data_bind.converter);
        let rust_number_to_list_view_model_core_type = rust_number_to_list_view_model
            .as_ref()
            .map(|view_model| view_model.object.type_key);
        if rust_number_to_list_view_model_core_type
            != cpp_data_bind.converter_number_to_list_view_model_core_type
        {
            return Err(format!(
                "{bind_prefix} number-to-list view model core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_number_to_list_view_model_core_type,
                rust_number_to_list_view_model_core_type
            ));
        }
        let rust_number_to_list_view_model_name = rust_number_to_list_view_model
            .as_ref()
            .and_then(|view_model| view_model.object.string_property("name"));
        if rust_number_to_list_view_model_name
            != cpp_data_bind
                .converter_number_to_list_view_model_name
                .as_deref()
        {
            return Err(format!(
                "{bind_prefix} number-to-list view model name mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_number_to_list_view_model_name,
                rust_number_to_list_view_model_name
            ));
        }
        let rust_context_source_path_ids =
            data_bind_context_source_path_ids(rust_file, rust_data_bind.object);
        if rust_context_source_path_ids != cpp_data_bind.data_bind_context_source_path_ids {
            return Err(format!(
                "{bind_prefix} DataBindContext source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.data_bind_context_source_path_ids, rust_context_source_path_ids
            ));
        }
        let rust_resolved_context_source_path_ids =
            data_bind_context_resolved_source_path_ids(rust_file, rust_data_bind.object);
        if rust_resolved_context_source_path_ids
            != cpp_data_bind.resolved_data_bind_context_source_path_ids
        {
            return Err(format!(
                "{bind_prefix} resolved DataBindContext source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.resolved_data_bind_context_source_path_ids,
                rust_resolved_context_source_path_ids
            ));
        }
        let rust_converter_interpolator_core_type =
            resolved_data_converter_interpolator_core_type(rust_file, rust_data_bind.converter);
        if rust_converter_interpolator_core_type != cpp_data_bind.converter_interpolator_core_type {
            return Err(format!(
                "{bind_prefix} converter interpolator core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_interpolator_core_type,
                rust_converter_interpolator_core_type
            ));
        }
        compare_data_converter_group_items_result(
            rust_file,
            &cpp_data_bind.converter_group_items,
            &resolved_data_bind_converter_group_items(rust_file, rust_data_bind.converter),
            &format!("{bind_prefix} converter group items"),
        )?;
        compare_data_converter_formula_tokens_result(
            &cpp_data_bind.converter_formula_tokens,
            &resolved_data_bind_converter_formula_tokens(rust_file, rust_data_bind.converter),
            &format!("{bind_prefix} converter formula tokens"),
        )?;
        let rust_target_core_type = rust_data_bind.target.map(|target| target.type_key);
        if rust_target_core_type != cpp_data_bind.target_core_type {
            return Err(format!(
                "{bind_prefix} target core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.target_core_type, rust_target_core_type
            ));
        }
        if rust_data_bind.target_local_id != cpp_data_bind.target_local {
            return Err(format!(
                "{bind_prefix} target local mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.target_local, rust_data_bind.target_local_id
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_data_bind.property_values,
            rust_data_bind.object,
            &bind_prefix,
        )?;
    }

    Ok(())
}

fn compare_corpus_state_machine_children(
    rust_file: &RuntimeFile,
    cpp_state_machine: &CppProbeStateMachine,
    rust_children: &nuxie_binary::RuntimeStateMachine<'_>,
    label: &str,
) -> Result<(), String> {
    if cpp_state_machine.layer_count != rust_children.layers.len()
        || cpp_state_machine.layers.len() != rust_children.layers.len()
    {
        return Err(format!(
            "{label} layer count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_state_machine.layer_count,
            cpp_state_machine.layers.len(),
            rust_children.layers.len()
        ));
    }
    for (layer_index, (cpp_layer, rust_layer)) in cpp_state_machine
        .layers
        .iter()
        .zip(&rust_children.layers)
        .enumerate()
    {
        let Some(cpp_layer) = cpp_layer.as_ref() else {
            return Err(format!("{label} layer {layer_index} is null in C++"));
        };
        if cpp_layer.index != layer_index {
            return Err(format!(
                "{label} layer {layer_index} C++ index mismatch, got {}",
                cpp_layer.index
            ));
        }
        if cpp_layer.core_type != rust_layer.object.type_key {
            return Err(format!(
                "{label} layer {layer_index} core type mismatch, C++ {}, Rust {}",
                cpp_layer.core_type, rust_layer.object.type_key
            ));
        }
        let rust_name = rust_layer
            .object
            .string_property("name")
            .unwrap_or_default();
        if cpp_layer.name != rust_name {
            return Err(format!(
                "{label} layer {layer_index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_layer.name, rust_name
            ));
        }
        if cpp_layer.state_count != rust_layer.state_count {
            return Err(format!(
                "{label} layer {layer_index} state count mismatch, C++ {}, Rust {}",
                cpp_layer.state_count, rust_layer.state_count
            ));
        }
        compare_state_machine_layer_states(
            rust_file,
            cpp_layer,
            rust_layer,
            &format!("{label} layer {layer_index}"),
        )?;
        compare_cpp_registry_properties_result(
            &cpp_layer.property_values,
            rust_layer.object,
            &format!("{label} layer {layer_index}"),
        )?;
    }

    if cpp_state_machine.input_count != rust_children.inputs.len()
        || cpp_state_machine.inputs.len() != rust_children.inputs.len()
    {
        return Err(format!(
            "{label} input count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_state_machine.input_count,
            cpp_state_machine.inputs.len(),
            rust_children.inputs.len()
        ));
    }
    for (input_index, (cpp_input, rust_input)) in cpp_state_machine
        .inputs
        .iter()
        .zip(&rust_children.inputs)
        .enumerate()
    {
        let Some(cpp_input) = cpp_input.as_ref() else {
            return Err(format!("{label} input {input_index} is null in C++"));
        };
        if cpp_input.index != input_index {
            return Err(format!(
                "{label} input {input_index} C++ index mismatch, got {}",
                cpp_input.index
            ));
        }
        if cpp_input.core_type != rust_input.type_key {
            return Err(format!(
                "{label} input {input_index} core type mismatch, C++ {}, Rust {}",
                cpp_input.core_type, rust_input.type_key
            ));
        }
        let rust_name = rust_input.string_property("name").unwrap_or_default();
        if cpp_input.name != rust_name {
            return Err(format!(
                "{label} input {input_index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_input.name, rust_name
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_input.property_values,
            rust_input,
            &format!("{label} input {input_index}"),
        )?;
    }

    if cpp_state_machine.listener_count != rust_children.listeners.len()
        || cpp_state_machine.listeners.len() != rust_children.listeners.len()
    {
        return Err(format!(
            "{label} listener count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_state_machine.listener_count,
            cpp_state_machine.listeners.len(),
            rust_children.listeners.len()
        ));
    }
    for (listener_index, (cpp_listener, rust_listener)) in cpp_state_machine
        .listeners
        .iter()
        .zip(&rust_children.listeners)
        .enumerate()
    {
        let Some(cpp_listener) = cpp_listener.as_ref() else {
            return Err(format!("{label} listener {listener_index} is null in C++"));
        };
        if cpp_listener.index != listener_index {
            return Err(format!(
                "{label} listener {listener_index} C++ index mismatch, got {}",
                cpp_listener.index
            ));
        }
        if cpp_listener.core_type != rust_listener.object.type_key {
            return Err(format!(
                "{label} listener {listener_index} core type mismatch, C++ {}, Rust {}",
                cpp_listener.core_type, rust_listener.object.type_key
            ));
        }
        let rust_name = rust_listener
            .object
            .string_property("name")
            .unwrap_or_default();
        if cpp_listener.name != rust_name {
            return Err(format!(
                "{label} listener {listener_index} name mismatch, C++ {:?}, Rust {:?}",
                cpp_listener.name, rust_name
            ));
        }
        if rust_listener.object.uint_property("targetId") != Some(cpp_listener.target_id) {
            return Err(format!(
                "{label} listener {listener_index} target id mismatch, C++ {}, Rust {:?}",
                cpp_listener.target_id,
                rust_listener.object.uint_property("targetId")
            ));
        }
        let rust_listener_path = data_bind_path_ids_for_referencer(rust_file, rust_listener.object);
        if cpp_listener.data_bind_path_ids != rust_listener_path {
            return Err(format!(
                "{label} listener {listener_index} data-bind path mismatch, C++ {:?}, Rust {:?}",
                cpp_listener.data_bind_path_ids, rust_listener_path
            ));
        }
        let rust_listener_resolved_path =
            resolved_data_bind_path_ids_for_referencer(rust_file, rust_listener.object);
        if cpp_listener.resolved_data_bind_path_ids != rust_listener_resolved_path {
            return Err(format!(
                "{label} listener {listener_index} resolved data-bind path mismatch, C++ {:?}, Rust {:?}",
                cpp_listener.resolved_data_bind_path_ids, rust_listener_resolved_path
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_listener.property_values,
            rust_listener.object,
            &format!("{label} listener {listener_index}"),
        )?;
        if cpp_listener.action_count != rust_listener.actions.len()
            || cpp_listener.actions.len() != rust_listener.actions.len()
        {
            return Err(format!(
                "{label} listener {listener_index} action count mismatch, C++ count/list {}/{}, Rust {}",
                cpp_listener.action_count,
                cpp_listener.actions.len(),
                rust_listener.actions.len()
            ));
        }
        for (action_index, (cpp_action, rust_action)) in cpp_listener
            .actions
            .iter()
            .zip(&rust_listener.actions)
            .enumerate()
        {
            let Some(cpp_action) = cpp_action.as_ref() else {
                return Err(format!(
                    "{label} listener {listener_index} action {action_index} is null in C++"
                ));
            };
            let rust_action = rust_action.object;
            if cpp_action.index != action_index {
                return Err(format!(
                    "{label} listener {listener_index} action {action_index} C++ index mismatch, got {}",
                    cpp_action.index
                ));
            }
            if cpp_action.core_type != rust_action.type_key {
                return Err(format!(
                    "{label} listener {listener_index} action {action_index} core type mismatch, C++ {}, Rust {}",
                    cpp_action.core_type, rust_action.type_key
                ));
            }
            if rust_action.uint_property("flags") != Some(cpp_action.flags) {
                return Err(format!(
                    "{label} listener {listener_index} action {action_index} flags mismatch, C++ {}, Rust {:?}",
                    cpp_action.flags,
                    rust_action.uint_property("flags")
                ));
            }
            let rust_action_path = data_bind_path_ids_for_referencer(rust_file, rust_action);
            if cpp_action.data_bind_path_ids != rust_action_path {
                return Err(format!(
                    "{label} listener {listener_index} action {action_index} data-bind path mismatch, C++ {:?}, Rust {:?}",
                    cpp_action.data_bind_path_ids, rust_action_path
                ));
            }
            let rust_action_resolved_path =
                resolved_data_bind_path_ids_for_referencer(rust_file, rust_action);
            if cpp_action.resolved_data_bind_path_ids != rust_action_resolved_path {
                return Err(format!(
                    "{label} listener {listener_index} action {action_index} resolved data-bind path mismatch, C++ {:?}, Rust {:?}",
                    cpp_action.resolved_data_bind_path_ids, rust_action_resolved_path
                ));
            }
            compare_cpp_registry_properties_result(
                &cpp_action.property_values,
                rust_action,
                &format!("{label} listener {listener_index} action {action_index}"),
            )?;
        }
        if cpp_listener.listener_input_type_count != rust_listener.listener_input_types.len()
            || cpp_listener.listener_input_types.len() != rust_listener.listener_input_types.len()
        {
            return Err(format!(
                "{label} listener {listener_index} listener input type count mismatch, C++ count/list {}/{}, Rust {}",
                cpp_listener.listener_input_type_count,
                cpp_listener.listener_input_types.len(),
                rust_listener.listener_input_types.len()
            ));
        }
        for (input_type_index, (cpp_input_type, rust_input_type)) in cpp_listener
            .listener_input_types
            .iter()
            .zip(&rust_listener.listener_input_types)
            .enumerate()
        {
            let Some(cpp_input_type) = cpp_input_type.as_ref() else {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} is null in C++"
                ));
            };
            if cpp_input_type.index != input_type_index {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} C++ index mismatch, got {}",
                    cpp_input_type.index
                ));
            }
            if cpp_input_type.core_type != rust_input_type.type_key {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} core type mismatch, C++ {}, Rust {}",
                    cpp_input_type.core_type, rust_input_type.type_key
                ));
            }
            if rust_input_type.uint_property("listenerTypeValue")
                != Some(cpp_input_type.listener_type_value)
            {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} listener type value mismatch, C++ {}, Rust {:?}",
                    cpp_input_type.listener_type_value,
                    rust_input_type.uint_property("listenerTypeValue")
                ));
            }
            let rust_input_type_path =
                data_bind_path_ids_for_referencer(rust_file, rust_input_type);
            if cpp_input_type.data_bind_path_ids != rust_input_type_path {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} data-bind path mismatch, C++ {:?}, Rust {:?}",
                    cpp_input_type.data_bind_path_ids, rust_input_type_path
                ));
            }
            let rust_input_type_resolved_path =
                resolved_data_bind_path_ids_for_referencer(rust_file, rust_input_type);
            if cpp_input_type.resolved_data_bind_path_ids != rust_input_type_resolved_path {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} resolved data-bind path mismatch, C++ {:?}, Rust {:?}",
                    cpp_input_type.resolved_data_bind_path_ids, rust_input_type_resolved_path
                ));
            }
            let rust_view_model_path_ids_buffer = rust_file
                .listener_input_type_view_model_path_ids_buffer_for_object(rust_input_type);
            if cpp_input_type.view_model_path_ids_buffer != rust_view_model_path_ids_buffer {
                return Err(format!(
                    "{label} listener {listener_index} input type {input_type_index} view-model path ids buffer mismatch, C++ {:?}, Rust {:?}",
                    cpp_input_type.view_model_path_ids_buffer, rust_view_model_path_ids_buffer
                ));
            }
            compare_cpp_registry_properties_result(
                &cpp_input_type.property_values,
                rust_input_type,
                &format!("{label} listener {listener_index} input type {input_type_index}"),
            )?;
        }
    }

    if cpp_state_machine.data_bind_count != rust_children.data_binds.len()
        || cpp_state_machine.data_binds.len() != rust_children.data_binds.len()
    {
        return Err(format!(
            "{label} data bind count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_state_machine.data_bind_count,
            cpp_state_machine.data_binds.len(),
            rust_children.data_binds.len()
        ));
    }
    for (data_bind_index, (cpp_data_bind, rust_data_bind)) in cpp_state_machine
        .data_binds
        .iter()
        .zip(&rust_children.data_binds)
        .enumerate()
    {
        let Some(cpp_data_bind) = cpp_data_bind.as_ref() else {
            return Err(format!(
                "{label} data bind {data_bind_index} is null in C++"
            ));
        };
        if cpp_data_bind.index != data_bind_index {
            return Err(format!(
                "{label} data bind {data_bind_index} C++ index mismatch, got {}",
                cpp_data_bind.index
            ));
        }
        if cpp_data_bind.core_type != rust_data_bind.type_key {
            return Err(format!(
                "{label} data bind {data_bind_index} core type mismatch, C++ {}, Rust {}",
                cpp_data_bind.core_type, rust_data_bind.type_key
            ));
        }
        if rust_data_bind.uint_property("propertyKey") != Some(cpp_data_bind.property_key) {
            return Err(format!(
                "{label} data bind {data_bind_index} property key mismatch, C++ {}, Rust {:?}",
                cpp_data_bind.property_key,
                rust_data_bind.uint_property("propertyKey")
            ));
        }
        if rust_data_bind.uint_property("flags") != Some(cpp_data_bind.flags) {
            return Err(format!(
                "{label} data bind {data_bind_index} flags mismatch, C++ {}, Rust {:?}",
                cpp_data_bind.flags,
                rust_data_bind.uint_property("flags")
            ));
        }
        if rust_data_bind.uint_property("converterId") != Some(cpp_data_bind.converter_id) {
            return Err(format!(
                "{label} data bind {data_bind_index} converter id mismatch, C++ {}, Rust {:?}",
                cpp_data_bind.converter_id,
                rust_data_bind.uint_property("converterId")
            ));
        }
        let rust_converter_core_type = rust_file
            .resolved_data_converter_for_data_bind_object(rust_data_bind)
            .map(|converter| converter.type_key);
        if rust_converter_core_type != cpp_data_bind.converter_core_type {
            return Err(format!(
                "{label} data bind {data_bind_index} converter core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_core_type, rust_converter_core_type
            ));
        }
        let rust_converter = rust_file.resolved_data_converter_for_data_bind_object(rust_data_bind);
        let rust_converter_output_type = data_converter_output_type(rust_file, rust_converter);
        if rust_converter_output_type != cpp_data_bind.converter_output_type {
            return Err(format!(
                "{label} data bind {data_bind_index} converter output type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_output_type, rust_converter_output_type
            ));
        }
        let rust_source_output_type = data_bind_source_output_type(rust_file, rust_data_bind);
        if rust_source_output_type != cpp_data_bind.data_bind_source_output_type {
            return Err(format!(
                "{label} data bind {data_bind_index} source output type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.data_bind_source_output_type, rust_source_output_type
            ));
        }
        let rust_output_type = data_bind_output_type(rust_file, rust_data_bind);
        if rust_output_type != cpp_data_bind.data_bind_output_type {
            return Err(format!(
                "{label} data bind {data_bind_index} output type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.data_bind_output_type, rust_output_type
            ));
        }
        let rust_target_supports_push = data_bind_target_supports_push(rust_file, rust_data_bind);
        if rust_target_supports_push != cpp_data_bind.target_supports_push {
            return Err(format!(
                "{label} data bind {data_bind_index} target-supports-push mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.target_supports_push, rust_target_supports_push
            ));
        }
        let rust_uses_persisting_list = data_bind_uses_persisting_list(rust_file, rust_data_bind);
        if rust_uses_persisting_list != cpp_data_bind.uses_persisting_list {
            return Err(format!(
                "{label} data bind {data_bind_index} persisting-list mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.uses_persisting_list, rust_uses_persisting_list
            ));
        }
        let rust_converter_source_path_ids =
            data_converter_operation_view_model_source_path_ids(rust_converter);
        if rust_converter_source_path_ids != cpp_data_bind.converter_source_path_ids {
            return Err(format!(
                "{label} data bind {data_bind_index} converter source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_source_path_ids, rust_converter_source_path_ids
            ));
        }
        let rust_number_to_list_view_model =
            resolved_number_to_list_view_model(rust_file, rust_converter);
        let rust_number_to_list_view_model_core_type = rust_number_to_list_view_model
            .as_ref()
            .map(|view_model| view_model.object.type_key);
        if rust_number_to_list_view_model_core_type
            != cpp_data_bind.converter_number_to_list_view_model_core_type
        {
            return Err(format!(
                "{label} data bind {data_bind_index} number-to-list view model core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_number_to_list_view_model_core_type,
                rust_number_to_list_view_model_core_type
            ));
        }
        let rust_number_to_list_view_model_name = rust_number_to_list_view_model
            .as_ref()
            .and_then(|view_model| view_model.object.string_property("name"));
        if rust_number_to_list_view_model_name
            != cpp_data_bind
                .converter_number_to_list_view_model_name
                .as_deref()
        {
            return Err(format!(
                "{label} data bind {data_bind_index} number-to-list view model name mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_number_to_list_view_model_name,
                rust_number_to_list_view_model_name
            ));
        }
        let rust_context_source_path_ids =
            data_bind_context_source_path_ids(rust_file, rust_data_bind);
        if rust_context_source_path_ids != cpp_data_bind.data_bind_context_source_path_ids {
            return Err(format!(
                "{label} data bind {data_bind_index} DataBindContext source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.data_bind_context_source_path_ids, rust_context_source_path_ids
            ));
        }
        let rust_resolved_context_source_path_ids =
            data_bind_context_resolved_source_path_ids(rust_file, rust_data_bind);
        if rust_resolved_context_source_path_ids
            != cpp_data_bind.resolved_data_bind_context_source_path_ids
        {
            return Err(format!(
                "{label} data bind {data_bind_index} resolved DataBindContext source path ids mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.resolved_data_bind_context_source_path_ids,
                rust_resolved_context_source_path_ids
            ));
        }
        let rust_converter_interpolator_core_type =
            resolved_data_converter_interpolator_core_type(rust_file, rust_converter);
        if rust_converter_interpolator_core_type != cpp_data_bind.converter_interpolator_core_type {
            return Err(format!(
                "{label} data bind {data_bind_index} converter interpolator core type mismatch, C++ {:?}, Rust {:?}",
                cpp_data_bind.converter_interpolator_core_type,
                rust_converter_interpolator_core_type
            ));
        }
        compare_data_converter_group_items_result(
            rust_file,
            &cpp_data_bind.converter_group_items,
            &resolved_data_bind_converter_group_items(rust_file, rust_converter),
            &format!("{label} data bind {data_bind_index} converter group items"),
        )?;
        compare_data_converter_formula_tokens_result(
            &cpp_data_bind.converter_formula_tokens,
            &resolved_data_bind_converter_formula_tokens(rust_file, rust_converter),
            &format!("{label} data bind {data_bind_index} converter formula tokens"),
        )?;
        compare_cpp_registry_properties_result(
            &cpp_data_bind.property_values,
            rust_data_bind,
            &format!("{label} data bind {data_bind_index}"),
        )?;
    }

    if cpp_state_machine.scripted_object_count != rust_children.scripted_objects.len()
        || cpp_state_machine.scripted_objects.len() != rust_children.scripted_objects.len()
    {
        return Err(format!(
            "{label} scripted object count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_state_machine.scripted_object_count,
            cpp_state_machine.scripted_objects.len(),
            rust_children.scripted_objects.len()
        ));
    }
    for (scripted_object_index, (cpp_scripted_object, rust_scripted_object)) in cpp_state_machine
        .scripted_objects
        .iter()
        .zip(&rust_children.scripted_objects)
        .enumerate()
    {
        if cpp_scripted_object.index != scripted_object_index {
            return Err(format!(
                "{label} scripted object {scripted_object_index} C++ index mismatch, got {}",
                cpp_scripted_object.index
            ));
        }
        if cpp_scripted_object.core_type != rust_scripted_object.object.type_key {
            return Err(format!(
                "{label} scripted object {scripted_object_index} core type mismatch, C++ {}, Rust {}",
                cpp_scripted_object.core_type, rust_scripted_object.object.type_key
            ));
        }
        if cpp_scripted_object.input_count != rust_scripted_object.inputs.len()
            || cpp_scripted_object.inputs.len() != rust_scripted_object.inputs.len()
        {
            return Err(format!(
                "{label} scripted object {scripted_object_index} input count mismatch, C++ count/list {}/{}, Rust {}",
                cpp_scripted_object.input_count,
                cpp_scripted_object.inputs.len(),
                rust_scripted_object.inputs.len()
            ));
        }
        for (input_index, (cpp_input, rust_input)) in cpp_scripted_object
            .inputs
            .iter()
            .zip(&rust_scripted_object.inputs)
            .enumerate()
        {
            if cpp_input.index != input_index {
                return Err(format!(
                    "{label} scripted object {scripted_object_index} input {input_index} C++ index mismatch, got {}",
                    cpp_input.index
                ));
            }
            if cpp_input.core_type != rust_input.type_key {
                return Err(format!(
                    "{label} scripted object {scripted_object_index} input {input_index} core type mismatch, C++ {}, Rust {}",
                    cpp_input.core_type, rust_input.type_key
                ));
            }
            let rust_name = rust_input.string_property("name").unwrap_or_default();
            if cpp_input.name != rust_name {
                return Err(format!(
                    "{label} scripted object {scripted_object_index} input {input_index} name mismatch, C++ {:?}, Rust {:?}",
                    cpp_input.name, rust_name
                ));
            }
            compare_cpp_registry_properties_result(
                &cpp_input.property_values,
                rust_input,
                &format!("{label} scripted object {scripted_object_index} input {input_index}"),
            )?;
        }
    }

    Ok(())
}

fn compare_state_machine_layer_states(
    rust_file: &RuntimeFile,
    cpp_layer: &CppProbeStateMachineLayer,
    rust_layer: &nuxie_binary::RuntimeStateMachineLayer<'_>,
    label: &str,
) -> Result<(), String> {
    if cpp_layer.states.len() != rust_layer.states.len() {
        return Err(format!(
            "{label} state list length mismatch, C++ {}, Rust {}",
            cpp_layer.states.len(),
            rust_layer.states.len()
        ));
    }

    for (state_index, (cpp_state, rust_state)) in
        cpp_layer.states.iter().zip(&rust_layer.states).enumerate()
    {
        let Some(cpp_state) = cpp_state.as_ref() else {
            return Err(format!("{label} state {state_index} is null in C++"));
        };
        if cpp_state.index != state_index {
            return Err(format!(
                "{label} state {state_index} C++ index mismatch, got {}",
                cpp_state.index
            ));
        }

        let Some(rust_state_object) = rust_state.object else {
            let layer_state_type_key = definition_by_name("LayerState")
                .expect("LayerState exists in schema")
                .type_key
                .int;
            if cpp_state.core_type != layer_state_type_key {
                return Err(format!(
                    "{label} placeholder state {state_index} core type mismatch, C++ {}, Rust placeholder LayerState {}",
                    cpp_state.core_type, layer_state_type_key
                ));
            }
            if cpp_state.transition_count != 0 || !rust_state.transitions.is_empty() {
                return Err(format!(
                    "{label} placeholder state {state_index} transition count mismatch, C++ {}, Rust {}",
                    cpp_state.transition_count,
                    rust_state.transitions.len()
                ));
            }
            if cpp_state.fire_action_count != 0 || !rust_state.fire_actions.is_empty() {
                return Err(format!(
                    "{label} placeholder state {state_index} fire action count mismatch, C++ {}, Rust {}",
                    cpp_state.fire_action_count,
                    rust_state.fire_actions.len()
                ));
            }
            if cpp_state.listener_action_count != 0 || !rust_state.listener_actions.is_empty() {
                return Err(format!(
                    "{label} placeholder state {state_index} listener action count mismatch, C++ {}, Rust {}",
                    cpp_state.listener_action_count,
                    rust_state.listener_actions.len()
                ));
            }
            if !cpp_state.blend_animations.is_empty() || !rust_state.blend_animations.is_empty() {
                return Err(format!(
                    "{label} placeholder state {state_index} blend animation count mismatch, C++ {}, Rust {}",
                    cpp_state.blend_animations.len(),
                    rust_state.blend_animations.len()
                ));
            }
            continue;
        };

        if cpp_state.core_type != rust_state_object.type_key {
            return Err(format!(
                "{label} state {state_index} core type mismatch, C++ {}, Rust {}",
                cpp_state.core_type, rust_state_object.type_key
            ));
        }
        if rust_state_object.uint_property("animationId") != cpp_state.animation_id {
            return Err(format!(
                "{label} state {state_index} animation id mismatch, C++ {:?}, Rust {:?}",
                cpp_state.animation_id,
                rust_state_object.uint_property("animationId")
            ));
        }
        let rust_animation_core_type = rust_state.animation.map(|animation| animation.type_key);
        if cpp_state.animation_core_type != rust_animation_core_type {
            return Err(format!(
                "{label} state {state_index} animation core type mismatch, C++ {:?}, Rust {:?}",
                cpp_state.animation_core_type, rust_animation_core_type
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_state.property_values,
            rust_state_object,
            &format!("{label} state {state_index}"),
        )?;

        compare_state_machine_fire_actions(
            cpp_state.fire_action_count,
            &cpp_state.fire_actions,
            &rust_state.fire_actions,
            &format!("{label} state {state_index} fire actions"),
        )?;
        compare_listener_actions(
            rust_file,
            cpp_state.listener_action_count,
            &cpp_state.listener_actions,
            &rust_state.listener_actions,
            &format!("{label} state {state_index} listener actions"),
        )?;

        if cpp_state.blend_animations.len() != rust_state.blend_animations.len() {
            return Err(format!(
                "{label} state {state_index} blend animation count mismatch, C++ {}, Rust {}",
                cpp_state.blend_animations.len(),
                rust_state.blend_animations.len()
            ));
        }
        for (blend_animation_index, (cpp_blend_animation, rust_blend_animation)) in cpp_state
            .blend_animations
            .iter()
            .zip(&rust_state.blend_animations)
            .enumerate()
        {
            compare_blend_animation(
                cpp_blend_animation.as_ref(),
                rust_blend_animation,
                blend_animation_index,
                &format!("{label} state {state_index} blend animation {blend_animation_index}"),
            )?;
        }

        if cpp_state.transition_count != rust_state.transitions.len()
            || cpp_state.transitions.len() != rust_state.transitions.len()
        {
            return Err(format!(
                "{label} state {state_index} transition count mismatch, C++ count/list {}/{}, Rust {}",
                cpp_state.transition_count,
                cpp_state.transitions.len(),
                rust_state.transitions.len()
            ));
        }

        for (transition_index, (cpp_transition, rust_transition)) in cpp_state
            .transitions
            .iter()
            .zip(&rust_state.transitions)
            .enumerate()
        {
            compare_state_transition(
                cpp_transition.as_ref(),
                rust_transition,
                rust_file,
                transition_index,
                &format!("{label} state {state_index} transition {transition_index}"),
            )?;
        }
    }

    Ok(())
}

fn compare_state_machine_fire_actions(
    cpp_fire_action_count: usize,
    cpp_fire_actions: &[Option<CppProbeStateMachineFireAction>],
    rust_fire_actions: &[nuxie_binary::RuntimeStateMachineFireAction<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_fire_action_count != rust_fire_actions.len()
        || cpp_fire_actions.len() != rust_fire_actions.len()
    {
        return Err(format!(
            "{label} count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_fire_action_count,
            cpp_fire_actions.len(),
            rust_fire_actions.len()
        ));
    }

    for (fire_action_index, (cpp_fire_action, rust_fire_action)) in
        cpp_fire_actions.iter().zip(rust_fire_actions).enumerate()
    {
        let Some(cpp_fire_action) = cpp_fire_action.as_ref() else {
            return Err(format!(
                "{label} fire action {fire_action_index} is null in C++"
            ));
        };
        if cpp_fire_action.index != fire_action_index {
            return Err(format!(
                "{label} fire action {fire_action_index} C++ index mismatch, got {}",
                cpp_fire_action.index
            ));
        }
        if cpp_fire_action.core_type != rust_fire_action.object.type_key {
            return Err(format!(
                "{label} fire action {fire_action_index} core type mismatch, C++ {}, Rust {}",
                cpp_fire_action.core_type, rust_fire_action.object.type_key
            ));
        }
        if rust_fire_action.object.uint_property("eventId") != cpp_fire_action.event_id {
            return Err(format!(
                "{label} fire action {fire_action_index} eventId mismatch, C++ {:?}, Rust {:?}",
                cpp_fire_action.event_id,
                rust_fire_action.object.uint_property("eventId")
            ));
        }
        if cpp_fire_action.event_local != rust_fire_action.event_local_index {
            return Err(format!(
                "{label} fire action {fire_action_index} event local mismatch, C++ {:?}, Rust {:?}",
                cpp_fire_action.event_local, rust_fire_action.event_local_index
            ));
        }
        let rust_event_core_type = rust_fire_action.event.map(|event| event.type_key);
        if cpp_fire_action.event_core_type != rust_event_core_type {
            return Err(format!(
                "{label} fire action {fire_action_index} event core type mismatch, C++ {:?}, Rust {:?}",
                cpp_fire_action.event_core_type, rust_event_core_type
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_fire_action.property_values,
            rust_fire_action.object,
            &format!("{label} fire action {fire_action_index}"),
        )?;
    }

    Ok(())
}

fn compare_listener_actions(
    rust_file: &RuntimeFile,
    cpp_action_count: usize,
    cpp_actions: &[Option<CppProbeListenerAction>],
    rust_actions: &[nuxie_binary::RuntimeListenerAction<'_>],
    label: &str,
) -> Result<(), String> {
    if cpp_action_count != rust_actions.len() || cpp_actions.len() != rust_actions.len() {
        return Err(format!(
            "{label} count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_action_count,
            cpp_actions.len(),
            rust_actions.len()
        ));
    }

    for (action_index, (cpp_action, rust_action)) in
        cpp_actions.iter().zip(rust_actions).enumerate()
    {
        let Some(cpp_action) = cpp_action.as_ref() else {
            return Err(format!("{label} action {action_index} is null in C++"));
        };
        if cpp_action.index != action_index {
            return Err(format!(
                "{label} action {action_index} C++ index mismatch, got {}",
                cpp_action.index
            ));
        }
        if cpp_action.core_type != rust_action.object.type_key {
            return Err(format!(
                "{label} action {action_index} core type mismatch, C++ {}, Rust {}",
                cpp_action.core_type, rust_action.object.type_key
            ));
        }
        if rust_action.object.uint_property("flags") != Some(cpp_action.flags) {
            return Err(format!(
                "{label} action {action_index} flags mismatch, C++ {}, Rust {:?}",
                cpp_action.flags,
                rust_action.object.uint_property("flags")
            ));
        }
        if rust_action.object.uint_property("eventId") != cpp_action.event_id {
            return Err(format!(
                "{label} action {action_index} eventId mismatch, C++ {:?}, Rust {:?}",
                cpp_action.event_id,
                rust_action.object.uint_property("eventId")
            ));
        }
        if cpp_action.event_local != rust_action.event_local_index {
            return Err(format!(
                "{label} action {action_index} event local mismatch, C++ {:?}, Rust {:?}",
                cpp_action.event_local, rust_action.event_local_index
            ));
        }
        let rust_event_core_type = rust_action.event.map(|event| event.type_key);
        if cpp_action.event_core_type != rust_event_core_type {
            return Err(format!(
                "{label} action {action_index} event core type mismatch, C++ {:?}, Rust {:?}",
                cpp_action.event_core_type, rust_event_core_type
            ));
        }
        let rust_action_path = data_bind_path_ids_for_referencer(rust_file, rust_action.object);
        if cpp_action.data_bind_path_ids != rust_action_path {
            return Err(format!(
                "{label} action {action_index} data-bind path mismatch, C++ {:?}, Rust {:?}",
                cpp_action.data_bind_path_ids, rust_action_path
            ));
        }
        let rust_action_resolved_path =
            resolved_data_bind_path_ids_for_referencer(rust_file, rust_action.object);
        if cpp_action.resolved_data_bind_path_ids != rust_action_resolved_path {
            return Err(format!(
                "{label} action {action_index} resolved data-bind path mismatch, C++ {:?}, Rust {:?}",
                cpp_action.resolved_data_bind_path_ids, rust_action_resolved_path
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_action.property_values,
            rust_action.object,
            &format!("{label} action {action_index}"),
        )?;
    }

    Ok(())
}

fn compare_blend_animation(
    cpp_blend_animation: Option<&CppProbeBlendAnimation>,
    rust_blend_animation: &nuxie_binary::RuntimeBlendAnimation<'_>,
    blend_animation_index: usize,
    label: &str,
) -> Result<(), String> {
    let Some(cpp_blend_animation) = cpp_blend_animation else {
        return Err(format!("{label} is null in C++"));
    };
    if cpp_blend_animation.index != blend_animation_index {
        return Err(format!(
            "{label} C++ index mismatch, got {}",
            cpp_blend_animation.index
        ));
    }
    if cpp_blend_animation.core_type != rust_blend_animation.object.type_key {
        return Err(format!(
            "{label} core type mismatch, C++ {}, Rust {}",
            cpp_blend_animation.core_type, rust_blend_animation.object.type_key
        ));
    }
    if rust_blend_animation.object.uint_property("animationId")
        != Some(cpp_blend_animation.animation_id)
    {
        return Err(format!(
            "{label} animationId mismatch, C++ {}, Rust {:?}",
            cpp_blend_animation.animation_id,
            rust_blend_animation.object.uint_property("animationId")
        ));
    }
    if cpp_blend_animation.animation_index != rust_blend_animation.animation_index {
        return Err(format!(
            "{label} animation index mismatch, C++ {:?}, Rust {:?}",
            cpp_blend_animation.animation_index, rust_blend_animation.animation_index
        ));
    }
    let rust_animation_core_type = rust_blend_animation
        .animation
        .map(|animation| animation.type_key);
    if cpp_blend_animation.animation_core_type != rust_animation_core_type {
        return Err(format!(
            "{label} animation core type mismatch, C++ {:?}, Rust {:?}",
            cpp_blend_animation.animation_core_type, rust_animation_core_type
        ));
    }
    compare_cpp_registry_properties_result(
        &cpp_blend_animation.property_values,
        rust_blend_animation.object,
        label,
    )
}

fn compare_state_transition(
    cpp_transition: Option<&CppProbeStateTransition>,
    rust_transition: &nuxie_binary::RuntimeStateTransition<'_>,
    rust_file: &RuntimeFile,
    transition_index: usize,
    label: &str,
) -> Result<(), String> {
    let Some(cpp_transition) = cpp_transition else {
        return Err(format!("{label} is null in C++"));
    };
    if cpp_transition.index != transition_index {
        return Err(format!(
            "{label} C++ index mismatch, got {}",
            cpp_transition.index
        ));
    }
    if cpp_transition.core_type != rust_transition.object.type_key {
        return Err(format!(
            "{label} core type mismatch, C++ {}, Rust {}",
            cpp_transition.core_type, rust_transition.object.type_key
        ));
    }
    if rust_transition.object.uint_property("stateToId") != Some(cpp_transition.state_to_id) {
        return Err(format!(
            "{label} stateToId mismatch, C++ {}, Rust {:?}",
            cpp_transition.state_to_id,
            rust_transition.object.uint_property("stateToId")
        ));
    }
    if cpp_transition.state_to_index != rust_transition.state_to_index {
        return Err(format!(
            "{label} stateToIndex mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.state_to_index, rust_transition.state_to_index
        ));
    }

    let rust_state_to_core_type = rust_transition.state_to.map(|state| state.type_key);
    if cpp_transition.state_to_core_type != rust_state_to_core_type {
        let layer_state_type_key = definition_by_name("LayerState")
            .expect("LayerState exists in schema")
            .type_key
            .int;
        if !(cpp_transition.state_to_core_type == Some(layer_state_type_key)
            && rust_transition.state_to.is_none())
        {
            return Err(format!(
                "{label} stateTo core type mismatch, C++ {:?}, Rust {:?}",
                cpp_transition.state_to_core_type, rust_state_to_core_type
            ));
        }
    }

    if rust_transition.object.uint_property("interpolatorId")
        != Some(cpp_transition.interpolator_id)
    {
        return Err(format!(
            "{label} interpolatorId mismatch, C++ {}, Rust {:?}",
            cpp_transition.interpolator_id,
            rust_transition.object.uint_property("interpolatorId")
        ));
    }
    let rust_interpolator_core_type = rust_transition.interpolator.map(|object| object.type_key);
    if cpp_transition.interpolator_core_type != rust_interpolator_core_type {
        return Err(format!(
            "{label} interpolator core type mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.interpolator_core_type, rust_interpolator_core_type
        ));
    }
    if rust_transition.object.uint_property("exitBlendAnimationId")
        != cpp_transition.exit_blend_animation_id
    {
        return Err(format!(
            "{label} exitBlendAnimationId mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.exit_blend_animation_id,
            rust_transition.object.uint_property("exitBlendAnimationId")
        ));
    }
    if cpp_transition.exit_blend_animation_index != rust_transition.exit_blend_animation_index {
        return Err(format!(
            "{label} exit blend animation index mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.exit_blend_animation_index, rust_transition.exit_blend_animation_index
        ));
    }
    let rust_exit_blend_animation_core_type = rust_transition
        .exit_blend_animation
        .map(|animation| animation.type_key);
    if cpp_transition.exit_blend_animation_core_type != rust_exit_blend_animation_core_type {
        return Err(format!(
            "{label} exit blend animation core type mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.exit_blend_animation_core_type, rust_exit_blend_animation_core_type
        ));
    }
    if cpp_transition.exit_animation_index != rust_transition.exit_animation_index {
        return Err(format!(
            "{label} exit animation index mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.exit_animation_index, rust_transition.exit_animation_index
        ));
    }
    let rust_exit_animation_core_type = rust_transition
        .exit_animation
        .map(|animation| animation.type_key);
    if cpp_transition.exit_animation_core_type != rust_exit_animation_core_type {
        return Err(format!(
            "{label} exit animation core type mismatch, C++ {:?}, Rust {:?}",
            cpp_transition.exit_animation_core_type, rust_exit_animation_core_type
        ));
    }
    compare_cpp_registry_properties_result(
        &cpp_transition.property_values,
        rust_transition.object,
        label,
    )?;
    compare_state_machine_fire_actions(
        cpp_transition.fire_action_count,
        &cpp_transition.fire_actions,
        &rust_transition.fire_actions,
        &format!("{label} fire actions"),
    )?;
    compare_listener_actions(
        rust_file,
        cpp_transition.listener_action_count,
        &cpp_transition.listener_actions,
        &rust_transition.listener_actions,
        &format!("{label} listener actions"),
    )?;

    if cpp_transition.condition_count != rust_transition.conditions.len()
        || cpp_transition.conditions.len() != rust_transition.conditions.len()
    {
        return Err(format!(
            "{label} condition count mismatch, C++ count/list {}/{}, Rust {}",
            cpp_transition.condition_count,
            cpp_transition.conditions.len(),
            rust_transition.conditions.len()
        ));
    }

    for (condition_index, (cpp_condition, rust_condition)) in cpp_transition
        .conditions
        .iter()
        .zip(&rust_transition.conditions)
        .enumerate()
    {
        let Some(cpp_condition) = cpp_condition.as_ref() else {
            return Err(format!(
                "{label} condition {condition_index} is null in C++"
            ));
        };
        if cpp_condition.index != condition_index {
            return Err(format!(
                "{label} condition {condition_index} C++ index mismatch, got {}",
                cpp_condition.index
            ));
        }
        if cpp_condition.core_type != rust_condition.type_key {
            return Err(format!(
                "{label} condition {condition_index} core type mismatch, C++ {}, Rust {}",
                cpp_condition.core_type, rust_condition.type_key
            ));
        }
        compare_cpp_registry_properties_result(
            &cpp_condition.property_values,
            rust_condition,
            &format!("{label} condition {condition_index}"),
        )?;
    }

    Ok(())
}

fn cpp_artboard_state_machines_are_auto_generated(state_machines: &[CppProbeStateMachine]) -> bool {
    let Some(state_machine) = state_machines.first() else {
        return false;
    };
    state_machines.len() == 1
        && state_machine.index == 0
        && state_machine.core_type
            == definition_by_name("StateMachine")
                .expect("StateMachine schema definition")
                .type_key
                .int
        && state_machine.name == "Auto Generated State Machine"
        && state_machine.layer_count == 0
        && state_machine.input_count == 0
        && state_machine.listener_count == 0
        && state_machine.data_bind_count == 0
        && state_machine.layers.is_empty()
        && state_machine.inputs.is_empty()
        && state_machine.listeners.is_empty()
        && state_machine.data_binds.is_empty()
}

fn compare_corpus_file_level_stored_property_values(
    cpp: &CppProbeFile,
    rust: &RuntimeFile,
    label: &str,
) -> Result<(), String> {
    let mut failures = Vec::new();

    let rust_assets = runtime_file_assets(rust);
    for (index, (cpp_asset, rust_asset)) in cpp.assets.iter().zip(&rust_assets).enumerate() {
        push_comparison_result(
            &mut failures,
            compare_cpp_registry_properties_result(
                &cpp_asset.property_values,
                rust_asset,
                &format!("{label}: file asset {index}"),
            ),
        );
    }

    let rust_view_models = runtime_file_view_models(rust);
    for (view_model_index, (cpp_view_model, rust_view_model)) in
        cpp.view_models.iter().zip(&rust_view_models).enumerate()
    {
        push_comparison_result(
            &mut failures,
            compare_cpp_registry_properties_result(
                &cpp_view_model.property_values,
                rust_view_model.object,
                &format!("{label}: view model {view_model_index}"),
            ),
        );

        for (property_index, (cpp_property, rust_property)) in cpp_view_model
            .properties
            .iter()
            .zip(&rust_view_model.properties)
            .enumerate()
        {
            push_comparison_result(
                &mut failures,
                compare_cpp_registry_properties_result(
                    &cpp_property.property_values,
                    rust_property,
                    &format!("{label}: view model {view_model_index} property {property_index}"),
                ),
            );
        }

        for (instance_index, (cpp_instance, rust_instance)) in cpp_view_model
            .instances
            .iter()
            .zip(&rust_view_model.instances)
            .enumerate()
        {
            push_comparison_result(
                &mut failures,
                compare_cpp_registry_properties_result(
                    &cpp_instance.property_values,
                    rust_instance.object,
                    &format!("{label}: view model {view_model_index} instance {instance_index}"),
                ),
            );
            for (value_index, (cpp_value, rust_value)) in cpp_instance
                .values
                .iter()
                .zip(&rust_instance.values)
                .enumerate()
            {
                push_comparison_result(
                    &mut failures,
                    compare_cpp_registry_properties_result(
                        &cpp_value.property_values,
                        rust_value.object,
                        &format!(
                            "{label}: view model {view_model_index} instance {instance_index} value {value_index}"
                        ),
                    ),
                );
                for (item_index, (cpp_item, rust_item)) in cpp_value
                    .items
                    .iter()
                    .zip(&rust_value.list_items)
                    .enumerate()
                {
                    push_comparison_result(
                        &mut failures,
                        compare_cpp_registry_properties_result(
                            &cpp_item.property_values,
                            rust_item,
                            &format!(
                                "{label}: view model {view_model_index} instance {instance_index} value {value_index} list item {item_index}"
                            ),
                        ),
                    );
                }
            }
        }
    }

    let rust_enums = runtime_data_enums(rust);
    for (enum_index, (cpp_enum, rust_enum)) in cpp.enums.iter().zip(&rust_enums).enumerate() {
        push_comparison_result(
            &mut failures,
            compare_cpp_registry_properties_result(
                &cpp_enum.property_values,
                rust_enum.object,
                &format!("{label}: data enum {enum_index}"),
            ),
        );

        for (value_index, (cpp_value, rust_value)) in
            cpp_enum.values.iter().zip(&rust_enum.values).enumerate()
        {
            push_comparison_result(
                &mut failures,
                compare_cpp_registry_properties_result(
                    &cpp_value.property_values,
                    rust_value,
                    &format!("{label}: data enum {enum_index} value {value_index}"),
                ),
            );
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn compare_corpus_artboard_stored_property_values(
    cpp: &CppProbeFile,
    rust: &RuntimeFile,
    label: &str,
) -> Result<(), String> {
    let mut failures = Vec::new();
    let ranges = runtime_artboard_ranges(rust);

    if cpp.artboards.len() != ranges.len() {
        return Err(format!(
            "{label}: artboard property-value count mismatch, C++ {}, Rust {}",
            cpp.artboards.len(),
            ranges.len()
        ));
    }

    for (artboard_index, (cpp_artboard, range)) in cpp.artboards.iter().zip(&ranges).enumerate() {
        let rust_objects = runtime_artboard_local_objects(rust, *range);
        if cpp_artboard.objects.len() != rust_objects.len() {
            failures.push(format!(
                "{label}: artboard {artboard_index} property-value object count mismatch, C++ {}, Rust {}",
                cpp_artboard.objects.len(),
                rust_objects.len()
            ));
            continue;
        }

        for (local_id, (cpp_object, rust_object)) in
            cpp_artboard.objects.iter().zip(&rust_objects).enumerate()
        {
            let (Some(cpp_object), Some(rust_object)) = (cpp_object, rust_object) else {
                continue;
            };
            if rust_object.type_name == "Artboard" {
                continue;
            }

            push_comparison_result(
                &mut failures,
                compare_cpp_registry_properties_result(
                    &cpp_object.property_values,
                    rust_object,
                    &format!("{label}: artboard {artboard_index} object {local_id}"),
                ),
            );
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn push_comparison_result(failures: &mut Vec<String>, result: Result<(), String>) {
    if let Err(err) = result {
        failures.push(err);
    }
}

fn collect_riv_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read directory {}: {err}", dir.display()));

    for entry in entries {
        let entry =
            entry.unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_riv_paths(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "riv") {
            paths.push(path);
        }
    }
}

fn collect_cpp_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read directory {}: {err}", dir.display()));

    for entry in entries {
        let entry =
            entry.unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_cpp_paths(&path, paths);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("cpp") {
            paths.push(path);
        }
    }
}

fn collect_hpp_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("failed to read directory {}: {err}", dir.display()));

    for entry in entries {
        let entry =
            entry.unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_hpp_paths(&path, paths);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("hpp") {
            paths.push(path);
        }
    }
}

fn cpp_generated_base_class_name(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix("class ") else {
            continue;
        };
        let class_name = rest
            .split(|character: char| character == ' ' || character == ':' || character == ';')
            .next()?;
        let Some(name) = class_name.strip_suffix("Base") else {
            continue;
        };
        return Some(name.to_owned());
    }

    None
}

fn cpp_header_data_converter_class_name(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix("class ") else {
            continue;
        };
        if rest.trim_end().ends_with(';') {
            continue;
        }
        let class_name = rest
            .split(|character: char| character == ' ' || character == ':' || character == ';')
            .next()?;
        if class_name.starts_with("DataConverter") || class_name == "ScriptedDataConverter" {
            return Some(class_name.to_owned());
        }
    }

    None
}

fn cpp_inline_output_type_return(compact_source: &str) -> Option<String> {
    let (_, after_signature) = compact_source.split_once("DataTypeoutputType()override")?;
    let (_, after_return) = after_signature.split_once("returnDataType::")?;
    let output_type = after_return
        .chars()
        .take_while(|character| *character == '_' || character.is_ascii_alphanumeric())
        .collect::<String>();
    (!output_type.is_empty()).then_some(output_type)
}

#[derive(Debug)]
struct CompactCppConstructor {
    initializers: String,
    body: String,
}

fn compact_cpp_constructor(source: &str, signature: &str) -> Option<CompactCppConstructor> {
    let signature_start = source.find(signature)?;
    let after_signature = signature_start + signature.len();
    let body_start = source[after_signature..].find('{')? + after_signature;
    let body_end = matching_compact_cpp_brace(source, body_start)?;

    Some(CompactCppConstructor {
        initializers: source[after_signature..body_start].to_owned(),
        body: source[body_start + 1..body_end].to_owned(),
    })
}

fn cpp_stored_members_in_hierarchy(definition: &Definition) -> BTreeMap<String, &'static str> {
    let mut members = BTreeMap::new();
    for owner in std::iter::once(definition.name).chain(definition.ancestors.iter().copied()) {
        let owner = definition_by_name(owner)
            .unwrap_or_else(|| panic!("missing ancestor definition {owner}"));
        for property in owner.properties {
            if property.stores_field {
                members.insert(cpp_member_name(property.name), property.name);
            }
        }
    }
    members
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

fn compact_cpp_member_assignment(source: &str, member: &str) -> Option<String> {
    let pattern = format!("m_{member}");
    let mut search_start = 0;
    while let Some(relative_start) = source[search_start..].find(&pattern) {
        let member_start = search_start + relative_start;
        let after_member = member_start + pattern.len();
        let Some(next) = source[after_member..].chars().next() else {
            return None;
        };
        let (after_assignment, terminator) = match next {
            '=' => (after_member + 1, ';'),
            '(' => (after_member + 1, ')'),
            '{' => (after_member + 1, '}'),
            _ => {
                search_start = after_member;
                continue;
            }
        };
        let value_end = source[after_assignment..].find(terminator)? + after_assignment;
        return Some(source[after_assignment..value_end].to_owned());
    }

    None
}

fn is_cpp_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn parse_cpp_core_field_type_id(header: &Path, class_name: &str) -> i32 {
    let contents = std::fs::read_to_string(header)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", header.display()));
    let mut in_class = false;

    for line in contents.lines() {
        let line = line.trim();
        if line == format!("class {class_name}") {
            in_class = true;
            continue;
        }
        if !in_class {
            continue;
        }
        if line == "};" {
            break;
        }

        let Some(value) = line.strip_prefix("static const int id = ") else {
            continue;
        };
        let value = value.strip_suffix(';').unwrap_or_else(|| {
            panic!("bad C++ field type id line in {}: {line}", header.display())
        });
        return value
            .parse::<i32>()
            .unwrap_or_else(|err| panic!("bad C++ field type id in {}: {err}", header.display()));
    }

    panic!(
        "missing C++ field type id for {class_name} in {}",
        header.display()
    );
}

fn compact_cpp_source(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn compact_cpp_function_body(source: &str, signature: &str) -> Option<String> {
    let signature_start = source.find(signature)?;
    let body_start = source[signature_start..].find('{')? + signature_start;
    let body_end = matching_compact_cpp_brace(source, body_start)?;
    Some(source[body_start + 1..body_end].to_owned())
}

fn compact_cpp_class_body(source: &str, class_name: &str) -> Option<String> {
    let class_start = source.find(&format!("class{class_name}{{"))?;
    let body_start = source[class_start..].find('{')? + class_start;
    let body_end = matching_compact_cpp_brace(source, body_start)?;
    Some(source[body_start + 1..body_end].to_owned())
}

fn compact_cpp_switch_cases(source: &str, switch_signature: &str) -> Option<BTreeSet<String>> {
    compact_cpp_switch_case_labels(source, switch_signature, "::id:")
}

fn compact_cpp_type_key_switch_cases(
    source: &str,
    switch_signature: &str,
) -> Option<BTreeSet<String>> {
    compact_cpp_switch_case_labels(source, switch_signature, "::typeKey:")
}

fn compact_cpp_switch_case_labels(
    source: &str,
    switch_signature: &str,
    suffix: &str,
) -> Option<BTreeSet<String>> {
    let switch_start = source.find(switch_signature)?;
    let body_start = source[switch_start..].find('{')? + switch_start;
    let body_end = matching_compact_cpp_brace(source, body_start)?;
    let body = &source[body_start + 1..body_end];
    let mut cases = BTreeSet::new();
    let label_suffix = suffix.strip_suffix(':').unwrap_or(suffix);

    for part in body.split("case").skip(1) {
        let Some((class_name, _)) = part.split_once(suffix) else {
            continue;
        };
        cases.insert(format!("{class_name}{label_suffix}"));
    }

    Some(cases)
}

fn assert_compact_contains_in_order(source: &str, snippets: &[&str], context: &str) {
    let mut search_start = 0usize;
    for snippet in snippets {
        let Some(offset) = source[search_start..].find(snippet) else {
            panic!("{context}: missing ordered snippet {snippet:?}");
        };
        search_start += offset + snippet.len();
    }
}

fn matching_compact_cpp_brace(source: &str, open_brace: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes()[open_brace..].iter().copied().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open_brace + offset);
                }
            }
            _ => {}
        }
    }

    None
}
