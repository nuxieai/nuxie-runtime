#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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

fn drifting_candidate_runner(directory: &Path) -> PathBuf {
    let path = directory.join("drifting-candidate.sh");
    let count = directory.join("candidate-count");
    executable(
        &path,
        &format!(
            "#!/bin/sh\nset -eu\ncount_file='{}'\ncount=0\nif [ -f \"$count_file\" ]; then count=$(cat \"$count_file\"); fi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nmedian=100\nif [ \"$count\" -gt {SCENE_SAMPLES_PER_LEG} ]; then median=150; fi\nIFS= read -r request\nprefix=${{request%?}}\nprintf '%s,\\\"selected_adapter\\\":{{\\\"backend\\\":\\\"metal\\\",\\\"name\\\":\\\"Integration GPU\\\",\\\"vendor\\\":\\\"Integration Vendor\\\",\\\"device\\\":\\\"Integration Device\\\",\\\"driver\\\":\\\"1.0\\\"}},\\\"measured_frame_median_ns\\\":%s,\\\"logical_flushes\\\":3,\\\"draws\\\":11,\\\"atomic_strategy_partitions\\\":2}}\\n' \"$prefix\" \"$median\"\n",
            count.display()
        ),
    );
    path
}

fn malformed_runner(directory: &Path) -> (PathBuf, PathBuf) {
    let path = directory.join("malformed.sh");
    let pid_file = directory.join("malformed.pid");
    executable(
        &path,
        &format!(
            "#!/bin/sh\nset -eu\nprintf '%s' \"$$\" > '{}'\nIFS= read -r request\nsleep 0.2\nprintf '{{not json}}\\n'\n",
            pid_file.display()
        ),
    );
    (path, pid_file)
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

fn fixed_host_sampler(directory: &Path, name: &str, idle_percent: u32) -> PathBuf {
    let path = directory.join(name);
    executable(
        &path,
        &format!("#!/bin/sh\nprintf 'r4-host-idle-percent={idle_percent}\\n'\n"),
    );
    path
}

fn alternating_host_sampler(directory: &Path) -> PathBuf {
    let path = directory.join("alternating-host-sampler.sh");
    let count = directory.join("host-sample-count");
    executable(
        &path,
        &format!(
            "#!/bin/sh\nset -eu\ncount_file='{}'\ncount=0\nif [ -f \"$count_file\" ]; then count=$(cat \"$count_file\"); fi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$count_file\"\nidle=98\nif [ $((count % 2)) -eq 0 ]; then idle=90; fi\nprintf 'r4-host-idle-percent=%s\\n' \"$idle\"\n",
            count.display()
        ),
    );
    path
}

fn coordinated_runner(
    directory: &Path,
    name: &str,
    runner_active: &Path,
    sampler_active: &Path,
    overlap: &Path,
) -> PathBuf {
    let path = directory.join(name);
    executable(
        &path,
        &format!(
            "#!/bin/sh\n# {name}\nset -eu\nrunner_active='{}'\nsampler_active='{}'\noverlap='{}'\ncleanup() {{ rm -f \"$runner_active\"; }}\ntrap cleanup EXIT HUP INT TERM\n: > \"$runner_active\"\n[ ! -e \"$sampler_active\" ] || : > \"$overlap\"\nsleep 0.005\n[ ! -e \"$sampler_active\" ] || : > \"$overlap\"\nIFS= read -r request\nprefix=${{request%?}}\nprintf '%s,\\\"selected_adapter\\\":{{\\\"backend\\\":\\\"metal\\\",\\\"name\\\":\\\"Integration GPU\\\",\\\"vendor\\\":\\\"Integration Vendor\\\",\\\"device\\\":\\\"Integration Device\\\",\\\"driver\\\":\\\"1.0\\\"}},\\\"measured_frame_median_ns\\\":100,\\\"logical_flushes\\\":3,\\\"draws\\\":11,\\\"atomic_strategy_partitions\\\":2}}\\n' \"$prefix\"\n",
            runner_active.display(),
            sampler_active.display(),
            overlap.display()
        ),
    );
    path
}

fn coordinated_host_sampler(
    directory: &Path,
    runner_active: &Path,
    sampler_active: &Path,
    overlap: &Path,
) -> PathBuf {
    let path = directory.join("coordinated-host-sampler.sh");
    executable(
        &path,
        &format!(
            "#!/bin/sh\nset -eu\nrunner_active='{}'\nsampler_active='{}'\noverlap='{}'\ncleanup() {{ rm -f \"$sampler_active\"; }}\ntrap cleanup EXIT HUP INT TERM\n: > \"$sampler_active\"\n[ ! -e \"$runner_active\" ] || : > \"$overlap\"\nsleep 0.01\n[ ! -e \"$runner_active\" ] || : > \"$overlap\"\nprintf 'r4-host-idle-percent=98\\n'\n",
            runner_active.display(),
            sampler_active.display(),
            overlap.display()
        ),
    );
    path
}

struct Gate<'a> {
    directory: &'a Path,
    baseline: &'a Path,
    a: &'a Path,
    b: &'a Path,
    max_renderer_ratio: &'a str,
    max_b_over_a: &'a str,
    max_control_drift: &'a str,
    max_repeat_drift: &'a str,
}

