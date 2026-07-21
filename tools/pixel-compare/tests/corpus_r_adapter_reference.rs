#![cfg(unix)]

use pixel_compare::RgbaImage;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[test]
fn static_corpus_selects_the_apple_paravirtual_reference() {
    let root = temporary_directory("adapter-reference-selection");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_png(root.join("reference-b.png"), [40, 50, 60, 255]);
    write_valid_provenance(&root, "Adapter A", "reference-a.png");
    write_valid_provenance(&root, "Apple Paravirtual device", "reference-b.png");
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"

[[entry.adapter_reference]]
adapter = "Apple Paravirtual device"
reference = "reference-b.png"
provenance = "reference-b.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Apple Paravirtual device")
        .env("FAKE_REPLAY_PNG", root.join("reference-b.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("exact adapter-bound: byte-exact=true")
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_adapter_references_require_one_runtime_revision() {
    assert_mixed_adapter_revision_is_rejected(
        "runtime_revision",
        "dddddddddddddddddddddddddddddddddddddddd",
    );
}

#[test]
fn static_adapter_references_require_one_dawn_revision() {
    assert_mixed_adapter_revision_is_rejected(
        "dawn_revision",
        "dddddddddddddddddddddddddddddddddddddddd",
    );
}

#[test]
fn static_adapter_reference_rejects_an_unknown_adapter_without_falling_back() {
    let root = temporary_directory("unknown-adapter-reference");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_valid_provenance(&root, "Adapter A", "reference-a.png");
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Unknown Adapter")
        .env("FAKE_REPLAY_PNG", root.join("reference-a.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "renderer adapter `Unknown Adapter` has no approved static reference for adapter-bound"
        ),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_adapter_reference_rejects_an_unreported_adapter_without_falling_back() {
    let root = temporary_directory("unreported-adapter-reference");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_valid_provenance(&root, "Adapter A", "reference-a.png");
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "")
        .env("FAKE_REPLAY_PNG", root.join("reference-a.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "renderer adapter identity missing for adapter-bound: candidate did not report `adapter=`"
        ),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_adapter_reference_rejects_provenance_that_does_not_match_the_png() {
    let root = temporary_directory("adapter-reference-provenance");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    fs::write(
        root.join("reference-a.provenance"),
        format!(
            concat!(
                "provenance_schema=1\n",
                "backend=metal\n",
                "renderer_implementation=cpp-dawn-webgpu\n",
                "capture_tool=renderer-replay-ffi-dawn\n",
                "adapter_device=Adapter A\n",
                "case_id=adapter-bound\n",
                "stream_sha256={}\n",
                "runtime_revision=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
                "dawn_revision=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n",
                "replay_sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\n",
                "png_sha256=0000000000000000000000000000000000000000000000000000000000000000\n",
                "frame_width=1\n",
                "frame_height=1\n",
                "frame=0\n",
                "mode=clockwise-atomic\n",
                "sample_count=1\n"
            ),
            sha256(root.join("fixture.rive-stream"))
        ),
    )
    .unwrap();
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Adapter A")
        .env("FAKE_REPLAY_PNG", root.join("reference-a.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("reference-a.provenance png_sha256 does not match reference-a.png"),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_adapter_reference_requires_provenance_for_every_known_adapter() {
    let root = temporary_directory("missing-adapter-reference-provenance");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Adapter A")
        .env("FAKE_REPLAY_PNG", root.join("reference-a.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "static adapter reference for adapter-bound adapter `Adapter A` must name provenance"
        ),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_adapter_reference_rejects_provenance_for_a_different_adapter() {
    let root = temporary_directory("mismatched-adapter-provenance");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_valid_provenance(&root, "Other Adapter", "reference-a.png");
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Adapter A")
        .env("FAKE_REPLAY_PNG", root.join("reference-a.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "reference-a.provenance adapter_device `Other Adapter` does not match manifest adapter `Adapter A`"
        ),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn static_adapter_reference_rejects_duplicate_adapter_names() {
    let root = temporary_directory("duplicate-adapter-reference");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_png(root.join("reference-b.png"), [40, 50, 60, 255]);
    write_valid_provenance(&root, "Adapter A", "reference-a.png");
    write_valid_provenance(&root, "Adapter A", "reference-b.png");
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-b.png"
provenance = "reference-b.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Adapter A")
        .env("FAKE_REPLAY_PNG", root.join("reference-a.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("adapter-bound repeats static adapter reference `Adapter A`"),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stub_baseline_checks_adapter_bound_entries_without_a_hardware_adapter() {
    let root = temporary_directory("adapter-reference-stub-baseline");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_png(root.join("transparent.png"), [0, 0, 0, 0]);
    write_valid_provenance(&root, "Adapter A", "reference-a.png");
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "")
        .env("FAKE_REPLAY_PNG", root.join("transparent.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "stub"])
        .args(["--output-dir", "artifacts"])
        .arg("--expect-all-fail")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("diverges adapter-bound: different-pixels=1"));

    fs::remove_dir_all(root).unwrap();
}

fn temporary_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "pixel-compare-{label}-{}-{}",
        std::process::id(),
        NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ))
}

fn write_png(path: impl AsRef<Path>, rgba: [u8; 4]) {
    RgbaImage::new(1, 1, rgba.to_vec())
        .unwrap()
        .write_png(path)
        .unwrap();
}

fn write_valid_provenance(root: &Path, adapter: &str, reference: &str) {
    let provenance = Path::new(reference).with_extension("provenance");
    fs::write(
        root.join(provenance),
        format!(
            concat!(
                "provenance_schema=1\n",
                "backend=metal\n",
                "renderer_implementation=cpp-dawn-webgpu\n",
                "capture_tool=renderer-replay-ffi-dawn\n",
                "adapter_device={}\n",
                "case_id=adapter-bound\n",
                "stream_sha256={}\n",
                "runtime_revision=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
                "dawn_revision=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n",
                "replay_sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\n",
                "png_sha256={}\n",
                "frame_width=1\n",
                "frame_height=1\n",
                "frame=0\n",
                "mode=clockwise-atomic\n",
                "sample_count=1\n"
            ),
            adapter,
            sha256(root.join("fixture.rive-stream")),
            sha256(root.join(reference)),
        ),
    )
    .unwrap();
}

fn assert_mixed_adapter_revision_is_rejected(field: &str, replacement: &str) {
    let root = temporary_directory(&format!("mixed-{field}"));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "fake renderer stream\n").unwrap();
    write_png(root.join("reference-a.png"), [10, 20, 30, 255]);
    write_png(root.join("reference-b.png"), [40, 50, 60, 255]);
    write_valid_provenance(&root, "Adapter A", "reference-a.png");
    write_valid_provenance(&root, "Apple Paravirtual device", "reference-b.png");
    let provenance_path = root.join("reference-b.provenance");
    let provenance = fs::read_to_string(&provenance_path).unwrap();
    let original = match field {
        "runtime_revision" => "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "dawn_revision" => "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        _ => panic!("unexpected revision field {field}"),
    };
    fs::write(
        &provenance_path,
        provenance.replace(
            &format!("{field}={original}"),
            &format!("{field}={replacement}"),
        ),
    )
    .unwrap();
    let replay = install_fake_replay(&root);
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "adapter-bound"
stream = "fixture.rive-stream"
reference = "reference-a.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"

[[entry.adapter_reference]]
adapter = "Adapter A"
reference = "reference-a.png"
provenance = "reference-a.provenance"

[[entry.adapter_reference]]
adapter = "Apple Paravirtual device"
reference = "reference-b.png"
provenance = "reference-b.provenance"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_ADAPTER", "Apple Paravirtual device")
        .env("FAKE_REPLAY_PNG", root.join("reference-b.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "adapter-bound adapter references disagree on {field}"
        )),
        "stderr:\n{stderr}"
    );

    fs::remove_dir_all(root).unwrap();
}

fn install_fake_replay(root: &Path) -> PathBuf {
    let path = root.join("fake-renderer-replay.sh");
    fs::write(
        &path,
        r#"#!/bin/sh
output=''
while [ "$#" -gt 0 ]; do
    case "$1" in
        --output) output="$2" ;;
    esac
    shift 2
done
cp "$FAKE_REPLAY_PNG" "$output"
printf 'adapter=%s\n' "$FAKE_ADAPTER"
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn sha256(path: impl AsRef<Path>) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}
