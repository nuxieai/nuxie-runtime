#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SCENE_SAMPLES_PER_LEG: usize = 16 * 7;

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let unique = format!(
            "rive-r4-timing-gate-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn executable(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap();
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).unwrap();
}

fn static_runner(directory: &Path, name: &str, median_ns: u64) -> PathBuf {
    let path = directory.join(name);
    executable(
        &path,
        &format!(
            "#!/bin/sh\n# {name}\nset -eu\nIFS= read -r request\nprefix=${{request%?}}\nprintf '%s,\\\"selected_adapter\\\":{{\\\"backend\\\":\\\"metal\\\",\\\"name\\\":\\\"Integration GPU\\\",\\\"vendor\\\":\\\"Integration Vendor\\\",\\\"device\\\":\\\"Integration Device\\\",\\\"driver\\\":\\\"1.0\\\"}},\\\"measured_frame_median_ns\\\":{median_ns},\\\"logical_flushes\\\":3,\\\"draws\\\":11,\\\"atomic_strategy_partitions\\\":2}}\\n' \"$prefix\"\n"
        ),
    );
    path
}

fn drifting_baseline_runner(directory: &Path) -> PathBuf {
    let path = directory.join("baseline.sh");
    let count = directory.join("baseline-count");
    executable(
        &path,
        &format!(
            "#!/bin/sh\nset -eu\ncount_file='{}'\ncount=0\nif [ -f \"$count_file\" ]; then count=$(cat \"$count_file\"); fi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nmedian=100\nif [ \"$count\" -gt {SCENE_SAMPLES_PER_LEG} ]; then median=150; fi\nIFS= read -r request\nprefix=${{request%?}}\nprintf '%s,\\\"selected_adapter\\\":{{\\\"backend\\\":\\\"metal\\\",\\\"name\\\":\\\"Integration GPU\\\",\\\"vendor\\\":\\\"Integration Vendor\\\",\\\"device\\\":\\\"Integration Device\\\",\\\"driver\\\":\\\"1.0\\\"}},\\\"measured_frame_median_ns\\\":%s,\\\"logical_flushes\\\":3,\\\"draws\\\":11,\\\"atomic_strategy_partitions\\\":2}}\\n' \"$prefix\" \"$median\"\n",
            count.display()
        ),
    );
    path
}

fn malformed_runner(directory: &Path) -> PathBuf {
    let path = directory.join("malformed.sh");
    executable(&path, "#!/bin/sh\ncat >/dev/null\nprintf '{not json}\\n'\n");
    path
}

fn mutating_runner(directory: &Path) -> PathBuf {
    let path = static_runner(directory, "mutating.sh", 100);
    let contents = std::fs::read_to_string(&path).unwrap();
    executable(
        &path,
        &format!(
            "#!/bin/sh\nset -eu\nif [ ! -e \"$0.mutated\" ]; then : > \"$0.mutated\"; printf '\\n# changed during timing gate\\n' >> \"$0\"; fi\n{}",
            contents.lines().skip(3).collect::<Vec<_>>().join("\n")
        ),
    );
    path
}

fn host_sampler(directory: &Path) -> PathBuf {
    let path = directory.join("host-sampler.sh");
    executable(
        &path,
        "#!/bin/sh\nprintf 'CPU usage: 1.0%% user, 1.0%% sys, 98.0%% idle\\n'\nprintf 'Processes: 1 total, 1 running\\n'\nprintf 'PID COMMAND %%CPU\\n1 renderer-perf 1.0\\n'\n",
    );
    path
}

struct Gate<'a> {
    directory: &'a Path,
    baseline: &'a Path,
    a: &'a Path,
    b: &'a Path,
    max_b_over_a: &'a str,
    max_control_drift: &'a str,
}

fn run_gate(gate: Gate<'_>) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../r4-timing-gate.sh");
    Command::new(script)
        .current_dir(gate.directory)
        .args(["--output-dir", "artifacts"])
        .env(
            "R4_TIMING_GATE_RENDERER_PERF",
            env!("CARGO_BIN_EXE_renderer-perf"),
        )
        .env(
            "R4_TIMING_GATE_COMPARATOR",
            env!("CARGO_BIN_EXE_r4-timing-compare"),
        )
        .env(
            "R4_TIMING_GATE_MANIFEST",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("renderer-scenes.toml"),
        )
        .env("R4_TIMING_GATE_BASELINE_RUNNER", gate.baseline)
        .env("R4_TIMING_GATE_A_RUNNER", gate.a)
        .env("R4_TIMING_GATE_B_RUNNER", gate.b)
        .env("R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO", "10")
        .env("R4_TIMING_GATE_MAX_B_OVER_A", gate.max_b_over_a)
        .env("R4_TIMING_GATE_MAX_CONTROL_DRIFT", gate.max_control_drift)
        .env("R4_TIMING_GATE_MIN_IDLE_PERCENT", "90")
        .env("R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT", "1")
        .env("R4_TIMING_GATE_HOST_SAMPLE_INTERVAL_SECONDS", "0.01")
        .env("R4_TIMING_GATE_HOST_SAMPLER", host_sampler(gate.directory))
        .output()
        .unwrap()
}