fn run_gate(gate: Gate<'_>) -> Output {
    let sampler = host_sampler(gate.directory);
    run_gate_with_sampler(gate, &sampler)
}

fn run_gate_with_sampler(gate: Gate<'_>, sampler: &Path) -> Output {
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
        .env(
            "R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO",
            gate.max_renderer_ratio,
        )
        .env("R4_TIMING_GATE_MAX_B_OVER_A", gate.max_b_over_a)
        .env("R4_TIMING_GATE_MAX_CONTROL_DRIFT", gate.max_control_drift)
        .env("R4_TIMING_GATE_MAX_REPEAT_DRIFT", gate.max_repeat_drift)
        .env("R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT", "1")
        .env("R4_TIMING_GATE_HOST_SAMPLER", sampler)
        .output()
        .unwrap()
}

fn process_is_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success()
}

fn metadata(directory: &Path) -> String {
    std::fs::read_to_string(directory.join("artifacts/metadata.env")).unwrap()
}

fn decision(directory: &Path) -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(directory.join("artifacts/gate-decision.json")).unwrap(),
    )
    .unwrap()
}

fn run_comparator(directory: &Path, a_first: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_r4-timing-compare"))
        .args([
            "--a-first",
            a_first.to_str().unwrap(),
            "--b-first",
            directory
                .join("artifacts/02-B.renderer-perf.json")
                .to_str()
                .unwrap(),
            "--b-second",
            directory
                .join("artifacts/03-B.renderer-perf.json")
                .to_str()
                .unwrap(),
            "--a-second",
            directory
                .join("artifacts/04-A.renderer-perf.json")
                .to_str()
                .unwrap(),
            "--max-renderer-ratio",
            "10",
            "--max-b-over-a",
            "1",
            "--max-control-drift",
            "1",
            "--max-repeat-drift",
            "1",
        ])
        .output()
        .unwrap()
}

fn assert_finalized_failure(directory: &Path) {
    let metadata = metadata(directory);
    assert!(metadata.contains("status=fail\n"), "{metadata}");
    assert!(metadata.contains("failure_phase="), "{metadata}");
    assert!(metadata.contains("failure_reason="), "{metadata}");
    assert!(metadata.contains("utc_finished="), "{metadata}");
    assert!(metadata.contains("artifact_dir="), "{metadata}");
    assert!(metadata.contains("decision_path="), "{metadata}");
    let decision = decision(directory);
    assert_eq!(decision["schema"], "rive-r4-timing-gate-decision-v1");
    assert_eq!(decision["status"], "fail");
    assert!(!decision["reason"].is_null());
}

#[test]
fn r4_gate_accepts_faster_b_and_samples_only_outside_timed_legs() {
    let directory = TempDir::new("faster");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 100);
    let b = static_runner(&directory.path, "b.sh", 90);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_renderer_ratio: "10",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
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
    let decision = decision(&directory.path);
    assert_eq!(decision["status"], "pass");
    assert_eq!(decision["comparison_available"], true);
    assert!(decision["reason"].is_null());
    assert!(
        metadata
            .lines()
            .any(|line| line.starts_with("artifact_dir=/") && !line.ends_with("=artifacts")),
        "{metadata}"
    );
    let samples = std::fs::read_to_string(directory.path.join("artifacts/host-idle.tsv")).unwrap();
    assert_eq!(samples.lines().count(), 9, "{samples}");
    assert!(!samples.contains("during"), "{samples}");
    let comparison =
        std::fs::read_to_string(directory.path.join("artifacts/comparison.json")).unwrap();
    assert!(
        comparison.contains("\"normalized_b_over_a\": 0.9"),
        "{comparison}"
    );

    let a_report = directory.path.join("artifacts/01-A.renderer-perf.json");
    let mut tampered: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&a_report).unwrap()).unwrap();
    tampered["scenes"][0]["control_selected_pair"]["candidate_ns"] = 999.into();
    let tampered_path = directory.path.join("tampered-pair.json");
    std::fs::write(
        &tampered_path,
        format!("{}\n", serde_json::to_string_pretty(&tampered).unwrap()),
    )
    .unwrap();
    let validation = run_comparator(&directory.path, &tampered_path);
    assert!(!validation.status.success());
    assert!(
        String::from_utf8_lossy(&validation.stderr).contains("inconsistent control-selected pair")
    );

    std::fs::write(&a_report, "{not renderer-perf JSON}\n").unwrap();
    let validation = run_comparator(&directory.path, &a_report);
    assert!(!validation.status.success());
    assert!(
        String::from_utf8_lossy(&validation.stderr).contains("invalid rive-renderer-perf-v2 JSON")
    );
}

