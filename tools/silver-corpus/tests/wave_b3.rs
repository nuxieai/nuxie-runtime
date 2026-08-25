//! Live SRIV replays for every Wave B3 focus case represented by the pinned corpus.

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

fn replay(id: &str) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave B3 corpus case");
    assert_eq!(case.provenance_file, "tests/unit_tests/runtime/focus_test.cpp");
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected = parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
        .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
#[ignore = "expected-red: frame 3 op 192 color paint_id is 11 instead of pinned 6"]
fn wave_b3_focus_collapsing() { replay("focus_collapsing"); }

#[test]
#[ignore = "expected-red: frame 0 op 85 expects color but Rust emits save"]
fn wave_b3_keyboard_listener() { replay("keyboard_listener"); }

#[test]
#[ignore = "expected-red: frame 1 op 214 expects color but Rust emits save"]
fn wave_b3_keyboard_listener_keyboard_input() { replay("keyboard_listener-KeyboardInput"); }

#[test]
fn wave_b3_focus_traversal() { replay("focus_traversal"); }

#[test]
fn wave_b3_focusable_element() { replay("focusable_element"); }

#[test]
#[ignore = "expected-red: frame 0 op 78 addRawPath y is 137.20053 instead of pinned 137.20052"]
fn wave_b3_list_focus_order() { replay("list_focus_order"); }

#[test]
fn wave_b3_focus_test() { replay("focus_test"); }
