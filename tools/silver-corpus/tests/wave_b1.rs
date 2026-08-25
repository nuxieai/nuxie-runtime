//! Live SRIV replays for every Wave B1 case already represented by the pinned corpus.

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
        .expect("Wave B1 corpus case");
    assert!(
        case.provenance_file
            .starts_with("tests/unit_tests/runtime/data_binding")
    );
    let actual = Execution::run(case, &runtime).expect("execute complete pinned action stream");
    let expected =
        parse_sriv(&std::fs::read(resolve_expected(&runtime, case)).expect("read pinned SRIV"))
            .expect("parse pinned SRIV");
    let actual = parse_sriv(actual.bytes()).expect("parse Rust SRIV");
    compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
}

#[test]
fn wave_b1_artboard_width_test() {
    replay("artboard_width_test");
}

#[test]
fn wave_b1_bidirectional_precedence_source_first() {
    replay("bidirectional_precedence-source_first");
}

#[test]
fn wave_b1_bidirectional_precedence_target_first() {
    replay("bidirectional_precedence-target_first");
}

#[test]
fn wave_b1_bidirectional_stateful_property() {
    replay("bidirectional_stateful_property");
}

#[test]
#[ignore = "expected-red: frame 1 op 255 expects rewind but Rust emits drawPath"]
fn wave_b1_computed_root_transform_list() {
    replay("computed_root_transform-list");
}

#[test]
fn wave_b1_computed_root_transform_nested_artboard() {
    replay("computed_root_transform-nested_artboard");
}

#[test]
fn wave_b1_custom_property_enum() {
    replay("custom_property_enum");
}

#[test]
fn wave_b1_custom_property_trigger_bind() {
    replay("custom_property_trigger_bind");
}

#[test]
fn wave_b1_data_bind_font_test() {
    replay("data_bind_font_test");
}

#[test]
#[ignore = "expected-red: frame 4 op 159 expects save but Rust emits restore"]
fn wave_b1_data_bind_keyframes_test() {
    replay("data_bind_keyframes_test");
}

#[test]
#[ignore = "expected-red: frame 0 op 81 addRawPath has 752 pinned fields but Rust emits 669"]
fn wave_b1_data_bind_solo_solos_to_values() {
    replay("data_bind_solo-solos-to-values");
}

#[test]
fn wave_b1_data_bind_solo_values_to_solos() {
    replay("data_bind_solo-values-to-solos");
}

#[test]
#[ignore = "expected-red: frame 1 op 30 expects save but Rust emits color"]
fn wave_b1_data_converter_interpolator_reset() {
    replay("data_converter_interpolator_reset");
}

#[test]
fn wave_b1_data_converter_to_number() {
    replay("data_converter_to_number");
}

#[test]
fn wave_b1_databind_artboard() {
    replay("databind_artboard");
}

#[test]
fn wave_b1_format_number_with_commas() {
    replay("format_number_with_commas");
}

#[test]
#[ignore = "expected-red: frame 2 op 115 transform tx is -197.96802 instead of 462.03198"]
fn wave_b1_image_fit_alignment() {
    replay("image_fit_alignment");
}

#[test]
fn wave_b1_image_fit_alignment_2() {
    replay("image_fit_alignment_2");
}

#[test]
fn wave_b1_image_fit_alignment_3() {
    replay("image_fit_alignment_3");
}

#[test]
fn wave_b1_image_fit_alignment_updated_test() {
    replay("image_fit_alignment_updated_test");
}

#[test]
#[ignore = "expected-red: frame 1 op 38 transform tx is 200 instead of 0"]
fn wave_b1_interpolation_zero_duration() {
    replay("interpolation_zero_duration");
}

#[test]
fn wave_b1_list_to_length_test() {
    replay("list_to_length_test");
}

#[test]
fn wave_b1_list_to_path() {
    replay("list_to_path");
}

#[test]
fn wave_b1_listener_view_model() {
    replay("listener_view_model");
}

#[test]
fn wave_b1_relative_data_bind_path() {
    replay("relative_data_bind_path");
}

#[test]
#[ignore = "expected-red: frame 1 op 48 expects color but Rust emits save"]
fn wave_b1_relative_data_bind_path_fire_trigger() {
    replay("relative_data_bind_path-fire-trigger");
}

#[test]
#[ignore = "expected-red: frame 1 op 72 expects makeRenderPath but Rust emits drawPath"]
fn wave_b1_relative_data_bind_path_listener() {
    replay("relative_data_bind_path-listener");
}

#[test]
#[ignore = "expected-red: frame 0 op 39 transform tx is 250 instead of 115.56351"]
fn wave_b1_relative_data_bind_path_scripted_input() {
    replay("relative_data_bind_path-scripted-input");
}

#[test]
fn wave_b1_relative_data_binding() {
    replay("relative_data_binding");
}

#[test]
fn wave_b1_state_transition_fire_trigger() {
    replay("state_transition_fire_trigger");
}

#[test]
#[ignore = "expected-red: frame 1 op 65 transform tx is 250.29443 instead of 250.07309"]
fn wave_b1_time_based_interpolation() {
    replay("time_based_interpolation");
}

#[test]
fn wave_b1_trigger_based_listeners() {
    replay("trigger_based_listeners");
}

#[test]
fn wave_b1_trigger_fires_single_change() {
    replay("trigger_fires_single_change");
}

#[test]
#[ignore = "expected-red: frame 0 op 10 expects save but Rust emits color"]
fn wave_b1_unbound_stateful_component() {
    replay("unbound_stateful_component");
}

#[test]
fn wave_b1_viewmodel_based_condition() {
    replay("viewmodel_based_condition");
}

#[test]
fn wave_b1_viewmodel_image_reset() {
    replay("viewmodel_image_reset");
}
