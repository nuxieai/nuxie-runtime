//! Exact safe-Rust port of pinned `malformed_file_import_test.cpp`.

use nuxie::{File, ImportResult, PersistentFactory, RuntimeFactoryHandle};
use nuxie_render_api::RecordingFactory;
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
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");

    for length in 0..=bytes.len() {
        let mut result = ImportResult::Success;
        let file = File::import(
            &bytes[..length],
            factory.clone(),
            Some(&mut result),
            None,
            None,
        );
        assert_eq!(
            file.is_some(),
            result == ImportResult::Success,
            "prefix length {length}: success and file nullability agree"
        );
    }
}

#[test]
fn wave_c3_malformed_import_002_full_file_still_imports() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&fixture(), factory, Some(&mut result), None, None);
    assert_eq!(result, ImportResult::Success);
    let file = file.expect("full pinned file imports after the guards");
    assert!(file.with_file(File::artboard).is_some());
}
