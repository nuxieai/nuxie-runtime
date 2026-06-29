use rive_binary::{RuntimeFile, read_runtime_file};
use rive_graph::GraphFile;
use rive_runtime::{ArtboardInstance, ComponentDirt, Mat2D, RuntimeComponent, TransformProperty};
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

fn push_bool_property(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: bool) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.push(u8::from(value));
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

fn push_keyframe_double(bytes: &mut Vec<u8>, frame: u64, value: f32, interpolation_type: u64) {
    push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
        push_uint_property(bytes, "KeyFrameDouble", "frame", frame);
        push_uint_property(
            bytes,
            "KeyFrameDouble",
            "interpolationType",
            interpolation_type,
        );
        push_f32_property(bytes, "KeyFrameDouble", "value", value);
    });
}

#[derive(Debug, Clone, Copy)]
struct LinearAnimationFixtureOptions {
    duration: u64,
    loop_value: u64,
    speed: f32,
    enable_work_area: bool,
    work_start: u64,
    work_end: u64,
    quantize: bool,
}

impl Default for LinearAnimationFixtureOptions {
    fn default() -> Self {
        Self {
            duration: 20,
            loop_value: 0,
            speed: 1.0,
            enable_work_area: false,
            work_start: 0,
            work_end: 0,
            quantize: false,
        }
    }
}

fn synthetic_linear_animation(
    file_id: u64,
    first_frame: u64,
    first_value: f32,
    second_frame: u64,
    second_value: f32,
    first_interpolation_type: u64,
    quantize: bool,
) -> Vec<u8> {
    synthetic_linear_animation_with_options(
        file_id,
        first_frame,
        first_value,
        second_frame,
        second_value,
        first_interpolation_type,
        LinearAnimationFixtureOptions {
            quantize,
            ..Default::default()
        },
    )
}

fn synthetic_linear_animation_with_options(
    file_id: u64,
    first_frame: u64,
    first_value: f32,
    second_frame: u64,
    second_value: f32,
    first_interpolation_type: u64,
    options: LinearAnimationFixtureOptions,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", options.duration);
            push_f32_property(bytes, "LinearAnimation", "speed", options.speed);
            push_uint_property(bytes, "LinearAnimation", "loopValue", options.loop_value);
            push_uint_property(bytes, "LinearAnimation", "workStart", options.work_start);
            push_uint_property(bytes, "LinearAnimation", "workEnd", options.work_end);
            if options.enable_work_area {
                push_bool_property(bytes, "LinearAnimation", "enableWorkArea", true);
            }
            if options.quantize {
                push_bool_property(bytes, "LinearAnimation", "quantize", true);
            }
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double(bytes, first_frame, first_value, first_interpolation_type);
        push_keyframe_double(bytes, second_frame, second_value, 0);
    })
}

fn read_cpp_probe_bytes(probe: &Path, label: &str, bytes: &[u8]) -> CppProbeFile {
    read_cpp_probe_bytes_with_args(probe, label, bytes, &[])
}

fn read_cpp_probe_bytes_with_args(
    probe: &Path,
    label: &str,
    bytes: &[u8],
    extra_args: &[String],
) -> CppProbeFile {
    let path = std::env::temp_dir().join(format!(
        "rive-rust-runtime-{}-{}.riv",
        std::process::id(),
        label.replace('/', "-")
    ));
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));

    let output = Command::new(probe)
        .arg("--no-advance")
        .arg("--instance-artboards")
        .arg("--runtime-update")
        .args(extra_args)
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

    assert_eq!(cpp_artboard.object_count, rust.slots().len());
    assert_eq!(cpp_artboard.objects.len(), rust.slots().len());
    for (local_id, (cpp_object, rust_slot)) in
        cpp_artboard.objects.iter().zip(rust.slots()).enumerate()
    {
        assert_eq!(rust_slot.local_id, local_id);
        match (cpp_object, rust_slot.type_name) {
            (Some(cpp_object), Some(type_name)) => {
                assert_eq!(cpp_object.local_id, local_id);
                assert_eq!(
                    cpp_object.core_type,
                    type_key_for_name(type_name),
                    "slot core type mismatch for local {local_id} in {label}"
                );
                assert_eq!(cpp_object.is_component, rust_slot.component_index.is_some());
            }
            (None, None) => {}
            _ => panic!("slot presence mismatch for local {local_id} in {label}"),
        }
    }

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

