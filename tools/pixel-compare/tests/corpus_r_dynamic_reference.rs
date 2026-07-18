#![cfg(unix)]

use pixel_compare::RgbaImage;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[test]
fn same_runner_reference_replay_replaces_a_missing_committed_reference() {
    let root = temporary_directory("dynamic-reference");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "ignored by fake replay\n").unwrap();
    RgbaImage::new(1, 1, vec![10, 20, 30, 255])
        .unwrap()
        .write_png(root.join("fixture.png"))
        .unwrap();
    let replay = install_fake_replay(&root, "Shared Test GPU");
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "same-runner"
stream = "fixture.rive-stream"
reference = "missing-committed-reference.png"
status = "exact"
frame = 0
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_REPLAY_PNG", root.join("fixture.png"))
        .env("FAKE_REPLAY_LOG", root.join("replay-order.log"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--reference-replay", replay.to_str().unwrap()])
        .args(["--reference-backend", "ffi-dawn"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("exact same-runner: byte-exact=true"));
    assert!(stdout.contains("adapter-check=matched"));
    assert_eq!(
        fs::read_to_string(root.join("replay-order.log")).unwrap(),
        "ffi-dawn\nrust-wgpu\n",
        "each case must render its reference before its candidate"
    );
    assert!(root.join("artifacts/same-runner-reference.png").is_file());
    assert!(root.join("artifacts/same-runner.png").is_file());
    let provenance_path = root.join("artifacts/same-runner.provenance.toml");
    let provenance = fs::read_to_string(&provenance_path).unwrap();
    assert!(provenance.contains(&format!(
        "stream_sha256 = \"{}\"",
        sha256(root.join("fixture.rive-stream"))
    )));
    assert!(provenance.contains(&format!(
        "reference_replay_sha256 = \"{}\"",
        sha256(&replay)
    )));
    assert!(provenance.contains(&format!(
        "candidate_replay_sha256 = \"{}\"",
        sha256(&replay)
    )));
    assert!(provenance.contains(&format!(
        "reference_png_sha256 = \"{}\"",
        sha256(root.join("artifacts/same-runner-reference.png"))
    )));
    assert!(provenance.contains(&format!(
        "candidate_png_sha256 = \"{}\"",
        sha256(root.join("artifacts/same-runner.png"))
    )));
    assert!(!root.join("missing-committed-reference.png").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_runner_reference_fails_closed_and_records_an_adapter_mismatch() {
    let root = temporary_directory("adapter-mismatch");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "ignored by fake replay\n").unwrap();
    RgbaImage::new(1, 1, vec![10, 20, 30, 255])
        .unwrap()
        .write_png(root.join("fixture.png"))
        .unwrap();
    let replay = install_fake_replay(&root, "Shared Test GPU");
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "wrong-adapter"
stream = "fixture.rive-stream"
reference = "missing.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_REPLAY_PNG", root.join("fixture.png"))
        .env("FAKE_REFERENCE_ADAPTER", "Reference GPU")
        .env("FAKE_CANDIDATE_ADAPTER", "Candidate GPU")
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--reference-replay", replay.to_str().unwrap()])
        .args(["--reference-backend", "ffi-dawn"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "renderer adapter mismatch for wrong-adapter: reference `Reference GPU`, candidate `Candidate GPU`"
    ));
    assert!(root.join("artifacts/wrong-adapter-reference.png").is_file());
    assert!(root.join("artifacts/wrong-adapter.png").is_file());
    let provenance = fs::read_to_string(root.join("artifacts/wrong-adapter.provenance.toml"))
        .expect("adapter failures retain provenance");
    assert!(provenance.contains("reference_adapter = \"Reference GPU\""));
    assert!(provenance.contains("candidate_adapter = \"Candidate GPU\""));
    assert!(provenance.contains("adapter_check = \"mismatch\""));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_runner_reference_fails_closed_when_an_adapter_is_unreported() {
    let root = temporary_directory("adapter-unreported");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "ignored by fake replay\n").unwrap();
    RgbaImage::new(1, 1, vec![10, 20, 30, 255])
        .unwrap()
        .write_png(root.join("fixture.png"))
        .unwrap();
    let replay = install_fake_replay(&root, "Shared Test GPU");
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "missing-adapter"
stream = "fixture.rive-stream"
reference = "missing.png"
status = "exact"
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_REPLAY_PNG", root.join("fixture.png"))
        .env("FAKE_OMIT_CANDIDATE_ADAPTER", "1")
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--reference-replay", replay.to_str().unwrap()])
        .args(["--reference-backend", "ffi-dawn"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(
        "renderer adapter identity missing for missing-adapter: candidate did not report `adapter=`"
    ));
    let provenance = fs::read_to_string(root.join("artifacts/missing-adapter.provenance.toml"))
        .expect("missing adapter retains provenance");
    assert!(provenance.contains("reference_adapter = \"Shared Test GPU\""));
    assert!(provenance.contains("candidate_adapter = \"unreported\""));
    assert!(provenance.contains("adapter_check = \"candidate-unreported\""));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_runner_reference_uses_the_manifest_tolerance_unchanged() {
    let root = temporary_directory("manifest-tolerance");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "ignored by fake replay\n").unwrap();
    RgbaImage::new(1, 1, vec![10, 20, 30, 255])
        .unwrap()
        .write_png(root.join("reference.png"))
        .unwrap();
    RgbaImage::new(1, 1, vec![12, 20, 30, 255])
        .unwrap()
        .write_png(root.join("candidate.png"))
        .unwrap();
    let replay = install_fake_replay(&root, "Shared Test GPU");
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "reviewed-tolerance"
stream = "fixture.rive-stream"
reference = "not-used.png"
status = "exact"
max_channel_delta = 2
max_different_pixels = 0
mode = "clockwise-atomic"
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("FAKE_REFERENCE_PNG", root.join("reference.png"))
        .env("FAKE_CANDIDATE_PNG", root.join("candidate.png"))
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--reference-replay", replay.to_str().unwrap()])
        .args(["--reference-backend", "ffi-dawn"])
        .args(["--output-dir", "artifacts"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "exact reviewed-tolerance: byte-exact=false different-pixels=0 max-channel-delta=2"
    ));
    assert!(!root.join("artifacts/reviewed-tolerance-diff.png").exists());
    let provenance =
        fs::read_to_string(root.join("artifacts/reviewed-tolerance.provenance.toml")).unwrap();
    assert!(provenance.contains("max_channel_delta = 2"));
    assert!(provenance.contains("max_different_pixels = 0"));

    fs::remove_dir_all(root).unwrap();
}

fn temporary_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "pixel-compare-{label}-{}-{}",
        std::process::id(),
        NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ))
}

