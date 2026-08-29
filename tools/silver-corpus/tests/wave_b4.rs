//! Exact pinned Silver replays for Wave B4 runtime test cases.

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

fn replay(id: &str, provenance: &str) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave B4 corpus case");
    assert_eq!(case.provenance_file, provenance);
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

fn follow_path(id: &str) {
    replay(
        id,
        "tests/unit_tests/runtime/follow_path_constraint_test.cpp",
    );
}

#[test]
fn wave_b4_follow_path_animate_shape() {
    follow_path("follow_path_animate_shape");
}

#[test]
fn wave_b4_follow_path_animate_solo() {
    follow_path("follow_path_animate_solo");
}

#[test]
fn wave_b4_follow_path_animate_target() {
    follow_path("follow_path_animate_target");
}

#[test]
fn wave_b4_text_follow_path_shape_length() {
    follow_path("text_follow_path_shape_length");
}

#[test]
fn wave_b4_follow_path_constraint() {
    follow_path("follow_path_constraint");
}

#[test]
fn wave_b4_gamepad_test() {
    replay("gamepad_test", "tests/unit_tests/runtime/gamepad_test.cpp");
}

#[test]
fn wave_b4_global_variables_test() {
    replay(
        "global_variables_test",
        "tests/unit_tests/runtime/global_viewmodels_test.cpp",
    );
}

#[test]
fn wave_b4_global_viewmodels_auto_instance() {
    replay(
        "global_viewmodels_test-auto_instance",
        "tests/unit_tests/runtime/global_viewmodels_test.cpp",
    );
}

#[test]
fn wave_b4_global_viewmodels_set_instance() {
    replay(
        "global_viewmodels_test-set_instance",
        "tests/unit_tests/runtime/global_viewmodels_test.cpp",
    );
}
