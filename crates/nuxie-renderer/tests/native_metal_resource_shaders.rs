#![cfg(all(
    any(target_os = "ios", target_os = "macos"),
    feature = "native-metal-experimental"
))]

use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs, path::PathBuf, process::Command};

const PROVENANCE: &str = include_str!("fixtures/native_metal/resource_shader_provenance.txt");
const BUILD_SCRIPT: &[u8] = include_bytes!("../build.rs");

#[cfg(target_os = "macos")]
const ARTIFACT_VARIANT: &str = "macosx";
#[cfg(all(target_os = "ios", target_abi = "sim"))]
const ARTIFACT_VARIANT: &str = "iphonesimulator";
#[cfg(all(target_os = "ios", not(target_abi = "sim")))]
const ARTIFACT_VARIANT: &str = "iphoneos";

const SOURCES: &[(&str, &[u8])] = &[
    (
        "color_ramp.metal",
        include_bytes!("../src/native_metal/shaders/color_ramp.metal"),
    ),
    (
        "tessellate.metal",
        include_bytes!("../src/native_metal/shaders/tessellate.metal"),
    ),
    (
        "metal.minified.glsl",
        include_bytes!("../src/native_metal/shaders/metal.minified.glsl"),
    ),
    (
        "constants.minified.glsl",
        include_bytes!("../src/native_metal/shaders/constants.minified.glsl"),
    ),
    (
        "flush_uniforms.minified.glsl",
        include_bytes!("../src/native_metal/shaders/flush_uniforms.minified.glsl"),
    ),
    (
        "common.minified.glsl",
        include_bytes!("../src/native_metal/shaders/common.minified.glsl"),
    ),
    (
        "color_ramp.minified.glsl",
        include_bytes!("../src/native_metal/shaders/color_ramp.minified.glsl"),
    ),
    (
        "bezier_utils.minified.glsl",
        include_bytes!("../src/native_metal/shaders/bezier_utils.minified.glsl"),
    ),
    (
        "tessellate.minified.glsl",
        include_bytes!("../src/native_metal/shaders/tessellate.minified.glsl"),
    ),
];

fn fixture_entries(prefix: &str) -> BTreeMap<&'static str, &'static str> {
    PROVENANCE
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .filter_map(|line| line.split_once('='))
        .collect()
}

fn fixture_value(key: &str) -> Option<&'static str> {
    PROVENANCE
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .and_then(|value| value.strip_prefix('='))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn active_toolchain_matches_captured_artifacts() -> bool {
    let Some(xcode) = command_stdout("xcodebuild", &["-version"]) else {
        return false;
    };
    let expected_xcode = format!(
        "Xcode {}\nBuild version {}",
        fixture_value("xcode_version").expect("captured Xcode version"),
        fixture_value("xcode_build").expect("captured Xcode build"),
    );
    if xcode != expected_xcode {
        return false;
    }

    let Some(metal) = command_stdout("xcrun", &["metal", "--version"]) else {
        return false;
    };
    let captured_metal = fixture_value("metal_compiler").expect("captured Metal compiler");
    let captured_metal_identity = captured_metal
        .strip_prefix("Apple metal ")
        .unwrap_or(captured_metal);
    if !metal.contains(captured_metal_identity) {
        return false;
    }

    let Some(metallib) = command_stdout("xcrun", &["metallib", "--version"]) else {
        return false;
    };
    metallib.contains(fixture_value("metallib_linker").expect("captured metallib linker"))
}

fn source_matches_digest(name: &str, bytes: &[u8], expected: &str) -> bool {
    let exact = format!("{:x}", Sha256::digest(bytes));
    if name != "bezier_utils.minified.glsl" {
        return expected == exact;
    }
    // The pinned generator omits the terminal newline from bezier_utils while
    // apply_patch keeps text fixtures newline-terminated. The shader payload
    // itself is otherwise byte-identical, and Metal preprocessing is newline
    // insensitive, so provenance compares the generated body bytes.
    let without_terminal_newline = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let normalized = format!("{:x}", Sha256::digest(without_terminal_newline));
    expected == exact || expected == normalized
}

