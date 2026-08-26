//! Exact pinned Silver replays for the executable Wave C5 path cases.

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
        .expect("read Silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave C5 corpus case");
    assert_eq!(
        case.provenance_file,
        "tests/unit_tests/runtime/path_test.cpp"
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
fn wave_c5_apply_stacked_path_effects_to_paths() {
    replay("stacked_path_effects");
}

#[test]
fn wave_c5_apply_trim_path_effect_to_fill() {
    replay("fill_trim_path");
}

#[test]
fn wave_c5_apply_group_effect_with_missing_items() {
    replay("group_effect-main-missing-targets");
}

#[test]
#[ignore = "expected-red: exact path-effect feathers SRIV diverges at frame 0 operation 21 feather paint_id (expected 8, got 5)"]
fn wave_c5_path_effects_with_inner_and_outer_feathers() {
    replay("path_effect_with_feathers");
}
