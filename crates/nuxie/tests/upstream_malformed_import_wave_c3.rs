//! Exact safe-Rust port of pinned `malformed_file_import_test.cpp`.

use nuxie::File;
use std::path::PathBuf;

fn fixture() -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let path = root.join("tests/unit_tests/assets/data_binding_test_2.riv");
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn wave_c3_malformed_import_001_truncated_file_never_crashes() {
    let bytes = fixture();
    let mut full_file = None;

    // Rust's `Result<File>` makes the upstream result/file nullability
    // invariant unrepresentable: an error cannot retain a partially-built
    // File. Still execute every exact prefix so each failed import drops all
    // partially-owned state before the next iteration.
    for length in 0..=bytes.len() {
        match File::import(&bytes[..length]) {
            Ok(file) if length == bytes.len() => full_file = Some(file),
            Ok(file) => drop(file),
            Err(_) => {}
        }
    }
}

#[test]
fn wave_c3_malformed_import_002_full_file_still_imports() {
    let file = File::import(&fixture()).expect("full pinned file imports after the guards");
    assert!(file.default_artboard().is_some());
}
