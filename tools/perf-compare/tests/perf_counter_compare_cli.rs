#![cfg(unix)]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write_runner(directory: &Path, name: &str, render_passes: u64) -> PathBuf {
    let path = directory.join(name);
    let python = format!(
        r#"import json, sys
r = json.loads(sys.stdin.readline())
r.update({{
    "selected_adapter": {{
        "backend": "metal",
        "name": "Integration GPU",
        "vendor": "Integration Vendor",
        "device": "Integration Device",
        "driver": "1.0"
    }},
    "measured_frame_median_ns": 10,
    "logical_flushes": 3,
    "draws": 11,
    "atomic_strategy_partitions": 2,
    "backend_work": {{
        "command_encoders": 1,
        "render_passes": {render_passes},
        "bind_groups_created": 0,
        "bind_group_sets": 0,
        "texture_bindings": 0,
        "buffer_clear_calls": 0,
        "buffer_clear_bytes": 0,
        "buffer_upload_calls": 0,
        "buffer_upload_bytes": 0,
        "texture_upload_calls": 0,
        "texture_upload_bytes": 0,
        "queue_submissions": 1,
        "gpu_draw_calls": 0,
        "gpu_draw_instances": 0,
        "tessellation_spans": 0,
        "path_patches": 0
    }}
}})
print(json.dumps(r))
"#
    );
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nset -eu\ntest \"$1\" = \"--renderer-perf-protocol\"\ntest \"$2\" = \"rive-renderer-perf-runner-v1\"\npython3 -c '{}'\n",
            python.replace('\'', "'\\''")
        ),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

fn sha256(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn cli_preserves_reports_before_counter_excess_failure() {
    let unique = format!(
        "rive-renderer-counter-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let directory = std::env::temp_dir().join(unique);
    std::fs::create_dir(&directory).unwrap();
    let baseline = write_runner(&directory, "baseline.sh", 1);
    let candidate = write_runner(&directory, "candidate.sh", 2);
    let json = directory.join("report.json");
    let markdown = directory.join("report.md");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("renderer-scenes.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_perf-counter-compare"))
        .args([
            "--manifest",
            manifest.to_str().unwrap(),
            "--baseline-runner",
            baseline.to_str().unwrap(),
            "--candidate-runner",
            candidate.to_str().unwrap(),
            "--baseline-source-id",
            "git:baseline-abc123",
            "--candidate-source-id",
            "git:candidate-def456",
            "--json",
            json.to_str().unwrap(),
            "--markdown",
            markdown.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("counter parity failed"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert_eq!(report["schema"], "rive-renderer-perf-counters-v2");
    assert_eq!(report["provenance"]["manifest_sha256"], sha256(&manifest));
    assert_eq!(
        report["provenance"]["baseline_runner_sha256"],
        sha256(&baseline)
    );
    assert_eq!(
        report["provenance"]["candidate_runner_sha256"],
        sha256(&candidate)
    );
    assert_eq!(
        report["provenance"]["generator_sha256"],
        sha256(Path::new(env!("CARGO_BIN_EXE_perf-counter-compare")))
    );
    assert_eq!(
        report["provenance"]["baseline_source_id"],
        "git:baseline-abc123"
    );
    assert_eq!(
        report["provenance"]["candidate_source_id"],
        "git:candidate-def456"
    );
    assert_eq!(report["ranked_excesses"].as_array().unwrap().len(), 16);
    assert_eq!(report["ranked_excesses"][0]["counter"], "render_passes");

    let markdown = std::fs::read_to_string(markdown).unwrap();
    assert!(markdown.contains("Baseline source identity: `git:baseline-abc123`"));
    assert!(markdown.contains("Candidate source identity: `git:candidate-def456`"));
    assert!(
        markdown.contains(
            "| 1 | gm-CubicStroke-clockwise-atomic | render_passes | 1 | 2 | 1 | 2.000 |"
        )
    );
    std::fs::remove_dir_all(directory).unwrap();
}
