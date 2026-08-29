//! Preserved host image-admission and C++ oracle-fingerprint contracts.
//!
//! These bounded codec assertions intentionally differ from the policy-free
//! Bitmap tests in upstream_wave_b5: the approved host policy rejects oversized
//! malformed fixtures before pixel allocation.

use nuxie_image_codec::{decoded_rgba_len, preflight_encoded_image, validate_encoded_image};
use nuxie_render_api::Mat2D;
use std::process::Command;

type CppProbeFile = serde_json::Value;
#[allow(dead_code)]
mod cpp_probe_support;
use cpp_probe_support::*;

/// Input list and hash construction must stay in lockstep with the
/// fingerprint block in tools/cpp-probe/build.sh.

#[cfg(unix)]
#[test]
fn probe_fingerprint_guard_flags_stale_binaries() {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!("nuxie-probe-guard-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create guard fixture dir");
    let write_fake_probe = |name: &str, script: &str| {
        let path = dir.join(name);
        std::fs::write(&path, script).expect("write fake probe");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("mark fake probe executable");
        path
    };

    // An out-of-date binary rejects --fingerprint as an unrecognized argument.
    let unrecognized = write_fake_probe(
        "probe-unrecognized-argument.sh",
        "#!/bin/sh\necho 'unrecognized argument --fingerprint' >&2\nexit 2\n",
    );
    assert_eq!(
        probe_staleness_error(&unrecognized, "make cpp-probe").as_deref(),
        Some("cpp-probe binary is stale — run make cpp-probe")
    );

    // A binary built from different sources reports a mismatched fingerprint.
    let mismatched = write_fake_probe(
        "probe-mismatched-fingerprint.sh",
        "#!/bin/sh\necho 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n",
    );
    assert_eq!(
        probe_staleness_error(&mismatched, "make cpp-probe-scripted").as_deref(),
        Some("cpp-probe binary is stale — run make cpp-probe-scripted")
    );

    // A binary reporting the current source fingerprint passes.
    let fresh = write_fake_probe(
        "probe-fresh.sh",
        &format!("#!/bin/sh\necho {}\n", expected_probe_fingerprint()),
    );
    assert_eq!(probe_staleness_error(&fresh, "make cpp-probe"), None);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn built_probe_reports_current_source_fingerprint() {
    let probe = probe_path().expect("fingerprinted C++ oracle required; run make cpp-probe");

    let output = Command::new(&probe)
        .arg("--fingerprint")
        .output()
        .expect("run cpp-probe --fingerprint");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        expected_probe_fingerprint()
    );
}

fn pinned_decoder_fixture(relative: &str, expected_len: usize) -> Vec<u8> {
    let bytes = std::fs::read(cpp_runtime_fixture(relative))
        .unwrap_or_else(|error| panic!("read pinned image decoder fixture {relative}: {error}"));
    assert_eq!(
        bytes.len(),
        expected_len,
        "pinned image decoder fixture length"
    );
    bytes
}

#[test]
fn upstream_png_decoder_fixture_contract() {
    let bytes = pinned_decoder_fixture("placeholder.png", 1_096);
    let decoded = validate_encoded_image(&bytes).expect("placeholder PNG fully decodes");
    assert_eq!((decoded.width, decoded.height), (226, 128));
    assert_eq!(
        decoded_rgba_len(decoded.width, decoded.height),
        Some(226 * 128 * 4)
    );
}

#[test]
fn upstream_jpeg_decoder_fixture_contract() {
    let bytes = pinned_decoder_fixture("open_source.jpg", 8_880);
    let decoded = validate_encoded_image(&bytes).expect("open-source JPEG fully decodes");
    assert_eq!((decoded.width, decoded.height), (350, 200));
    assert_eq!(
        decoded_rgba_len(decoded.width, decoded.height),
        Some(350 * 200 * 4)
    );
}

#[test]
fn upstream_bad_jpeg_fixture_is_rejected_before_oversized_allocation() {
    let bytes = pinned_decoder_fixture("bad.jpg", 88_731);
    assert_eq!(preflight_encoded_image(&bytes), None);
    assert_eq!(validate_encoded_image(&bytes), None);
}

#[test]
fn upstream_bad_png_fixture_is_rejected_before_oversized_allocation() {
    let bytes = pinned_decoder_fixture("bad.png", 534_283);
    assert_eq!(preflight_encoded_image(&bytes), None);
    assert_eq!(validate_encoded_image(&bytes), None);
}

#[test]
fn upstream_webp_decoder_fixture_contract() {
    let bytes = pinned_decoder_fixture("1.webp", 30_320);
    let decoded = validate_encoded_image(&bytes).expect("WebP fully decodes");
    assert_eq!((decoded.width, decoded.height), (550, 368));
    assert_eq!(
        decoded_rgba_len(decoded.width, decoded.height),
        Some(550 * 368 * 4)
    );
}
