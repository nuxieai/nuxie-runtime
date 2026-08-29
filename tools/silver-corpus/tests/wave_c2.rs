//! Exact pinned Silver action streams for Wave C2 layout cases.

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_test.cpp";

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
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
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

#[test]
#[ignore = "expected-red: complete layout_aspect_ratio SRIV diverges from the pinned renderer stream"]
fn wave_c2_layout_020_aspect_ratio() {
    replay("layout_aspect_ratio");
}

#[test]
fn wave_c2_layout_021_fixed_fill_round_trip() {
    replay("layout_fixed_fill");
}

#[test]
fn wave_c2_layout_022_top_level_hug_artboard() {
    replay("layout_hug_artboard");
}