#[test]
fn mutated_instance_transform_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let label = "synthetic/runtime_mutated_transform_hierarchy.riv";
    let bytes = synthetic_transform_hierarchy();
    let x_key = property_key_for_name("Node", "x");
    let cpp = read_cpp_probe_bytes_with_args(
        &probe,
        label,
        &bytes,
        &[
            "--runtime-set-double".to_owned(),
            "1".to_owned(),
            x_key.to_string(),
            "12.0".to_owned(),
        ],
    );
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    assert!(rust.set_transform_property(1, TransformProperty::X, 12.0));
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

#[test]
fn mutable_instance_transform_does_not_mutate_source_object() {
    let label = "synthetic/runtime_transform_source_separation.riv";
    let bytes = synthetic_transform_hierarchy();
    let (runtime, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let child_slot = rust.slot(1).expect("child slot should exist");
    let source_child = runtime
        .object(child_slot.source_global_id as usize)
        .expect("source child should exist");

    assert_eq!(source_child.double_property("x"), Some(2.0));

    assert!(rust.set_transform_property(1, TransformProperty::X, 12.0));
    assert_eq!(rust.component(1).unwrap().transform.x, 12.0);
    assert_eq!(
        source_child.double_property("x"),
        Some(2.0),
        "mutating instance transform state must not mutate imported source data"
    );

    rust.update_components();

    assert_eq!(
        rust.component(1).unwrap().transform.local_transform.0[4],
        12.0
    );
    assert_eq!(source_child.double_property("x"), Some(2.0));
}

#[test]
fn linear_animation_apply_interpolates_keyframe_double_transform() {
    let label = "synthetic/runtime_linear_animation_interpolation.riv";
    let bytes = synthetic_linear_animation(8201, 0, 2.0, 10, 12.0, 1, false);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert_eq!(rust.linear_animations().len(), 1);
    assert_eq!(
        rust.linear_animation(0).unwrap().keyed_objects[0].keyed_properties[0]
            .key_frames
            .len(),
        2
    );

    assert!(rust.apply_linear_animation(0, 0.5, 1.0));
    assert_close(
        rust.component(1).unwrap().transform.x,
        7.0,
        "interpolated x",
    );
    assert!(
        rust.component(1)
            .unwrap()
            .dirt
            .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
    );
}

#[test]
fn linear_animation_apply_holds_when_interpolation_type_is_zero() {
    let label = "synthetic/runtime_linear_animation_hold.riv";
    let bytes = synthetic_linear_animation(8202, 0, 4.0, 10, 12.0, 0, false);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert!(rust.apply_linear_animation(0, 0.5, 1.0));
    assert_close(rust.component(1).unwrap().transform.x, 4.0, "held x");
}

#[test]
fn linear_animation_apply_uses_first_and_last_keyframes_outside_range() {
    let before_label = "synthetic/runtime_linear_animation_before_first.riv";
    let before_bytes = synthetic_linear_animation(8203, 10, 4.0, 20, 12.0, 1, false);
    let (_, mut before) = read_rust_instance_from_bytes(&before_bytes, before_label);

    assert!(before.apply_linear_animation(0, 0.0, 1.0));
    assert_close(
        before.component(1).unwrap().transform.x,
        4.0,
        "before first x",
    );

    let after_label = "synthetic/runtime_linear_animation_after_last.riv";
    let after_bytes = synthetic_linear_animation(8204, 0, 4.0, 10, 12.0, 1, false);
    let (_, mut after) = read_rust_instance_from_bytes(&after_bytes, after_label);

    assert!(after.apply_linear_animation(0, 2.0, 1.0));
    assert_close(
        after.component(1).unwrap().transform.x,
        12.0,
        "after last x",
    );
}

#[test]
fn linear_animation_apply_quantizes_seconds_before_sampling() {
    let label = "synthetic/runtime_linear_animation_quantized.riv";
    let bytes = synthetic_linear_animation(8205, 0, 2.0, 10, 12.0, 1, true);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert!(rust.apply_linear_animation(0, 0.99, 1.0));
    assert_close(rust.component(1).unwrap().transform.x, 11.0, "quantized x");
}

#[test]
fn linear_animation_apply_mixes_with_current_transform_value() {
    let label = "synthetic/runtime_linear_animation_mixed.riv";
    let bytes = synthetic_linear_animation(8206, 0, 2.0, 10, 12.0, 1, false);
    let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);

    assert!(rust.apply_linear_animation(0, 1.0, 0.25));
    assert_close(rust.component(1).unwrap().transform.x, 4.5, "mixed x");
}

