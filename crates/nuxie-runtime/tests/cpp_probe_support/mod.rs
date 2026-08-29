//! Shared fixture bytes and C++ oracle invocation for the native and pending probe families.
use super::{CppProbeFile, Mat2D};
use nuxie_schema::definition_by_name;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn cpp_probe_temp_path(prefix: &str, label: &str) -> PathBuf {
    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}-{}.riv",
        std::process::id(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed),
        label.replace('/', "-")
    ))
}

pub(super) fn default_probe_path() -> PathBuf {
    let os = match std::env::consts::OS {
        "macos" => "macosx",
        other => other,
    };

    repo_root()
        .join("tools/cpp-probe/build")
        .join(os)
        .join("bin/debug/rive_cpp_probe")
}

pub(super) fn probe_path() -> Option<PathBuf> {
    let path = if let Some(path) = std::env::var_os("RIVE_CPP_PROBE") {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            repo_root().join(path)
        }
    } else {
        let path = default_probe_path();
        if !path.exists() {
            return None;
        }
        path
    };

    verify_probe_fingerprint(&path, "make cpp-probe");
    Some(path)
}

pub(super) fn expected_probe_fingerprint() -> &'static str {
    static FINGERPRINT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FINGERPRINT.get_or_init(|| {
        use sha2::{Digest, Sha256};

        let probe_dir = repo_root().join("tools/cpp-probe");
        let mut manifest = String::from("nuxie-cpp-probe-source/v1\n");
        for input in [
            "main.cpp",
            "testing_random_provider.cpp",
            "build/premake5.lua",
            "build.sh",
        ] {
            let path = probe_dir.join(input);
            let bytes = std::fs::read(&path).unwrap_or_else(|error| {
                panic!("cannot read cpp-probe source {}: {error}", path.display())
            });
            manifest.push_str(&format!("{input}:{:x}\n", Sha256::digest(&bytes)));
        }
        format!("{:x}", Sha256::digest(manifest.as_bytes()))
    })
}

pub(super) fn probe_staleness_error(probe: &Path, rebuild: &str) -> Option<String> {
    let output = match Command::new(probe).arg("--fingerprint").output() {
        Ok(output) => output,
        Err(error) => {
            return Some(format!("cannot run cpp-probe {}: {error}", probe.display()));
        }
    };
    let reported = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || reported.trim() != expected_probe_fingerprint() {
        return Some(format!("cpp-probe binary is stale — run {rebuild}"));
    }
    None
}

pub(super) fn verify_probe_fingerprint(probe: &Path, rebuild: &str) {
    if let Some(message) = probe_staleness_error(probe, rebuild) {
        panic!("{message}");
    }
}

pub(super) fn cpp_runtime_fixture(relative: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(relative)
}

pub(super) fn read_cpp_probe_fixture_with_args(
    probe: &Path,
    relative: &str,
    extra_args: &[String],
) -> CppProbeFile {
    let fixture = cpp_runtime_fixture(relative);
    let output = Command::new(probe)
        .arg("--instance-artboards")
        .arg("--runtime-update")
        .args(extra_args)
        .arg("--file")
        .arg(&fixture)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));
    assert!(
        output.status.success(),
        "C++ probe failed for {}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        fixture.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid probe JSON for {}: {err}", fixture.display()))
}

pub(super) fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
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

pub(super) fn type_key_for_name(type_name: &str) -> u16 {
    definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
        .type_key
        .int
}

pub(super) fn property_key_for_name(type_name: &str, property_name: &str) -> u16 {
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

pub(super) fn push_object_with_properties(
    bytes: &mut Vec<u8>,
    type_name: &str,
    properties: impl FnOnce(&mut Vec<u8>),
) {
    push_var_uint(bytes, u64::from(type_key_for_name(type_name)));
    properties(bytes);
    push_var_uint(bytes, 0);
}

pub(super) fn push_uint_property(
    bytes: &mut Vec<u8>,
    type_name: &str,
    property_name: &str,
    value: u64,
) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    push_var_uint(bytes, value);
}

pub(super) fn push_f32_property(
    bytes: &mut Vec<u8>,
    type_name: &str,
    property_name: &str,
    value: f32,
) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn push_bool_property(
    bytes: &mut Vec<u8>,
    type_name: &str,
    property_name: &str,
    value: bool,
) {
    let key = property_key_for_name(type_name, property_name);
    push_var_uint(bytes, u64::from(key));
    bytes.push(u8::from(value));
}

pub(super) fn synthetic_runtime_file(
    file_id: u64,
    object_stream: impl FnOnce(&mut Vec<u8>),
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, file_id);
    push_var_uint(&mut bytes, 0);
    object_stream(&mut bytes);
    bytes
}

pub(super) fn push_transform_node(
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

pub(super) fn synthetic_transform_hierarchy() -> Vec<u8> {
    synthetic_runtime_file(8101, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 4.0, 5.0, 0.5);
        push_transform_node(bytes, 1, 7.0, 11.0, 2.0, 3.0, 0.25);
    })
}