fn metadata(directory: &Path) -> String {
    std::fs::read_to_string(directory.join("artifacts/metadata.env")).unwrap()
}

fn assert_finalized_failure(directory: &Path) {
    let metadata = metadata(directory);
    assert!(metadata.contains("status=fail\n"), "{metadata}");
    assert!(metadata.contains("failure_phase="), "{metadata}");
    assert!(metadata.contains("failure_reason="), "{metadata}");
    assert!(metadata.contains("utc_finished="), "{metadata}");
    assert!(metadata.contains("artifact_dir="), "{metadata}");
}

#[test]
fn r4_gate_accepts_faster_b_and_retains_in_leg_process_samples() {
    let directory = TempDir::new("faster");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 100);
    let b = static_runner(&directory.path, "b.sh", 90);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_b_over_a: "1",
        max_control_drift: "1",
    });

    assert!(
        output.status.success(),
        "stderr={} metadata={} comparator={}",
        String::from_utf8_lossy(&output.stderr),
        metadata(&directory.path),
        std::fs::read_to_string(directory.path.join("artifacts/comparator.stderr"))
            .unwrap_or_default()
    );
    let metadata = metadata(&directory.path);
    assert!(metadata.contains("status=pass\n"), "{metadata}");
    assert!(
        metadata
            .lines()
            .any(|line| line.starts_with("artifact_dir=/") && !line.ends_with("=artifacts")),
        "{metadata}"
    );
    let samples = std::fs::read_to_string(directory.path.join("artifacts/host-idle.tsv")).unwrap();
    assert!(samples.contains("during-0001"), "{samples}");
    let comparison =
        std::fs::read_to_string(directory.path.join("artifacts/comparison.json")).unwrap();
    assert!(
        comparison.contains("\"candidate_b_over_a\": 0.9"),
        "{comparison}"
    );

    let malformed = directory.path.join("artifacts/01-A.renderer-perf.json");
    std::fs::write(&malformed, "{not renderer-perf JSON}\n").unwrap();
    let validation = Command::new(env!("CARGO_BIN_EXE_r4-timing-compare"))
        .args([
            "--a-first",
            malformed.to_str().unwrap(),
            "--b-first",
            directory
                .path
                .join("artifacts/02-B.renderer-perf.json")
                .to_str()
                .unwrap(),
            "--b-second",
            directory
                .path
                .join("artifacts/03-B.renderer-perf.json")
                .to_str()
                .unwrap(),
            "--a-second",
            directory
                .path
                .join("artifacts/04-A.renderer-perf.json")
                .to_str()
                .unwrap(),
            "--max-b-over-a",
            "1",
            "--max-control-drift",
            "1",
        ])
        .output()
        .unwrap();
    assert!(!validation.status.success());
    assert!(
        String::from_utf8_lossy(&validation.stderr).contains("invalid rive-renderer-perf-v1 JSON")
    );
}

#[test]
fn r4_gate_rejects_slow_b_and_finalizes_metadata() {
    let directory = TempDir::new("slower");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 100);
    let b = static_runner(&directory.path, "b.sh", 120);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_b_over_a: "1.05",
        max_control_drift: "1",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(metadata(&directory.path).contains("failure_phase=validate-comparison"));
}

#[test]
fn r4_gate_rejects_cpp_control_drift() {
    let directory = TempDir::new("control-drift");
    let baseline = drifting_baseline_runner(&directory.path);
    let a = static_runner(&directory.path, "a.sh", 100);
    let b = static_runner(&directory.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_b_over_a: "1",
        max_control_drift: "1.1",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(metadata(&directory.path).contains("C++\\ control\\ drift\\ failed"));
}

#[test]
fn r4_gate_rejects_malformed_report_json() {
    let directory = TempDir::new("malformed");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let malformed = malformed_runner(&directory.path);
    let b = static_runner(&directory.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &malformed,
        b: &b,
        max_b_over_a: "1",
        max_control_drift: "1",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(metadata(&directory.path).contains("failure_phase=run-01-A"));
}

#[test]
fn r4_gate_rejects_runner_identity_and_mutation() {
    let identity = TempDir::new("identity");
    let baseline = static_runner(&identity.path, "baseline.sh", 100);
    let a = static_runner(&identity.path, "a.sh", 100);
    let output = run_gate(Gate {
        directory: &identity.path,
        baseline: &baseline,
        a: &a,
        b: &a,
        max_b_over_a: "1",
        max_control_drift: "1",
    });
    assert!(!output.status.success());
    assert_finalized_failure(&identity.path);
    assert!(metadata(&identity.path).contains("A\\ and\\ B\\ runners\\ must\\ be\\ distinct"));

    let mutation = TempDir::new("mutation");
    let baseline = static_runner(&mutation.path, "baseline.sh", 100);
    let a = mutating_runner(&mutation.path);
    let b = static_runner(&mutation.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &mutation.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_b_over_a: "1",
        max_control_drift: "1",
    });
    assert!(!output.status.success());
    assert_finalized_failure(&mutation.path);
    assert!(
        metadata(&mutation.path).contains("A\\ runner\\ changed\\ during\\ the\\ gate"),
        "{}",
        metadata(&mutation.path)
    );
}
