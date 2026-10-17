//! Exact pinned Silver action streams for Wave C2 layout cases.

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_test.cpp";

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

fn replay(id: &str) {
    replay_result(id).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

fn replay_result(id: &str) -> Result<(), silver_corpus::Difference> {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read Silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .unwrap_or_else(|| panic!("missing Silver case {id}"));
    assert_eq!(case.provenance_file, PROVENANCE);
    assert!(
        case.actions
            .executable()
            .is_some_and(|actions| !actions.is_empty()),
        "the shared manifest retains the complete pinned action stream"
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual)
}

#[test]
fn wave_c2_layout_014_collapsing_and_soloing() {
    replay("collapsing_elements");
}

#[test]
fn wave_c2_layout_015_animating_layout_display() {
    replay("layout_display");
}

#[test]
fn wave_c2_layout_016_background_and_foreground_paints() {
    replay("layout_paint");
}

#[test]
fn wave_c2_layout_017_animation_time_databound() {
    replay("layout_anim_bound");
}

#[test]
fn wave_c2_layout_018_animation_nested_artboards() {
    replay("layout_anim_nested");
}

#[test]
fn wave_c2_layout_019_animation_component_list() {
    replay("layout_anim_component_list");
}

// Layout divergence (stock Taffy, CSS semantics): the aspect-ratio box solves
// to 142x133 where Yoga keeps the authored 2:1 ratio (the Taffy fork already
// did); the first difference is pinned so anything that moves it still fails.
#[test]
fn wave_c2_layout_020_aspect_ratio() {
    let difference =
        replay_result("layout_aspect_ratio").expect_err("layout divergence should remain present");
    assert_eq!(
        difference.to_string(),
        "frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133)"
    );
}

#[test]
fn wave_c2_layout_021_fixed_fill_round_trip() {
    replay("layout_fixed_fill");
}

#[test]
fn wave_c2_layout_022_top_level_hug_artboard() {
    replay("layout_hug_artboard");
}