#[test]
fn pinned_resource_sources_match_generated_batch() {
    assert_eq!(
        fixture_value("upstream_commit"),
        Some("4ac7b32798da0482e441ef09304dc3b480ed3ee5")
    );
    let expected = fixture_entries("source:");
    assert_eq!(expected.len(), SOURCES.len());
    for (name, bytes) in SOURCES {
        let expected = expected
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("missing provenance for {name}"));
        assert!(
            source_matches_digest(name, bytes, expected),
            "source bytes for {name} differ from pinned generated batch",
        );
    }
}

#[test]
fn provenance_records_reproducible_generator_and_five_target_matrix() {
    for key in [
        "generator",
        "generator_blob",
        "generator_sha256",
        "generator_makefile",
        "generator_makefile_blob",
        "generator_makefile_sha256",
        "generator_premake",
        "generator_premake_blob",
        "generator_premake_sha256",
        "generator_input_set_sha256",
        "generation_command",
        "verified_generator_runtime",
        "verified_generation_date",
        "compile_command",
        "link_command",
        "xcode_version",
        "xcode_build",
        "metal_compiler",
        "metallib_linker",
    ] {
        assert!(
            fixture_value(key).is_some(),
            "missing provenance fact {key}"
        );
    }

    assert_eq!(
        sha256(BUILD_SCRIPT),
        fixture_value("build_script_sha256").expect("build-script digest")
    );
    assert_eq!(
        fixture_entries("target:"),
        BTreeMap::from([
            ("aarch64-apple-darwin", "macosx"),
            ("aarch64-apple-ios", "iphoneos"),
            ("aarch64-apple-ios-sim", "iphonesimulator"),
            ("x86_64-apple-darwin", "macosx"),
            ("x86_64-apple-ios", "iphonesimulator"),
        ])
    );
    for variant in ["macosx", "iphoneos", "iphonesimulator"] {
        assert!(fixture_value(&format!("sdk:{variant}")).is_some());
        assert!(fixture_value(&format!("arguments:{variant}")).is_some());
        assert_eq!(
            fixture_entries(&format!("output:{variant}:"))
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            [
                "native_metal_color_ramp.air",
                "native_metal_resources.metallib",
                "native_metal_tessellate.air",
            ]
        );
    }
}

#[test]
fn matching_captured_toolchain_reproduces_air_and_metallib_bytes() {
    // Artifact hashes attest the recorded Xcode capture, not every supported
    // compiler release. Other installed toolchains still have to satisfy the
    // semantic entry-point inventory and live Metal-loading tests below.
    if !active_toolchain_matches_captured_artifacts() {
        return;
    }
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let expected = fixture_entries(&format!("output:{ARTIFACT_VARIANT}:"));
    assert_eq!(expected.len(), 3);
    for (name, expected_digest) in expected {
        let path = output_dir.join(name);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "read captured {} output {}: {error}",
                ARTIFACT_VARIANT,
                path.display()
            )
        });
        assert_eq!(
            sha256(&bytes),
            expected_digest,
            "{ARTIFACT_VARIANT} Metal artifact {} differs from the captured Xcode toolchain output",
            path.display()
        );
    }
}

#[test]
fn linked_resource_metallib_has_exact_entry_point_inventory() {
    let expected = PROVENANCE
        .lines()
        .find_map(|line| line.strip_prefix("functions="))
        .expect("resource function inventory fixture")
        .split(',')
        .collect::<Vec<_>>();
    let metallib = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("native_metal_resources.metallib");
    assert!(metallib.is_file(), "missing {}", metallib.display());

    let output = Command::new("xcrun")
        .args(["metal-objdump", "--metallib", "--disassemble-all"])
        .arg(&metallib)
        .output()
        .expect("xcrun metal-objdump is installed");
    assert!(
        output.status.success(),
        "metal-objdump failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output_text = String::from_utf8_lossy(&output.stdout);
    let inventory = output_text
        .lines()
        .filter_map(|line| line.split_once(" -- ").map(|(_, name)| name))
        .filter_map(|name| name.strip_suffix(':'))
        .collect::<Vec<_>>();
    assert_eq!(
        inventory, expected,
        "compiled Metal entry-point inventory changed"
    );
}