#[test]
fn r4_gate_captures_a_above_the_final_ratio_when_b_passes() {
    let directory = TempDir::new("slow-a-fast-enough-b");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 300);
    let b = static_runner(&directory.path, "b.sh", 150);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_renderer_ratio: "2",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
    });

    assert!(
        output.status.success(),
        "stderr={} metadata={}",
        String::from_utf8_lossy(&output.stderr),
        metadata(&directory.path)
    );
    let comparison =
        std::fs::read_to_string(directory.path.join("artifacts/comparison.json")).unwrap();
    let comparison: serde_json::Value = serde_json::from_str(&comparison).unwrap();
    assert_eq!(comparison["schema"], "rive-r4-timing-comparison-v3");
    assert_eq!(comparison["worst_b_scene"]["candidate_over_cpp"], 1.5);
    assert_eq!(comparison["overall_pass"], true);
}

#[test]
fn r4_gate_rejects_b_above_the_final_renderer_ratio() {
    let directory = TempDir::new("slow-a-too-slow-b");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 300);
    let b = static_runner(&directory.path, "b.sh", 250);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_renderer_ratio: "2",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(
        metadata(&directory.path)
            .contains("post-tail\\ B\\ worst-scene\\ renderer/C++\\ timing\\ failed")
    );
    let comparison =
        std::fs::read_to_string(directory.path.join("artifacts/comparison.json")).unwrap();
    let comparison: serde_json::Value = serde_json::from_str(&comparison).unwrap();
    assert_eq!(comparison["overall_pass"], false);
    assert_eq!(
        comparison["checks"]["post_tail_worst_scene"]["passed"],
        false
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
        max_renderer_ratio: "10",
        max_b_over_a: "1.05",
        max_control_drift: "1",
        max_repeat_drift: "1",
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
        max_renderer_ratio: "10",
        max_b_over_a: "1",
        max_control_drift: "1.1",
        max_repeat_drift: "2",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(metadata(&directory.path).contains("C++\\ control\\ drift\\ failed"));
}

