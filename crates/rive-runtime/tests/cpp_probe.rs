use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::GraphFile;
use rive_runtime::{ArtboardInstance, ComponentDirt, Mat2D, RuntimeComponent};
use rive_schema::definition_by_name;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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

fn type_key_for_name(type_name: &str) -> u16 {
    definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int
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
    push_var_uint(bytes, u64::from(type_key_for_name(type_name)));
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value);
}

fn push_f32_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.extend_from_slice(&value.to_le_bytes());
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

fn push_transform_node(
    bytes: &mut Vec<u8>,
    parent_id: u64,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    opacity: f32,
) {
    push_object_with_properties(bytes, "Node", |bytes| {
        push_uint_property(bytes, "Node", "parentId", parent_id);
        push_f32_property(bytes, "Node", "x", x);
        push_f32_property(bytes, "Node", "y", y);
        push_f32_property(bytes, "Node", "scaleX", scale_x);
        push_f32_property(bytes, "Node", "scaleY", scale_y);
        push_f32_property(bytes, "Node", "opacity", opacity);
    });
}

fn synthetic_transform_hierarchy() -> Vec<u8> {
    synthetic_runtime_file(8101, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 4.0, 5.0, 0.5);
        push_transform_node(bytes, 1, 7.0, 11.0, 2.0, 3.0, 0.25);
    })
}

fn read_cpp_probe_bytes(probe: &Path, label: &str, bytes: &[u8]) -> CppProbeFile {
    let path = std::env::temp_dir().join(format!(
        "rive-rust-runtime-{}-{}.riv",
        std::process::id(),
        label.replace('/', "-")
    ));
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));

    let output = Command::new(probe)
        .arg("--no-advance")
        .arg("--runtime-update")
        .arg("--file")
        .arg(&path)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));

    let _ = std::fs::remove_file(&path);

    assert!(
        output.status.success(),
        "C++ probe failed for {label}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid probe JSON for {label}: {err}"))
}

fn read_rust_instance_from_bytes(bytes: &[u8], label: &str) -> (RuntimeFile, ArtboardInstance) {
    let runtime = read_runtime_file(bytes).unwrap_or_else(|err| {
        panic!("failed to import {label}: {err:#}");
    });
    let graph = GraphFile::from_runtime_file(&runtime).unwrap_or_else(|err| {
        panic!("failed to build Rust graph for {label}: {err:#}");
    });
    let artboard = graph
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing Rust artboard for {label}"));
    let instance = ArtboardInstance::from_graph(&runtime, artboard).unwrap_or_else(|err| {
        panic!("failed to build Rust artboard instance for {label}: {err:#}");
    });

    (runtime, instance)
}

#[test]
fn runtime_update_matches_cpp_for_transform_hierarchy() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_transform_hierarchy.riv";
    let bytes = synthetic_transform_hierarchy();
    let cpp = read_cpp_probe_bytes(&probe, label, &bytes);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let report = rust.update_components();

    let cpp_artboard = cpp
        .artboards
        .first()
        .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
    let cpp_update = cpp_artboard
        .runtime_update
        .as_ref()
        .unwrap_or_else(|| panic!("missing C++ runtimeUpdate for {label}"));

    assert_eq!(cpp_update.did_update, report.did_update);
    assert_eq!(
        cpp_update.has_components_dirt,
        rust.has_dirt(ComponentDirt::COMPONENTS)
    );

    for cpp_component in &cpp_update.components {
        let rust_component = rust
            .component(cpp_component.local_id)
            .unwrap_or_else(|| panic!("missing Rust component {}", cpp_component.local_id));
        compare_component(cpp_component, rust_component, label);
    }
}

fn compare_component(cpp: &CppRuntimeComponent, rust: &RuntimeComponent, label: &str) {
    assert_eq!(
        cpp.graph_order, rust.graph_order,
        "graph order mismatch for {label}"
    );
    assert_eq!(
        cpp.dirt, rust.dirt.0,
        "dirt mismatch for local {} in {label}",
        cpp.local_id
    );
    assert_eq!(
        cpp.collapsed,
        rust.is_collapsed(),
        "collapsed flag mismatch for local {} in {label}",
        cpp.local_id
    );

    let rust_local = rust
        .capabilities
        .transform
        .then_some(rust.transform.local_transform);
    compare_mat2d(
        cpp.local_transform,
        rust_local,
        "local transform",
        cpp.local_id,
        label,
    );

    let rust_world = rust
        .capabilities
        .world_transform
        .then_some(rust.transform.world_transform);
    compare_mat2d(
        cpp.world_transform,
        rust_world,
        "world transform",
        cpp.local_id,
        label,
    );

    let rust_opacity = rust
        .capabilities
        .transform
        .then_some(rust.transform.render_opacity);
    compare_optional_f32(
        cpp.render_opacity,
        rust_opacity,
        "render opacity",
        cpp.local_id,
        label,
    );
}

fn compare_mat2d(
    cpp: Option<[f32; 6]>,
    rust: Option<Mat2D>,
    field: &str,
    local_id: usize,
    label: &str,
) {
    match (cpp, rust) {
        (Some(cpp), Some(rust)) => {
            for (index, (cpp_value, rust_value)) in cpp.into_iter().zip(rust.0).enumerate() {
                assert!(
                    (cpp_value - rust_value).abs() <= 0.0001,
                    "{field}[{index}] mismatch for local {local_id} in {label}: C++ {cpp_value}, Rust {rust_value}"
                );
            }
        }
        (None, None) => {}
        _ => panic!("{field} presence mismatch for local {local_id} in {label}"),
    }
}

fn compare_optional_f32(
    cpp: Option<f32>,
    rust: Option<f32>,
    field: &str,
    local_id: usize,
    label: &str,
) {
    match (cpp, rust) {
        (Some(cpp), Some(rust)) => assert!(
            (cpp - rust).abs() <= 0.0001,
            "{field} mismatch for local {local_id} in {label}: C++ {cpp}, Rust {rust}"
        ),
        (None, None) => {}
        _ => panic!("{field} presence mismatch for local {local_id} in {label}"),
    }
}

#[derive(Debug, Deserialize)]
struct CppProbeFile {
    artboards: Vec<CppArtboard>,
}

#[derive(Debug, Deserialize)]
struct CppArtboard {
    #[serde(rename = "runtimeUpdate")]
    runtime_update: Option<CppRuntimeUpdate>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeUpdate {
    #[serde(rename = "didUpdate")]
    did_update: bool,
    #[serde(rename = "hasComponentsDirt")]
    has_components_dirt: bool,
    components: Vec<CppRuntimeComponent>,
}

#[derive(Debug, Deserialize)]
struct CppRuntimeComponent {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "graphOrder")]
    graph_order: usize,
    dirt: u16,
    collapsed: bool,
    #[serde(rename = "worldTransform")]
    world_transform: Option<[f32; 6]>,
    #[serde(rename = "localTransform")]
    local_transform: Option<[f32; 6]>,
    #[serde(rename = "renderOpacity")]
    render_opacity: Option<f32>,
}
