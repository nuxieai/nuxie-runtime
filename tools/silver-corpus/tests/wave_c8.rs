//! Exact pinned Silver replays for executable Wave C8 rendering cases.

use nuxie_runtime::{runtime_random_call_count, set_runtime_random_test_values};
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
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
#[ignore = "expected-red: juice frame 0, op 40 (blendMode): expected blendMode, got makeRenderPaint"]
fn wave_c8_render_001_juice_silver() {
    replay("juice");
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
#[ignore = "expected-red: superbowl frame 0, op 2825 (color), field paint_id: expected 220, got 208"]
fn wave_c8_render_007_superbowl_data_binding() {
    replay("superbowl");
}

#[test]
#[ignore = "expected-red: bankcard frame 0, op 22 (blendMode): expected blendMode, got makeRenderPaint"]
fn wave_c8_render_009_bank_card_data_binding() {
    replay("bankcard");
}

#[test]
#[ignore = "expected-red: ai_assitant frame 0, op 82 (makeLinearGradient): expected makeLinearGradient, got feather"]
fn wave_c8_render_010_ai_assistant_data_binding() {
    replay("ai_assitant");
}

#[test]
#[ignore = "expected-red: rewards_demo frame 0, op 22 (blendMode): expected blendMode, got makeRenderPaint"]
fn wave_c8_render_012_rewards_demo_data_binding() {
    replay("rewards_demo");
}

#[test]
#[ignore = "expected-red: spotify_kids_demo frame 0, op 200 (blendMode): expected blendMode, got makeRenderPaint"]
fn wave_c8_render_013_spotify_kids_demo_data_binding() {
    replay("spotify_kids_demo");
}

#[test]
fn wave_c8_render_014_spotify_kids_app_icon_data_binding() {
    replay("spotify_kids_app_icon");
}

#[test]
#[ignore = "expected-red: hunter_x_demo frame 0, op 488 (blendMode): expected blendMode, got makeRenderPaint"]
fn wave_c8_render_015_hunter_x_demo_data_binding() {
    replay("hunter_x_demo");
}

#[test]
#[ignore = "expected-red: car_widgets_v01 frame 0, op 222 (blendMode): expected blendMode, got makeRenderPaint"]
fn wave_c8_render_017_car_widgets_data_binding() {
    replay("car_widgets_v01");
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
#[ignore = "expected-red: collapse_data_binds-test_1 frame 10, op 760 (rewind): expected rewind, got drawPath"]
fn wave_c8_render_020_collapsed_data_binds_hidden_layout() {
    replay("collapse_data_binds-test_1");
}

#[test]
#[ignore = "expected-red: collapse_data_binds-test_2 frame 15, op 315 (addRawPath): expected 151 fields, got 256"]
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
#[ignore = "expected-red: interactive_scrolling frame 0, op 42 (transform), field xy: expected -0.0 (0x80000000), got 0"]
fn wave_c8_render_027_interactive_and_non_interactive_scrolling() {
    replay("interactive_scrolling");
}

#[test]
#[ignore = "expected-red: interpolate_to_end frame 1, op 63 (addRawPath): expected 954 fields, got 975"]
fn wave_c8_render_028_interpolator_advance_until_settled() {
    replay("interpolate_to_end");
}

#[test]
#[ignore = "expected-red: drag_event frame 23, op 602 (save): expected save, got color"]
fn wave_c8_render_032_pointer_drag_event() {
    replay("drag_event");
}

#[test]
fn wave_c8_render_033_recursive_data_binding_artboards_skipped() {
    replay("recursive_data_bind");
}

#[test]
#[ignore = "expected-red: collapsable_data_binding frame 0, op 14 (save): expected save, got color"]
fn wave_c8_render_034_collapsable_data_binds_added_when_uncollapsed() {
    replay("collapsable_data_binding");
}

#[test]
#[ignore = "expected-red: virtualize_blendmode frame 0, op 33 (color): expected color, got save"]
fn wave_c8_render_035_virtualized_list_blended_initial_state() {
    let _random_values = set_runtime_random_test_values(&[]);
    assert_eq!(runtime_random_call_count(), 0);
    replay("virtualize_blendmode");
}

#[test]
fn wave_c8_render_036_advance_blend_modes_inputs() {
    let _random_values = set_runtime_random_test_values(&[]);
    assert_eq!(runtime_random_call_count(), 0);
    replay("advance_blend_mode-inputs");
}

#[test]
fn wave_c8_render_037_advance_blend_modes_view_model() {
    let _random_values = set_runtime_random_test_values(&[]);
    assert_eq!(runtime_random_call_count(), 0);
    replay("advance_blend_mode-vms");
}

#[test]
#[ignore = "expected-red: transition_artboard_condition_test frame 0, op 16 (frameSize), field width: expected 983, got 984"]
fn wave_c8_render_038_transition_conditions_based_on_artboards() {
    replay("transition_artboard_condition_test");
}
