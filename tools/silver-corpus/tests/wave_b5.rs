//! Exact live SRIV replays for the Silver cases in Wave B5.

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

fn replay(id: &str, test_name: &str) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave B5 corpus case");
    assert_eq!(
        case.provenance_file,
        "tests/unit_tests/runtime/hittest_test.cpp"
    );
    assert_eq!(case.provenance_test, test_name);
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
fn wave_b5_shape_clipped_by_parent_layout() {
    replay("hittest_ab1", "Shape clipped by parent layout");
}

#[test]
fn wave_b5_shape_clipped_by_parent_artboard() {
    replay("hittest_ab1_parent", "Shape clipped by parent artboard");
}

#[test]
fn wave_b5_shape_clipped_by_parent_and_grand_parent_artboard() {
    replay(
        "hittest_ab1_grand_parent",
        "Shape clipped by parent and grand-parent artboard",
    );
}

#[test]
fn wave_b5_artboard_list_component_with_scrolling_behavior() {
    replay(
        "hittest_ab_2_non_virtualized",
        "Artboard list component with scrolling behavior",
    );
}

#[test]
fn wave_b5_artboard_list_component_with_scrolling_behavior_virtualized_and_carousel() {
    replay(
        "hittest_ab_2_virtualized",
        "Artboard list component with scrolling behavior virtualized and carousel",
    );
}

#[test]
fn wave_b5_hit_testing_text_in_multiple_layouts_rotated_and_scaled() {
    replay(
        "hittest_ab_text_parent",
        "Hit testing text in multiple layouts rotated and scaled",
    );
}

#[test]
fn wave_b5_hit_testing_shapes_in_layouts() {
    replay("hittest_ab_shape_parent", "Hit testing shapes in layouts");
}

#[test]
fn wave_b5_hit_testing_objects_inside_shapes() {
    replay("hittest_nested", "Hit testing objects inside shapes");
}

#[test]
fn wave_b5_pointer_exit_works_correctly() {
    replay("pointer_exit", "Pointer exit works correctly");
}

#[test]
fn wave_b5_hit_testing_multi_touch_events() {
    replay("multitouch", "Hit testing multi touch events");
}

#[test]
fn wave_b5_multitouch_with_nested_artboard_and_pointer_exit_event() {
    replay(
        "multitouch_enter",
        "Multitouch with nested artboard and pointer exit event",
    );
}

#[test]
fn wave_b5_multitouch_with_list_and_pointer_exit_event() {
    replay(
        "multitouch_enter-MainList",
        "Multitouch with list and pointer exit event",
    );
}

#[test]
fn wave_b5_multitouch_with_multi_scroll() {
    replay(
        "multitouch_enter-MultiScroll",
        "Multitouch with multi scroll",
    );
}

#[test]
fn wave_b5_hit_test_leaves_in_collapsed_layouts() {
    replay(
        "hittest_collapsed_layouts",
        "Hit test leaves in collapsed layouts",
    );
}
