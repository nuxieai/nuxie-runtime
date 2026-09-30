//! Null-factory regression from upstream 57628249 runtime/file_test.cpp.
use nuxie_runtime::{File, ImportResult};

#[test]
fn importing_with_a_null_factory_fails_instead_of_crashing() {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/juice.riv")).unwrap();
    let mut result = ImportResult::Success;
    let file = File::import(&bytes, None, Some(&mut result), None, None);
    assert!(file.is_none());
    assert_eq!(result, ImportResult::Malformed);
}