pub(super) fn push_keyframe_double(
    bytes: &mut Vec<u8>,
    frame: u64,
    value: f32,
    interpolation_type: u64,
) {
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

pub(super) fn push_keyframe_double_with_interpolator(
    bytes: &mut Vec<u8>,
    frame: u64,
    value: f32,
    interpolation_type: u64,
    interpolator_id: u64,
) {
    push_object_with_properties(bytes, "KeyFrameDouble", |bytes| {
        push_uint_property(bytes, "KeyFrameDouble", "frame", frame);
        push_uint_property(
            bytes,
            "KeyFrameDouble",
            "interpolationType",
            interpolation_type,
        );
        push_uint_property(bytes, "KeyFrameDouble", "interpolatorId", interpolator_id);
        push_f32_property(bytes, "KeyFrameDouble", "value", value);
    });
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LinearAnimationFixtureOptions {
    pub(super) duration: u64,
    pub(super) loop_value: u64,
    pub(super) speed: f32,
    pub(super) enable_work_area: bool,
    pub(super) work_start: u64,
    pub(super) work_end: u64,
    pub(super) quantize: bool,
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

pub(super) fn synthetic_linear_animation(
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

pub(super) fn synthetic_linear_animation_with_options(
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

pub(super) fn synthetic_linear_animation_wrong_type_interpolator(file_id: u64) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_transform_node(bytes, 0, 2.0, 3.0, 1.0, 1.0, 1.0);
        push_object_with_properties(bytes, "LinearAnimation", |bytes| {
            push_uint_property(bytes, "LinearAnimation", "fps", 10);
            push_uint_property(bytes, "LinearAnimation", "duration", 20);
        });
        push_object_with_properties(bytes, "KeyedObject", |bytes| {
            push_uint_property(bytes, "KeyedObject", "objectId", 1);
        });
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "y")),
            );
        });
        push_keyframe_double(bytes, 0, 3.0, 1);
        push_keyframe_double(bytes, 10, 13.0, 0);
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "x")),
            );
        });
        push_keyframe_double_with_interpolator(bytes, 0, 12.0, 1, 1);
        push_keyframe_double(bytes, 10, 22.0, 0);
        push_object_with_properties(bytes, "KeyedProperty", |bytes| {
            push_uint_property(
                bytes,
                "KeyedProperty",
                "propertyKey",
                u64::from(property_key_for_name("Node", "y")),
            );
        });
        push_keyframe_double(bytes, 0, 13.0, 1);
        push_keyframe_double(bytes, 10, 23.0, 0);
    })
}

pub(super) fn synthetic_state_machine_missing_system_state(
    file_id: u64,
    missing: &'static str,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        for type_name in ["AnyState", "EntryState", "ExitState"] {
            if type_name != missing {
                push_object_with_properties(bytes, type_name, |_| {});
            }
        }
    })
}

pub(super) fn synthetic_state_machine_bad_transition_target(
    file_id: u64,
    state_to_id: u64,
) -> Vec<u8> {
    synthetic_runtime_file(file_id, |bytes| {
        push_object_with_properties(bytes, "Backboard", |_| {});
        push_object_with_properties(bytes, "Artboard", |_| {});
        push_object_with_properties(bytes, "StateMachine", |_| {});
        push_object_with_properties(bytes, "StateMachineLayer", |_| {});
        push_object_with_properties(bytes, "AnyState", |_| {});
        push_object_with_properties(bytes, "EntryState", |_| {});
        push_object_with_properties(bytes, "StateTransition", |bytes| {
            push_uint_property(bytes, "StateTransition", "stateToId", state_to_id);
        });
        push_object_with_properties(bytes, "LayerState", |_| {});
        push_object_with_properties(bytes, "ExitState", |_| {});
    })
}

pub(super) fn read_cpp_probe_bytes(probe: &Path, label: &str, bytes: &[u8]) -> CppProbeFile {
    read_cpp_probe_bytes_with_args(probe, label, bytes, &[])
}

pub(super) fn read_cpp_probe_bytes_with_args(
    probe: &Path,
    label: &str,
    bytes: &[u8],
    extra_args: &[String],
) -> CppProbeFile {
    let path = cpp_probe_temp_path("rive-rust-runtime", label);
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

    assert!(
        output.status.success(),
        "C++ probe failed for {label}\npath: {}\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        path.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if std::env::var_os("RIVE_KEEP_CPP_PROBE_FIXTURES").is_some() {
        eprintln!("kept C++ probe fixture for {label}: {}", path.display());
    } else {
        let _ = std::fs::remove_file(&path);
    }

    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("invalid probe JSON for {label}: {err}"))
}

pub(super) fn cpp_probe_accepts_bytes(probe: &Path, label: &str, bytes: &[u8]) -> bool {
    let path = cpp_probe_temp_path("rive-rust-runtime", label);
    std::fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    let status = Command::new(probe)
        .arg("--no-advance")
        .arg("--instance-artboards")
        .arg("--file")
        .arg(&path)
        .status()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", probe.display()));
    let _ = std::fs::remove_file(path);
    status.success()
}

pub(super) fn assert_close(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "{label} mismatch: expected {expected}, got {actual}"
    );
}

pub(super) fn compare_mat2d(
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

pub(super) fn compare_optional_f32(
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
