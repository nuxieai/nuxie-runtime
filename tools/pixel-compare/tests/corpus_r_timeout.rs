#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[test]
fn rejects_a_zero_replay_timeout() {
    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .args(["--replay-timeout-seconds", "0"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("--replay-timeout-seconds must be a positive integer, got `0`"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn times_out_and_reaps_a_sleeping_replay_with_case_diagnostics() {
    let root = temporary_directory("sleeping-replay");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("fixture.rive-stream"),
        "sleeping replay fixture\n",
    )
    .unwrap();
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "sleeping-replay"
stream = "fixture.rive-stream"
reference = "unused.png"
status = "exact"
frame = 0
max_channel_delta = 0
max_different_pixels = 0
mode = "clockwise-atomic"
"#,
    )
    .unwrap();
    let replay = install_sleeping_replay(&root);
    let pid_path = root.join("replay.pid");

    let started = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .env("REPLAY_PID", &pid_path)
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--reference-replay", replay.to_str().unwrap()])
        .args(["--reference-backend", "ffi-dawn"])
        .args(["--output-dir", "artifacts"])
        .args(["--replay-timeout-seconds", "1"])
        .output()
        .unwrap();
    let elapsed = started.elapsed();

    assert!(!output.status.success());
    assert!(
        elapsed < Duration::from_secs(3),
        "a two-second replay should be stopped near the one-second deadline; elapsed={elapsed:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("sleeping replay stdout"),
        "stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("sleeping replay stderr"),
        "stderr:\n{stderr}"
    );
    assert!(
        stderr.contains(
            "renderer replay timed out after 1 second for entry `sleeping-replay` with backend `ffi-dawn`"
        ),
        "stderr:\n{stderr}"
    );
    let pid = fs::read_to_string(&pid_path).unwrap();
    assert!(
        !Command::new("kill")
            .args(["-0", pid.trim()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success(),
        "timed-out replay process {} must be reaped",
        pid.trim()
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn drains_verbose_replay_output_while_the_child_is_running() {
    let root = temporary_directory("verbose-replay");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("fixture.rive-stream"), "verbose replay fixture\n").unwrap();
    fs::write(
        root.join("corpus.toml"),
        r#"
[[entry]]
id = "verbose-replay"
stream = "fixture.rive-stream"
reference = "unused.png"
status = "exact"
frame = 0
max_channel_delta = 0
max_different_pixels = 0
mode = "msaa"
"#,
    )
    .unwrap();
    let replay = install_verbose_failed_replay(&root);

    let output = Command::new(env!("CARGO_BIN_EXE_corpus-r"))
        .current_dir(&root)
        .args(["--manifest", "corpus.toml"])
        .args(["--replay", replay.to_str().unwrap()])
        .args(["--backend", "rust-wgpu"])
        .args(["--reference-replay", replay.to_str().unwrap()])
        .args(["--reference-backend", "ffi-dawn"])
        .args(["--output-dir", "artifacts"])
        .args(["--replay-timeout-seconds", "10"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("stdout-line-9999"), "stdout tail missing");
    assert!(stderr.contains("stderr-line-9999"), "stderr tail missing");
    assert!(
        stderr.contains("reference renderer replay failed for verbose-replay"),
        "stderr:\n{stderr}"
    );
    assert!(!stderr.contains("timed out"), "stderr:\n{stderr}");

    fs::remove_dir_all(root).unwrap();
}

fn temporary_directory(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "pixel-compare-{label}-{}-{}",
        std::process::id(),
        NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ))
}

fn install_sleeping_replay(root: &Path) -> PathBuf {
    let path = root.join("sleeping-renderer-replay.sh");
    fs::write(
        &path,
        r#"#!/bin/sh
printf '%s\n' "$$" > "$REPLAY_PID"
printf 'sleeping replay stdout\n'
printf 'sleeping replay stderr\n' >&2
exec sleep 2
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn install_verbose_failed_replay(root: &Path) -> PathBuf {
    let path = root.join("verbose-failed-renderer-replay.sh");
    fs::write(
        &path,
        r#"#!/bin/sh
i=0
while [ "$i" -lt 10000 ]; do
    printf 'stdout-line-%s\n' "$i"
    printf 'stderr-line-%s\n' "$i" >&2
    i=$((i + 1))
done
exit 7
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}