#[test]
fn r4_gate_rejects_candidate_repeat_drift() {
    let directory = TempDir::new("candidate-repeat-drift");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = drifting_candidate_runner(&directory.path);
    let b = static_runner(&directory.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_renderer_ratio: "10",
        max_b_over_a: "2",
        max_control_drift: "1",
        max_repeat_drift: "1.1",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(
        metadata(&directory.path).contains("normalized\\ A\\ repeat\\ drift\\ failed"),
        "{}",
        metadata(&directory.path)
    );
}

#[test]
fn r4_gate_never_overlaps_host_sampling_with_runner_work() {
    let directory = TempDir::new("host-sampling-serialization");
    let runner_active = directory.path.join("runner-active");
    let sampler_active = directory.path.join("sampler-active");
    let overlap = directory.path.join("overlap-observed");
    let baseline = coordinated_runner(
        &directory.path,
        "baseline.sh",
        &runner_active,
        &sampler_active,
        &overlap,
    );
    let a = coordinated_runner(
        &directory.path,
        "a.sh",
        &runner_active,
        &sampler_active,
        &overlap,
    );
    let b = coordinated_runner(
        &directory.path,
        "b.sh",
        &runner_active,
        &sampler_active,
        &overlap,
    );
    let sampler =
        coordinated_host_sampler(&directory.path, &runner_active, &sampler_active, &overlap);
    let output = run_gate_with_sampler(
        Gate {
            directory: &directory.path,
            baseline: &baseline,
            a: &a,
            b: &b,
            max_renderer_ratio: "10",
            max_b_over_a: "1",
            max_control_drift: "1",
            max_repeat_drift: "1",
        },
        &sampler,
    );

    assert!(
        output.status.success(),
        "stderr={} metadata={}",
        String::from_utf8_lossy(&output.stderr),
        metadata(&directory.path)
    );
    assert!(!overlap.exists(), "host sampling overlapped timed work");
}

#[test]
fn r4_gate_rejects_malformed_report_json() {
    let directory = TempDir::new("malformed");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let (malformed, pid_file) = malformed_runner(&directory.path);
    let b = static_runner(&directory.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &directory.path,
        baseline: &baseline,
        a: &malformed,
        b: &b,
        max_renderer_ratio: "10",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
    });

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    assert!(metadata(&directory.path).contains("failure_phase=run-01-A"));
    let pid = std::fs::read_to_string(pid_file)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    assert!(
        !process_is_alive(pid),
        "malformed runner {pid} was not reaped"
    );
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
        max_renderer_ratio: "10",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
    });
    assert!(!output.status.success());
    assert_finalized_failure(&identity.path);
    assert!(
        metadata(&identity.path)
            .contains("baseline\\,\\ A\\,\\ and\\ B\\ runners\\ must\\ be\\ pairwise\\ distinct")
    );

    let baseline_alias = TempDir::new("baseline-alias");
    let baseline = static_runner(&baseline_alias.path, "baseline.sh", 100);
    let b = static_runner(&baseline_alias.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &baseline_alias.path,
        baseline: &baseline,
        a: &baseline,
        b: &b,
        max_renderer_ratio: "10",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
    });
    assert!(!output.status.success());
    assert_finalized_failure(&baseline_alias.path);
    assert!(
        metadata(&baseline_alias.path)
            .contains("baseline\\,\\ A\\,\\ and\\ B\\ runners\\ must\\ be\\ pairwise\\ distinct")
    );

    let mutation = TempDir::new("mutation");
    let baseline = static_runner(&mutation.path, "baseline.sh", 100);
    let a = mutating_runner(&mutation.path);
    let b = static_runner(&mutation.path, "b.sh", 100);
    let output = run_gate(Gate {
        directory: &mutation.path,
        baseline: &baseline,
        a: &a,
        b: &b,
        max_renderer_ratio: "10",
        max_b_over_a: "1",
        max_control_drift: "1",
        max_repeat_drift: "1",
    });
    assert!(!output.status.success());
    assert_finalized_failure(&mutation.path);
    assert!(
        metadata(&mutation.path).contains("A\\ runner\\ changed\\ during\\ the\\ gate"),
        "{}",
        metadata(&mutation.path)
    );
}

#[test]
fn r4_gate_accepts_stable_low_absolute_idle() {
    let directory = TempDir::new("stable-low-idle");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 100);
    let b = static_runner(&directory.path, "b.sh", 100);
    let sampler = fixed_host_sampler(&directory.path, "low-idle.sh", 5);
    let output = run_gate_with_sampler(
        Gate {
            directory: &directory.path,
            baseline: &baseline,
            a: &a,
            b: &b,
            max_renderer_ratio: "10",
            max_b_over_a: "1",
            max_control_drift: "1",
            max_repeat_drift: "1",
        },
        &sampler,
    );

    assert!(
        output.status.success(),
        "stderr={} metadata={}",
        String::from_utf8_lossy(&output.stderr),
        metadata(&directory.path)
    );
    let decision = decision(&directory.path);
    assert_eq!(decision["status"], "pass");
    assert_eq!(decision["comparison_available"], true);
    assert_eq!(decision["idle_spread_percent"], 0.0);
    let samples = std::fs::read_to_string(directory.path.join("artifacts/host-idle.tsv")).unwrap();
    assert_eq!(samples.lines().count(), 9, "{samples}");
    assert!(samples.lines().skip(1).all(|line| line.contains("\t5\t")));
}

#[test]
fn r4_gate_rejects_load_spread_before_writing_a_comparison() {
    let directory = TempDir::new("idle-spread");
    let baseline = static_runner(&directory.path, "baseline.sh", 100);
    let a = static_runner(&directory.path, "a.sh", 100);
    let b = static_runner(&directory.path, "b.sh", 100);
    let sampler = alternating_host_sampler(&directory.path);
    let output = run_gate_with_sampler(
        Gate {
            directory: &directory.path,
            baseline: &baseline,
            a: &a,
            b: &b,
            max_renderer_ratio: "10",
            max_b_over_a: "1",
            max_control_drift: "1",
            max_repeat_drift: "1",
        },
        &sampler,
    );

    assert!(!output.status.success());
    assert_finalized_failure(&directory.path);
    let decision = decision(&directory.path);
    assert_eq!(decision["phase"], "validate-host-load");
    assert_eq!(decision["comparison_available"], false);
    assert_eq!(decision["idle_spread_percent"], 8.0);
    assert!(!directory.path.join("artifacts/comparison.json").exists());
}
