#![cfg(unix)]

use serde_json::Value;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn write_runner(directory: &Path, name: &str, median_ns: u64) -> PathBuf {
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
    "measured_frame_median_ns": {median_ns},
    "logical_flushes": 3,
    "draws": 11
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

#[test]
fn cli_emits_reports_before_a_threshold_failure() {
    let unique = format!(
        "rive-renderer-perf-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let directory = std::env::temp_dir().join(unique);
    std::fs::create_dir(&directory).unwrap();
    let baseline = write_runner(&directory, "baseline.sh", 10);
    let candidate = write_runner(&directory, "candidate.sh", 20);
    let json = directory.join("report.json");
    let markdown = directory.join("report.md");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("renderer-scenes.toml");

    let output = Command::new(env!("CARGO_BIN_EXE_renderer-perf"))
        .args([
            "--manifest",
            manifest.to_str().unwrap(),
            "--baseline-runner",
            baseline.to_str().unwrap(),
            "--candidate-runner",
            candidate.to_str().unwrap(),
            "--max-ratio",
            "1.5",
            "--json",
            json.to_str().unwrap(),
            "--markdown",
            markdown.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("threshold failed"));
    let report: Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert_eq!(report["aggregate"]["ratio"], 2.0);
    assert_eq!(report["scenes"][0]["structural"]["logical_flushes"], 3);
    let markdown = std::fs::read_to_string(markdown).unwrap();
    assert!(markdown.contains("logical flushes"));
    std::fs::remove_dir_all(directory).unwrap();
}