#[test]
fn linear_animation_apply_does_not_mutate_source_object() {
    let label = "synthetic/runtime_linear_animation_source_separation.riv";
    let bytes = synthetic_linear_animation(8207, 0, 2.0, 10, 12.0, 1, false);
    let (runtime, mut rust) = read_rust_instance_from_bytes(&bytes, label);
    let child_slot = rust.slot(1).expect("child slot should exist");
    let source_child = runtime
        .object(child_slot.source_global_id as usize)
        .expect("source child should exist");

    assert_eq!(source_child.double_property("x"), Some(2.0));

    assert!(rust.apply_linear_animation(0, 1.0, 1.0));
    assert_close(rust.component(1).unwrap().transform.x, 12.0, "applied x");
    assert_eq!(source_child.double_property("x"), Some(2.0));
}

#[test]
fn linear_animation_apply_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let cases = [
        (
            "synthetic/runtime_linear_animation_cpp_exact.riv",
            synthetic_linear_animation(8211, 0, 2.0, 10, 12.0, 1, false),
            1.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_before_first.riv",
            synthetic_linear_animation(8212, 10, 4.0, 20, 12.0, 1, false),
            0.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_interpolated.riv",
            synthetic_linear_animation(8213, 0, 2.0, 10, 12.0, 1, false),
            0.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_held.riv",
            synthetic_linear_animation(8214, 0, 4.0, 10, 12.0, 0, false),
            0.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_after_last.riv",
            synthetic_linear_animation(8215, 0, 4.0, 10, 12.0, 1, false),
            2.0,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_quantized.riv",
            synthetic_linear_animation(8216, 0, 2.0, 10, 12.0, 1, true),
            0.99,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_cpp_mixed.riv",
            synthetic_linear_animation(8217, 0, 2.0, 10, 12.0, 1, false),
            1.0,
            0.25,
        ),
    ];

    for (label, bytes, seconds, mix) in cases {
        let cpp = read_cpp_probe_bytes_with_args(
            &probe,
            label,
            &bytes,
            &[
                "--runtime-apply-animation".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
                mix.to_string(),
            ],
        );
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        rust.apply_linear_animation(0, seconds, mix);
        let report = rust.update_components();

        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

#[test]
fn linear_animation_instance_advance_matches_cpp_probe() {
    let Some(probe) = probe_path() else {
        eprintln!("skipping C++ runtime comparison; set RIVE_CPP_PROBE to enable");
        return;
    };

    let cases = [
        (
            "synthetic/runtime_linear_animation_instance_one_shot.riv",
            synthetic_linear_animation_with_options(
                8221,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    ..Default::default()
                },
            ),
            1.5,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_loop.riv",
            synthetic_linear_animation_with_options(
                8222,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    loop_value: 1,
                    ..Default::default()
                },
            ),
            1.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_ping_pong.riv",
            synthetic_linear_animation_with_options(
                8223,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    loop_value: 2,
                    ..Default::default()
                },
            ),
            1.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_work_area.riv",
            synthetic_linear_animation_with_options(
                8224,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 20,
                    enable_work_area: true,
                    work_start: 2,
                    work_end: 8,
                    ..Default::default()
                },
            ),
            0.3,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_negative_speed.riv",
            synthetic_linear_animation_with_options(
                8225,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    speed: -1.0,
                    ..Default::default()
                },
            ),
            0.25,
            1.0,
        ),
        (
            "synthetic/runtime_linear_animation_instance_mixed.riv",
            synthetic_linear_animation_with_options(
                8226,
                0,
                2.0,
                10,
                12.0,
                1,
                LinearAnimationFixtureOptions {
                    duration: 10,
                    ..Default::default()
                },
            ),
            0.5,
            0.25,
        ),
    ];

    for (label, bytes, seconds, mix) in cases {
        let cpp = read_cpp_probe_bytes_with_args(
            &probe,
            label,
            &bytes,
            &[
                "--runtime-advance-animation".to_owned(),
                "0".to_owned(),
                seconds.to_string(),
                mix.to_string(),
            ],
        );
        let (_, mut rust) = read_rust_instance_from_bytes(&bytes, label);
        let mut animation = rust
            .linear_animation_instance(0)
            .unwrap_or_else(|| panic!("missing Rust animation instance for {label}"));
        let keep_going = rust.advance_linear_animation_instance(&mut animation, seconds);
        rust.apply_linear_animation_instance(&animation, mix);
        let report = rust.update_components();

        let cpp_artboard = cpp
            .artboards
            .first()
            .unwrap_or_else(|| panic!("missing C++ artboard for {label}"));
        let cpp_animation = cpp_artboard
            .runtime_animation_advances
            .first()
            .unwrap_or_else(|| panic!("missing C++ animation advance report for {label}"));
        let runtime_animation = rust
            .linear_animation(0)
            .unwrap_or_else(|| panic!("missing Rust runtime animation for {label}"));
        let keep_going_after = rust.linear_animation_instance_keep_going(&animation);
        compare_animation_advance(
            cpp_animation,
            runtime_animation,
            &animation,
            keep_going,
            keep_going_after,
            label,
        );
        compare_cpp_runtime_update(&cpp, &rust, &report, label);
    }
}

fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "{label} mismatch: expected {expected}, got {actual}"
    );
}

fn compare_animation_advance(
    cpp: &CppRuntimeAnimationAdvance,
    runtime_animation: &rive_runtime::RuntimeLinearAnimation,
    rust: &rive_runtime::LinearAnimationInstance,
    keep_going: bool,
    keep_going_after: bool,
    label: &str,
) {
    assert_eq!(cpp.animation_index, rust.animation_index(), "{label}");
    assert_eq!(cpp.advanced, keep_going, "{label} advance return mismatch");
    assert_eq!(
        cpp.keep_going, keep_going_after,
        "{label} keepGoing mismatch"
    );
    assert_close(cpp.time, rust.time(), &format!("{label} time"));
    assert_close(
        cpp.direction,
        rust.direction(),
        &format!("{label} direction"),
    );
    assert_close(
        cpp.directed_speed,
        rust.directed_speed(runtime_animation),
        &format!("{label} directedSpeed"),
    );
    assert_close(
        cpp.total_time,
        rust.total_time(),
        &format!("{label} totalTime"),
    );
    assert_close(
        cpp.last_total_time,
        rust.last_total_time(),
        &format!("{label} lastTotalTime"),
    );
    assert_close(
        cpp.spilled_time,
        rust.spilled_time(),
        &format!("{label} spilledTime"),
    );
    assert_eq!(cpp.did_loop, rust.did_loop(), "{label} didLoop mismatch");
    assert_eq!(
        cpp.loop_value as u64,
        rust.loop_value().unwrap_or(runtime_animation.loop_value),
        "{label} loopValue override mismatch"
    );
}

fn compare_cpp_runtime_update(
    cpp: &CppProbeFile,
    rust: &ArtboardInstance,
    report: &rive_runtime::UpdateComponentsReport,
    label: &str,
) {
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
    #[serde(rename = "objectCount")]
    object_count: usize,
    objects: Vec<Option<CppObject>>,
    #[serde(rename = "runtimeUpdate")]
    runtime_update: Option<CppRuntimeUpdate>,
    #[serde(default, rename = "runtimeAnimationAdvances")]
    runtime_animation_advances: Vec<CppRuntimeAnimationAdvance>,
}

#[derive(Debug, Deserialize)]
struct CppObject {
    #[serde(rename = "localId")]
    local_id: usize,
    #[serde(rename = "coreType")]
    core_type: u16,
    #[serde(rename = "isComponent")]
    is_component: bool,
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
struct CppRuntimeAnimationAdvance {
    #[serde(rename = "animationIndex")]
    animation_index: usize,
    advanced: bool,
    #[serde(rename = "keepGoing")]
    keep_going: bool,
    time: f32,
    direction: f32,
    #[serde(rename = "directedSpeed")]
    directed_speed: f32,
    #[serde(rename = "totalTime")]
    total_time: f32,
    #[serde(rename = "lastTotalTime")]
    last_total_time: f32,
    #[serde(rename = "spilledTime")]
    spilled_time: f32,
    #[serde(rename = "didLoop")]
    did_loop: bool,
    #[serde(rename = "loopValue")]
    loop_value: i64,
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
