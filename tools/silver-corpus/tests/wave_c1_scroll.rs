//! Exact pinned Silver replays from `layout_scroll_test.cpp` Wave C1 cases.

use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

const PROVENANCE: &str = "tests/unit_tests/runtime/layout_scroll_test.cpp";

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

fn replay(id: &str, expected_actions: usize) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave C1 layout-scroll case");
    assert_eq!(case.provenance_file, PROVENANCE);
    assert_eq!(
        case.actions
            .executable()
            .map(<[silver_corpus::Action]>::len),
        Some(expected_actions),
        "complete pinned helper/action stream"
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
#[ignore = "expected-red: exact carousel swipe diverges at frame 1/op145 (expected rewind, got drawPath)"]
fn carousel_snap_swipe_right_settles_past_index_zero() {
    replay("layout_scroll_snap_carousel", 23);
}

#[test]
#[ignore = "expected-red: exact layout snap-padding diverges at frame 0/op38 (expected makeRenderPaint, got frameSize)"]
fn scroll_snap_respects_viewport_padding_layouts() {
    replay("layout_scroll_snap_padding_layouts", 23);
}

#[test]
#[ignore = "expected-red: exact list snap-padding diverges at frame 0/op24 (expected makeRenderPaint, got frameSize)"]
fn scroll_snap_respects_viewport_padding_list() {
    replay("layout_scroll_snap_padding_list", 23);
}

#[test]
#[ignore = "expected-red: exact virtualized snap-padding diverges at frame 0/op24 (expected makeRenderPaint, got frameSize)"]
fn scroll_snap_respects_viewport_padding_virtualized_list() {
    replay("layout_scroll_snap_padding_virtualized", 23);
}

#[test]
#[ignore = "expected-red: exact layout drag-multiplier diverges at frame 0/op38 (expected makeRenderPaint, got frameSize)"]
fn scroll_drag_multiplier_layouts() {
    replay("layout_scroll_drag_multiplier_layouts", 39);
}

#[test]
#[ignore = "expected-red: exact list drag-multiplier diverges at frame 0/op24 (expected makeRenderPaint, got frameSize)"]
fn scroll_drag_multiplier_list() {
    replay("layout_scroll_drag_multiplier_list", 39);
}

#[test]
#[ignore = "expected-red: exact virtualized drag-multiplier diverges at frame 0/op24 (expected makeRenderPaint, got frameSize)"]
fn scroll_drag_multiplier_virtualized_list() {
    replay("layout_scroll_drag_multiplier_virtualized", 39);
}

#[test]
#[ignore = "expected-red: exact hidden-item stream diverges at frame 0/op130 (expected negative-zero xy, got positive-zero)"]
fn scroll_constraint_scroll_index_with_hidden_items() {
    replay("layout_scroll_visibility", 910);
}

#[test]
#[ignore = "expected-red: exact index-intent stream diverges at frame 0/op69 (expected negative-zero xy, got positive-zero)"]
fn scroll_constraint_index_intent_across_hidden_layout() {
    replay("scroll_intent", 114);
}
