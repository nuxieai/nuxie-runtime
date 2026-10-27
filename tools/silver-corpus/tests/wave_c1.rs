//! Exact pinned grid/stack Silver action streams for Wave C1.

use silver_corpus::{
    Action, Execution, Status, compare_sriv, parse_sriv, read_manifest, resolve_expected,
};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp";

fn workspace_root() -> PathBuf {
    Path::new(option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")))
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

fn replay_result(id: &str) -> (Status, Result<(), silver_corpus::Difference>) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let manifest_case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave C1 corpus case");
    assert_eq!(manifest_case.provenance_file, PROVENANCE);

    assert_eq!(
        manifest_case.actions.executable().map(<[Action]>::len),
        Some(362),
        "the manifest must retain the complete 120-frame helper stream",
    );
    let case = manifest_case;

    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    (case.status, compare_sriv(&expected, &actual))
}

fn replay(id: &str) {
    let (status, result) = replay_result(id);
    assert_eq!(status, Status::Exact, "{id} must remain classified exact");
    result.unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
fn wave_c1_layout_grid_stack_001_grid_with_layouts() {
    replay("layout_grid_stack_grid_with_layouts");
}

#[test]
fn wave_c1_layout_grid_stack_002_stack_with_layouts() {
    replay("layout_grid_stack_stack_with_layouts");
}

#[test]
fn wave_c1_layout_grid_stack_003_grid_with_layouts_size_changing() {
    replay("layout_grid_stack_grid_with_layouts_size_changing");
}

#[test]
fn wave_c1_layout_grid_stack_004_grid_with_layouts_span() {
    replay("layout_grid_stack_grid_with_layouts_span");
}

#[test]
fn wave_c1_layout_grid_stack_005_grid_with_layouts_size_span_changing() {
    replay("layout_grid_stack_grid_with_layouts_size_span_changing");
}

#[test]
#[ignore = "intentional CSS automatic minima: cell19 height and participant29 width; see README.md geometry audit"]
fn wave_c1_layout_grid_stack_006_grid_with_layout_participants() {
    // Keep the original full-stream equality assertion available explicitly.
    // The shared runner executes this as a classified CSS divergence.
    let (status, result) = replay_result("layout_grid_stack_grid_with_layout_participants");
    assert_eq!(status, Status::Diverges);
    result.unwrap_or_else(|difference| panic!("grid participants: {difference}"));
}
