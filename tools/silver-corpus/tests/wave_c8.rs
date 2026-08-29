//! Exact pinned Silver replays for executable Wave C8 rendering cases.

use nuxie_runtime::source::math::random::RandomProvider;
use silver_corpus::{
    Difference, Execution, Status, compare_sriv, parse_sriv, read_manifest, resolve_expected,
};
use std::path::{Path, PathBuf};

// Pinned RandomProvider is process-global. Serialize this binary's fixture
// replays so a TESTING FIFO cannot affect another concurrently running case.
static REPLAY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct TestingRandom;
impl Drop for TestingRandom {
    fn drop(&mut self) {
        RandomProvider::clear_testing_mode();
    }
}

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
    let _lock = REPLAY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_, result) = replay_locked(id);
    result.unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

fn replay_exact_locked(id: &str) {
    let (status, result) = replay_locked(id);
    assert_eq!(status, Status::Exact, "{id} should be classified exact");
    result.unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

fn replay_divergence(id: &str, expected: &str) {
    let _lock = REPLAY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (status, result) = replay_locked(id);
    assert_eq!(
        status,
        Status::Diverges,
        "{id} should be classified diverges"
    );
    let difference = result.expect_err("signed divergence should remain present");
    assert_eq!(difference.to_string(), expected);
}

fn replay_locked(id: &str) -> (Status, Result<(), Difference>) {
    let runtime = runtime_root();
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))
        .expect("read Silver corpus manifest");
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .expect("Wave C8 corpus case");
    assert_eq!(
        case.provenance_file,
        "tests/unit_tests/runtime/serialized_rendering_test.cpp"
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    (case.status, compare_sriv(&expected, &actual))
}

#[test]
fn wave_c8_render_002_hide_silver() {
    replay("hide_test");
}

#[test]
fn wave_c8_render_003_n_slice_silver() {
    replay("n_slice_triangle");
}

#[test]
fn wave_c8_render_004_lock_icon_listener_silver() {
    replay("lock_icon_demo");
}

#[test]
fn wave_c8_render_005_validate_text_run_listener_works() {
    replay("text_listener_simpler");
}

#[test]
fn wave_c8_render_006_validate_text_with_modifiers_and_dashes() {
    replay("text_stroke_test");
}

#[test]
fn wave_c8_render_007_superbowl_data_binding() {
    replay("superbowl");
}

#[test]
fn wave_c8_render_009_bank_card_data_binding() {
    replay("bankcard");
}

#[test]
fn wave_c8_render_010_ai_assistant_data_binding() {
    replay("ai_assitant");
}

#[test]
fn wave_c8_render_012_rewards_demo_data_binding() {
    replay_divergence(
        "rewards_demo",
        "frame 0, op 1461 (addRawPath): expected 44 fields, got 46",
    );
}

#[test]
fn wave_c8_render_013_spotify_kids_demo_data_binding() {
    replay("spotify_kids_demo");
}

#[test]
fn wave_c8_render_014_spotify_kids_app_icon_data_binding() {
    replay("spotify_kids_app_icon");
}

#[test]
fn wave_c8_render_015_hunter_x_demo_data_binding() {
    replay_divergence(
        "hunter_x_demo",
        "frame 0, op 5055 (addRawPath): expected 20 fields, got 22",
    );
}

#[test]
fn wave_c8_render_017_car_widgets_data_binding() {
    replay_divergence(
        "car_widgets_v01",
        "frame 0, op 10306 (addRawPath): expected 60 fields, got 56",
    );
}

#[test]
fn wave_c8_render_018_vertical_align_ellipsis() {
    replay("vertical_align_ellipsis");
}

#[test]
fn wave_c8_render_019_event_triggers_another_event() {
    replay("event_trigger_event");
}

#[test]
fn wave_c8_render_020_collapsed_data_binds_hidden_layout() {
    replay_divergence(
        "collapse_data_binds-test_1",
        "frame 0, op 100 (transform), field tx: expected 411.31592, got 410.13672",
    );
}

#[test]
fn wave_c8_render_021_collapsed_data_binds_property_group_solo() {
    replay("collapse_data_binds-test_2");
}

#[test]
fn wave_c8_render_022_collapsed_data_bound_layout_styles_update() {
    replay("collapse_data_binds-test_3");
}

#[test]
fn wave_c8_render_026_target_to_source_different_types() {
    replay("saturation");
}

#[test]
fn wave_c8_render_027_interactive_and_non_interactive_scrolling() {
    replay("interactive_scrolling");
}

#[test]
fn wave_c8_render_032_pointer_drag_event() {
    replay("drag_event");
}

#[test]
fn wave_c8_render_033_recursive_data_binding_artboards_skipped() {
    replay("recursive_data_bind");
}

#[test]
fn wave_c8_render_034_collapsable_data_binds_added_when_uncollapsed() {
    replay("collapsable_data_binding");
}

#[test]
fn wave_c8_render_036_advance_blend_modes_inputs() {
    let _lock = REPLAY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    RandomProvider::clear_randoms();
    let _random_values = TestingRandom;
    assert_eq!(RandomProvider::total_calls(), 0);
    replay_exact_locked("advance_blend_mode-inputs");
}

#[test]
fn wave_c8_render_037_advance_blend_modes_view_model() {
    let _lock = REPLAY_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    RandomProvider::clear_randoms();
    let _random_values = TestingRandom;
    assert_eq!(RandomProvider::total_calls(), 0);
    replay_exact_locked("advance_blend_mode-vms");
}

#[test]
fn wave_c8_render_038_transition_conditions_based_on_artboards() {
    replay("transition_artboard_condition_test");
}