fn install_fake_replay(root: &std::path::Path, adapter: &str) -> PathBuf {
    let path = root.join("fake-renderer-replay.sh");
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
output=''
backend=''
while [ "$#" -gt 0 ]; do
    case "$1" in
        --output) output="$2" ;;
        --backend) backend="$2" ;;
    esac
    shift 2
done
if [ -n "$FAKE_REPLAY_LOG" ]; then
    printf '%s\n' "$backend" >> "$FAKE_REPLAY_LOG"
fi
if [ "$backend" = 'ffi-dawn' ]; then
    selected_png="${{FAKE_REFERENCE_PNG:-$FAKE_REPLAY_PNG}}"
    selected_adapter="${{FAKE_REFERENCE_ADAPTER:-{adapter}}}"
else
    selected_png="${{FAKE_CANDIDATE_PNG:-$FAKE_REPLAY_PNG}}"
    selected_adapter="${{FAKE_CANDIDATE_ADAPTER:-{adapter}}}"
fi
cp "$selected_png" "$output"
if [ "$backend" = 'ffi-dawn' ]; then
    omit_adapter="${{FAKE_OMIT_REFERENCE_ADAPTER:-}}"
else
    omit_adapter="${{FAKE_OMIT_CANDIDATE_ADAPTER:-}}"
fi
if [ -z "$omit_adapter" ]; then
    printf 'adapter=%s\n' "$selected_adapter"
fi
printf 'backend=%s output=%s\n' "$backend" "$output"
"#
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn sha256(path: impl AsRef<std::path::Path>) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}
